//! End-to-end CLI tests that drive the actual compiled `mog` binary. All scratch
//! files live in a `tempfile::tempdir()`, so these never touch the repo or any
//! real user files.

use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

/// Build a fresh `mog` command bound to the compiled test binary.
fn mog() -> Command {
    Command::cargo_bin("mog").expect("mog binary builds")
}

/// Write a file under `dir` and return its full path as a String.
fn write_file(dir: &Path, name: &str, contents: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, contents).expect("write scratch file");
    path.to_str().expect("utf8 path").to_string()
}

/// A .mog script whose single step uppercases the whole file.
const UPPER_MOG: &str = r#"{ "steps": [ { "action": "to_upper" } ] }"#;

/// A .mog script that flags (and thus changes) lines containing STORED.
const FLAG_MOG: &str = r#"{ "steps": [ { "action": "flag_matching",
    "options": { "pattern": "STORED", "message": "must be immutable", "comment": "--" } } ] }"#;

#[test]
fn help_succeeds_and_lists_key_flags() {
    mog()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--in-place"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--script"));
}

#[test]
fn single_file_transforms_to_stdout() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .args(["-m", &script, &input])
        .assert()
        .success()
        // Transformed content goes to stdout (the summary goes to stderr).
        .stdout(predicate::eq("HELLO"));
}

#[test]
fn dry_run_over_multiple_files_reports_and_leaves_disk_unchanged() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let a = write_file(dir.path(), "a.txt", "alpha");
    let b = write_file(dir.path(), "b.txt", "beta");

    mog()
        .args(["-m", &script, &a, &b, "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt"))
        .stdout(predicate::str::contains("Processed 2 file(s)"));

    // The input files on disk must be untouched by --dry-run.
    assert_eq!(fs::read_to_string(&a).unwrap(), "alpha");
    assert_eq!(fs::read_to_string(&b).unwrap(), "beta");
}

#[test]
fn in_place_rewrites_file_with_backup() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "doc.txt", "hello");

    mog()
        .args(["-m", &script, &input, "--in-place", "--backup", ".bak"])
        .assert()
        .success();

    // Input now holds the transformed content.
    assert_eq!(fs::read_to_string(&input).unwrap(), "HELLO");
    // The backup holds the ORIGINAL content.
    let backup = format!("{input}.bak");
    assert_eq!(fs::read_to_string(&backup).unwrap(), "hello");
}

#[test]
fn out_dir_writes_output_and_refuses_existing_without_overwrite() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");
    let out_dir = dir.path().join("out");
    let out_dir_str = out_dir.to_str().unwrap();

    // First run writes out/in.txt.
    mog()
        .args(["-m", &script, &input, "-o", out_dir_str])
        .assert()
        .success();
    let produced = out_dir.join("in.txt");
    assert_eq!(fs::read_to_string(&produced).unwrap(), "HELLO");

    // Second run to the same existing file without --overwrite must fail.
    mog()
        .args(["-m", &script, &input, "-o", out_dir_str])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // With --overwrite it succeeds.
    mog()
        .args(["-m", &script, &input, "-o", out_dir_str, "--overwrite"])
        .assert()
        .success();
}

#[test]
fn multiple_inputs_without_output_mode_errors() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let a = write_file(dir.path(), "a.txt", "alpha");
    let b = write_file(dir.path(), "b.txt", "beta");

    mog()
        .args(["-m", &script, &a, &b])
        .assert()
        .failure()
        .stderr(predicate::str::contains("output mode"));
}

#[test]
fn unknown_action_in_script_errors() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "bad.mog",
        r#"{ "steps": [ { "action": "nope" } ] }"#,
    );
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .args(["-m", &script, &input])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown action 'nope'"));
}

#[test]
fn missing_script_path_errors() {
    let dir = tempdir().unwrap();
    let input = write_file(dir.path(), "in.txt", "hello");
    let missing = dir.path().join("nope.mog");

    mog()
        .args(["-m", missing.to_str().unwrap(), &input])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read script"));
}

#[test]
fn bare_script_name_resolves_by_appending_mog() {
    let dir = tempdir().unwrap();
    // A .mog on disk; `-m upper` (no extension) must resolve to `upper.mog`.
    write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .current_dir(dir.path())
        .args(["-m", "upper", &input])
        .assert()
        .success()
        .stdout(predicate::eq("HELLO"));
}

#[test]
fn wrong_extension_script_is_refused_without_parsing() {
    let dir = tempdir().unwrap();
    // A file with valid mog JSON but the wrong extension: it must be refused
    // BEFORE parsing, so the error is about the extension, not the content.
    let notes = write_file(dir.path(), "notes.txt", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .args(["-m", &notes, &input])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Bad file extension"))
        .stderr(predicate::str::contains(".mog files"))
        .stderr(predicate::str::contains("notes.txt"));
}

#[test]
fn internal_dot_script_name_is_refused() {
    let dir = tempdir().unwrap();
    // `my.v2` has extension `.v2`; it must NOT be silently turned into `my.v2.mog`.
    write_file(dir.path(), "my.v2", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .current_dir(dir.path())
        .args(["-m", "my.v2", &input])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Bad file extension"));
}

#[test]
fn run_mog_child_with_wrong_extension_is_refused() {
    let dir = tempdir().unwrap();
    // Parent references a non-.mog child via run_mog; resolution must refuse it.
    let parent = write_file(
        dir.path(),
        "parent.mog",
        r#"{ "steps": [ { "action": "run_mog", "options": { "file": "child.txt" } } ] }"#,
    );
    // The child exists and is valid mog JSON, but its extension is wrong.
    write_file(dir.path(), "child.txt", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .args(["-m", &parent, &input])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Bad file extension"));
}

#[test]
fn missing_input_file_errors() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let missing = dir.path().join("nope.txt");

    mog()
        .args(["-m", &script, missing.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read"));
}

#[test]
fn glob_pattern_matches_multiple_files() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    write_file(dir.path(), "one.txt", "one");
    write_file(dir.path(), "two.txt", "two");

    // Run from inside the tempdir so the glob expands relative to it. Use
    // --dry-run so multiple inputs don't require an output mode.
    mog()
        .current_dir(dir.path())
        .args(["-m", &script, "*.txt", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed 2 file(s)"));
}

#[test]
fn stdin_is_filtered_to_stdout_when_no_inputs() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    // No input args: read stdin, write transformed text to stdout.
    mog()
        .args(["-m", &script])
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout(predicate::eq("HELLO WORLD"));
}

#[test]
fn dash_input_reads_stdin() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    mog()
        .args(["-m", &script, "-"])
        .write_stdin("abc")
        .assert()
        .success()
        .stdout(predicate::eq("ABC"));
}

#[test]
fn diff_output_contains_plus_and_minus_lines() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello\n");

    mog()
        .args(["-m", &script, &input, "--diff", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-hello"))
        .stdout(predicate::str::contains("+HELLO"));
}

#[test]
fn exclude_skips_matching_file() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    write_file(dir.path(), "keep1.txt", "one");
    write_file(dir.path(), "keep2.txt", "two");
    write_file(dir.path(), "skip.txt", "skip");

    // Three files match the glob; the excluded one is dropped, leaving two.
    mog()
        .current_dir(dir.path())
        .args(["-m", &script, "*.txt", "--dry-run", "--exclude", "*skip*"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed 2 file(s)"))
        .stdout(predicate::str::contains("keep1.txt"))
        .stdout(predicate::str::contains("keep2.txt"))
        .stdout(predicate::str::contains("skip.txt").not());
}

#[test]
fn lossy_processes_invalid_utf8_without_error() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    // Write raw bytes with a byte (0xFF) that is invalid as lone UTF-8.
    let path = dir.path().join("bad.txt");
    fs::write(&path, [b'a', 0xFF, b'b']).unwrap();
    let path_str = path.to_str().unwrap();

    // Default (auto) no longer fails: it falls back to Windows-1252 ("ANSI").
    // Force UTF-8 output so the transformed text is inspectable as a string
    // (preserve mode would re-emit Windows-1252 bytes).
    mog()
        .args(["-m", &script, path_str, "--output-encoding", "utf-8"])
        .assert()
        .success()
        .stdout(predicate::str::contains("A"))
        .stdout(predicate::str::contains("B"));

    // Forcing strict UTF-8 fails on the invalid byte.
    mog()
        .args(["-m", &script, path_str, "--encoding", "utf-8"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("valid UTF-8"));

    // Strict UTF-8 + --lossy replaces the invalid byte and succeeds.
    mog()
        .args(["-m", &script, path_str, "--encoding", "utf-8", "--lossy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("A"))
        .stdout(predicate::str::contains("B"));
}

// ---- Phase E1: --json report, --check exit codes, `mog --test` verb ----

/// Parse the command's stdout as JSON, failing the test on invalid JSON.
fn stdout_json(assert: assert_cmd::assert::Assert) -> serde_json::Value {
    let out = assert.get_output().stdout.clone();
    serde_json::from_slice(&out).expect("stdout is valid JSON")
}

#[test]
fn json_dry_run_emits_structured_report() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let a = write_file(dir.path(), "a.txt", "alpha");
    let b = write_file(dir.path(), "b.txt", "BETA"); // already upper -> unchanged

    let assert = mog()
        .args(["-m", &script, &a, &b, "--dry-run", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);

    assert_eq!(v["summary"]["files"], 2);
    assert_eq!(v["summary"]["changed"], 1);
    assert_eq!(v["summary"]["errors"], 0);
    let files = v["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    // Every file record carries the documented shape.
    for f in files {
        assert!(f["path"].is_string());
        assert!(f["changed"].is_boolean());
        assert!(f["bytes_before"].is_u64());
        assert!(f["bytes_after"].is_u64());
        assert!(f["lines_added"].is_u64());
        assert!(f["lines_removed"].is_u64());
    }
    assert!(v["flags"].is_array());
    assert!(v["flag_counts"].is_object());
}

#[test]
fn json_reports_flags_with_per_tag_counts() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "flag.mog", FLAG_MOG);
    let input = write_file(dir.path(), "in.sql", "a STORED,\nb int\n");

    let assert = mog()
        .args(["-m", &script, &input, "--dry-run", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);

    let flags = v["flags"].as_array().unwrap();
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0]["tag"], "FIXME");
    assert_eq!(flags[0]["line"], 1);
    assert!(flags[0]["file"].as_str().unwrap().contains("in.sql"));
    assert_eq!(v["flag_counts"]["FIXME"], 1);
}

#[test]
fn json_in_stdout_content_mode_requires_dry_run() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    // Single file to stdout + --json without --dry-run/--check: the report would
    // collide with the transformed content, so this is refused (as JSON error).
    let assert = mog()
        .args(["-m", &script, &input, "--json"])
        .assert()
        .failure();
    let v = stdout_json(assert);
    assert_eq!(v["error"], true);
    assert!(v["message"].as_str().unwrap().contains("content"));
}

#[test]
fn json_top_level_error_carries_step_and_action() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "bad.mog",
        r#"{ "steps": [ { "action": "to_upper" }, { "action": "nope" } ] }"#,
    );

    // In stdin mode the run fails as a whole (no per-file report), so the error
    // bubbles up and is emitted as a top-level JSON error object.
    let assert = mog()
        .args(["-m", &script, "--dry-run", "--json"])
        .write_stdin("hello")
        .assert()
        .failure();
    let v = stdout_json(assert);
    assert_eq!(v["error"], true);
    // The unknown action is in step 2.
    assert_eq!(v["action"], "nope");
    assert_eq!(v["step"], 2);
}

#[test]
fn json_malformed_script_is_reported_as_json_error() {
    let dir = tempdir().unwrap();
    // Not valid JSON: a parse/validation failure before any file is processed.
    let script = write_file(dir.path(), "bad.mog", "{ this is not json }");
    let input = write_file(dir.path(), "in.txt", "hello");

    let assert = mog()
        .args(["-m", &script, &input, "--dry-run", "--json"])
        .assert()
        .failure();
    let v = stdout_json(assert);
    assert_eq!(v["error"], true);
    assert!(v["message"].is_string());
}

#[test]
fn json_per_file_execution_error_lands_in_files_array() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "bad.mog",
        r#"{ "steps": [ { "action": "nope" } ] }"#,
    );
    let input = write_file(dir.path(), "in.txt", "hello");

    // A per-file run error is captured in that file's record (not the top level).
    let assert = mog()
        .args(["-m", &script, &input, "--dry-run", "--json"])
        .assert()
        .failure();
    let v = stdout_json(assert);
    assert_eq!(v["summary"]["errors"], 1);
    let f = &v["files"][0];
    assert!(f["error"]
        .as_str()
        .unwrap()
        .contains("Unknown action 'nope'"));
}

#[test]
fn check_clean_exits_zero_with_no_output() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    // Already uppercase: nothing would change.
    let input = write_file(dir.path(), "in.txt", "HELLO");

    mog()
        .args(["-m", &script, &input, "--check"])
        .assert()
        .code(0)
        .stdout(predicate::str::is_empty());
}

#[test]
fn check_dirty_exits_one_and_lists_path() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello"); // would change

    mog()
        .args(["-m", &script, &input, "--check"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("in.txt"));

    // The file on disk is untouched (check writes nothing).
    assert_eq!(fs::read_to_string(&input).unwrap(), "hello");
}

#[test]
fn check_with_json_reports_offending_and_exit_code() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    let assert = mog()
        .args(["-m", &script, &input, "--check", "--json"])
        .assert()
        .code(1);
    let v = stdout_json(assert);
    assert_eq!(v["check"]["clean"], false);
    assert!(v["check"]["offending"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p.as_str().unwrap().contains("in.txt")));
}

/// Write a recipe directory: `<stem>/<stem>.mog` plus its `tests/input.txt` and
/// `tests/expected.txt` fixtures. Returns the `.mog` path.
fn write_test_fixtures(
    dir: &Path,
    stem: &str,
    script: &str,
    input: &str,
    expected: &str,
) -> String {
    let mog_dir = dir.join(stem);
    fs::create_dir_all(mog_dir.join("tests")).expect("create recipe tests dir");
    let mog_path = write_file(&mog_dir, &format!("{stem}.mog"), script);
    write_file(&mog_dir.join("tests"), "input.txt", input);
    write_file(&mog_dir.join("tests"), "expected.txt", expected);
    mog_path
}

#[test]
fn test_verb_passes_matching_fixture() {
    let dir = tempdir().unwrap();
    let mog_path = write_test_fixtures(dir.path(), "upper", UPPER_MOG, "hello\n", "HELLO\n");

    mog()
        .args(["--test", &mog_path])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("1 passed, 0 failed"));
}

#[test]
fn test_verb_fails_mismatched_fixture_with_diff() {
    let dir = tempdir().unwrap();
    let mog_path = write_test_fixtures(dir.path(), "upper", UPPER_MOG, "hello\n", "WRONG\n");

    mog()
        .args(["--test", &mog_path])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("FAIL"))
        // A unified diff: golden (-) vs actual (+).
        .stdout(predicate::str::contains("-WRONG"))
        .stdout(predicate::str::contains("+HELLO"));
}

#[test]
fn test_verb_scans_a_directory() {
    let dir = tempdir().unwrap();
    write_test_fixtures(dir.path(), "one", UPPER_MOG, "a\n", "A\n");
    write_test_fixtures(dir.path(), "two", UPPER_MOG, "b\n", "B\n");

    mog()
        .args(["--test", dir.path().to_str().unwrap()])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed, 0 failed"));
}

#[test]
fn test_verb_json_reports_results() {
    let dir = tempdir().unwrap();
    let mog_path = write_test_fixtures(dir.path(), "upper", UPPER_MOG, "hello\n", "HELLO\n");

    let assert = mog().args(["--test", &mog_path, "--json"]).assert().code(0);
    let v = stdout_json(assert);
    assert_eq!(v["summary"]["total"], 1);
    assert_eq!(v["summary"]["passed"], 1);
    assert_eq!(v["tests"][0]["pass"], true);
}

// ---- Progressive disclosure: --list-actions --compact, --describe ----

#[test]
fn list_actions_full_emits_params() {
    let assert = mog().args(["--list-actions", "--json"]).assert().success();
    let v = stdout_json(assert);
    let arr = v.as_array().expect("array of descriptors");
    assert!(arr.len() >= 40);
    // The full view carries params and a tier on every entry.
    for entry in arr {
        assert!(entry.get("params").is_some(), "full entry lacks params");
        let tier = entry["tier"].as_str().expect("tier");
        assert!(tier == "core" || tier == "full");
    }
}

#[test]
fn list_actions_compact_omits_params_and_carries_tier() {
    let assert = mog()
        .args(["--list-actions", "--compact", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    let arr = v.as_array().expect("array of compact descriptors");
    assert!(arr.len() >= 40);
    for entry in arr {
        // Compact entries are identity + summary only: no params, always a tier.
        assert!(entry.get("params").is_none(), "compact entry has params");
        assert!(entry.get("name").is_some());
        assert!(entry.get("summary").is_some());
        let tier = entry["tier"].as_str().expect("tier on every compact entry");
        assert!(tier == "core" || tier == "full");
    }
    // At least the curated Core set should be present.
    let cores = arr.iter().filter(|e| e["tier"] == "core").count();
    assert!(
        cores >= 12,
        "expected a dozen-plus core actions, got {cores}"
    );
}

#[test]
fn compact_without_list_actions_errors() {
    mog()
        .args(["--compact"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--compact only applies to --list-actions",
        ));
}

#[test]
fn describe_known_action_emits_full_descriptor() {
    let assert = mog()
        .args(["--describe", "replace", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["name"], "replace");
    assert!(v.get("params").is_some(), "describe carries params");
    assert!(v.get("example").is_some(), "core action carries an example");
    assert_eq!(v["tier"], "core");
}

#[test]
fn describe_resolves_alias_to_canonical() {
    // "rr" is the alias for replace_regex; --describe resolves it.
    let assert = mog()
        .args(["--describe", "rr", "--json"])
        .assert()
        .success();
    let v = stdout_json(assert);
    assert_eq!(v["name"], "replace_regex");
}

#[test]
fn describe_unknown_action_json_error() {
    // Under --json the error is a JSON object on stdout, exit code 1.
    let assert = mog()
        .args(["--describe", "no_such_action", "--json"])
        .assert()
        .code(1);
    let v = stdout_json(assert);
    assert_eq!(v["error"], true);
    assert!(v["message"].as_str().unwrap().contains("no_such_action"));
}

#[test]
fn describe_unknown_action_plain_error() {
    // Without --json the error goes to stderr with a nonzero exit.
    mog()
        .args(["--describe", "no_such_action"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("unknown action 'no_such_action'"));
}

// ---- Guardrails: --refuse-binary and --max-shrink ----
//
// Both were previously covered only by shell smoke-tests. These lock the real
// CLI behavior: the trigger conditions, the exact user-facing messages, and the
// opt-in (no false-positive / no-op-when-off) contract.

/// A .mog whose single step drops every line matching `pattern`.
fn drop_lines_mog(pattern: &str) -> String {
    format!(
        r#"{{ "steps": [ {{ "action": "remove_lines_matching", "options": {{ "pattern": "{pattern}" }} }} ] }}"#
    )
}

/// A file containing a NUL byte is refused under --refuse-binary, and the message
/// names the offending file.
#[test]
fn refuse_binary_rejects_file_with_nul_byte() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    let path = dir.path().join("bin.dat");
    fs::write(&path, [b'a', 0x00, b'b']).unwrap();
    let path_str = path.to_str().unwrap();

    mog()
        .args(["-m", &script, path_str, "--refuse-binary"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "looks binary (contains a NUL byte)",
        ))
        .stderr(predicate::str::contains("bin.dat"));
}

/// Without --refuse-binary the same NUL-containing file is processed (a lone NUL
/// is valid UTF-8), proving the guard is opt-in rather than the default.
#[test]
fn nul_byte_file_processed_without_refuse_binary() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    let path = dir.path().join("bin.dat");
    fs::write(&path, [b'a', 0x00, b'b']).unwrap();
    let path_str = path.to_str().unwrap();

    mog()
        .args(["-m", &script, path_str])
        .assert()
        .success()
        .stdout(predicate::str::contains("A"))
        .stdout(predicate::str::contains("B"));
}

/// --refuse-binary also guards stdin, and the message is labelled "stdin".
#[test]
fn refuse_binary_rejects_stdin_with_nul_byte() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    mog()
        .args(["-m", &script, "--refuse-binary"])
        .write_stdin(vec![b'a', 0x00, b'b'])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "stdin looks binary (contains a NUL byte)",
        ));
}

/// Ordinary text with --refuse-binary present passes: the flag never
/// false-positives on NUL-free input.
#[test]
fn refuse_binary_allows_plain_text() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello");

    mog()
        .args(["-m", &script, &input, "--refuse-binary"])
        .assert()
        .success()
        .stdout(predicate::eq("HELLO"));
}

/// --max-shrink fails a run that drops more than the allowed percentage of lines.
#[test]
fn max_shrink_fails_when_too_many_lines_dropped() {
    let dir = tempdir().unwrap();
    // Pattern "^" matches the start of every line, so all lines are removed
    // (100% shrink), well over the 50% ceiling.
    let script = write_file(dir.path(), "drop.mog", &drop_lines_mog("^"));
    let input = write_file(dir.path(), "in.txt", "a\nb\nc\nd");

    mog()
        .args(["-m", &script, &input, "--max-shrink", "50"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("over --max-shrink"))
        .stderr(predicate::str::contains("dropped"));
}

/// --max-shrink allows a run whose line loss stays within the threshold.
#[test]
fn max_shrink_allows_small_line_loss() {
    let dir = tempdir().unwrap();
    // Drop only the line "b" -> 1 of 4 lines = 25% shrink, under the ceiling.
    let script = write_file(dir.path(), "drop.mog", &drop_lines_mog("^b$"));
    let input = write_file(dir.path(), "in.txt", "a\nb\nc\nd");

    mog()
        .args(["-m", &script, &input, "--max-shrink", "50"])
        .assert()
        .success()
        .stdout(predicate::eq("a\nc\nd"));
}

/// The threshold is inclusive: the guard trips only on strictly greater shrink,
/// so exactly 50% dropped at --max-shrink 50 still passes.
#[test]
fn max_shrink_boundary_is_inclusive() {
    let dir = tempdir().unwrap();
    // Drop "b" and "c" -> 2 of 4 lines = exactly 50%.
    let script = write_file(dir.path(), "drop.mog", &drop_lines_mog("^[bc]$"));
    let input = write_file(dir.path(), "in.txt", "a\nb\nc\nd");

    mog()
        .args(["-m", &script, &input, "--max-shrink", "50"])
        .assert()
        .success()
        .stdout(predicate::eq("a\nd"));
}

// ---- Phase 4: --stream (bounded-memory stdin filter) ----

/// A multi-MiB input with MIXED line endings (LF, CRLF, and lone CR) so the
/// streamed blocks (cut at LF only) exercise the concat-safety of the EOL bytes.
/// Comfortably exceeds the 256 KiB stream block size to force several blocks.
fn big_mixed_eol_input() -> String {
    let mut s = String::with_capacity(1_200_000);
    let mut i = 0u64;
    while s.len() < 1_100_000 {
        // Rotate LF / CRLF / lone-CR terminators; content has mixed case + spaces.
        let term = match i % 3 {
            0 => "\n",
            1 => "\r\n",
            _ => "\r",
        };
        s.push_str(&format!("  Line_{i} Value_{i}  {term}"));
        i += 1;
    }
    s
}

/// The streamed output must be byte-for-byte identical to the whole-file output
/// for an all-streamable pipeline over multi-block, mixed-EOL input.
#[test]
fn stream_output_is_byte_identical_to_whole_file() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = big_mixed_eol_input();

    let whole = mog()
        .args(["-m", &script, "--encoding", "utf-8"])
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let streamed = mog()
        .args(["-m", &script, "--stream", "--encoding", "utf-8"])
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(streamed, whole, "streamed output diverged from whole-file");
}

/// A multi-step all-streamable pipeline also streams byte-identically.
#[test]
fn stream_multi_step_pipeline_matches_whole_file() {
    let dir = tempdir().unwrap();
    // trim -> upper -> tabs_to_spaces: all streamable.
    let script = write_file(
        dir.path(),
        "clean.mog",
        r#"{"steps":[{"action":"trim_whitespace"},{"action":"to_upper"},{"action":"tabs_to_spaces"}]}"#,
    );
    let input = big_mixed_eol_input();

    let whole = mog()
        .args(["-m", &script, "--encoding", "utf-8"])
        .write_stdin(input.clone())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let streamed = mog()
        .args(["-m", &script, "--stream", "--encoding", "utf-8"])
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(streamed, whole);
}

/// --stream refuses a non-streamable pipeline, naming the offending step.
#[test]
fn stream_rejects_non_streamable_pipeline() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "sort.mog",
        r#"{"steps":[{"action":"sort_lines"}]}"#,
    );

    mog()
        .args(["-m", &script, "--stream", "--encoding", "utf-8"])
        .write_stdin("b\na\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not streamable"))
        .stderr(predicate::str::contains("sort_lines"));
}

/// --stream requires explicit UTF-8 (the default 'auto' is refused, since
/// per-block auto-detection would not be byte-identical).
#[test]
fn stream_requires_utf8_encoding() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    mog()
        .args(["-m", &script, "--stream"])
        .write_stdin("hello\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --encoding utf-8"));
}

/// --stream on a single FILE input streams to stdout, byte-identical to the
/// whole-file path, over multi-block mixed-EOL input.
#[test]
fn stream_single_file_matches_whole_file() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let path = dir.path().join("big.txt");
    fs::write(&path, big_mixed_eol_input()).unwrap();
    let path = path.to_str().unwrap();

    let whole = mog()
        .args(["-m", &script, path, "--encoding", "utf-8"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let streamed = mog()
        .args(["-m", &script, path, "--stream", "--encoding", "utf-8"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(streamed, whole, "file stream diverged from whole-file");
}

/// --stream --in-place rewrites the file in bounded memory, byte-identical to the
/// whole-file in-place path, and honors --backup.
#[test]
fn stream_in_place_matches_whole_file_and_backs_up() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let original = big_mixed_eol_input();

    // Reference: whole-file in-place on a copy.
    let ref_path = dir.path().join("ref.txt");
    fs::write(&ref_path, &original).unwrap();
    mog()
        .args([
            "-m",
            &script,
            ref_path.to_str().unwrap(),
            "--in-place",
            "--encoding",
            "utf-8",
        ])
        .assert()
        .success();
    let ref_bytes = fs::read(&ref_path).unwrap();

    // Streamed in-place on another copy, with a backup.
    let path = dir.path().join("doc.txt");
    fs::write(&path, &original).unwrap();
    mog()
        .args([
            "-m",
            &script,
            path.to_str().unwrap(),
            "--stream",
            "--in-place",
            "--backup",
            ".bak",
            "--encoding",
            "utf-8",
        ])
        .assert()
        .success();

    // The streamed rewrite equals the whole-file rewrite, byte-for-byte.
    assert_eq!(
        fs::read(&path).unwrap(),
        ref_bytes,
        "in-place stream diverged"
    );
    // The backup holds the ORIGINAL bytes.
    let bak = format!("{}.bak", path.to_str().unwrap());
    assert_eq!(fs::read(&bak).unwrap(), original.as_bytes());
}

/// --stream with --dry-run is refused (streaming emits content; dry-run must not).
#[test]
fn stream_with_dry_run_is_refused() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    mog()
        .args([
            "-m",
            &script,
            "--stream",
            "--encoding",
            "utf-8",
            "--dry-run",
        ])
        .write_stdin("hello\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("incompatible with --dry-run"));
}

/// --stream --out-dir writes the transformed file into the output directory in
/// bounded memory (byte-identical to the whole-file out-dir path).
#[test]
fn stream_with_out_dir_writes_result() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello\n");
    let out = dir.path().join("out");

    mog()
        .args([
            "-m",
            &script,
            &input,
            "--stream",
            "--encoding",
            "utf-8",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("out-dir"));

    let written = fs::read_to_string(out.join("in.txt")).expect("output file written");
    assert_eq!(written, "HELLO\n");
    // The original input is left untouched.
    assert_eq!(fs::read_to_string(&input).unwrap(), "hello\n");
}

/// --stream --out-dir refuses to clobber an existing destination without --overwrite.
#[test]
fn stream_out_dir_refuses_existing_without_overwrite() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);
    let input = write_file(dir.path(), "in.txt", "hello\n");
    let out = dir.path().join("out");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("in.txt"), "OLD").unwrap();

    mog()
        .args([
            "-m",
            &script,
            &input,
            "--stream",
            "--encoding",
            "utf-8",
            "--out-dir",
            out.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
}

/// --refuse-binary still guards the streamed path (NUL byte in stdin).
#[test]
fn stream_honors_refuse_binary() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "upper.mog", UPPER_MOG);

    mog()
        .args([
            "-m",
            &script,
            "--stream",
            "--encoding",
            "utf-8",
            "--refuse-binary",
        ])
        .write_stdin(vec![b'a', 0x00, b'b', b'\n'])
        .assert()
        .failure()
        .stderr(predicate::str::contains("looks binary"));
}

// ---- Phase 4: --parallel (intra-file, byte-identical) ----

/// --parallel on a streamable pipeline yields byte-identical output to sequential,
/// for bare --parallel (all cores) and --parallel=N.
#[test]
fn parallel_output_matches_sequential() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "clean.mog",
        r#"{"steps":[{"action":"trim_whitespace"},{"action":"to_upper"}]}"#,
    );
    let path = dir.path().join("big.txt");
    fs::write(&path, big_mixed_eol_input()).unwrap();
    let path = path.to_str().unwrap();

    let seq = mog()
        .args(["-m", &script, path, "--encoding", "utf-8"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    for degree in [&["--parallel"][..], &["--parallel", "4"][..]] {
        let mut args = vec!["-m", &script, path, "--encoding", "utf-8"];
        args.extend_from_slice(degree);
        let par = mog()
            .args(&args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        assert_eq!(par, seq, "parallel {degree:?} diverged from sequential");
    }
}

/// --parallel on a NON-streamable pipeline (sort_lines) falls back to sequential
/// and still produces correct output.
#[test]
fn parallel_non_streamable_falls_back_correctly() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "sort.mog",
        r#"{"steps":[{"action":"sort_lines"}]}"#,
    );
    let input = write_file(dir.path(), "in.txt", "c\na\nb\n");

    mog()
        .args(["-m", &script, &input, "--parallel"])
        .assert()
        .success()
        .stdout(predicate::eq("a\nb\nc\n"));
}

/// --stream and --parallel are mutually exclusive.
#[test]
fn stream_and_parallel_conflict() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);

    mog()
        .args([
            "-m",
            &script,
            "--stream",
            "--parallel",
            "--encoding",
            "utf-8",
        ])
        .write_stdin("hi\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("different strategies"));
}

// ---- Phase 4 CI/CD: --in-place, --files-from ----

/// --in-place rewrites the file and leaves no temp file behind.
#[test]
fn in_place_rewrites_and_leaves_no_temp() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);
    let input = write_file(dir.path(), "doc.txt", "hello");

    mog()
        .args(["-m", &script, &input, "--in-place"])
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&input).unwrap(), "HELLO");
    let leftover: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".mog-tmp."))
        .collect();
    assert!(leftover.is_empty(), "temp file left behind: {leftover:?}");
}

/// --files-from FILE adds the listed paths to the inputs.
#[test]
fn files_from_file_processes_listed_inputs() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);
    let a = write_file(dir.path(), "a.txt", "aaa");
    let b = write_file(dir.path(), "b.txt", "bbb");
    let list = write_file(dir.path(), "list.txt", &format!("{a}\n{b}\n"));

    mog()
        .args(["-m", &script, "--files-from", &list, "--in-place"])
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&a).unwrap(), "AAA");
    assert_eq!(fs::read_to_string(&b).unwrap(), "BBB");
}

/// --files-from - reads the path list from stdin.
#[test]
fn files_from_stdin_processes_listed_inputs() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);
    let a = write_file(dir.path(), "a.txt", "x");

    mog()
        .args(["-m", &script, "--files-from", "-", "--in-place"])
        .write_stdin(format!("{a}\n"))
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&a).unwrap(), "X");
}

/// An empty --files-from list is a no-op success (CI: no changed files).
#[test]
fn files_from_empty_list_is_noop_success() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);

    mog()
        .args(["-m", &script, "--files-from", "-", "--in-place"])
        .write_stdin("\n  \n")
        .assert()
        .success()
        .stderr(predicate::str::contains("nothing to do"));
}

// ---- Phase 5: --explain ----

/// --explain renders the pipeline's intent: name, description, each step's own
/// description, and a descriptor fallback for steps without one.
#[test]
fn explain_renders_pipeline_intent() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "p.mog",
        r#"{"name":"Demo","description":"do things","steps":[{"action":"to_upper"},{"action":"trim_whitespace","description":"trim it"}]}"#,
    );

    mog()
        .args(["-m", &script, "--explain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Demo"))
        .stdout(predicate::str::contains("do things"))
        .stdout(predicate::str::contains("trim it")) // step's own description
        .stdout(predicate::str::contains("Uppercase")); // descriptor fallback for to_upper

    mog()
        .args(["-m", &script, "--explain", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"text\": \"trim it\""));
}

// ---- Phase 4: --sample (huge-file preview) ----

/// --sample N previews the transform on just the first N lines.
#[test]
fn sample_previews_first_n_lines() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);
    let input = write_file(dir.path(), "big.txt", "a\nb\nc\nd\ne\n");

    mog()
        .args(["-m", &script, &input, "--sample", "2"])
        .assert()
        .success()
        .stdout(predicate::eq("A\nB\n"));
}

/// --sample reads from stdin too.
#[test]
fn sample_from_stdin() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);

    mog()
        .args(["-m", &script, "--sample", "2"])
        .write_stdin("x\ny\nz\n")
        .assert()
        .success()
        .stdout(predicate::eq("X\nY\n"));
}

/// --sample is a preview and refuses the file-writing modes.
#[test]
fn sample_refuses_in_place() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "u.mog", UPPER_MOG);
    let input = write_file(dir.path(), "x.txt", "a\nb\n");

    mog()
        .args(["-m", &script, &input, "--sample", "1", "--in-place"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("read-only preview"));
}

#[test]
fn mog_output_encoding_property_is_honored() {
    // A recipe may declare `output_encoding`; here it forces a UTF-8 BOM prefix
    // without the caller passing --output-encoding.
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "bom.mog",
        r#"{"output_encoding":"utf-8-bom","steps":[{"action":"to_upper"}]}"#,
    );
    let input = write_file(dir.path(), "in.txt", "hello");
    mog()
        .args(["-m", &script, &input])
        .assert()
        .success()
        .stdout(predicate::eq(&b"\xEF\xBB\xBFHELLO"[..]));
}

#[test]
fn explicit_output_encoding_flag_overrides_mog_property() {
    // An explicit --output-encoding wins over the recipe's output_encoding.
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "bom.mog",
        r#"{"output_encoding":"utf-8-bom","steps":[{"action":"to_upper"}]}"#,
    );
    let input = write_file(dir.path(), "in.txt", "hello");
    mog()
        .args(["-m", &script, &input, "--output-encoding", "utf-8"])
        .assert()
        .success()
        .stdout(predicate::eq(&b"HELLO"[..]));
}

/// A two-step .mog for progress tests (uppercase, then prefix each line).
const TWO_STEP_MOG: &str =
    r#"{"steps":[{"action":"to_upper"},{"action":"prefix_lines","options":{"text":"> "}}]}"#;

#[test]
fn progress_prints_step_lines_to_stderr_only() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "two.mog", TWO_STEP_MOG);
    mog()
        .args(["-m", &script, "--progress"])
        .write_stdin("hello\n")
        .assert()
        .success()
        // Content is clean on stdout (no progress noise).
        .stdout(predicate::eq("> HELLO\n"))
        // Each step is announced on stderr with the i/N counter and action.
        .stderr(predicate::str::contains("step 1/2: to_upper"))
        .stderr(predicate::str::contains("step 2/2: prefix_lines"));
}

#[test]
fn progress_without_flag_is_silent() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "two.mog", TWO_STEP_MOG);
    mog()
        .args(["-m", &script])
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout(predicate::eq("> HELLO\n"))
        .stderr(predicate::str::contains("step ").not());
}

#[test]
fn progress_is_suppressed_and_noted_for_multiple_files() {
    let dir = tempdir().unwrap();
    let script = write_file(dir.path(), "two.mog", TWO_STEP_MOG);
    let a = write_file(dir.path(), "a.txt", "a");
    let b = write_file(dir.path(), "b.txt", "b");
    mog()
        .args(["-m", &script, &a, &b, "--dry-run", "--progress"])
        .assert()
        .success()
        // A note explains the suppression; no per-step lines are emitted.
        .stderr(predicate::str::contains(
            "--progress applies to single-file runs",
        ))
        .stderr(predicate::str::contains("step 1/").not());
}

/// The {{@filename}} placeholder is expanded per input file (proves the per-file
/// file-context interpolation is wired through the normal file path).
#[test]
fn filename_placeholder_expands_per_file() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "stamp.mog",
        r##"{"steps":[{"action":"prepend","options":{"text":"# source: {{@filename}}\n"}}]}"##,
    );
    let input = write_file(dir.path(), "data.txt", "hello\n");
    mog()
        .args(["-m", &script, &input])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("# source: data.txt\n"));
}

/// {{@random}} is a 32-hex token, and --pin-seed makes it reproducible.
#[test]
fn random_placeholder_is_stable_under_pin_seed() {
    let dir = tempdir().unwrap();
    let script = write_file(
        dir.path(),
        "rand.mog",
        r#"{"steps":[{"action":"prepend","options":{"text":"{{@random}}\n"}}]}"#,
    );
    let input = write_file(dir.path(), "in.txt", "x\n");
    let run = || {
        let out = mog()
            .args(["-m", &script, &input, "--pin-seed", "0"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        String::from_utf8(out).unwrap()
    };
    let a = run();
    assert_eq!(a, run()); // reproducible under the same seed
    let token = a.lines().next().unwrap();
    assert_eq!(token.len(), 32);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}
