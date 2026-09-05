//! The client side of the marketplace: fetch and verify the signed catalog,
//! then `install` / `update` / `list` / `show` / `search` against it.
//!
//! Trust flow (spec sections 9, 11-12): verify the index *signature* with the
//! public key compiled into `mog`, then verify each downloaded recipe against the
//! `sha256` in the now-trusted index. Discovery is hybrid, work from a verified
//! local cache, refresh from the registry when asked. The registry base can be an
//! `http(s)` URL or a local directory path (the latter makes end-to-end testing
//! trivial and doubles as a private mirror).

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use ed25519_dalek::VerifyingKey;
use serde::Serialize;

use crate::market_index::{self, Index, IndexEntry, Revocations};

/// Default registry base, compiled in; overridable with `MOG_MARKET_URL`. Empty
/// until the public registry exists.
pub const MARKET_DEFAULT_URL: &str = "https://kjustinkeener.github.io/mog-market/";

/// Cap on CLI search/list results (context economy, spec section 9).
const RESULT_CAP: usize = 25;
/// Browse-all (`market list`) upper bound -- effectively uncapped for a curated
/// library, just a guard against a pathologically large merged catalog.
const LIST_CAP: usize = 2000;

pub fn market_base_url() -> Result<String> {
    if let Ok(u) = std::env::var("MOG_MARKET_URL") {
        if !u.trim().is_empty() {
            return Ok(u.trim().to_string());
        }
    }
    if !MARKET_DEFAULT_URL.is_empty() {
        return Ok(MARKET_DEFAULT_URL.to_string());
    }
    bail!("no marketplace URL configured (set MOG_MARKET_URL, or build with MARKET_DEFAULT_URL)")
}

/// The verifying key to check the catalog with: `MOG_MARKET_PUBKEY` (for a mirror
/// or dev) else the key compiled into this build. Verification is never skipped.
fn resolve_pubkey() -> Result<VerifyingKey> {
    if let Ok(k) = std::env::var("MOG_MARKET_PUBKEY") {
        if !k.trim().is_empty() {
            return market_index::verifying_key_from_b64(k.trim());
        }
    }
    market_index::embedded_public_key().ok_or_else(|| {
        anyhow!("this mog build has no marketplace public key (and MOG_MARKET_PUBKEY is unset); cannot verify the catalog")
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

/// Fetch bytes from the registry: HTTP(S) via ureq, or a local filesystem path.
fn fetch_bytes(base: &str, rel: &str) -> Result<Vec<u8>> {
    if is_http(base) {
        let url = join(base, rel);
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| anyhow!("GET {url}: {e}"))?;
        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .with_context(|| format!("read body of {url}"))?;
        Ok(buf)
    } else {
        let path = Path::new(base).join(rel.trim_start_matches('/'));
        fs::read(&path).with_context(|| format!("read '{}'", path.display()))
    }
}

fn fetch_bytes_opt(base: &str, rel: &str) -> Option<Vec<u8>> {
    fetch_bytes(base, rel).ok()
}

fn cache_dir(root: &Path) -> PathBuf {
    root.join(".cache")
}

/// Fetch `index.json` + `index.json.sig`, verify the signature, cache the
/// verified bytes (for offline search), and return the parsed catalog.
pub fn fetch_and_verify_index(root: &Path, base: &str) -> Result<Index> {
    let vk = resolve_pubkey()?;
    let bytes = fetch_bytes(base, "index.json")?;
    let sig = String::from_utf8(fetch_bytes(base, "index.json.sig")?)
        .context("index.json.sig is not valid utf-8")?;
    market_index::verify(&vk, &bytes, sig.trim())
        .context("index signature verification failed (refusing to trust the catalog)")?;
    let index = Index::from_json_bytes(&bytes)?;

    let cache = cache_dir(root);
    let _ = fs::create_dir_all(&cache);
    let _ = fs::write(cache.join("index.json"), &bytes);
    let _ = fs::write(cache.join("index.json.sig"), sig.trim());
    Ok(index)
}

pub fn load_cached_index(root: &Path) -> Result<Option<Index>> {
    let p = cache_dir(root).join("index.json");
    if !p.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&p)?;
    // The cache was verified when written; re-verify when a key is available.
    if let Ok(vk) = resolve_pubkey() {
        if let Ok(sig) = fs::read_to_string(cache_dir(root).join("index.json.sig")) {
            market_index::verify(&vk, &bytes, sig.trim())
                .context("cached index failed re-verification")?;
        }
    }
    Ok(Some(Index::from_json_bytes(&bytes)?))
}

/// The catalog to work from: the verified cache unless `refresh` is set or the
/// cache is empty, in which case fetch from the registry.
pub fn ensure_index(root: &Path, base: &str, refresh: bool) -> Result<Index> {
    if !refresh {
        if let Some(idx) = load_cached_index(root)? {
            return Ok(idx);
        }
    }
    fetch_and_verify_index(root, base)
}

/// The signed deny list, if published (verified). A missing list is empty.
fn fetch_revocations(base: &str) -> Result<Revocations> {
    let Some(bytes) = fetch_bytes_opt(base, "revocations.json") else {
        return Ok(Revocations::default());
    };
    if let Ok(vk) = resolve_pubkey() {
        if let Some(sig) = fetch_bytes_opt(base, "revocations.json.sig") {
            let sig = String::from_utf8(sig).unwrap_or_default();
            market_index::verify(&vk, &bytes, sig.trim())
                .context("revocation list signature verification failed")?;
        }
    }
    Revocations::from_json_bytes(&bytes)
}

// --- install / update --------------------------------------------------------

/// The install manifest lives in the managed recipe dir: `<root>/mogs/market/.installed.json`.
fn manifest_path(root: &Path) -> PathBuf {
    crate::library::market_dir(root).join(".installed.json")
}

fn read_installed(root: &Path) -> BTreeMap<String, u32> {
    fs::read(manifest_path(root))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn write_installed(root: &Path, m: &BTreeMap<String, u32>) -> Result<()> {
    fs::create_dir_all(crate::library::market_dir(root))?;
    let mut v = serde_json::to_vec_pretty(m)?;
    v.push(b'\n');
    fs::write(manifest_path(root), v)?;
    Ok(())
}

/// Download + verify a recipe (body and fixtures) and write it into the install
/// source. Does not do collision/revocation checks; callers gate those.
fn install_entry(root: &Path, base: &str, entry: &IndexEntry) -> Result<()> {
    let mog_bytes = fetch_bytes(base, &entry.path)?;
    let got = market_index::sha256_hex(&mog_bytes);
    if got != entry.sha256 {
        bail!(
            "hash mismatch for '{}': expected {} got {} (refusing)",
            entry.name,
            entry.sha256,
            got
        );
    }

    // Fixtures live beside the recipe in the registry.
    let dir = Path::new(&entry.path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    let mut fixtures: Vec<(String, Vec<u8>)> = Vec::new();
    for (fname, fhash) in &entry.fixtures {
        let rel = if dir.is_empty() {
            fname.clone()
        } else {
            format!("{dir}/{fname}")
        };
        let fbytes = fetch_bytes(base, &rel)?;
        if market_index::sha256_hex(&fbytes) != *fhash {
            bail!(
                "hash mismatch for fixture '{fname}' of '{}' (refusing)",
                entry.name
            );
        }
        fixtures.push((fname.clone(), fbytes));
    }

    // Each recipe installs into its own directory (mirrors the one-dir-per-recipe
    // factory layout) so fixtures with identical keys (e.g. `tests/input.txt`) never
    // clobber another recipe's goldens under a shared `community/tests/`.
    let dst = crate::library::market_dir(root).join(&entry.name);
    fs::create_dir_all(&dst).with_context(|| format!("create '{}'", dst.display()))?;
    fs::write(dst.join(format!("{}.mog", entry.name)), &mog_bytes)?;
    for (fname, fbytes) in &fixtures {
        let fpath = dst.join(fname);
        if let Some(parent) = fpath.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create '{}'", parent.display()))?;
        }
        fs::write(fpath, fbytes)?;
    }

    let mut installed = read_installed(root);
    installed.insert(entry.name.clone(), entry.version);
    write_installed(root, &installed)?;
    Ok(())
}

fn remove_installed(root: &Path, name: &str) -> Result<()> {
    let dst = crate::library::market_dir(root).join(name);
    // Each recipe owns its directory (`<name>/`), so remove the whole tree.
    let _ = fs::remove_dir_all(&dst);
    let mut installed = read_installed(root);
    installed.remove(name);
    write_installed(root, &installed)?;
    Ok(())
}

pub fn install(root: Option<&Path>, base: &str, name: &str, json: bool) -> Result<i32> {
    let root = root.ok_or_else(|| anyhow!("no library root (set MOG_HOME or --mog-dir)"))?;
    let index = ensure_index(root, base, false)?;
    if index.get(name).is_none() {
        bail!("no recipe '{name}' in the marketplace");
    }
    let revs = fetch_revocations(base).unwrap_or_default();

    let mut installing: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut done: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut installed_now: Vec<(String, u32)> = Vec::new();
    install_recursive(
        root,
        base,
        &index,
        &revs,
        name,
        &mut installing,
        &mut done,
        &mut installed_now,
    )?;

    if json {
        #[derive(Serialize)]
        struct Item {
            name: String,
            version: u32,
        }
        let items: Vec<Item> = installed_now
            .iter()
            .map(|(n, v)| Item {
                name: n.clone(),
                version: *v,
            })
            .collect();
        println!("{}", serde_json::json!({ "installed": items }));
    } else if installed_now.is_empty() {
        println!("{name} is already installed and up to date");
    } else {
        for (n, v) in &installed_now {
            let tag = if n == name { "" } else { "  (dependency)" };
            println!("installed {n} v{v} -> {n}/{n}.mog{tag}");
        }
    }
    Ok(0)
}

/// Install `name` and everything it depends on, transitively, dependencies
/// first. Cycles are detected via `installing`; recipes already installed at the
/// same version are skipped.
#[allow(clippy::too_many_arguments)]
fn install_recursive(
    root: &Path,
    base: &str,
    index: &Index,
    revs: &Revocations,
    name: &str,
    installing: &mut std::collections::HashSet<String>,
    done: &mut std::collections::HashSet<String>,
    out: &mut Vec<(String, u32)>,
) -> Result<()> {
    if done.contains(name) {
        return Ok(());
    }
    let entry = index
        .get(name)
        .ok_or_else(|| anyhow!("dependency '{name}' is not in the marketplace"))?
        .clone();
    if revs.is_revoked(name, entry.version) {
        bail!(
            "'{name}' is revoked (version {}); refusing to install",
            entry.version
        );
    }
    if !installing.insert(name.to_string()) {
        bail!("dependency cycle detected at '{name}'");
    }
    for dep in &entry.dependencies {
        install_recursive(root, base, index, revs, dep, installing, done, out)?;
    }
    // Skip the download when this exact version is already installed.
    if read_installed(root).get(name).copied() != Some(entry.version) {
        install_entry(root, base, &entry)?;
        out.push((name.to_string(), entry.version));
    }
    installing.remove(name);
    done.insert(name.to_string());
    Ok(())
}

/// The recipe diff `sync` computes against the catalog: which installed recipes
/// are new / content-changed / removed-by-revocation / orphaned. Content is keyed
/// by SHA-256 (never a monotonic version integer), so a recipe re-published with
/// no version bump still updates -- "pull whatever the catalog now says, replace
/// what differs." Orphans (on disk, gone from the catalog) are reported but only
/// removed under `prune`; revoked recipes are always removed.
#[derive(Serialize, Default, Debug)]
pub struct SyncPlan {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub revoked: Vec<String>,
    pub orphaned: Vec<String>,
    pub catalog: usize,
}

impl SyncPlan {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.changed.is_empty()
            && self.revoked.is_empty()
            && self.orphaned.is_empty()
    }
}

/// Sync the local library to the catalog by content hash. When `check` is set
/// nothing is written -- it just returns the plan (for `mog update --check` and
/// the Studio badge). `prune` removes orphaned installs (present on disk, absent
/// from the catalog); revoked recipes are removed regardless.
pub fn sync(root: &Path, base: &str, check: bool, prune: bool) -> Result<SyncPlan> {
    let index = fetch_and_verify_index(root, base)?; // always refresh
    let revs = fetch_revocations(base).unwrap_or_default();

    let mut plan = SyncPlan {
        catalog: index.entries.len(),
        ..Default::default()
    };

    // Pull every non-revoked catalog entry whose on-disk bytes are missing or
    // differ.
    for e in &index.entries {
        if revs.is_revoked(&e.name, e.version) {
            continue; // handled by the revocation sweep below
        }
        let path = crate::library::market_dir(root)
            .join(&e.name)
            .join(format!("{}.mog", e.name));
        let local_sha = fs::read(&path).ok().map(|b| market_index::sha256_hex(&b));
        match local_sha {
            Some(s) if s == e.sha256 => {} // already current
            Some(_) => {
                if !check {
                    install_entry(root, base, e)?;
                }
                plan.changed.push(e.name.clone());
            }
            None => {
                if !check {
                    install_entry(root, base, e)?;
                }
                plan.added.push(e.name.clone());
            }
        }
    }

    // Sweep what is installed but should not be: revoked (always removed) or
    // orphaned (removed only under `prune`).
    for (name, ver) in &read_installed(root) {
        if revs.is_revoked(name, *ver) {
            if !check {
                remove_installed(root, name)?;
            }
            plan.revoked.push(name.clone());
        } else if index.get(name).is_none() {
            if prune && !check {
                remove_installed(root, name)?;
            }
            plan.orphaned.push(name.clone());
        }
    }

    plan.added.sort();
    plan.changed.sort();
    plan.revoked.sort();
    plan.orphaned.sort();
    Ok(plan)
}

// --- discovery: list / search / show (local-first, catalog-augmented) --------
//
// The library on disk is just the already-installed slice of the marketplace, so
// list/search/show present ONE unified view: local recipes (always, offline) plus
// catalog recipes not yet installed (when a registry is configured and reachable).

#[derive(Serialize)]
struct Row {
    name: String,
    /// Short human browse blurb (from the recipe's `summary`). Additive: present
    /// only when the recipe declares one; catalog-only entries have None. The
    /// human render prefers this, but the AI/`--json` channel still carries the
    /// full `description` below regardless.
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    /// Natural-language discovery triggers from the recipe's `task_phrases`.
    /// Weighted strongly by search; empty for catalog-only rows (the signed index
    /// does not yet carry the field).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    task_phrases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    featured: bool,
    version: u32,
    /// "installed" (on disk) or "available" (catalog, not on disk).
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    download_count: Option<u64>,
}

/// The recipe's unique name: the file stem of a resolvable like `user/foo.mog`.
fn recipe_stem(resolvable: &str) -> String {
    Path::new(resolvable)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Seeded popularity for the shipped factory recipes: an editorial estimate of how
/// commonly each is likely to be reached for, so a fresh install with no live
/// catalog still gets a sensible popularity-ordered no-query browse (spec §9.1
/// makes popularity an advisory sort input, never a visibility gate). This is a
/// BASELINE only: once real GitHub Release `download_count`s accumulate in the
/// catalog, `popularity()` adds them on top of this seed rather than replacing it.
const FACTORY_BASELINE: &[(&str, u64)] = &[
    // DATA-ENG SPOTLIGHT: recipes touching current data-eng tech, seeded above the
    // everyday-util tier so they lead the catalog. Ordered so adjacent entries are
    // DIFFERENT kinds of tool (warehouse-migration / reshape / format-interchange /
    // transform / schema / orchestration / IaC) -- the top few read as a varied
    // cross-section, not a run of near-identical converters. First-match wins in
    // factory_baseline(), so these override any lower duplicate seeds below.
    ("dbt-model-from-raw-sql", 90000),        // transform (dbt)
    ("databricks-to-snowflake", 89000),       // warehouse migration
    ("pivot-csv", 88000),                     // tabular reshape
    ("json-to-jsonl", 87000),                 // format interchange (JSONL)
    ("terraform-to-opentofu", 86000),         // IaC migration
    ("bigquery-to-duckdb", 85000),            // warehouse -> local engine (DuckDB)
    ("protobuf-to-json-schema", 84000),       // streaming/schema
    ("unpivot-csv", 83000),                   // tabular reshape
    ("csv-to-json", 82000),                   // format interchange
    ("airflow1-to-2-imports", 81000),         // orchestration migration
    ("clickhouse-tables-to-postgres", 80000), // warehouse migration
    ("json-schema-to-sql-ddl", 79000),        // schema/DDL
    ("transpose-csv", 78000),                 // tabular reshape
    ("avro-to-sql-ddl", 77000),               // streaming/schema
    ("snowflake-tables-to-postgres", 76000),  // warehouse migration
    ("env-to-k8s-secret", 75000),             // k8s/infra
    ("jsonl-to-json", 74000),                 // format interchange
    ("dedupe-records", 73000),                // data cleaning
    ("csv-to-sql", 72000),                    // tabular -> SQL
    ("duckdb-to-snowflake", 71000),           // warehouse migration
    ("fixed-width-to-csv", 70000),            // ingest -> tabular
    ("mssql-to-postgres", 69000),             // db migration
    ("avro-to-json-schema", 68000),           // streaming/schema
    ("fill-down-csv", 67000),                 // tabular reshape
    ("logfmt-to-json", 66000),                // log/format interchange
    ("bigquery-tables-to-redshift", 65000),   // warehouse migration
    ("jsonl-to-yaml", 64000),                 // format interchange
    ("k8s-apiversion-bump", 63000),           // k8s/infra
    ("csv-to-insert-template", 62000),        // tabular -> SQL
    ("duckdb-tables-to-bigquery", 61000),     // warehouse migration
    // Everyday whitespace / EOL / list hygiene: the most-reached-for cases.
    ("strip-trailing-whitespace", 48213),
    ("crlf-to-lf", 29817),
    ("repo-tidy", 880),
    ("sort-dedupe", 25446),
    ("comment-out", 17204),
    ("tabs-to-spaces", 20307),
    ("dedupe-keep-order", 590),
    ("tidy-list", 560),
    ("md-bulletize", 32659),
    // Redaction / encoding / escaping / paste / terminal output.
    ("redact-secrets", 41327),
    ("redact-pii", 700),
    ("redact-emails", 690),
    ("redact-ipv4", 680),
    ("redact-phone-numbers", 675),
    ("redact-json-fields", 685),
    ("strip-notebook-outputs", 440),
    ("sharegpt-to-openai", 436),
    ("openai-to-sharegpt", 434),
    ("shift-dates", 400),
    ("license-header-apply", 408),
    ("extract-json-from-llm", 445),
    ("rag-chunk", 420),
    ("hl7-to-csv", 330),
    ("edi-x12-to-csv", 328),
    ("redact-uuids", 670),
    ("redact-jwt", 665),
    ("mask-credit-cards", 660),
    ("redact-mac-addresses", 650),
    ("paste-cleanup", 660),
    ("emdash-cleanup", 640),
    ("dash-cleanup", 635),
    ("ai-glyph-cleanup", 630),
    ("emdash-audit", 620),
    ("emdash-review-list", 615),
    ("no-dash-guard", 610),
    ("base64-encode", 23612),
    ("url-encode", 596),
    ("url-decode", 594),
    ("html-unescape", 498),
    ("csv-to-json", 44118),
    ("json-to-csv", 565),
    ("yaml-to-json", 555),
    ("json-to-yaml", 545),
    ("base64-decode", 540),
    ("json-to-jsonl", 530),
    ("jsonl-to-json", 525),
    ("toml-to-json", 515),
    ("claude-transcript-to-text", 510),
    ("json-to-toml", 505),
    ("html-escape", 500),
    ("strip-markdown", 495),
    ("env-to-json", 488),
    ("json-to-env", 485),
    ("json-escape", 480),
    ("csv-to-markdown", 475),
    ("xml-to-json", 468),
    ("json-to-xml", 465),
    ("querystring-to-json", 452),
    ("json-to-querystring", 448),
    ("strip-ansi", 460),
    ("markdown-table-to-csv", 445),
    ("csv-to-yaml", 442),
    ("yaml-to-csv", 438),
    ("logfmt-to-json", 430),
    ("json-to-logfmt", 420),
    ("csv-to-sql", 27503),
    ("csv-to-html", 472),
    ("html-table-to-csv", 444),
    ("access-log-to-csv", 426),
    ("env-to-compose", 414),
    ("env-to-k8s-configmap", 412),
    ("properties-to-json", 408),
    ("commonjs-to-esm", 380),
    ("esm-to-commonjs", 378),
    ("remove-console-logs", 376),
    ("quote-lines", 360),
    ("join-lines-comma", 362),
    ("bump-copyright-year", 372),
    ("stacktrace-sanitize", 370),
    ("properties-to-yaml", 406),
    ("fixed-width-to-csv", 350),
    ("ini-to-toml", 410),
    ("toml-to-ini", 400),
    ("yaml-to-toml", 452),
    ("toml-to-yaml", 450),
    ("ini-to-json", 448),
    ("json-to-ini", 446),
    // Zero-decision list/format helpers (sort, canonicalize, escape, paste).
    ("sort-ip-addresses", 392),
    ("sort-versions", 388),
    ("normalize-url-list", 384),
    ("shell-quote-list", 378),
    ("sort-imports", 372),
    ("normalize-decimals", 366),
    // Column cleanup: mixed human formats to one canonical shape (flag what is ambiguous).
    ("normalize-dates-to-iso", 371),
    ("normalize-phone-numbers", 369),
    ("split-name-column", 367),
    ("paste-column", 362),
    // Table reshaping (CSV/TSV).
    ("transpose-csv", 360),
    ("unpivot-csv", 356),
    ("pivot-csv", 35461),
    // Two-input compare / reconcile / lookup (primary text vs a reference source).
    ("new-lines-only", 358),
    ("allowlist-filter", 352),
    ("list-diff", 348),
    ("reconcile-exports", 344),
    ("lookup-enrich-by-id", 356),
    // Pseudonymization / privacy.
    ("pseudonymize-id-column", 368),
    // Time / template / codemod / escaping (the self-contained-action unlocks).
    ("log-epoch-to-iso", 364),
    ("csv-to-insert-template", 358),
    ("comments-slashes-to-hash", 346),
    ("lines-to-json-strings", 342),
    // Docs / prose / interchange helpers.
    ("srt-to-vtt", 350),
    ("vtt-to-srt", 349),
    ("shift-subtitle-timing", 348),
    ("one-sentence-per-line", 344),
    ("markdown-demote-headings", 336),
    ("markdown-promote-headings", 334),
    ("vcard-to-csv", 348),
    ("ical-to-csv", 346),
    ("csv-to-tsv", 470),
    ("tsv-to-csv", 468),
    ("strip-yaml-frontmatter", 360),
    ("strip-html-comments", 356),
    ("strip-sql-comments", 352),
    ("dotenv-to-exports", 358),
    ("ipynb-to-python", 372),
    ("markdown-toc", 362),
    ("markdown-uncheck-tasks", 340),
    ("camel-to-snake", 366),
    ("kebab-to-snake", 360),
    ("snake-to-kebab", 358),
    ("git-diff-added-lines", 354),
    ("domains-to-hosts-block", 344),
    ("bibtex-to-ris", 330),
    ("tsv-to-markdown", 456),
    ("markdown-table-to-tsv", 448),
    ("markdown-table-to-yaml", 438),
    ("fill-down-csv", 372),
    ("remove-trailing-commas", 364),
    ("strip-markdown-links", 372),
    ("jsonl-to-csv", 486),
    ("package-lock-to-inventory", 441),
    ("clf-access-log-to-json", 430),
    ("mbox-to-csv", 418),
    ("properties-to-env", 356),
    ("env-to-properties", 354),
    ("lines-to-checklist", 368),
    ("list-to-json-array", 374),
    ("sql-values-from-list", 350),
    ("quote-as-blockquote", 360),
    ("strip-line-numbers", 362),
    ("add-line-numbers", 360),
    ("unwrap-code-fence", 368),
    ("markdown-strip-emphasis", 366),
    ("slugify-lines", 370),
    ("tsv-to-json", 462),
    ("markdown-table-to-json", 440),
    ("straighten-typography", 366),
    ("number-list", 364),
    ("hosts-to-domains", 342),
    ("format-json", 38704),
    ("minify-json", 468),
    ("strip-hash-comments", 358),
    ("wrap-in-code-block", 372),
    ("json-keys", 380),
    ("csv-first-column", 356),
    ("csv-swap-first-two-columns", 352),
    ("normalize-spaces", 374),
    ("url-encode", 496),
    ("url-decode", 494),
    ("html-unescape", 476),
    ("fix-mojibake", 430),
    ("remove-accents", 420),
    ("strip-repl-prompts", 400),
    ("iso-to-epoch", 366),
    ("env-to-example", 410),
    ("strip-email-quotes", 396),
    ("strip-zero-width", 404),
    ("k8s-apiversion-bump", 388),
    ("airflow1-to-2-imports", 372),
    ("ris-to-bibtex", 332),
    ("align-columns", 398),
    ("csv-to-vcard", 350),
    ("csv-to-ical", 348),
    ("normalize-log-level", 386),
    ("email-headers-to-csv", 378),
    ("csv-to-xml", 372),
    ("xml-to-csv", 371),
    ("junit-xml-to-csv", 369),
    ("har-to-csv", 367),
    ("openapi-paths-to-csv", 366),
    ("sarif-to-csv", 364),
    ("k8s-manifests-to-csv", 362),
    ("git-log-to-csv", 365),
    ("ldapsearch-to-csv", 344),
    ("unwrap-paragraphs", 388),
    ("csv-to-sql-update", 358),
    ("strip-jsonc-comments", 392),
    ("markdown-to-slack", 384),
    ("count-unique", 376),
    ("json5-to-json", 466),
    ("jsonl-to-yaml", 460),
    ("yaml-to-jsonl", 458),
    ("reverse-lines", 382),
    ("sort-lines", 470),
    ("remove-blank-lines", 480),
    ("squeeze-blank-lines", 466),
    ("spaces-to-tabs", 430),
    // Warehouse-SQL dialect converters (the modern-data-stack pillar; Snowflake hub).
    ("oracle-tables-to-postgres", 340),
    ("sqlserver-tables-to-postgres", 339),
    ("bigquery-tables-to-postgres", 339),
    ("redshift-tables-to-postgres", 338),
    ("databricks-tables-to-postgres", 337),
    ("snowflake-tables-to-mysql", 336),
    ("postgres-tables-to-sqlite", 334),
    ("bigquery-tables-to-mysql", 333),
    ("postgres-tables-to-mysql", 338),
    ("sqlserver-tables-to-mysql", 337),
    ("oracle-tables-to-mysql", 335),
    ("sqlite-tables-to-postgres", 335),
    ("mysql-tables-to-sqlite", 333),
    ("snowflake-tables-to-bigquery", 336),
    ("bigquery-tables-to-snowflake", 334),
    ("redshift-tables-to-snowflake", 332),
    ("postgres-tables-to-snowflake", 330),
    ("snowflake-tables-to-postgres", 21938),
    ("databricks-tables-to-snowflake", 326),
    ("snowflake-tables-to-databricks", 324),
    ("snowflake-tables-to-duckdb", 344),
    ("duckdb-tables-to-snowflake", 342),
    ("bigquery-tables-to-duckdb", 340),
    ("duckdb-tables-to-bigquery", 338),
    ("snowflake-tables-to-trino", 334),
    ("trino-tables-to-snowflake", 332),
    ("redshift-tables-to-bigquery", 330),
    ("bigquery-tables-to-redshift", 328),
    ("dbt-model-from-raw-sql", 320),
    ("postgres-tables-to-bigquery", 335),
    ("mysql-tables-to-bigquery", 333),
    ("mysql-tables-to-snowflake", 336),
    // 2026-08-29 batch: legacy-DB -> modern-analytics migrations, grounded on real
    // GET_DDL / mssql-scripter exports and execution-validated.
    ("oracle-tables-to-snowflake", 340),
    ("sqlserver-tables-to-snowflake", 338),
    ("oracle-tables-to-duckdb", 325),
    ("sqlserver-tables-to-duckdb", 326),
    // ClickHouse migration paths (analytics-DB hub; both ingest and export), grounded on
    // real SHOW CREATE / pg_dump / SHOW CREATE exports and execution-validated.
    ("mysql-tables-to-clickhouse", 330),
    ("postgres-tables-to-clickhouse", 329),
    ("clickhouse-tables-to-bigquery", 329),
    ("clickhouse-tables-to-snowflake", 328),
    ("clickhouse-tables-to-postgres", 327),
    // Non-table object grids: views and stored procedures (SSMS-grounded,
    // compile-validated). Basic bodies translate; the rest is flagged.
    ("sqlserver-views-to-postgres", 305),
    ("sqlserver-procs-to-postgres", 300),
    // Code generation: DB artifact -> host-language code (new axis). Grounded on real
    // schemas and compile-validated (dotnet for C#, tsc --strict for TypeScript). POCO
    // is the stack-neutral foundation.
    ("postgres-tables-to-csharp", 295),
    ("sqlserver-tables-to-csharp", 294),
    ("mysql-tables-to-csharp", 293),
    ("postgres-tables-to-python-dataclass", 292),
    ("sqlserver-tables-to-python-dataclass", 285),
    ("mysql-tables-to-python-dataclass", 284),
    ("postgres-tables-to-typescript", 291),
    ("sqlserver-tables-to-typescript", 290),
    ("mysql-tables-to-typescript", 289),
    ("postgres-tables-to-go", 288),
    ("sqlserver-tables-to-go", 287),
    ("mysql-tables-to-go", 286),
    ("snowflake-tables-to-csharp", 282),
    ("snowflake-tables-to-python-dataclass", 281),
    ("snowflake-tables-to-typescript", 280),
    ("snowflake-tables-to-go", 279),
    ("bigquery-tables-to-csharp", 275),
    ("bigquery-tables-to-python-dataclass", 274),
    ("bigquery-tables-to-typescript", 273),
    ("bigquery-tables-to-go", 272),
    // Domain-specific conversions and composable helpers (nicher).
    ("normalize-log-for-diff", 322),
    ("sql-in-clause", 300),
    ("sql-column-lineage", 296),
    ("oracle-types-to-postgres", 292),
    ("sql-canonicalize-identifiers", 288),
    ("sql-qualify-columns", 284),
    ("MySQL-To-PGSQL", 240),
    ("mysql-strip-dump-noise", 120),
    ("mssql-strip-server-noise", 110),
    ("strip-clickhouse-showcreate-noise", 105),
    // Pure composition fragments (plumbing): lowest, they are reached indirectly.
    ("sql-finalize-whitespace", 90),
    ("sql-requote-quote-to-backtick", 88),
    ("sql-requote-backtick-to-quote", 86),
    ("sql-colon-cast-to-cast", 84),
    // 2026-08-26 batch: warehouse-matrix holes (Snowflake/Redshift/Databricks
    // cross-pairs), schema translation, IaC + LLM/dataset formats, k8s Secret,
    // and per-source paste-cleanup packs. Values track each recipe's cluster.
    ("snowflake-tables-to-redshift", 331),
    ("redshift-tables-to-databricks", 327),
    ("redshift-tables-to-mysql", 329),
    ("databricks-tables-to-bigquery", 329),
    ("databricks-tables-to-mysql", 327),
    ("databricks-tables-to-redshift", 325),
    ("json-schema-to-sql-ddl", 356),
    ("sql-ddl-to-json-schema", 15683),
    ("avro-to-json-schema", 346),
    ("protobuf-to-json-schema", 344),
    ("avro-to-sql-ddl", 346),
    ("terraform-to-opentofu", 386),
    ("terraform-plan-to-changes", 372),
    ("anthropic-to-openai-messages", 446),
    ("openai-to-anthropic-messages", 444),
    ("alpaca-to-openai-messages", 442),
    ("alpaca-to-sharegpt", 440),
    ("env-to-k8s-secret", 411),
    ("paste-cleanup-excel", 530),
    ("paste-cleanup-slack", 520),
    ("paste-cleanup-web", 525),
    ("paste-cleanup-pdf", 500),
    // Demos for the new numbers/sort-key/encode actions.
    ("sort-csv-by-column", 390),
    ("zero-pad-numbers", 386),
    ("rot13", 372),
    // Enterprise segment/positional formats (best-effort, lossy dumps; niche).
    ("edifact-to-csv", 326),
    ("swift-mt-to-csv", 324),
    ("idoc-to-csv", 322),
    ("cobol-copybook-to-fields", 320),
    // Demo for the per-match $uuid replacement token.
    ("assign-uuid-per-line", 358),
    // Demos for the block / dedupe-by-key / positional-replace actions.
    ("dedupe-by-first-field", 388),
    ("sort-records", 384),
    ("last-separator-to-and", 360),
    // Action-coverage sweep: one recipe per engine action not previously exercised
    // by a factory recipe (mostly thin single-action utilities).
    // Case.
    ("snake-to-camel", 470),
    ("snake-to-pascal", 468),
    ("sentence-case-lines", 360),
    ("invert-case", 300),
    ("mocking-case", 190),
    ("case-preserving-rename", 430),
    // Escape / encode.
    ("escape-csv-field", 410),
    ("xml-escape", 430),
    ("regex-escape", 420),
    ("escape-c-string", 350),
    ("text-to-hex", 300),
    ("hex-to-text", 300),
    ("json-string-unescape", 400),
    // Text / numbers.
    ("html-to-text", 460),
    ("keep-digits-only", 420),
    ("strip-control-chars", 380),
    ("bump-numbers", 340),
    ("round-decimals", 380),
    ("scale-numbers", 350),
    // Whitespace / affix.
    ("dedent-block", 420),
    ("indent-block", 430),
    ("outdent-one-level", 360),
    ("trim-leading-space", 380),
    ("unwrap-to-one-line", 400),
    ("join-lines-with-space", 340),
    ("append-suffix", 380),
    ("lines-to-sql-in-list", 410),
    // EOL.
    ("lf-to-crlf", 560),
    ("eol-to-cr-mac", 240),
    // Line ops.
    ("dedupe-records", 18729),
    ("find-duplicate-lines", 400),
    ("collapse-adjacent-dupes", 430),
    ("shuffle-lines", 320),
    ("split-on-comma", 420),
    ("insert-after-match", 380),
    // Replace.
    ("map-status-codes", 420),
    ("replace-first-only", 380),
    ("replace-nth-occurrence", 350),
    ("notepad-escapes-replace", 340),
    // Data / JSON.
    ("explode-list-column", 400),
    ("mask-field", 460),
    ("fill-from-list", 380),
    ("remove-json-key", 440),
    // Detect (flagging) / compose.
    ("scan-for-pii", 450),
    ("scan-for-secrets", 450),
    ("dockerfile-flag-unpinned", 448),
    ("github-actions-flag-unpinned", 447),
    ("annotate-blocks", 300),
    // Grounded warehouse/dialect converters + strip fragments + query converters (2026-08-28).
    ("bigquery-query-to-duckdb", 345),
    ("databricks-tables-to-duckdb", 305),
    ("mysql-tables-to-duckdb", 305),
    ("mysql-tables-to-postgres", 305),
    ("postgres-tables-to-duckdb", 305),
    ("redshift-tables-to-duckdb", 305),
    ("snowflake-query-to-duckdb", 345),
    ("snowflake-query-to-postgres", 345),
    ("strip-databricks-export-noise", 250),
    ("strip-mysql-ddl-noise", 250),
    ("strip-pgdump-noise", 250),
    ("strip-snowflake-getddl-noise", 250),
    ("strip-oracle-getddl-noise", 250),
    ("po-to-csv", 250),
    ("csv-to-po", 250),
    ("sqlserver-tables-to-bigquery", 250),
    ("oracle-tables-to-bigquery", 250),
    ("wrap-email-quotes", 300),
    ("unwrap-email-quotes", 300),
];

/// The seeded baseline for a factory recipe, if any (None for user-authored ones).
fn factory_baseline(name: &str) -> Option<u64> {
    FACTORY_BASELINE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| *c)
}

/// Effective popularity = seeded baseline + any real catalog download_count. None
/// only when neither exists, so it stays omitted from JSON for uncounted recipes.
fn popularity(name: &str, catalog_count: Option<u64>) -> Option<u64> {
    match (factory_baseline(name), catalog_count) {
        (None, None) => None,
        (seed, real) => Some(seed.unwrap_or(0) + real.unwrap_or(0)),
    }
}

/// Merge the local library with the catalog (if reachable) into one view.
fn gather(root: &Path) -> Vec<Row> {
    let local = crate::market::scan(root);
    let installed = read_installed(root);
    let mut local_by_name: BTreeMap<String, &crate::market::Entry> = BTreeMap::new();
    for e in &local {
        local_by_name.entry(recipe_stem(&e.resolvable)).or_insert(e);
    }

    // The catalog is optional: list/search work offline against the local library.
    let catalog = market_base_url()
        .ok()
        .and_then(|b| ensure_index(root, &b, false).ok());

    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(idx) = &catalog {
        for e in &idx.entries {
            seen.insert(e.name.clone());
            let status = if local_by_name.contains_key(&e.name) {
                "installed"
            } else {
                "available"
            };
            rows.push(Row {
                name: e.name.clone(),
                // Catalog index entries carry only a description, no summary.
                summary: None,
                description: e.description.clone(),
                tags: e.tags.clone(),
                task_phrases: Vec::new(),
                category: e.category.clone(),
                featured: e.featured,
                version: e.version,
                status,
                download_count: popularity(&e.name, e.download_count),
            });
        }
    }
    for (name, e) in &local_by_name {
        if seen.contains(name) {
            continue;
        }
        let status = "installed";
        rows.push(Row {
            name: name.clone(),
            summary: e.summary.clone(),
            description: e.description.clone(),
            tags: e.tags.clone(),
            task_phrases: e.task_phrases.clone(),
            category: None,
            featured: false,
            version: installed.get(name).copied().unwrap_or(1),
            status,
            download_count: popularity(name, None),
        });
    }
    rows
}

/// Width the human browse blurb is clamped to (Studio-style terse row).
const BLURB_WIDTH: usize = 72;

/// The short human browse blurb for a row: its `summary` when present, else a
/// truncated first sentence of the long, agent-oriented `description`. Clamped to
/// [`BLURB_WIDTH`] chars with an ASCII `...` when cut. The AI (`--json`/MCP)
/// channel keeps the full `description` regardless; this is human display only.
fn short_blurb(summary: Option<&str>, description: Option<&str>) -> String {
    if let Some(s) = summary {
        let s = s.trim();
        if !s.is_empty() {
            return clamp_blurb(s);
        }
    }
    let d = description.unwrap_or("").trim();
    clamp_blurb(first_sentence(d))
}

/// The first sentence of `s`: text up to the first period that ends a sentence
/// (one followed by whitespace or end of text). A period mid-token (version
/// numbers, `e.g.`) does not cut. Returns the whole string when none is found.
fn first_sentence(s: &str) -> &str {
    let mut search_from = 0;
    while let Some(rel) = s[search_from..].find('.') {
        let abs = search_from + rel;
        match s[abs + 1..].chars().next() {
            None => return s[..abs].trim_end(),
            Some(c) if c.is_whitespace() => return s[..abs].trim_end(),
            _ => search_from = abs + 1,
        }
    }
    s
}

/// Trim `s` to [`BLURB_WIDTH`] chars, appending an ASCII `...` when it was cut.
fn clamp_blurb(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() <= BLURB_WIDTH {
        return s.to_string();
    }
    let head: String = s.chars().take(BLURB_WIDTH.saturating_sub(3)).collect();
    format!("{}...", head.trim_end())
}

/// Render the merged rows. `total` is the pre-cap match/library count so the
/// header can say "{shown} of {total}"; `rows` is already truncated to the cap.
fn print_rows(rows: &[Row], total: usize, json: bool) {
    if json {
        println!(
            "{}",
            serde_json::json!({ "results": rows, "count": rows.len() })
        );
        return;
    }
    // Studio-style count header before the rows.
    println!("{} of {} recipes", rows.len(), total);
    for r in rows {
        let mark = match r.status {
            "installed" => "✓",
            _ => " ",
        };
        let star = if r.featured { "★" } else { " " };
        let short = short_blurb(r.summary.as_deref(), r.description.as_deref());
        println!("{mark}{star} {:<28}  {short}", r.name);
    }
    println!(
        "({} shown; ✓ installed, ★ featured; `mog market install <name>`)",
        rows.len()
    );
}

pub fn list(root: Option<&Path>, json: bool) -> Result<i32> {
    let root = root.ok_or_else(|| anyhow!("no library root (set MOG_HOME or --mog-dir)"))?;
    let mut rows = gather(root);
    // The no-query browse default (spec section 9): featured first, then by
    // popularity (seeded baseline + any real download_count), then by name.
    rows.sort_by(|a, b| {
        b.featured
            .cmp(&a.featured)
            .then(
                b.download_count
                    .unwrap_or(0)
                    .cmp(&a.download_count.unwrap_or(0)),
            )
            .then(a.name.cmp(&b.name))
    });
    // No RESULT_CAP here: `list` is the browse-all view, so it returns the whole
    // library (search stays capped to top matches). A high safety bound only.
    let total = rows.len();
    rows.truncate(LIST_CAP);
    print_rows(&rows, total, json);
    Ok(0)
}

pub fn search(root: Option<&Path>, query: &str, json: bool) -> Result<i32> {
    let root = root.ok_or_else(|| anyhow!("no library root (set MOG_HOME or --mog-dir)"))?;
    let rows = gather(root);

    let tokens = crate::market::tokenize(query);
    let terms = crate::market::expand_query(&tokens);
    let mut scored: Vec<(i32, Row)> = rows
        .into_iter()
        .map(|r| {
            // Relevance is primary; a small featured nudge only breaks ties.
            let s = crate::market::score_fields(
                Some(&r.name),
                &r.tags,
                &r.task_phrases,
                r.description.as_deref(),
                &terms,
            );
            (s * 2 + i32::from(r.featured), r)
        })
        .filter(|(s, _)| *s > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));

    let total = scored.len();
    let rows: Vec<Row> = scored
        .into_iter()
        .take(RESULT_CAP)
        .map(|(_, r)| r)
        .collect();
    print_rows(&rows, total, json);
    Ok(0)
}

pub fn show(root: Option<&Path>, name: &str, no_content: bool, json: bool) -> Result<i32> {
    let root = root.ok_or_else(|| anyhow!("no library root (set MOG_HOME or --mog-dir)"))?;
    // Local-first: if the name resolves to an installed recipe, show the full local
    // detail (content + fixture). This is a targeted single-recipe resolve (reads one
    // file, not the whole ~N-recipe store), matching how `-m` finds the same script.
    // Resolve without a base dir so a bare market name never picks up a stray local
    // `.mog` in the cwd.
    if crate::library::resolve_script(Path::new(name), None, Some(root)).is_ok() {
        return crate::market::show(Some(root), Path::new(name), no_content, json);
    }
    // Otherwise the catalog entry, if a marketplace is configured and reachable.
    let Ok(base) = market_base_url() else {
        bail!("no recipe '{name}' installed (and no marketplace configured to search)");
    };
    let index = ensure_index(root, &base, false)?;
    let entry = index
        .get(name)
        .ok_or_else(|| anyhow!("no recipe '{name}' installed or in the marketplace"))?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "entry": entry, "installed": false })
        );
    } else {
        println!(
            "{}  v{}  (available, not installed)",
            entry.name, entry.version
        );
        if let Some(d) = &entry.description {
            println!("  {d}");
        }
        if let Some(c) = &entry.category {
            println!("  category: {c}");
        }
        if !entry.tags.is_empty() {
            println!("  tags: {}", entry.tags.join(", "));
        }
        if entry.featured {
            println!("  featured");
        }
        println!("  sha256: {}", entry.sha256);
        println!("  install with `mog market install {name}`");
    }
    Ok(0)
}
