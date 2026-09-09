//! market-admin: registry-side maintenance for the Mog marketplace. It
//! generates the signing keypair and builds + signs the catalog `index.json`.
//! This is NOT shipped to end users; the registry's CI (or a maintainer) runs it
//! on merge. It reuses the `mog` library's `market_index` contract so the format
//! and crypto can never drift from what the client verifies.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use mog::market_index::{self, build_index, load_curation, sign, signing_key_from_b64, Curation};
use mog::selfupdate::{EngineManifest, EnginePlatform};

#[derive(Parser)]
#[command(name = "market-admin", about = "Mog marketplace registry maintenance")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate a fresh ed25519 keypair. Save the signing key as a CI secret and
    /// paste the verifying key into `MARKET_PUBLIC_KEY_B64`.
    Keygen,
    /// Build and sign `index.json` from a directory of mogs.
    Index {
        /// Directory of `.mog` mogs (with sibling fixtures).
        #[arg(long)]
        mogs: PathBuf,
        /// Optional `curation.json`: name -> {category, featured, version}.
        #[arg(long)]
        curation: Option<PathBuf>,
        /// Signing key: base64, or `@path` to a file containing it.
        #[arg(long)]
        sign_key: String,
        /// Output dir for `index.json` + `index.json.sig` (default: --mogs).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Value for the index's `generated_at` field (e.g. an ISO timestamp).
        /// Passed in, not read from the clock, so generation stays reproducible.
        #[arg(long)]
        generated_at: Option<String>,
    },
    /// Build and sign `engine.json`: the per-platform engine binaries that
    /// `mog update` self-replaces from. Each `--platform` local binary is hashed;
    /// the client verifies that hash before swapping. With `--url-base` the
    /// manifest points at that absolute host (a GitHub Release on the engine repo)
    /// and no binary is written into `<out>`; without it the binary is copied into
    /// `<out>/bin/` and referenced relative to the catalog base URL.
    Engine {
        /// Published version string (display only; the hash is the freshness check).
        #[arg(long)]
        version: String,
        /// A platform binary as `<os>-<arch>=<local-path>`, e.g.
        /// `windows-x86_64=./dist/mog.exe`. Repeatable, one per platform.
        #[arg(long = "platform", value_name = "KEY=PATH")]
        platforms: Vec<String>,
        /// Absolute base URL the binaries are hosted at (e.g. a GitHub Release
        /// download prefix). When set, the manifest path is `<url-base>/<filename>`
        /// and nothing is copied into `<out>/bin/`.
        #[arg(long)]
        url_base: Option<String>,
        /// Signing key: base64, or `@path` to a file containing it.
        #[arg(long)]
        sign_key: String,
        /// Output dir for `engine.json` + `engine.json.sig` (+ `bin/` unless `--url-base`).
        #[arg(long)]
        out: PathBuf,
    },
    /// Build and sign `studio.json`: the per-platform Studio binaries that
    /// `mog install studio` downloads. Same shape as `engine`; publish it beside
    /// the engine manifest (CI order: build dist mog.exe, embed it into Studio,
    /// then publish both). `--url-base` behaves as for `engine`.
    Studio {
        /// Published version string (display only; the hash is the freshness check).
        #[arg(long)]
        version: String,
        /// A platform binary as `<os>-<arch>=<local-path>`, e.g.
        /// `windows-x86_64=./MogStudio/.../mog-studio.exe`. Repeatable.
        #[arg(long = "platform", value_name = "KEY=PATH")]
        platforms: Vec<String>,
        /// Absolute base URL the binaries are hosted at (see `engine --url-base`).
        #[arg(long)]
        url_base: Option<String>,
        /// Signing key: base64, or `@path` to a file containing it.
        #[arg(long)]
        sign_key: String,
        /// Output dir for `studio.json` + `studio.json.sig` (+ `bin/` unless `--url-base`).
        #[arg(long)]
        out: PathBuf,
    },
}

/// Build + sign a per-platform binary manifest (`engine.json` / `studio.json`).
/// Hashes each `KEY=PATH` binary and writes the signed `<name>.json` (+ `.sig`).
/// With `url_base` the manifest path is `<url_base>/<filename>` (the binary is
/// hosted elsewhere, e.g. a GitHub Release); without it the binary is copied into
/// `<out>/bin/` and referenced relative to the catalog base URL. Shared by the
/// `engine` and `studio` commands so their format can never drift.
fn publish_manifest(
    name: &str,
    version: String,
    platforms: &[String],
    url_base: Option<&str>,
    sign_key: &str,
    out: &std::path::Path,
) -> Result<()> {
    let sk = read_sign_key(sign_key)?;
    std::fs::create_dir_all(out)?;
    // Only materialize <out>/bin when hosting the binaries in the catalog repo.
    // With --url-base the binaries live elsewhere (a GitHub Release), so <out>
    // holds signed JSON only.
    if url_base.is_none() {
        std::fs::create_dir_all(out.join("bin"))?;
    }
    let mut map = std::collections::BTreeMap::new();
    for spec in platforms {
        let (key, src) = spec
            .split_once('=')
            .with_context(|| format!("--platform must be KEY=PATH, got '{spec}'"))?;
        let src = PathBuf::from(src);
        let bytes = std::fs::read(&src)
            .with_context(|| format!("read platform binary '{}'", src.display()))?;
        let sha = market_index::sha256_hex(&bytes);
        let fname = src
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{name}-{key}"));
        let path = match url_base {
            // Absolute host: the binary is published as a release asset named
            // `fname`; the client fetches this URL directly, ignoring the base.
            Some(base) => format!("{}/{}", base.trim_end_matches('/'), fname),
            // Catalog-relative: copy the binary in beside the manifest.
            None => {
                let rel = format!("bin/{fname}");
                std::fs::write(out.join(&rel), &bytes)?;
                rel
            }
        };
        map.insert(key.to_string(), EnginePlatform { path, sha256: sha });
    }
    let manifest = EngineManifest {
        version,
        platforms: map,
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    let sig = sign(&sk, &bytes);
    let man_path = out.join(format!("{name}.json"));
    let sig_path = out.join(format!("{name}.json.sig"));
    std::fs::write(&man_path, &bytes)?;
    std::fs::write(&sig_path, format!("{sig}\n"))?;
    eprintln!(
        "wrote {} ({} platforms) + {}",
        man_path.display(),
        manifest.platforms.len(),
        sig_path.display()
    );
    Ok(())
}

fn read_sign_key(arg: &str) -> Result<SigningKey> {
    let b64 = if let Some(path) = arg.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("read sign-key file '{path}'"))?
    } else {
        arg.to_string()
    };
    signing_key_from_b64(&b64)
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Keygen => {
            let sk = market_index::generate_keypair();
            println!(
                "signing key  (SECRET, keep private): {}",
                market_index::signing_key_to_b64(&sk)
            );
            println!(
                "verifying key (public, embed in mog): {}",
                market_index::verifying_key_to_b64(&sk.verifying_key())
            );
        }
        Cmd::Index {
            mogs,
            curation,
            sign_key,
            out,
            generated_at,
        } => {
            let cur: Curation = match curation {
                Some(p) => load_curation(&p)?,
                None => Curation::new(),
            };
            let sk = read_sign_key(&sign_key)?;
            let out_dir = out.unwrap_or_else(|| mogs.clone());
            // Paths in the index are relative to the output dir (where index.json
            // lives), so mogs may sit in a `recipes/` subdir and the client
            // still fetches base/<path> correctly.
            let index = build_index(&mogs, &out_dir, &cur, generated_at)?;
            let bytes = index.to_json_bytes()?;
            let sig = sign(&sk, &bytes);

            std::fs::create_dir_all(&out_dir)?;
            let idx_path = out_dir.join("index.json");
            let sig_path = out_dir.join("index.json.sig");
            std::fs::write(&idx_path, &bytes)?;
            std::fs::write(&sig_path, format!("{sig}\n"))?;
            eprintln!(
                "wrote {} ({} entries) + {}",
                idx_path.display(),
                index.entries.len(),
                sig_path.display()
            );
        }
        Cmd::Engine {
            version,
            platforms,
            url_base,
            sign_key,
            out,
        } => publish_manifest(
            "engine",
            version,
            &platforms,
            url_base.as_deref(),
            &sign_key,
            &out,
        )?,
        Cmd::Studio {
            version,
            platforms,
            url_base,
            sign_key,
            out,
        } => publish_manifest(
            "studio",
            version,
            &platforms,
            url_base.as_deref(),
            &sign_key,
            &out,
        )?,
    }
    Ok(())
}
