//! `for_each_block` runs a sub-.mog per region with the block header's captures
//! bound as constants. Needs a base dir so the child script resolves, so this
//! uses a tempdir rather than the `execute` (no-base) path.

use std::fs;

use mog::{execute_at, parse_mog};
use tempfile::tempdir;

#[test]
fn binds_header_capture_and_runs_child_per_block() {
    let dir = tempdir().unwrap();
    // The child sees each table block with {{table}} bound from the header match.
    fs::write(
        dir.path().join("per-table.mog"),
        r#"{
          "steps": [
            { "action": "insert_before_matching",
              "options": { "pattern": "^CREATE TABLE", "text": "-- table: {{table}}" } }
          ]
        }"#,
    )
    .unwrap();

    let parent = parse_mog(
        r#"{
          "steps": [
            { "action": "for_each_block",
              "options": {
                "block_start": "CREATE TABLE (\\w+) \\(",
                "block_end": "^\\)",
                "bind": { "table": "${1}" },
                "run": "per-table.mog"
              } }
          ]
        }"#,
    )
    .unwrap();

    let input = "CREATE TABLE users (\n  id int\n);\nCREATE TABLE orders (\n  id int\n);\n";
    let out = execute_at(&parent, input, Some(dir.path())).unwrap();
    assert_eq!(
        out,
        "-- table: users\nCREATE TABLE users (\n  id int\n);\n\
         -- table: orders\nCREATE TABLE orders (\n  id int\n);\n"
    );
}

#[test]
fn non_matching_input_is_unchanged() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("noop.mog"),
        r#"{ "steps": [ { "action": "to_upper" } ] }"#,
    )
    .unwrap();
    let parent = parse_mog(
        r#"{ "steps": [ { "action": "for_each_block",
              "options": { "block_start": "CREATE TABLE (\\w+)", "block_end": "^\\)",
                           "run": "noop.mog" } } ] }"#,
    )
    .unwrap();
    let input = "SELECT 1;\nSELECT 2;\n";
    assert_eq!(execute_at(&parent, input, Some(dir.path())).unwrap(), input);
}
