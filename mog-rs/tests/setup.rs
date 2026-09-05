//! End-to-end tests for `mog setup`. Each test drives a temp `MOG_HOME` via
//! `--mog-dir`, so nothing touches the repo or a real user library. MCP
//! registration is exercised only in `--no-mcp` / `--print` form, so the tests
//! never require `claude` or `node` to exist.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn mog() -> Command {
    Command::cargo_bin("mog").expect("mog binary builds")
}

/// A fresh, empty temp dir to use as the library root (setup creates the
/// sources itself).
fn home() -> TempDir {
    tempfile::tempdir().unwrap()
}

fn exists(root: &Path, rel: &str) -> bool {
    root.join(rel).exists()
}

#[test]
fn setup_writes_recipes_into_market_dir() {
    let h = home();
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();

    // No old-layout source subfolders at the outer root.
    assert!(!exists(h.path(), "user"), "no user/ source folder");
    assert!(
        !exists(h.path(), "community"),
        "no community/ source folder"
    );
    assert!(!exists(h.path(), "factory"), "no factory/ source folder");

    // The two mogs/ subdirs exist.
    assert!(exists(h.path(), "mogs/market"), "mogs/market created");
    assert!(exists(h.path(), "mogs/user"), "mogs/user created");

    // A representative recipe and BOTH its fixtures landed under mogs/market
    // (one directory per recipe: <name>/<name>.mog + <name>/tests/{input,expected}).
    assert!(exists(h.path(), "mogs/market/tidy-list/tidy-list.mog"));
    assert!(exists(h.path(), "mogs/market/tidy-list/tests/input.txt"));
    assert!(exists(h.path(), "mogs/market/tidy-list/tests/expected.txt"));
    // And the SQL converters + their fixtures.
    assert!(exists(
        h.path(),
        "mogs/market/sqlserver-tables-to-postgres/sqlserver-tables-to-postgres.mog"
    ));
    assert!(exists(
        h.path(),
        "mogs/market/sqlserver-tables-to-postgres/tests/input.sql"
    ));
    assert!(exists(
        h.path(),
        "mogs/market/sqlserver-tables-to-postgres/tests/expected.sql"
    ));
}

#[test]
fn factory_script_is_resolvable_and_listed() {
    let h = home();
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();

    // `mog market list` surfaces the installed factory recipe.
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "market", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tidy-list"));

    // Its fixture passes immediately, from the library.
    let mog_path = h
        .path()
        .join("mogs")
        .join("market")
        .join("tidy-list")
        .join("tidy-list.mog");
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "--test",
            mog_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

#[test]
fn factory_script_resolves_by_bare_name() {
    let h = home();
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();

    // `-m tidy-list.mog` resolves through the flat library and runs.
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "-m",
            "tidy-list.mog",
        ])
        .write_stdin("  banana\napple\nbanana\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Apple"))
        .stdout(predicate::str::contains("Banana"));
}

#[test]
fn print_writes_nothing() {
    let h = home();
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "setup",
            "--print",
            "--no-mcp",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry run"));

    // Nothing was created on disk.
    assert!(!exists(h.path(), "mogs/market/tidy-list/tidy-list.mog"));
}

#[test]
fn rerun_is_idempotent_and_leaves_user_untouched() {
    let h = home();
    // First run bootstraps the store.
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();
    assert!(exists(h.path(), "mogs/market/tidy-list/tidy-list.mog"));

    // A user recipe under mogs/user must survive a re-run untouched.
    let user_recipe = h.path().join("mogs").join("user").join("mine.mog");
    fs::write(&user_recipe, r#"{ "steps": [] }"#).unwrap();

    // Second run: idempotent; the market set is rewritten, the user dir is left intact.
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();

    assert!(exists(h.path(), "mogs/market/tidy-list/tidy-list.mog"));
    assert!(user_recipe.exists(), "user recipe left untouched");
}

#[test]
fn mcp_prints_in_engine_registration_command() {
    let h = home();
    // --print guarantees the print path (never runs `claude`, even on a box that
    // has it). The registration now points at the in-engine server: `-- <exe> mcp`
    // with a MOG_HOME env, and no Node entry point or MOG_BIN.
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "setup",
            "--print",
            "--no-store",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude mcp add -s user mog"))
        .stdout(predicate::str::contains(" mcp"))
        .stdout(predicate::str::contains("MOG_HOME="))
        // The retired Node wiring must be gone.
        .stdout(predicate::str::contains("node ").not())
        .stdout(predicate::str::contains("MOG_BIN").not());
}

#[test]
fn json_summary_reports_actions() {
    let h = home();
    let assert = mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "setup",
            "--no-mcp",
            "--json",
        ])
        .assert()
        .success();
    let out = assert.get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("stdout is valid JSON");
    assert_eq!(v["store"]["wrote"], serde_json::json!(true));
    assert!(
        v["store"]["factory_files"].as_array().unwrap().len() >= 5,
        "at least the five factory scripts plus fixtures are reported"
    );
    assert_eq!(v["mcp"]["skipped"], serde_json::json!(true));
    assert!(v["caveat"].as_str().unwrap().contains("NEW session"));
}
