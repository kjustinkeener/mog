//! End-to-end coverage for `mog update`: the recipe sync (content-hash diff) and
//! the engine self-update check, both driven against a local directory acting as
//! the registry (the same path a real GitHub-Release mirror would occupy). The
//! signing key is generated per test and injected via `MOG_MARKET_PUBKEY`; env is
//! process-global so the tests in this file serialize on a mutex.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use mog::market_index::{self, build_index};
use mog::selfupdate::{EngineManifest, EnginePlatform};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn mog_json(name: &str, desc: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "description": "{desc}",
  "tags": ["test"],
  "steps": [
    {{ "description": "trim.", "action": "trim_trailing_whitespace" }}
  ]
}}
"#
    )
}

/// Stand up a registry dir with the given `(stem, description)` recipes, build and
/// sign `index.json`, and return `(registry_path, pubkey_b64)`.
fn build_registry(registry: &Path, recipes: &[(&str, &str)]) -> String {
    let mog_root = registry.join("recipes");
    for (stem, desc) in recipes {
        let dir = mog_root.join(stem);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{stem}.mog")), mog_json(stem, desc)).unwrap();
    }
    // path_base = registry so entry.path is "recipes/<stem>/<stem>.mog".
    let index = build_index(&mog_root, registry, &BTreeMap::new(), Some("test".into())).unwrap();
    let bytes = index.to_json_bytes().unwrap();
    let sk = market_index::generate_keypair();
    fs::write(registry.join("index.json"), &bytes).unwrap();
    fs::write(
        registry.join("index.json.sig"),
        market_index::sign(&sk, &bytes),
    )
    .unwrap();
    market_index::verifying_key_to_b64(&sk.verifying_key())
}

#[test]
fn sync_adds_updates_and_is_idempotent() {
    let _g = env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let registry = tmp.path().join("registry");
    let lib = tmp.path().join("lib");
    fs::create_dir_all(&lib).unwrap();

    let pubkey = build_registry(&registry, &[("alpha", "Alpha v1"), ("beta", "Beta v1")]);
    std::env::set_var("MOG_MARKET_PUBKEY", &pubkey);
    let base = registry.to_string_lossy().to_string();

    // First sync: both recipes are new.
    let plan = mog::market_client::sync(&lib, &base, false, false).unwrap();
    assert_eq!(plan.added, vec!["alpha", "beta"]);
    assert!(plan.changed.is_empty());
    assert!(lib.join("mogs/market/alpha/alpha.mog").exists());
    assert!(lib.join("mogs/market/beta/beta.mog").exists());

    // Re-sync with an unchanged registry: no-op.
    let plan = mog::market_client::sync(&lib, &base, false, false).unwrap();
    assert!(plan.is_empty(), "expected no changes, got {plan:?}");

    // Change one recipe's body and re-sign: sync detects it by hash and rewrites.
    let registry2 = tmp.path().join("registry2");
    let pubkey2 = build_registry(
        &registry2,
        &[("alpha", "Alpha v2 CHANGED"), ("beta", "Beta v1")],
    );
    std::env::set_var("MOG_MARKET_PUBKEY", &pubkey2);
    let base2 = registry2.to_string_lossy().to_string();
    // Bust the verified index cache written under the shared lib root.
    let _ = fs::remove_file(lib.join(".cache/index.json"));
    let plan = mog::market_client::sync(&lib, &base2, false, false).unwrap();
    assert_eq!(plan.changed, vec!["alpha"]);
    assert!(plan.added.is_empty());
    let body = fs::read_to_string(lib.join("mogs/market/alpha/alpha.mog")).unwrap();
    assert!(body.contains("Alpha v2 CHANGED"));
}

#[test]
fn sync_check_writes_nothing() {
    let _g = env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let registry = tmp.path().join("registry");
    let lib = tmp.path().join("lib");
    fs::create_dir_all(&lib).unwrap();

    let pubkey = build_registry(&registry, &[("alpha", "Alpha")]);
    std::env::set_var("MOG_MARKET_PUBKEY", &pubkey);
    let base = registry.to_string_lossy().to_string();

    let plan = mog::market_client::sync(&lib, &base, true, false).unwrap();
    assert_eq!(plan.added, vec!["alpha"]);
    assert!(
        !lib.join("mogs/market/alpha/alpha.mog").exists(),
        "--check must not write recipes"
    );
}

/// Regression: two recipes each carry a `tests/input.txt` fixture (identical key,
/// different content). The install layout must give every recipe its own directory
/// so their goldens never clobber under a shared `community/tests/`.
#[test]
fn sync_keeps_per_mog_fixtures_from_colliding() {
    let _g = env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let registry = tmp.path().join("registry");
    let lib = tmp.path().join("lib");
    fs::create_dir_all(&lib).unwrap();

    // Build a registry by hand so each recipe gets a sibling `tests/input.txt`
    // with distinct bytes (the shared key that used to collide).
    let mog_root = registry.join("recipes");
    for (stem, fixture) in [("alpha", "ALPHA INPUT\n"), ("beta", "BETA INPUT\n")] {
        let dir = mog_root.join(stem);
        fs::create_dir_all(dir.join("tests")).unwrap();
        fs::write(dir.join(format!("{stem}.mog")), mog_json(stem, stem)).unwrap();
        fs::write(dir.join("tests/input.txt"), fixture).unwrap();
        fs::write(dir.join("tests/expected.txt"), fixture).unwrap();
    }
    let index = build_index(&mog_root, &registry, &BTreeMap::new(), Some("test".into())).unwrap();
    let bytes = index.to_json_bytes().unwrap();
    let sk = market_index::generate_keypair();
    fs::write(registry.join("index.json"), &bytes).unwrap();
    fs::write(
        registry.join("index.json.sig"),
        market_index::sign(&sk, &bytes),
    )
    .unwrap();
    std::env::set_var(
        "MOG_MARKET_PUBKEY",
        market_index::verifying_key_to_b64(&sk.verifying_key()),
    );
    let base = registry.to_string_lossy().to_string();

    let plan = mog::market_client::sync(&lib, &base, false, false).unwrap();
    assert_eq!(plan.added, vec!["alpha", "beta"]);

    // Both goldens must survive, each with ITS OWN content (no clobber).
    let alpha = fs::read_to_string(lib.join("mogs/market/alpha/tests/input.txt")).unwrap();
    let beta = fs::read_to_string(lib.join("mogs/market/beta/tests/input.txt")).unwrap();
    assert_eq!(alpha, "ALPHA INPUT\n");
    assert_eq!(beta, "BETA INPUT\n");
}

#[test]
fn engine_check_compares_running_binary_by_hash() {
    let _g = env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let registry = tmp.path().join("registry");
    fs::create_dir_all(&registry).unwrap();

    // Sign an engine manifest whose entry for THIS platform points at a binary
    // whose hash differs from the running test binary -> update available.
    let sk = market_index::generate_keypair();
    std::env::set_var(
        "MOG_MARKET_PUBKEY",
        market_index::verifying_key_to_b64(&sk.verifying_key()),
    );
    let mut platforms = BTreeMap::new();
    platforms.insert(
        mog::selfupdate::platform_key(),
        EnginePlatform {
            path: "bin/mog".into(),
            sha256: market_index::sha256_hex(b"a different build"),
        },
    );
    let manifest = EngineManifest {
        version: "9.9.9".into(),
        platforms,
    };
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(registry.join("engine.json"), &bytes).unwrap();
    fs::write(
        registry.join("engine.json.sig"),
        market_index::sign(&sk, &bytes),
    )
    .unwrap();

    let base = registry.to_string_lossy().to_string();
    let status = mog::selfupdate::check(&base).unwrap();
    assert!(status.update_available);
    assert_eq!(status.version, "9.9.9");

    // A manifest matching the running binary's own hash -> no update.
    let exe = std::env::current_exe().unwrap();
    let cur = market_index::sha256_hex(&fs::read(&exe).unwrap());
    let mut platforms = BTreeMap::new();
    platforms.insert(
        mog::selfupdate::platform_key(),
        EnginePlatform {
            path: "bin/mog".into(),
            sha256: cur,
        },
    );
    let manifest = EngineManifest {
        version: "0.1.0".into(),
        platforms,
    };
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(registry.join("engine.json"), &bytes).unwrap();
    fs::write(
        registry.join("engine.json.sig"),
        market_index::sign(&sk, &bytes),
    )
    .unwrap();
    let status = mog::selfupdate::check(&base).unwrap();
    assert!(!status.update_available);
}
