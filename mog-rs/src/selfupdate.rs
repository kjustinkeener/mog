//! The engine track of `mog update`: replace the running `mog` binary in place
//! with a newer signed build published in the registry, alongside the mog
//! sync in [`crate::market_client::sync`].
//!
//! Trust mirrors the catalog: a signed `engine.json` manifest lists, per
//! platform, the download path and the SHA-256 of the binary. The manifest
//! signature is checked with the same key that verifies the mog index, then
//! the downloaded bytes are checked against the now-trusted hash before the swap.
//!
//! The swap uses `self_replace`, which handles the Windows case where a running
//! executable cannot be overwritten: it renames the live image aside and writes
//! the new bytes into the original path. The *file* is new immediately (so the
//! next `mog` invocation is the new code); a process that already loaded the old
//! image keeps running it until it respawns.

use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::market_index::{self, sha256_hex};

/// One platform's binary in the engine manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePlatform {
    /// Registry-relative path to the binary (its download location).
    pub path: String,
    /// SHA-256 (hex) of the binary bytes.
    pub sha256: String,
}

/// The signed `engine.json`: the newest published engine build per platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineManifest {
    /// Human-facing version of the published build (display only; the hash is the
    /// actual freshness check).
    #[serde(default)]
    pub version: String,
    /// Keyed by `"<os>-<arch>"`, e.g. `"windows-x86_64"`.
    pub platforms: std::collections::BTreeMap<String, EnginePlatform>,
}

/// The current platform key, matching the manifest's `platforms` keys.
pub fn platform_key() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// The verifying key for the engine manifest: `MOG_MARKET_PUBKEY` (mirror/dev)
/// else the key compiled into this build. Verification is never skipped.
fn resolve_pubkey() -> Result<VerifyingKey> {
    if let Ok(k) = std::env::var("MOG_MARKET_PUBKEY") {
        if !k.trim().is_empty() {
            return market_index::verifying_key_from_b64(k.trim());
        }
    }
    market_index::embedded_public_key().ok_or_else(|| {
        anyhow!("this mog build has no marketplace public key (and MOG_MARKET_PUBKEY is unset); cannot verify an engine update")
    })
}

fn is_http(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn join(base: &str, rel: &str) -> String {
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        rel.trim_start_matches('/')
    )
}

/// Fetch bytes from the registry: HTTP(S) via ureq, or a local filesystem path
/// (the latter makes end-to-end testing trivial and doubles as a private mirror).
fn fetch_bytes(base: &str, rel: &str) -> Result<Vec<u8>> {
    if is_http(base) {
        let url = join(base, rel);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| anyhow!("GET {url}: {e}"))?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf)
            .with_context(|| format!("read body of {url}"))?;
        Ok(buf)
    } else {
        let path = Path::new(base).join(rel.trim_start_matches('/'));
        std::fs::read(&path).with_context(|| format!("read '{}'", path.display()))
    }
}

/// Fetch and signature-verify the engine manifest. A missing manifest (no engine
/// track published for this registry) returns `None` rather than erroring.
pub fn fetch_manifest(base: &str) -> Result<Option<EngineManifest>> {
    let Ok(bytes) = fetch_bytes(base, "engine.json") else {
        return Ok(None);
    };
    let vk = resolve_pubkey()?;
    let sig = String::from_utf8(fetch_bytes(base, "engine.json.sig")?)
        .context("engine.json.sig is not valid utf-8")?;
    market_index::verify(&vk, &bytes, sig.trim())
        .context("engine manifest signature verification failed (refusing to trust it)")?;
    let manifest: EngineManifest = serde_json::from_slice(&bytes).context("parse engine.json")?;
    Ok(Some(manifest))
}

/// What an engine check found.
pub struct EngineStatus {
    /// The published build for this platform, if the registry has one.
    pub available: Option<EnginePlatform>,
    /// Published version string (display only).
    pub version: String,
    /// True when a published build exists and differs from the running binary.
    pub update_available: bool,
}

/// Compare the running binary against the manifest without changing anything.
pub fn check(base: &str) -> Result<EngineStatus> {
    let Some(manifest) = fetch_manifest(base)? else {
        return Ok(EngineStatus {
            available: None,
            version: String::new(),
            update_available: false,
        });
    };
    let key = platform_key();
    let Some(plat) = manifest.platforms.get(&key).cloned() else {
        return Ok(EngineStatus {
            available: None,
            version: manifest.version,
            update_available: false,
        });
    };
    let exe = std::env::current_exe().context("locate the running mog binary")?;
    let cur_sha = std::fs::read(&exe)
        .ok()
        .map(|b| sha256_hex(&b))
        .unwrap_or_default();
    let update_available = cur_sha != plat.sha256;
    Ok(EngineStatus {
        update_available,
        available: Some(plat),
        version: manifest.version,
    })
}

/// Download the newer build (verifying its hash) and swap it in over the running
/// binary. Returns `false` when already current (nothing done). Safe to call from
/// a long-lived host (MCP / Studio sidecar): the swap never overwrites the live
/// image's open bytes, so the caller's process keeps running until it respawns.
pub fn apply(base: &str) -> Result<bool> {
    let status = check(base)?;
    if !status.update_available {
        return Ok(false);
    }
    let plat = status
        .available
        .ok_or_else(|| anyhow!("no engine build published for {}", platform_key()))?;

    let bytes = fetch_bytes(base, &plat.path)?;
    let got = sha256_hex(&bytes);
    if got != plat.sha256 {
        bail!(
            "hash mismatch for the downloaded engine binary: expected {} got {} (refusing)",
            plat.sha256,
            got
        );
    }

    // Stage the new bytes beside the current exe, then let self_replace rename the
    // running image aside and move the staged file into its place.
    let exe = std::env::current_exe().context("locate the running mog binary")?;
    let dir = exe.parent().unwrap_or_else(|| Path::new("."));
    let staged = dir.join(format!(".mog-update-{}.tmp", std::process::id()));
    std::fs::write(&staged, &bytes).with_context(|| format!("stage '{}'", staged.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&staged)?.permissions();
        perm.set_mode(0o755);
        let _ = std::fs::set_permissions(&staged, perm);
    }

    let result = self_replace::self_replace(&staged);
    let _ = std::fs::remove_file(&staged);
    result.context("swap in the new mog binary")?;
    Ok(true)
}
