//! End-to-end tests for the unified `mog market` surface (list / search / show /
//! add / rm / bless). The library on disk is the already-installed slice of the
//! marketplace, so list/search/show are local-first (they work offline). Each
//! test drives a temp MOG_HOME, so nothing touches the repo or a real library.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use mog::market_index::{self, build_index};
use predicates::prelude::*;
use tempfile::TempDir;

/// Stand up a signed local registry dir with the given recipe stems and return
/// `(base_url, pubkey_b64)`. Mirrors the helper in `update.rs`; used to drive the
/// remote-catalog path offline. The child command must receive both `MOG_MARKET_URL`
/// (the returned base) and `MOG_MARKET_PUBKEY` (the returned key) via `.env()`.
fn build_registry(registry: &Path, stems: &[&str]) -> (String, String) {
    let recipe_root = registry.join("recipes");
    for stem in stems {
        let dir = recipe_root.join(stem);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{stem}.mog")),
            format!(r#"{{ "name": "{stem}", "steps": [ {{ "action": "to_upper" }} ] }}"#),
        )
        .unwrap();
    }
    let index = build_index(
        &recipe_root,
        registry,
        &BTreeMap::new(),
        Some("test".into()),
    )
    .unwrap();
    let bytes = index.to_json_bytes().unwrap();
    let sk = market_index::generate_keypair();
    fs::write(registry.join("index.json"), &bytes).unwrap();
    fs::write(
        registry.join("index.json.sig"),
        market_index::sign(&sk, &bytes),
    )
    .unwrap();
    (
        registry.to_string_lossy().to_string(),
        market_index::verifying_key_to_b64(&sk.verifying_key()),
    )
}

/// A .mog that uppercases the whole file.
const UPPER_MOG: &str = r#"{ "steps": [ { "action": "to_upper" } ] }"#;

fn mog() -> Command {
    let mut c = Command::cargo_bin("mog").expect("mog binary builds");
    // These tests exercise the LOCAL library; keep them offline and deterministic.
    c.env_remove("MOG_MARKET_URL")
        .env_remove("MOG_MARKET_PUBKEY");
    c
}

/// A fresh, empty temp MOG_HOME (flat layout: no source subfolders).
fn home() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// The managed recipe dir under a store `home`: `<home>/mogs/market`. The engine
/// scans, resolves, and manages recipes here.
fn market(home: &Path) -> PathBuf {
    home.join("mogs").join("market")
}

/// Write a recipe (one directory per recipe) into the managed recipe dir of
/// `home`: `mogs/market/<stem>/<stem>.mog` plus `<stem>/tests/input.txt` and
/// `expected.txt`.
fn put_script(home: &Path, stem: &str, body: &str, input: &str, expected: &str) {
    let recipe_dir = market(home).join(stem);
    fs::create_dir_all(recipe_dir.join("tests")).unwrap();
    fs::write(recipe_dir.join(format!("{stem}.mog")), body).unwrap();
    fs::write(recipe_dir.join("tests/input.txt"), input).unwrap();
    fs::write(recipe_dir.join("tests/expected.txt"), expected).unwrap();
}

/// Write a loose recipe directory in a scratch dir (an `add` source):
/// `<stem>/<stem>.mog` + `<stem>/tests/{input,expected}.txt`.
fn loose_script(dir: &Path, stem: &str, body: &str, input: &str, expected: &str) -> PathBuf {
    let recipe_dir = dir.join(stem);
    fs::create_dir_all(recipe_dir.join("tests")).unwrap();
    fs::write(recipe_dir.join(format!("{stem}.mog")), body).unwrap();
    fs::write(recipe_dir.join("tests/input.txt"), input).unwrap();
    fs::write(recipe_dir.join("tests/expected.txt"), expected).unwrap();
    recipe_dir.join(format!("{stem}.mog"))
}

fn stdout_json(assert: assert_cmd::assert::Assert) -> serde_json::Value {
    let out = assert.get_output().stdout.clone();
    serde_json::from_slice(&out).expect("stdout is valid JSON")
}

/// The `name`s from a `mog market list/search --json` result.
fn result_names(v: &serde_json::Value) -> Vec<String> {
    v["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap().to_string())
        .collect()
}

/// A temp MOG_HOME bootstrapped with the embedded factory set (no MCP).
fn factory_home() -> TempDir {
    let h = home();
    mog()
        .env("MOG_HOME", h.path())
        .args(["setup", "--no-mcp"])
        .assert()
        .success();
    h
}

// ---- list --------------------------------------------------------------------

#[test]
fn list_empty_exits_zero() {
    let h = home();
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list"])
        .assert()
        .success();
}

#[test]
fn list_shows_local_recipe_json() {
    let h = home();
    put_script(
        h.path(),
        "tagged",
        r#"{ "name": "Tagged", "description": "does things", "tags": ["sql","cleanup"],
             "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    let results = v["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    let s = &results[0];
    assert_eq!(s["name"], "tagged");
    assert_eq!(s["description"], "does things");
    assert_eq!(s["status"], "installed");
    let tags: Vec<&str> = s["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert_eq!(tags, vec!["sql", "cleanup"]);
    assert!(
        s.get("content").is_none(),
        "list must not carry script bodies"
    );
}

#[test]
fn list_orders_factory_by_seeded_popularity() {
    let h = factory_home();
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    let names = result_names(&v);
    // The highest seeded baseline sorts first (the top data-eng override seed).
    assert_eq!(
        names.first().map(String::as_str),
        Some("dbt-model-from-raw-sql")
    );
    // download_count is surfaced for factory recipes and ordered non-increasing.
    let counts: Vec<u64> = v["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            r["download_count"]
                .as_u64()
                .expect("factory rows carry download_count")
        })
        .collect();
    assert!(
        counts.windows(2).all(|w| w[0] >= w[1]),
        "list is popularity-descending"
    );
}

#[test]
fn list_human_prefers_summary_and_prints_count_header() {
    let h = home();
    put_script(
        h.path(),
        "blurbed",
        r#"{ "name": "blurbed",
             "summary": "Tidy a list fast",
             "description": "A long, verbose, agent-oriented description that a person would never want to skim in a browse row because it goes on and on.",
             "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list"])
        .assert()
        .success()
        // Studio-style count header before the rows.
        .stdout(predicate::str::contains("1 of 1 recipes"))
        // The human row shows the short summary, not the long description.
        .stdout(predicate::str::contains("Tidy a list fast"))
        .stdout(predicate::str::contains("verbose, agent-oriented").not())
        // Terse row: no version token.
        .stdout(predicate::str::contains(" v1 ").not());
}

#[test]
fn list_json_carries_both_summary_and_full_description() {
    let h = home();
    put_script(
        h.path(),
        "blurbed",
        r#"{ "name": "blurbed",
             "summary": "Tidy a list fast",
             "description": "The full agent-oriented description, kept intact for --json and MCP.",
             "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    let s = &v["results"].as_array().unwrap()[0];
    // Both fields present; the description is NOT truncated in the AI channel.
    assert_eq!(s["summary"], "Tidy a list fast");
    assert_eq!(
        s["description"],
        "The full agent-oriented description, kept intact for --json and MCP."
    );
}

#[test]
fn list_human_falls_back_to_truncated_description_without_summary() {
    let h = home();
    let long = "This first sentence is deliberately far longer than the seventy two character human browse clamp so it must be cut. A second sentence follows.";
    put_script(
        h.path(),
        "nosum",
        &format!(
            r#"{{ "name": "nosum", "description": {desc}, "steps": [ {{ "action": "to_upper" }} ] }}"#,
            desc = serde_json::to_string(long).unwrap()
        ),
        "a\n",
        "A\n",
    );
    // Human: a truncated first sentence with an ASCII ellipsis; the second
    // sentence never appears.
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "This first sentence is deliberately",
        ))
        .stdout(predicate::str::contains("..."))
        .stdout(predicate::str::contains("A second sentence follows").not());
    // JSON: summary is absent (no field), description is the full untruncated text.
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "list", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    let s = &v["results"].as_array().unwrap()[0];
    assert!(s.get("summary").is_none(), "no summary field when absent");
    assert_eq!(s["description"], long);
}

#[test]
fn deny_unknown_fields_still_parses_summary() {
    // The new top-level `summary` key parses under deny_unknown_fields, and the
    // local `show` surfaces it on its own line.
    let h = home();
    put_script(
        h.path(),
        "withsum",
        r#"{ "name": "withsum", "summary": "short blurb",
             "description": "long text", "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "show", "withsum"])
        .assert()
        .success()
        .stdout(predicate::str::contains("summary:"))
        .stdout(predicate::str::contains("short blurb"))
        .stdout(predicate::str::contains("description: long text"));
}

// ---- search ------------------------------------------------------------------

#[test]
fn search_finds_factory_converter() {
    let h = factory_home();
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "search", "mysql", "--json"])
        .assert()
        .success();
    let found = result_names(&stdout_json(assert));
    assert!(
        found.iter().any(|n| n == "MySQL-To-PGSQL"),
        "search mysql should find the converter, got {found:?}"
    );
}

#[test]
fn search_by_tag_cleanup() {
    let h = factory_home();
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "search", "cleanup", "--json"])
        .assert()
        .success();
    let found = result_names(&stdout_json(assert));
    for want in [
        "mssql-strip-server-noise",
        "mysql-strip-dump-noise",
        "tidy-list",
    ] {
        assert!(
            found.iter().any(|n| n == want),
            "search cleanup should find {want}, got {found:?}"
        );
    }
}

#[test]
fn search_case_insensitive() {
    let h = factory_home();
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "search", "MYSQL", "--json"])
        .assert()
        .success();
    let found = result_names(&stdout_json(assert));
    assert!(
        found.iter().any(|n| n == "MySQL-To-PGSQL"),
        "uppercase search should still match, got {found:?}"
    );
}

#[test]
fn search_no_match_empty() {
    let h = factory_home();
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "search", "zzzqqqxyz", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["results"].as_array().unwrap().len(), 0);
    assert_eq!(v["count"], 0);
}

#[test]
fn search_ranks_best_match_first() {
    let h = factory_home();
    // A natural multi-word phrase that is not a substring of the recipe name or
    // tags as a whole must still rank the right recipe first (tokenized +
    // synonym-aware): "color"/"terminal" hit strip-ansi's tags.
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args([
            "market",
            "search",
            "strip color codes from terminal output",
            "--json",
        ])
        .assert()
        .success();
    let found = result_names(&stdout_json(assert));
    assert_eq!(
        found.first().map(|s| s.as_str()),
        Some("strip-ansi"),
        "ranked search should put strip-ansi first, got {found:?}"
    );
}

#[test]
fn search_matches_task_phrases_not_in_name_or_description() {
    // The distinctive words live ONLY in task_phrases; name/description/tags
    // never mention them. A search on that phrasing must still find the recipe.
    let h = home();
    put_script(
        h.path(),
        "widget-tidy",
        r#"{ "name": "widget-tidy",
             "description": "Applies a mechanical transform to each line.",
             "task_phrases": ["zap the flux capacitor", "defrobnicate the sprocket"],
             "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "search", "flux capacitor", "--json"])
        .assert()
        .success();
    let found = result_names(&stdout_json(assert));
    assert!(
        found.iter().any(|n| n == "widget-tidy"),
        "task_phrases should drive discovery, got {found:?}"
    );
}

#[test]
fn show_json_and_deny_unknown_fields_carry_task_phrases() {
    let h = home();
    put_script(
        h.path(),
        "phrased",
        r#"{ "name": "phrased",
             "task_phrases": ["convert oracle sql to postgres"],
             "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "show", "phrased", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["task_phrases"][0], "convert oracle sql to postgres");
}

// ---- show --------------------------------------------------------------------

#[test]
fn show_local_prints_metadata_and_content() {
    let h = home();
    put_script(
        h.path(),
        "u",
        r#"{ "name": "U", "description": "d", "steps": [ { "action": "to_upper" } ] }"#,
        "a\n",
        "A\n",
    );
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "show", "u"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name:"))
        .stdout(predicate::str::contains("fixture:"))
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("to_upper"));
}

#[test]
fn show_no_content_omits_body_json() {
    let h = home();
    put_script(h.path(), "u", UPPER_MOG, "a\n", "A\n");
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "show", "u", "--no-content", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["resolvable"], "u/u.mog");
    assert_eq!(v["fixture"]["pass"], true);
    assert!(v.get("content").is_none(), "no-content must omit the body");
}

#[test]
fn show_not_found_errors() {
    let h = home();
    // A configured, reachable registry that simply does not carry "nope": the
    // recipe is neither local nor in the catalog, so show must error clearly.
    let reg = tempfile::tempdir().unwrap();
    let (base, pubkey) = build_registry(reg.path(), &["known"]);
    mog()
        .env("MOG_HOME", h.path())
        .env("MOG_MARKET_URL", &base)
        .env("MOG_MARKET_PUBKEY", &pubkey)
        .args(["market", "show", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no recipe"));
}

// ---- add ---------------------------------------------------------------------

#[test]
fn add_installs_and_verifies_fixture() {
    let h = home();
    let scratch = tempfile::tempdir().unwrap();
    let file = loose_script(scratch.path(), "new", UPPER_MOG, "hi\n", "HI\n");

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("new/new.mog"));

    assert!(market(h.path()).join("new/new.mog").is_file());
    assert!(market(h.path()).join("new/tests/input.txt").is_file());
    assert!(market(h.path()).join("new/tests/expected.txt").is_file());

    // It now resolves and runs via -m.
    mog()
        .env("MOG_HOME", h.path())
        .args(["-m", "new.mog"])
        .write_stdin("abc")
        .assert()
        .success()
        .stdout(predicate::eq("ABC"));
}

#[test]
fn add_aborts_on_failing_fixture_and_leaves_library_unchanged() {
    let h = home();
    let scratch = tempfile::tempdir().unwrap();
    let file = loose_script(scratch.path(), "bad", UPPER_MOG, "hi\n", "WRONG\n");

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("install aborted"));

    assert!(!market(h.path()).join("bad/bad.mog").exists());
    assert!(!market(h.path()).join("bad/tests/input.txt").exists());
    assert!(!market(h.path()).join("bad/tests/expected.txt").exists());
}

#[test]
fn add_missing_fixture_aborts() {
    let h = home();
    let scratch = tempfile::tempdir().unwrap();
    let file = scratch.path().join("nofix.mog");
    fs::write(&file, UPPER_MOG).unwrap();

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("input"));
    assert!(!market(h.path()).join("nofix/nofix.mog").exists());
}

#[test]
fn add_idempotent_noop_then_force_on_difference() {
    let h = home();
    let scratch = tempfile::tempdir().unwrap();
    let file = loose_script(scratch.path(), "id", UPPER_MOG, "hi\n", "HI\n");

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file.to_str().unwrap()])
        .assert()
        .success();

    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file.to_str().unwrap(), "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["noop"], true);
    assert_eq!(v["installed"], false);

    let file2 = loose_script(
        scratch.path(),
        "id",
        r#"{ "description": "changed", "steps": [ { "action": "to_upper" } ] }"#,
        "hi\n",
        "HI\n",
    );
    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file2.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force"));

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", file2.to_str().unwrap(), "--force"])
        .assert()
        .success();
    let installed = fs::read_to_string(market(h.path()).join("id/id.mog")).unwrap();
    assert!(installed.contains("changed"));
}

#[test]
fn add_refuses_non_mog_source_file() {
    let h = home();
    let scratch = tempfile::tempdir().unwrap();
    let data = scratch.path().join("data.json");
    fs::write(&data, UPPER_MOG).unwrap();

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "add", data.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Bad file extension"))
        .stderr(predicate::str::contains(".mog files"));
    assert!(!market(h.path()).join("data.mog").exists());
}

// ---- rm ----------------------------------------------------------------------

#[test]
fn rm_removes_recipe_and_fixtures() {
    let h = home();
    put_script(h.path(), "gone", UPPER_MOG, "a\n", "A\n");

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "rm", "gone.mog"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));

    assert!(!market(h.path()).join("gone").exists());
    assert!(!market(h.path()).join("gone/gone.mog").exists());
    assert!(!market(h.path()).join("gone/tests/input.txt").exists());
    assert!(!market(h.path()).join("gone/tests/expected.txt").exists());
}

// ---- bless -------------------------------------------------------------------

#[test]
fn bless_shows_diff_and_writes_only_with_yes() {
    let h = home();
    put_script(h.path(), "b", UPPER_MOG, "hi\n", "OLD\n");

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "bless", "b.mog"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-OLD"))
        .stdout(predicate::str::contains("+HI"))
        .stdout(predicate::str::contains("not written"));
    assert_eq!(
        fs::read_to_string(market(h.path()).join("b/tests/expected.txt")).unwrap(),
        "OLD\n"
    );

    mog()
        .env("MOG_HOME", h.path())
        .args(["market", "bless", "b.mog", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("blessed"));
    assert_eq!(
        fs::read_to_string(market(h.path()).join("b/tests/expected.txt")).unwrap(),
        "HI\n"
    );

    mog()
        .env("MOG_HOME", h.path())
        .args(["--test", market(h.path()).join("b/b.mog").to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn bless_no_change_is_noop() {
    let h = home();
    put_script(h.path(), "ok", UPPER_MOG, "hi\n", "HI\n");
    let assert = mog()
        .env("MOG_HOME", h.path())
        .args(["market", "bless", "ok.mog", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["changed"], false);
    assert_eq!(v["written"], false);
}
