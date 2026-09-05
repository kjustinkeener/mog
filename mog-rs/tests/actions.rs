//! Deterministic per-action unit tests. These complement `golden.rs`, covering
//! the actions that had no dedicated test. Non-deterministic actions
//! (`shuffle_lines`, `random_case`) are intentionally NOT tested here.
//!
//! Expected values are derived by reading the action source, which is the source
//! of truth for the engine's behavior.

use mog::model::Step;
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

// ---------------------------------------------------------------------------
// replace_regex / replace_regex_multiline
// ---------------------------------------------------------------------------

#[test]
fn replace_regex_capture_group_dollar_one() {
    // Two captures, reordered via $1/$2 expansion.
    let out = run_action(
        "replace_regex",
        json!({ "find": r"(\w+) (\w+)", "replace_with": "$2-$1" }),
        "hello world",
    );
    assert_eq!(out, "world-hello");
}

#[test]
fn replace_regex_multiline_caret_matches_each_line() {
    // With (?m), `^` matches at the start of every line, so every line gets the
    // prefix. (Without multiline, only the very first line would.)
    let out = run_action(
        "replace_regex_multiline",
        json!({ "find": "^", "replace_with": "# " }),
        "foo\nbar\nbaz",
    );
    assert_eq!(out, "# foo\n# bar\n# baz");
}

#[test]
fn replace_regex_multiline_dollar_matches_each_line_end() {
    // `$` under (?m) matches before each newline and at end of string.
    let out = run_action(
        "replace_regex_multiline",
        json!({ "find": "$", "replace_with": ";" }),
        "a\nb\nc",
    );
    assert_eq!(out, "a;\nb;\nc;");
}

// ---------------------------------------------------------------------------
// remove_empty_lines
// ---------------------------------------------------------------------------

#[test]
fn remove_empty_lines_keeps_whitespace_only_lines_by_default() {
    // Default (include_whitespace = false) only drops truly empty lines; a line
    // of spaces is preserved.
    let out = run_action("remove_empty_lines", json!({}), "a\n\nb\n   \nc");
    assert_eq!(out, "a\nb\n   \nc");
}

#[test]
fn remove_empty_lines_include_whitespace_drops_blank_lines() {
    let out = run_action(
        "remove_empty_lines",
        json!({ "include_whitespace": true }),
        "a\n\nb\n   \nc",
    );
    assert_eq!(out, "a\nb\nc");
}

// ---------------------------------------------------------------------------
// remove_consecutive_duplicate_lines
// ---------------------------------------------------------------------------

#[test]
fn remove_consecutive_duplicate_lines_collapses_adjacent() {
    // Only adjacent duplicates collapse; the final "a" survives because it is
    // not adjacent to the earlier "a".
    let out = run_action(
        "remove_consecutive_duplicate_lines",
        json!({}),
        "a\na\nb\nb\nb\na",
    );
    assert_eq!(out, "a\nb\na");
}

// ---------------------------------------------------------------------------
// join_lines
// ---------------------------------------------------------------------------

#[test]
fn join_lines_with_separator() {
    let out = run_action("join_lines", json!({ "separator": ", " }), "a\nb\nc");
    assert_eq!(out, "a, b, c");
}

#[test]
fn join_lines_default_no_separator() {
    let out = run_action("join_lines", json!({}), "a\nb\nc");
    assert_eq!(out, "abc");
}

#[test]
fn join_lines_reappends_trailing_eol() {
    // A trailing newline in the input is preserved after joining.
    let out = run_action("join_lines", json!({}), "a\nb\n");
    assert_eq!(out, "ab\n");
}

// ---------------------------------------------------------------------------
// whitespace: trim_left / trim_both / tabs / spaces / eol_to_space
// ---------------------------------------------------------------------------

#[test]
fn trim_whitespace_left_strips_leading_spaces_and_tabs() {
    let out = run_action("trim_whitespace_left", json!({}), "  a\n\tb\n   c");
    assert_eq!(out, "a\nb\nc");
}

#[test]
fn trim_whitespace_strips_both_ends() {
    let out = run_action("trim_whitespace", json!({}), "  hello  \n  world  ");
    assert_eq!(out, "hello\nworld");
}

#[test]
fn tabs_to_spaces_custom_width() {
    let out = run_action("tabs_to_spaces", json!({ "width": 2 }), "a\tb");
    assert_eq!(out, "a  b");
}

#[test]
fn spaces_to_tabs_all_runs() {
    // Each exact run of `width` (default 4) spaces becomes one tab.
    let out = run_action("spaces_to_tabs", json!({}), "a    b");
    assert_eq!(out, "a\tb");
}

#[test]
fn spaces_to_tabs_leading_only() {
    // Only the leading run is converted; interior spaces are untouched.
    let out = run_action(
        "spaces_to_tabs",
        json!({ "leading_only": true }),
        "    a    b",
    );
    assert_eq!(out, "\ta    b");
}

#[test]
fn spaces_to_tabs_leading_only_keeps_remainder_spaces() {
    // 6 leading spaces at width 4 -> one tab + two remainder spaces.
    let out = run_action("spaces_to_tabs", json!({ "leading_only": true }), "      x");
    assert_eq!(out, "\t  x");
}

#[test]
fn eol_to_space_converts_all_line_endings() {
    let out = run_action("eol_to_space", json!({}), "a\nb\r\nc\rd");
    assert_eq!(out, "a b c d");
}

#[test]
fn trim_and_eol_to_space_trims_then_joins() {
    // Each line is trimmed, joined with single spaces, trailing newline dropped.
    let out = run_action("trim_and_eol_to_space", json!({}), "  a  \n  b  \n");
    assert_eq!(out, "a b");
}

// ---------------------------------------------------------------------------
// eol_cr
// ---------------------------------------------------------------------------

#[test]
fn eol_cr_converts_to_carriage_returns() {
    let out = run_action("eol_cr", json!({}), "a\nb\r\nc");
    assert_eq!(out, "a\rb\rc");
}

// ---------------------------------------------------------------------------
// case: to_upper / to_lower / to_sentence / invert_case
// ---------------------------------------------------------------------------

#[test]
fn to_upper_uppercases() {
    let out = run_action("to_upper", json!({}), "Hello World");
    assert_eq!(out, "HELLO WORLD");
}

#[test]
fn to_lower_lowercases() {
    let out = run_action("to_lower", json!({}), "Hello World");
    assert_eq!(out, "hello world");
}

#[test]
fn to_sentence_lowercases_then_capitalizes_each_sentence() {
    // blend = false lowercases the whole text first, then capitalizes the first
    // letter of each sentence (after . ! ?).
    let out = run_action("to_sentence", json!({}), "hELLO. wORLD! fOO? bar");
    assert_eq!(out, "Hello. World! Foo? Bar");
}

#[test]
fn to_sentence_blend_keeps_remainder_case() {
    // blend = true keeps the remainder untouched, only forcing the first letter
    // of each sentence to uppercase.
    let out = run_action("to_sentence", json!({ "blend": true }), "hELLO. wORLD!");
    assert_eq!(out, "HELLO. WORLD!");
}

#[test]
fn invert_case_swaps_per_char() {
    let out = run_action("invert_case", json!({}), "Hello World");
    assert_eq!(out, "hELLO wORLD");
}

// ---------------------------------------------------------------------------
// affix: prepend / append
// ---------------------------------------------------------------------------

#[test]
fn prepend_adds_text_in_front() {
    let out = run_action("prepend", json!({ "text": ">> " }), "world");
    assert_eq!(out, ">> world");
}

#[test]
fn append_adds_text_at_end() {
    let out = run_action("append", json!({ "text": "!" }), "hello");
    assert_eq!(out, "hello!");
}

// ---------------------------------------------------------------------------
// alias resolution: aliases must behave identically to their canonical names
// ---------------------------------------------------------------------------

#[test]
fn alias_r_matches_replace() {
    let opts = json!({ "find": "cat", "replace_with": "dog" });
    let canonical = run_action("replace", opts.clone(), "cat cat");
    let alias = run_action("r", opts, "cat cat");
    assert_eq!(alias, canonical);
    assert_eq!(alias, "dog dog");
}

#[test]
fn alias_dedupe_matches_remove_duplicate_lines() {
    let input = "a\nb\na\nc\nb";
    let canonical = run_action("remove_duplicate_lines", json!({}), input);
    let alias = run_action("dedupe", json!({}), input);
    assert_eq!(alias, canonical);
    assert_eq!(alias, "a\nb\nc");
}

#[test]
fn alias_sort_matches_sort_lines() {
    let input = "banana\napple\ncherry";
    let canonical = run_action("sort_lines", json!({}), input);
    let alias = run_action("sort", json!({}), input);
    assert_eq!(alias, canonical);
    assert_eq!(alias, "apple\nbanana\ncherry");
}
