//! The marketplace trust contract: the signed `index.json` catalog format,
//! plus the SHA-256 hashing and ed25519 signing/verification that let the client
//! trust a downloaded mog without trusting the host it came from.
//!
//! One signature over the whole
//! index, plus a per-entry content hash, transitively protects every mog:
//! `install`/`update` verify the index signature with the public key compiled
//! into `mog`, then verify each downloaded mog against the hash in the
//! now-trusted index.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Current `index.json` schema version. Bumped only on a breaking format change
/// (distinct from a mog's own `version`).
pub const INDEX_SCHEMA: u32 = 1;

/// The marketplace public signing key (base64 of the 32-byte ed25519 verifying key),
/// compiled into `mog` so `install`/`update` verify the index signature offline.
/// Empty until the production keypair is generated; [`embedded_public_key`]
/// returns `None` while empty, so callers must decide whether an unconfigured
/// build may talk to the marketplace at all.
pub const MARKET_PUBLIC_KEY_B64: &str = "3B0+jzypvM9vuIT1S03MSfVTxryOuCdWutCP7z4gW+4=";

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !*b
}

/// One mog in the catalog. The author-authored searchable fields (`name`,
/// `description`, `tags`) are copied here so search works offline against the
/// cached index; the curatorial fields (`category`, `featured`, `version`, the
/// hashes, `download_count`) are set by the registry, never by the submitter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Names of other marketplace mogs this mog composes; installed transitively.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Reviewer-assigned hierarchical path, e.g. `sql/dialect-conversion`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Curated front-page flag: a soft ranking boost and the no-query browse
    /// default. Never a hard visibility gate.
    #[serde(default, skip_serializing_if = "is_false")]
    pub featured: bool,
    /// Monotonic mog version; a backward-compatible revision bumps it.
    pub version: u32,
    /// Registry-relative path to the mog `.mog` (its download location).
    pub path: String,
    /// SHA-256 (hex) of the mog `.mog` bytes.
    pub sha256: String,
    /// SHA-256 (hex) of each sibling fixture, keyed by file name. Sorted (BTree)
    /// so the serialized index is deterministic, hence signable.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fixtures: BTreeMap<String, String>,
    /// Advisory popularity (GitHub Release `download_count`), if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

/// The full catalog. Signed as a whole (its exact JSON bytes), so a single
/// signature plus each entry's `sha256` protects every mog transitively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    pub schema: u32,
    /// Set by the generator (an ISO-8601 timestamp); passed in rather than read
    /// from the clock so index generation stays reproducible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            schema: INDEX_SCHEMA,
            generated_at: None,
            entries: Vec::new(),
        }
    }

    /// Look up a single mog by its (globally unique) name.
    pub fn get(&self, name: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// The exact bytes that get signed and verified: deterministic pretty JSON
    /// with a trailing newline. Signing and verifying both operate on these
    /// bytes, never on a re-serialization, so field-ordering can never desync
    /// the two sides.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        let mut v = serde_json::to_vec_pretty(self).context("serialize index")?;
        v.push(b'\n');
        Ok(v)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Index> {
        serde_json::from_slice(bytes).context("parse index.json")
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

/// The signed deny list: mogs pulled after publication (malicious, legal
/// takedown, unfixable break). The client removes/blocks these on `update`, and
/// refuses to `install` them. Published and signed alongside the index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Revocations {
    #[serde(default)]
    pub schema: u32,
    /// Revoked identifiers: a bare `name` (all versions) or `name@version`.
    #[serde(default)]
    pub revoked: Vec<String>,
}

impl Revocations {
    pub fn is_revoked(&self, name: &str, version: u32) -> bool {
        let at = format!("{name}@{version}");
        self.revoked.iter().any(|r| r == name || r == &at)
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        let mut v = serde_json::to_vec_pretty(self).context("serialize revocations")?;
        v.push(b'\n');
        Ok(v)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> Result<Revocations> {
        serde_json::from_slice(bytes).context("parse revocations")
    }
}

/// SHA-256 of `bytes` as a lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

// --- ed25519 signing / verification -----------------------------------------
//
// The private (signing) key lives only as a CI secret and signs the index on
// merge; the public (verifying) key is compiled into `mog`. Keys and signatures
// are stored/transported as base64 of their raw byte form.

/// Generate a fresh ed25519 keypair (registry setup). The signing half becomes a
/// CI secret; the verifying half is pasted into [`MARKET_PUBLIC_KEY_B64`].
pub fn generate_keypair() -> SigningKey {
    SigningKey::generate(&mut rand::rngs::OsRng)
}

pub fn signing_key_to_b64(sk: &SigningKey) -> String {
    B64.encode(sk.to_bytes())
}

pub fn verifying_key_to_b64(vk: &VerifyingKey) -> String {
    B64.encode(vk.to_bytes())
}

pub fn signature_to_b64(sig: &Signature) -> String {
    B64.encode(sig.to_bytes())
}

pub fn signing_key_from_b64(s: &str) -> Result<SigningKey> {
    let bytes = B64.decode(s.trim()).context("decode signing key")?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("signing key must be 32 bytes, got {}", bytes.len()))?;
    Ok(SigningKey::from_bytes(&arr))
}

pub fn verifying_key_from_b64(s: &str) -> Result<VerifyingKey> {
    let bytes = B64.decode(s.trim()).context("decode verifying key")?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("verifying key must be 32 bytes, got {}", bytes.len()))?;
    VerifyingKey::from_bytes(&arr).context("invalid ed25519 verifying key")
}

pub fn signature_from_b64(s: &str) -> Result<Signature> {
    let bytes = B64.decode(s.trim()).context("decode signature")?;
    let arr: [u8; 64] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("signature must be 64 bytes, got {}", bytes.len()))?;
    Ok(Signature::from_bytes(&arr))
}

/// Sign arbitrary bytes (e.g. `index.to_json_bytes()`), returning the base64
/// signature to store next to the file (`index.json.sig`).
pub fn sign(sk: &SigningKey, msg: &[u8]) -> String {
    signature_to_b64(&sk.sign(msg))
}

/// Verify a base64 signature over `msg` with `vk`. Returns an error (never
/// panics) on any decode or verification failure.
pub fn verify(vk: &VerifyingKey, msg: &[u8], sig_b64: &str) -> Result<()> {
    let sig = signature_from_b64(sig_b64)?;
    vk.verify(msg, &sig)
        .map_err(|_| anyhow!("signature verification failed"))
}

/// The public key compiled into this build, if one is configured. `None` means
/// this `mog` was built without a marketplace key and cannot verify the catalog.
pub fn embedded_public_key() -> Option<VerifyingKey> {
    if MARKET_PUBLIC_KEY_B64.is_empty() {
        return None;
    }
    verifying_key_from_b64(MARKET_PUBLIC_KEY_B64).ok()
}

// --- index generation (registry / CI side) ----------------------------------

use std::path::Path;

fn default_version() -> u32 {
    1
}

/// Reviewer-assigned curation for one mog, read from the registry's
/// `curation.json` sidecar (spec section 4: the submitter cannot set these).
#[derive(Debug, Clone, Deserialize)]
pub struct CurationEntry {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub featured: bool,
    #[serde(default = "default_version")]
    pub version: u32,
}

impl Default for CurationEntry {
    fn default() -> Self {
        CurationEntry {
            category: None,
            featured: false,
            version: 1,
        }
    }
}

/// `name -> curation`. Mogs absent from the map default to no category, not
/// featured, version 1.
pub type Curation = BTreeMap<String, CurationEntry>;

pub fn load_curation(path: &Path) -> Result<Curation> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read curation '{}'", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse curation '{}'", path.display()))
}

fn collect_mog_files(root: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in
        std::fs::read_dir(root).with_context(|| format!("read dir '{}'", root.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect_mog_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("mog") {
            out.push(path);
        }
    }
    Ok(())
}

fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Hash a mog's fixtures under its `tests/` directory: `input.*` and
/// `expected.*`. Keys are the fixture path relative to the mog directory
/// (e.g. `tests/input.txt`) so the manifest is unambiguous. A mog with no
/// `tests/` directory contributes an empty map.
fn fixture_hashes(mog_path: &Path) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    let dir = mog_path.parent().unwrap_or_else(|| Path::new("."));
    let tests_dir = dir.join("tests");
    let entries = match std::fs::read_dir(&tests_dir) {
        Ok(e) => e,
        Err(_) => return Ok(map),
    };
    for entry in entries {
        let entry = entry?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with("input.") || fname.starts_with("expected.") {
            let bytes = std::fs::read(entry.path())?;
            map.insert(format!("tests/{fname}"), sha256_hex(&bytes));
        }
    }
    Ok(map)
}

/// Walk `recipe_root` for `.mog` files and build the catalog: author fields
/// (`description`, `tags`) copied from each mog, curatorial fields from
/// `curation`, content hashes computed over the mog and its fixtures. Entries
/// are sorted by name so the output, and therefore its signature, is
/// deterministic.
pub fn build_index(
    mog_root: &Path,
    path_base: &Path,
    curation: &Curation,
    generated_at: Option<String>,
) -> Result<Index> {
    let mut mog_files = Vec::new();
    collect_mog_files(mog_root, &mut mog_files)?;

    let mut entries = Vec::new();
    for mog_path in mog_files {
        let stem = mog_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("bad recipe file name: {}", mog_path.display()))?
            .to_string();
        let bytes = std::fs::read(&mog_path)
            .with_context(|| format!("read recipe '{}'", mog_path.display()))?;
        // Author metadata only; the registry does full validation at review time.
        let meta: crate::model::Mog = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse recipe '{}'", mog_path.display()))?;
        let cur = curation.get(&stem).cloned().unwrap_or_default();

        entries.push(IndexEntry {
            name: stem.clone(),
            description: meta.description,
            tags: meta.tags,
            dependencies: meta.dependencies,
            category: cur.category,
            featured: cur.featured,
            version: cur.version,
            path: rel_slash(path_base, &mog_path),
            sha256: sha256_hex(&bytes),
            fixtures: fixture_hashes(&mog_path)?,
            download_count: None,
            published: None,
            updated: None,
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Index {
        schema: INDEX_SCHEMA,
        generated_at,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        // NIST test vector: SHA-256("abc").
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sign_verify_roundtrip_and_tamper() {
        let sk = generate_keypair();
        let vk = sk.verifying_key();
        let msg = b"the exact index bytes";
        let sig = sign(&sk, msg);

        assert!(verify(&vk, msg, &sig).is_ok());
        // A tampered message must fail.
        assert!(verify(&vk, b"the exact index bytez", &sig).is_err());
        // A different key must fail.
        let other = generate_keypair().verifying_key();
        assert!(verify(&other, msg, &sig).is_err());
    }

    #[test]
    fn key_and_signature_b64_roundtrip() {
        let sk = generate_keypair();
        let vk = sk.verifying_key();

        let sk2 = signing_key_from_b64(&signing_key_to_b64(&sk)).unwrap();
        assert_eq!(sk.to_bytes(), sk2.to_bytes());

        let vk2 = verifying_key_from_b64(&verifying_key_to_b64(&vk)).unwrap();
        assert_eq!(vk.to_bytes(), vk2.to_bytes());

        let sig = sk.sign(b"payload");
        let sig2 = signature_from_b64(&signature_to_b64(&sig)).unwrap();
        assert_eq!(sig.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn index_json_roundtrips_and_is_signable() {
        let mut idx = Index::new();
        idx.entries.push(IndexEntry {
            name: "tidy-list".into(),
            description: Some("de-duplicate, sort, and trim a list".into()),
            tags: vec!["list".into(), "cleanup".into()],
            dependencies: vec![],
            category: Some("text/list".into()),
            featured: true,
            version: 1,
            path: "recipes/tidy-list/tidy-list.mog".into(),
            sha256: sha256_hex(b"{\"steps\":[]}"),
            fixtures: BTreeMap::from([
                ("tests/input.txt".into(), sha256_hex(b"in")),
                ("tests/expected.txt".into(), sha256_hex(b"out")),
            ]),
            download_count: Some(42),
            published: None,
            updated: None,
        });

        // Round-trips exactly.
        let bytes = idx.to_json_bytes().unwrap();
        assert_eq!(Index::from_json_bytes(&bytes).unwrap(), idx);

        // Serialization is deterministic (so the signature is stable).
        assert_eq!(idx.to_json_bytes().unwrap(), bytes);

        // Sign the exact bytes, verify against them.
        let sk = generate_keypair();
        let sig = sign(&sk, &bytes);
        assert!(verify(&sk.verifying_key(), &bytes, &sig).is_ok());
    }

    #[test]
    fn revocation_matching() {
        let r = Revocations {
            schema: 1,
            revoked: vec!["evil-recipe".into(), "buggy@3".into()],
        };
        // Bare name revokes every version.
        assert!(r.is_revoked("evil-recipe", 1));
        assert!(r.is_revoked("evil-recipe", 99));
        // name@version revokes only that version.
        assert!(r.is_revoked("buggy", 3));
        assert!(!r.is_revoked("buggy", 2));
        assert!(!r.is_revoked("innocent", 1));
    }

    #[test]
    fn embedded_key_is_present_and_valid() {
        // The production verifying key is compiled in; it must parse as a real
        // ed25519 key so the client can verify the catalog signature offline.
        assert!(embedded_public_key().is_some());
    }

    #[test]
    fn build_index_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // One directory per mog: <name>/<name>.mog + <name>/tests/{input,expected}.
        let mog_dir = root.join("tidy-list");
        std::fs::create_dir_all(mog_dir.join("tests")).unwrap();
        std::fs::write(
            mog_dir.join("tidy-list.mog"),
            br#"{"name":"Tidy List","description":"clean a list","tags":["list"],"steps":[{"action":"sort_lines"}]}"#,
        )
        .unwrap();
        std::fs::write(mog_dir.join("tests/input.txt"), b"b\na\n").unwrap();
        std::fs::write(mog_dir.join("tests/expected.txt"), b"a\nb\n").unwrap();

        let mut cur = Curation::new();
        cur.insert(
            "tidy-list".into(),
            CurationEntry {
                category: Some("text/list".into()),
                featured: true,
                version: 2,
            },
        );

        let idx = build_index(root, root, &cur, Some("2026-08-23".into())).unwrap();
        assert_eq!(idx.entries.len(), 1);
        let e = &idx.entries[0];
        assert_eq!(e.name, "tidy-list");
        assert_eq!(e.description.as_deref(), Some("clean a list"));
        assert_eq!(e.tags, vec!["list".to_string()]);
        assert_eq!(e.category.as_deref(), Some("text/list"));
        assert!(e.featured);
        assert_eq!(e.version, 2);
        assert_eq!(e.path, "tidy-list/tidy-list.mog");
        assert_eq!(e.fixtures.len(), 2);

        // The generated index is signable and verifies.
        let sk = generate_keypair();
        let bytes = idx.to_json_bytes().unwrap();
        let sig = sign(&sk, &bytes);
        assert!(verify(&sk.verifying_key(), &bytes, &sig).is_ok());
    }
}
