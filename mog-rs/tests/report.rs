//! End-to-end tests for the run dashboard (`--report`, `mog report`), the
//! extended run report JSON, and the `config.toml` opt-out surface. Every test
//! drives a temp `MOG_HOME` via `--mog-dir`, so nothing touches the repo or a
//! real user library.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

fn mog() -> Command {
    Command::cargo_bin("mog").expect("mog binary builds")
}

fn home() -> TempDir {
    tempfile::tempdir().unwrap()
}

/// A .mog whose single step uppercases the whole file (so a run changes it).
const UPPER_MOG: &str = r#"{ "name": "Upper", "steps": [ { "action": "to_upper" } ] }"#;

fn write_file(dir: &Path, name: &str, contents: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, contents).expect("write scratch file");
    path.to_str().expect("utf8 path").to_string()
}

/// Write `n` files that each change under the uppercasing mog, returning paths.
fn write_n_changed(dir: &Path, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| write_file(dir, &format!("f{i}.txt"), &format!("hello {i}")))
        .collect()
}

/// Count how many files carry a diff in a dashboard HTML. The dashboard renders
/// client-side, so the template's note strings appear once regardless of the
/// data; the injected report JSON is the source of truth, and `"diff":` occurs
/// exactly once per file whose diff was computed (stats-only files omit it).
fn diffed_files(html: &str) -> usize {
    html.matches("\"diff\":").count()
}

#[test]
fn report_flag_writes_self_contained_html_with_injected_json() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello world");
    let out = dir.path().join("run.html");

    mog()
        .args(["-m", &script, &input, "--report", out.to_str().unwrap()])
        .assert()
        .success();

    let html = fs::read_to_string(&out).expect("report html written");
    // The injection token is fully replaced.
    assert!(
        !html.contains("__MOG_REPORT_JSON__"),
        "no injection token should remain"
    );
    // The report JSON is spliced in, with the new metadata + a per-file diff.
    assert!(html.contains("\"mog\""), "mog metadata present");
    assert!(html.contains("\"mode\""), "mode present");
    assert!(html.contains("\"when\""), "when present");
    assert!(html.contains("\"diff\""), "changed file carries a diff");
    assert!(
        html.contains("HELLO WORLD"),
        "diff shows transformed content"
    );
    // Self-contained: no external requests.
    assert!(
        !html.contains("http://") && !html.contains("https://"),
        "template + report must be free of external URLs"
    );
}

#[test]
fn report_flag_defaults_to_mog_home_reports_dir() {
    let h = home();
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hi");

    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "-m",
            &script,
            &input,
            "--report",
        ])
        .assert()
        .success();

    let reports = h.path().join("reports");
    let files: Vec<_> = fs::read_dir(&reports)
        .expect("reports dir created")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        files.len(),
        1,
        "one run-<timestamp>.html written: {files:?}"
    );
    assert!(files[0].starts_with("run-") && files[0].ends_with(".html"));
}

#[test]
fn json_run_report_has_additive_fields() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    let assert = mog()
        .args(["-m", &script, &input, "--dry-run", "--json"])
        .assert()
        .success();
    let out = assert.get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");

    assert_eq!(v["mog"]["name"], serde_json::json!("Upper"));
    assert_eq!(v["mode"], serde_json::json!("dry-run"));
    assert!(v["when"].as_str().is_some(), "when is present");
    assert!(v["summary"]["lines_added"].is_number());
    // The model-facing --json is lean: per-file stats, but never diff text.
    assert!(
        v["files"][0].get("diff").is_none(),
        "the lean --json output carries no diff text"
    );
    assert!(
        v["files"][0]["bytes_before"].is_number(),
        "per-file stats are still present"
    );
}

#[test]
fn mog_report_renders_json_from_stdin_to_stdout() {
    let report = r#"{"mog":{"name":"demo","path":"demo/demo.mog"},"mode":"apply","files":[]}"#;
    mog()
        .args(["report"])
        .write_stdin(report)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\":\"demo\""))
        .stdout(predicate::str::contains("__MOG_REPORT_JSON__").not());
}

#[test]
fn mog_report_writes_html_file() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.html");
    let report = r#"{"mog":{"name":"demo","path":"p"},"mode":"apply","files":[]}"#;
    mog()
        .args(["report", out.to_str().unwrap()])
        .write_stdin(report)
        .assert()
        .success();
    let html = fs::read_to_string(&out).expect("html written");
    assert!(!html.contains("__MOG_REPORT_JSON__"));
    assert!(html.contains("\"name\":\"demo\""));
}

#[test]
fn config_set_then_get_round_trips_and_writes_toml() {
    let h = home();
    let dir = h.path().to_str().unwrap().to_string();

    mog()
        .args(["--mog-dir", &dir, "config", "set", "reports", "off"])
        .assert()
        .success();

    // The file exists and records the value.
    let toml = fs::read_to_string(h.path().join("config.toml")).expect("config.toml written");
    assert!(toml.contains("reports = false"), "toml: {toml}");

    // `get` reads it back.
    mog()
        .args(["--mog-dir", &dir, "config", "get", "reports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn config_rejects_unknown_key_and_bad_bool() {
    let h = home();
    let dir = h.path().to_str().unwrap().to_string();
    mog()
        .args(["--mog-dir", &dir, "config", "set", "bogus", "x"])
        .assert()
        .failure();
    mog()
        .args(["--mog-dir", &dir, "config", "set", "reports", "maybe"])
        .assert()
        .failure();
}

#[test]
fn json_report_is_lean_no_diff_and_caps_file_list() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let inputs = write_n_changed(dir.path(), 6);

    let mut args: Vec<String> = vec!["-m".into(), script];
    args.extend(inputs.clone());
    args.extend([
        "--dry-run".into(),
        "--json".into(),
        "--report-limit".into(),
        "max_files=3".into(),
    ]);

    let assert = mog().args(&args).assert().success();
    let out = assert.get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    let v: serde_json::Value = serde_json::from_slice(&out).expect("valid JSON");

    // files[] is enumeration-capped, but the summary counts stay exact.
    assert_eq!(
        v["files"].as_array().unwrap().len(),
        3,
        "files[] capped to max_files"
    );
    assert_eq!(v["summary"]["files"], serde_json::json!(6));
    assert_eq!(v["summary"]["changed"], serde_json::json!(6));
    // The model-facing serialization carries no diff text at all.
    assert!(
        !text.contains("\"diff\""),
        "lean --json must not carry diff text: {text}"
    );
}

#[test]
fn diff_skip_over_bytes_leaves_large_file_stats_only() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello world"); // 11 bytes
    let out = dir.path().join("run.html");

    mog()
        .args([
            "-m",
            &script,
            &input,
            "--report",
            out.to_str().unwrap(),
            "--report-limit",
            "diff_skip_over_bytes=5",
        ])
        .assert()
        .success();

    let html = fs::read_to_string(&out).unwrap();
    // A file over the size gate is never diffed: no diff is injected, so its
    // transformed content (which would only appear inside a diff) is absent.
    assert_eq!(
        diffed_files(&html),
        0,
        "no diff should be computed for a file over the skip cap"
    );
    assert!(
        !html.contains("HELLO WORLD"),
        "the transformed content never materializes without a diff"
    );
}

#[test]
fn per_file_diff_truncated_with_note() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let content: String = (0..20).map(|i| format!("line {i}\n")).collect();
    let input = write_file(dir.path(), "in.txt", &content);
    let out = dir.path().join("run.html");

    mog()
        .args([
            "-m",
            &script,
            &input,
            "--report",
            out.to_str().unwrap(),
            "--report-limit",
            "max_diff_lines=5",
        ])
        .assert()
        .success();

    let html = fs::read_to_string(&out).unwrap();
    assert!(
        html.contains("more lines, truncated"),
        "the per-file diff is truncated with a note"
    );
}

#[test]
fn total_diff_bytes_budget_makes_later_files_stats_only() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let inputs = write_n_changed(dir.path(), 3);
    let out = dir.path().join("run.html");

    let mut args: Vec<String> = vec!["-m".into(), script];
    args.extend(inputs);
    args.extend([
        "--dry-run".into(),
        "--report".into(),
        out.to_str().unwrap().into(),
        "--report-limit".into(),
        "total_diff_bytes=1".into(),
    ]);
    mog().args(&args).assert().success();

    let html = fs::read_to_string(&out).unwrap();
    // A 1-byte budget lets exactly the first-computed file's diff through; the
    // other two collapse to stats-only.
    assert_eq!(
        diffed_files(&html),
        1,
        "only one of three changed files should carry a diff"
    );
}

#[test]
fn report_full_and_report_limit_override_config_with_correct_precedence() {
    let h = home();
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let inputs = write_n_changed(dir.path(), 3);

    // config.toml caps the diff sample to 1 file.
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "config",
            "set",
            "report.max_diff_files",
            "1",
        ])
        .assert()
        .success();

    let run = |out_name: &str, extra: &[&str]| -> usize {
        let out = dir.path().join(out_name);
        let mut args: Vec<String> = vec![
            "--mog-dir".into(),
            h.path().to_str().unwrap().into(),
            "-m".into(),
            script.clone(),
        ];
        args.extend(inputs.iter().cloned());
        args.extend([
            "--dry-run".into(),
            "--report".into(),
            out.to_str().unwrap().into(),
        ]);
        args.extend(extra.iter().map(|s| s.to_string()));
        mog().args(&args).assert().success();
        diffed_files(&fs::read_to_string(&out).unwrap())
    };

    // config alone: caps the diff sample to 1 file.
    assert_eq!(run("a.html", &[]), 1, "config cap of 1 applies");
    // --report-full lifts the sample cap over config: all 3 files diffed.
    assert_eq!(
        run("b.html", &["--report-full"]),
        3,
        "--report-full overrides config"
    );
    // --report-limit is the highest precedence, re-capping to 2 over --report-full.
    assert_eq!(
        run(
            "c.html",
            &["--report-full", "--report-limit", "max_diff_files=2"]
        ),
        2,
        "--report-limit overrides --report-full"
    );
}

#[test]
fn config_report_cap_cli_round_trips_and_unlimited_lifts() {
    let h = home();
    let dir = h.path().to_str().unwrap().to_string();

    mog()
        .args([
            "--mog-dir",
            &dir,
            "config",
            "set",
            "report.max_diff_files",
            "5",
        ])
        .assert()
        .success();
    mog()
        .args(["--mog-dir", &dir, "config", "get", "report.max_diff_files"])
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));

    // `unlimited` (and `0`) lift the cap; `get` shows it as unlimited.
    mog()
        .args([
            "--mog-dir",
            &dir,
            "config",
            "set",
            "report.max_diff_files",
            "unlimited",
        ])
        .assert()
        .success();
    mog()
        .args(["--mog-dir", &dir, "config", "get", "report.max_diff_files"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unlimited"));

    // The [report] table lives under config.toml without disturbing other keys.
    let toml = fs::read_to_string(h.path().join("config.toml")).expect("config.toml");
    assert!(
        toml.contains("[report]"),
        "toml has a [report] table: {toml}"
    );
}

#[test]
fn setup_no_reports_writes_config_and_factory_templates() {
    let h = home();
    mog()
        .args([
            "--mog-dir",
            h.path().to_str().unwrap(),
            "setup",
            "--no-mcp",
            "--no-reports",
        ])
        .assert()
        .success();

    // config.toml opts out of reports.
    let toml = fs::read_to_string(h.path().join("config.toml")).expect("config.toml written");
    assert!(toml.contains("reports = false"), "toml: {toml}");

    // The factory templates are written into the store for users to see/copy.
    assert!(
        h.path()
            .join("templates")
            .join("factory")
            .join("default.html")
            .is_file(),
        "factory template written to store"
    );
}

#[test]
fn setup_without_no_reports_leaves_config_absent() {
    let h = home();
    mog()
        .args(["--mog-dir", h.path().to_str().unwrap(), "setup", "--no-mcp"])
        .assert()
        .success();
    assert!(
        !h.path().join("config.toml").exists(),
        "config.toml is only written on --no-reports"
    );
    // Templates are still bootstrapped regardless of the reports opt-out.
    assert!(h
        .path()
        .join("templates")
        .join("factory")
        .join("default.html")
        .is_file());
}
