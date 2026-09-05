//! Golden and unit tests for the mog engine.

use std::path::PathBuf;

use mog::model::Step;
use mog::{execute, parse_mog};
use serde_json::json;

fn step(action: &str, options: serde_json::Value) -> Step {
    let map = options.as_object().cloned().unwrap_or_default();
    Step {
        description: None,
        section: None,
        action: Some(action.to_string()),
        disabled: false,
        only_lines_matching: None,
        except_lines_matching: None,
        match_ignore_case: false,
        scope: None,
        options: map,
    }
}

fn run_action(action: &str, options: serde_json::Value, input: &str) -> String {
    let f = mog::engine::resolve(action).expect("action resolves");
    f(input, &step(action, options)).expect("action runs")
}

#[test]
fn golden_tidy_list() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mog_path = manifest.join("factory/tidy-list/tidy-list.mog");
    let txt_path = manifest.join("factory/tidy-list/tests/input.txt");

    let script = std::fs::read_to_string(&mog_path).expect("read tidy-list.mog");
    let input = std::fs::read_to_string(&txt_path).expect("read tidy-list.txt");

    let mog = parse_mog(&script).expect("parse mog");
    let output = execute(&mog, &input).expect("execute");

    assert_eq!(
        output, "Apple\nBanana\nCherry\nDate\nElderberry\nFig\n",
        "golden tidy-list output mismatch"
    );
}

#[test]
fn replace_case_sensitive() {
    let out = run_action(
        "replace",
        json!({ "find": "cat", "replace_with": "dog", "ignore_case": false }),
        "cat CAT Cat",
    );
    assert_eq!(out, "dog CAT Cat");
}

#[test]
fn replace_case_insensitive() {
    let out = run_action(
        "replace",
        json!({ "find": "cat", "replace_with": "dog", "ignore_case": true }),
        "cat CAT Cat",
    );
    assert_eq!(out, "dog dog dog");
}

#[test]
fn replace_ignore_case_literal_not_regex() {
    // The find must be treated literally even under ignore_case.
    let out = run_action(
        "replace",
        json!({ "find": "a.c", "replace_with": "X", "ignore_case": true }),
        "a.c abc A.C",
    );
    assert_eq!(out, "X abc X");
}

#[test]
fn sort_lines_numeric() {
    let out = run_action("sort_lines", json!({ "numeric": true }), "10\n2\n1");
    assert_eq!(out, "1\n2\n10");
}

#[test]
fn sort_lines_ignore_case() {
    let out = run_action(
        "sort_lines",
        json!({ "ignore_case": true }),
        "banana\nApple\ncherry",
    );
    assert_eq!(out, "Apple\nbanana\ncherry");
}

#[test]
fn remove_duplicate_lines_keeps_first() {
    let out = run_action(
        "remove_duplicate_lines",
        json!({ "ignore_case": true }),
        "Apple\napple\nBanana\nAPPLE\nbanana",
    );
    assert_eq!(out, "Apple\nBanana");
}

#[test]
fn eol_crlf_and_lf_conversion() {
    let to_crlf = run_action("eol_crlf", json!({}), "a\nb\nc");
    assert_eq!(to_crlf, "a\r\nb\r\nc");

    let to_lf = run_action("eol_lf", json!({}), "a\r\nb\r\nc");
    assert_eq!(to_lf, "a\nb\nc");
}

#[test]
fn regex_backreference_doubled_chars() {
    // (\w)\1 matches a doubled character - proves fancy-regex backrefs work.
    let out = run_action(
        "replace_regex",
        json!({ "find": r"(\w)\1", "replace_with": "[$1]" }),
        "book keeper letter",
    );
    assert_eq!(out, "b[o]k k[e]per le[t]er");
}

#[test]
fn reverse_lines_preserves_crlf() {
    let out = run_action("reverse_lines", json!({}), "a\r\nb\r\nc\r\n");
    assert_eq!(out, "c\r\nb\r\na\r\n");
}

#[test]
fn line_actions_preserve_no_trailing_newline() {
    let out = run_action("reverse_lines", json!({}), "a\nb\nc");
    assert_eq!(out, "c\nb\na");
}

#[test]
fn trim_whitespace_right_keeps_crlf() {
    let out = run_action("trim_whitespace_right", json!({}), "a   \r\nb\t\r\n");
    assert_eq!(out, "a\r\nb\r\n");
}

#[test]
fn to_proper_lowercases_rest() {
    let out = run_action("to_proper", json!({}), "hELLO wORLD");
    assert_eq!(out, "Hello World");
}

#[test]
fn to_proper_blend_keeps_rest() {
    let out = run_action("to_proper", json!({ "blend": true }), "hELLO wORLD");
    assert_eq!(out, "HELLO WORLD");
}

#[test]
fn tabs_to_spaces_default_width() {
    let out = run_action("tabs_to_spaces", json!({}), "a\tb");
    assert_eq!(out, "a    b");
}

#[test]
fn squeeze_spaces_preserves_indent_and_tabs() {
    // Leading indentation is kept; interior runs of 2+ spaces collapse to one.
    let out = run_action(
        "squeeze_spaces",
        json!({}),
        "\t\tfoo   bar    baz\n    a  b",
    );
    assert_eq!(out, "\t\tfoo bar baz\n    a b");
}

#[test]
fn replace_whole_word_does_not_bite_into_tokens() {
    let out = run_action(
        "replace",
        json!({ "find": "datetime", "replace_with": "timestamp", "whole_word": true }),
        "datetime sysdatetime datetime2",
    );
    // Standalone only: not inside sysdatetime, and datetime2 keeps its suffix.
    assert_eq!(out, "timestamp sysdatetime datetime2");
}

#[test]
fn replace_whole_word_with_ignore_case_and_delimiters() {
    let out = run_action(
        "replace",
        json!({ "find": "bit", "replace_with": "boolean", "whole_word": true, "ignore_case": true }),
        "BIT bitmap [bit]",
    );
    assert_eq!(out, "boolean bitmap [boolean]");
}

#[test]
fn replace_regex_decode_replacement_makes_real_newline() {
    let out = run_action(
        "replace_regex",
        json!({ "find": "\\|", "replace_with": "\\n", "decode_replacement": true }),
        "a|b|c",
    );
    assert_eq!(out, "a\nb\nc");
}

#[test]
fn replace_regex_without_decode_keeps_escape_literal() {
    let out = run_action(
        "replace_regex",
        json!({ "find": "\\|", "replace_with": "\\n" }),
        "a|b",
    );
    assert_eq!(out, "a\\nb");
}

#[test]
fn replace_regex_decode_preserves_backrefs() {
    let out = run_action(
        "replace_regex",
        json!({ "find": r"(\w)=(\w)", "replace_with": "$1\\t$2", "decode_replacement": true }),
        "a=b",
    );
    assert_eq!(out, "a\tb");
}

#[test]
fn insert_before_and_after_matching_lines() {
    let before = run_action(
        "insert_before_matching",
        json!({ "pattern": "^CREATE", "text": "-- new" }),
        "x\nCREATE TABLE t\ny",
    );
    assert_eq!(before, "x\n-- new\nCREATE TABLE t\ny");

    let after = run_action(
        "insert_after_matching",
        json!({ "pattern": "^CREATE", "text": ";" }),
        "CREATE a\nCREATE b",
    );
    assert_eq!(after, "CREATE a\n;\nCREATE b\n;");
}

#[test]
fn unknown_action_errors() {
    let mog = parse_mog(r#"{ "steps": [ { "action": "nope" } ] }"#).unwrap();
    let err = execute(&mog, "x").unwrap_err();
    assert!(err.to_string().contains("Unknown action 'nope'"));
}

#[test]
fn disabled_step_is_skipped() {
    let mog = parse_mog(
        r#"{ "steps": [
            { "action": "to_upper", "disabled": true },
            { "action": "append", "options": { "text": "!" } }
        ] }"#,
    )
    .unwrap();
    let out = execute(&mog, "hi").unwrap();
    assert_eq!(out, "hi!");
}

#[test]
fn json_comments_are_rejected() {
    // .mog is strict JSON: a comment is a parse error, not silently ignored.
    let err = parse_mog(
        r#"{
            // leading comment
            "steps": [ { "action": "to_upper" } ]
        }"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("failed to parse .mog JSON"));
}

#[test]
fn trailing_commas_are_rejected() {
    // .mog is strict JSON: a trailing comma is a parse error.
    let err = parse_mog(r#"{ "steps": [ { "action": "to_upper" }, ] }"#).unwrap_err();
    assert!(err.to_string().contains("failed to parse .mog JSON"));
}
