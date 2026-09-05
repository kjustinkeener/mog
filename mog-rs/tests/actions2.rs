//! Deterministic tests for the newer batch of actions (line filtering, per-line
//! affixing, numbering, blank/whitespace, encoding), scoped steps, and the
//! `run_mog` compose action.

use std::fs;

use mog::model::Step;
use mog::{execute_at, parse_mog};
use serde_json::{json, Value};
use tempfile::tempdir;

/// Build a plain step (no scope) from an action name + options.
fn step(action: &str, options: Value) -> Step {
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

fn run_action(action: &str, options: Value, input: &str) -> String {
    let f = mog::engine::resolve(action).expect("action resolves");
    f(input, &step(action, options)).expect("action runs")
}

// ---------------------------------------------------------------------------
// replace_extended (Notepad++ "Extended" mode: literal replace with escapes)
// ---------------------------------------------------------------------------

#[test]
fn replace_extended_crlf_in_find_matches_real_crlf() {
    // The key case: `\r\n` in `find` decodes to a real CR LF and matches it.
    let input = "x a\r\nb y"; // contains actual CR LF between "a" and "b"
    let out = run_action(
        "replace_extended",
        json!({ "find": "a\\r\\nb", "replace_with": "Z" }),
        input,
    );
    assert_eq!(out, "x Z y");
}

#[test]
fn replace_extended_tab_in_find_matches_real_tab() {
    let input = "x\ty"; // real tab
    let out = run_action(
        "replace_extended",
        json!({ "find": "x\\ty", "replace_with": "Q" }),
        input,
    );
    assert_eq!(out, "Q");
}

#[test]
fn replace_extended_hex_escape_matches_literal_a() {
    // \x41 decodes to "A".
    let out = run_action(
        "replace_extended",
        json!({ "find": "\\x41", "replace_with": "Q" }),
        "bAd",
    );
    assert_eq!(out, "bQd");
}

#[test]
fn replace_extended_unicode_escape_decodes() {
    // é decodes to "é".
    let out = run_action(
        "replace_extended",
        json!({ "find": "\\u00e9", "replace_with": "e" }),
        "caf\u{00e9}",
    );
    assert_eq!(out, "cafe");
}

#[test]
fn replace_extended_unknown_escape_stays_literal() {
    // `\q` stays the two literal chars backslash-q: it matches a literal
    // backslash-q in the input and does NOT match a bare "q".
    let matched = run_action(
        "replace_extended",
        json!({ "find": "\\q", "replace_with": "Z" }),
        "a\\qb",
    );
    assert_eq!(matched, "aZb");

    let bare_q_untouched = run_action(
        "replace_extended",
        json!({ "find": "\\q", "replace_with": "Z" }),
        "aqb",
    );
    assert_eq!(bare_q_untouched, "aqb");
}

#[test]
fn replace_extended_decodes_escapes_in_replace_with() {
    // A `\n` in replace_with becomes a real newline.
    let out = run_action(
        "replace_extended",
        json!({ "find": "y", "replace_with": "x\\ny" }),
        "y",
    );
    assert_eq!(out, "x\ny");
}

#[test]
fn replace_extended_ignore_case_on_decoded_strings() {
    // find decodes to "A<tab>B"; with ignore_case it matches "a<tab>b".
    let out = run_action(
        "replace_extended",
        json!({ "find": "A\\tB", "replace_with": "Z", "ignore_case": true }),
        "a\tb",
    );
    assert_eq!(out, "Z");
}

#[test]
fn replace_extended_double_backslash_decodes_to_single() {
    // `\\` decodes to one backslash, matching a literal backslash in the input.
    let out = run_action(
        "replace_extended",
        json!({ "find": "a\\\\b", "replace_with": "/" }),
        "a\\b",
    );
    assert_eq!(out, "/");
}

// ---------------------------------------------------------------------------
// keep_lines_matching / remove_lines_matching
// ---------------------------------------------------------------------------

#[test]
fn keep_lines_matching_filters_to_matches() {
    let out = run_action(
        "keep_lines_matching",
        json!({ "pattern": "ERROR|WARN" }),
        "INFO ok\nERROR boom\nWARN careful\nDEBUG noise",
    );
    assert_eq!(out, "ERROR boom\nWARN careful");
}

#[test]
fn keep_lines_matching_context_keeps_neighbours() {
    let input = "a
b HIT
c
d
e HIT
f";
    let both = run_action(
        "keep_lines_matching",
        json!({ "pattern": "HIT", "context": 1 }),
        input,
    );
    assert_eq!(both, input, "one line each side covers every line here");
    let before_only = run_action(
        "keep_lines_matching",
        json!({ "pattern": "HIT", "before": 2, "after": 0 }),
        input,
    );
    assert_eq!(
        before_only,
        "a
b HIT
c
d
e HIT"
    );
}

#[test]
fn keep_lines_matching_context_windows_do_not_duplicate() {
    // Both matches claim the middle line; it must appear once, in order.
    let out = run_action(
        "keep_lines_matching",
        json!({ "pattern": "HIT", "context": 1 }),
        "a HIT
b
c HIT",
    );
    assert_eq!(
        out,
        "a HIT
b
c HIT"
    );
}

#[test]
fn keep_lines_matching_context_clamps_at_edges() {
    let out = run_action(
        "keep_lines_matching",
        json!({ "pattern": "HIT", "context": 5 }),
        "a
HIT
b",
    );
    assert_eq!(
        out,
        "a
HIT
b"
    );
}

#[test]
fn keep_lines_matching_ignore_case() {
    let out = run_action(
        "keep_lines_matching",
        json!({ "pattern": "error", "ignore_case": true }),
        "Error one\nokay\nERROR two",
    );
    assert_eq!(out, "Error one\nERROR two");
}

#[test]
fn remove_lines_matching_drops_matches_and_preserves_trailing_eol() {
    let out = run_action(
        "remove_lines_matching",
        json!({ "pattern": "^\\s*#" }),
        "# comment\nkeep me\n  # indented comment\nalso keep\n",
    );
    assert_eq!(out, "keep me\nalso keep\n");
}

// ---------------------------------------------------------------------------
// prefix_lines / suffix_lines / wrap_lines
// ---------------------------------------------------------------------------

#[test]
fn prefix_lines_adds_text_to_each_line() {
    let out = run_action("prefix_lines", json!({ "text": "> " }), "a\nb\nc");
    assert_eq!(out, "> a\n> b\n> c");
}

#[test]
fn suffix_lines_adds_text_to_each_line() {
    let out = run_action("suffix_lines", json!({ "text": ";" }), "a\nb");
    assert_eq!(out, "a;\nb;");
}

#[test]
fn wrap_lines_wraps_each_line() {
    let out = run_action(
        "wrap_lines",
        json!({ "prefix": "[", "suffix": "]" }),
        "a\nb",
    );
    assert_eq!(out, "[a]\n[b]");
}

// ---------------------------------------------------------------------------
// number_lines
// ---------------------------------------------------------------------------

#[test]
fn number_lines_defaults_pad_to_equal_width() {
    // 12 lines => widths 1 and 2 mix; numbers are right-aligned to width 2.
    let input = (0..12).map(|_| "x").collect::<Vec<_>>().join("\n");
    let out = run_action("number_lines", json!({}), &input);
    let first = out.lines().next().unwrap();
    let last = out.lines().last().unwrap();
    assert_eq!(first, " 1. x");
    assert_eq!(last, "12. x");
}

#[test]
fn number_lines_custom_start_and_separator() {
    let out = run_action(
        "number_lines",
        json!({ "start": 5, "separator": ") " }),
        "a\nb\nc",
    );
    assert_eq!(out, "5) a\n6) b\n7) c");
}

// ---------------------------------------------------------------------------
// squeeze_blank_lines
// ---------------------------------------------------------------------------

#[test]
fn squeeze_blank_lines_collapses_runs() {
    let out = run_action("squeeze_blank_lines", json!({}), "a\n\n\n\nb\n\nc");
    assert_eq!(out, "a\n\nb\n\nc");
}

#[test]
fn squeeze_blank_lines_include_whitespace() {
    // Whitespace-only lines count as blank when include_whitespace is set.
    let out = run_action(
        "squeeze_blank_lines",
        json!({ "include_whitespace": true }),
        "a\n\n   \n\t\nb",
    );
    assert_eq!(out, "a\n\nb");
}

// ---------------------------------------------------------------------------
// collapse_whitespace
// ---------------------------------------------------------------------------

#[test]
fn collapse_whitespace_collapses_runs_per_line() {
    let out = run_action("collapse_whitespace", json!({}), "a   b\t\tc\n d    e");
    assert_eq!(out, "a b c\n d e");
}

// ---------------------------------------------------------------------------
// indent / outdent
// ---------------------------------------------------------------------------

#[test]
fn indent_default_four_spaces() {
    let out = run_action("indent", json!({}), "a\nb");
    assert_eq!(out, "    a\n    b");
}

#[test]
fn indent_with_custom_text() {
    let out = run_action("indent", json!({ "text": "// " }), "a\nb");
    assert_eq!(out, "// a\n// b");
}

#[test]
fn outdent_removes_up_to_spaces() {
    // Line 1 has 6 leading spaces; only 4 are removed. Line 2 has 2 (all gone).
    let out = run_action("outdent", json!({}), "      a\n  b\nc");
    assert_eq!(out, "  a\nb\nc");
}

#[test]
fn outdent_collapses_leading_tab_as_one_level() {
    let out = run_action("outdent", json!({}), "\tindented\nplain");
    assert_eq!(out, "indented\nplain");
}

// ---------------------------------------------------------------------------
// encoding helpers
// ---------------------------------------------------------------------------

#[test]
fn url_encode_and_decode_round_trip() {
    let encoded = run_action("url_encode", json!({}), "a b/c?d=e&f");
    assert_eq!(encoded, "a%20b%2Fc%3Fd%3De%26f");
    let decoded = run_action("url_decode", json!({}), &encoded);
    assert_eq!(decoded, "a b/c?d=e&f");
}

#[test]
fn html_encode_escapes_five_characters() {
    let out = run_action("html_encode", json!({}), r#"<a href="x">'&'</a>"#);
    assert_eq!(out, "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;");
}

#[test]
fn html_decode_reverses_encode() {
    let original = r#"<tag attr="v">a & b</tag>"#;
    let encoded = run_action("html_encode", json!({}), original);
    let decoded = run_action("html_decode", json!({}), &encoded);
    assert_eq!(decoded, original);
}

#[test]
fn base64_encode_and_decode_round_trip() {
    let encoded = run_action("base64_encode", json!({}), "Hello, mog!");
    assert_eq!(encoded, "SGVsbG8sIG1vZyE=");
    let decoded = run_action("base64_decode", json!({}), &encoded);
    assert_eq!(decoded, "Hello, mog!");
}

// ---------------------------------------------------------------------------
// Part B: scoped steps (only_lines_matching / except_lines_matching)
// ---------------------------------------------------------------------------

#[test]
fn scope_only_lines_matching_uppercases_just_matches() {
    let mog = parse_mog(
        r#"{ "steps": [
            { "action": "to_upper", "only_lines_matching": "error" }
        ] }"#,
    )
    .unwrap();
    let out = execute_at(&mog, "an error here\nall fine\nanother error", None).unwrap();
    assert_eq!(out, "AN ERROR HERE\nall fine\nANOTHER ERROR");
}

#[test]
fn scope_except_lines_matching_skips_matches() {
    // replace applies to every line EXCEPT those matching "keep".
    let mog = parse_mog(
        r#"{ "steps": [
            { "action": "replace",
              "options": { "find": "cat", "replace_with": "dog" },
              "except_lines_matching": "keep" }
        ] }"#,
    )
    .unwrap();
    let out = execute_at(&mog, "cat one\nkeep cat\ncat two", None).unwrap();
    assert_eq!(out, "dog one\nkeep cat\ndog two");
}

#[test]
fn scope_match_ignore_case() {
    let mog = parse_mog(
        r#"{ "steps": [
            { "action": "to_upper",
              "only_lines_matching": "todo",
              "match_ignore_case": true }
        ] }"#,
    )
    .unwrap();
    let out = execute_at(&mog, "TODO first\nplain\ntodo second", None).unwrap();
    assert_eq!(out, "TODO FIRST\nplain\nTODO SECOND");
}

#[test]
fn scope_preserves_crlf_and_trailing_newline() {
    let mog = parse_mog(
        r#"{ "steps": [
            { "action": "prefix_lines",
              "options": { "text": "> " },
              "only_lines_matching": "a" }
        ] }"#,
    )
    .unwrap();
    let out = execute_at(&mog, "apple\r\nberry\r\n", None).unwrap();
    assert_eq!(out, "> apple\r\nberry\r\n");
}

// ---------------------------------------------------------------------------
// Part C: run_mog compose action
// ---------------------------------------------------------------------------

#[test]
fn run_mog_runs_child_inline() {
    let dir = tempdir().unwrap();
    let child = r#"{ "steps": [ { "action": "to_upper" } ] }"#;
    fs::write(dir.path().join("child.mog"), child).unwrap();

    let parent = r#"{ "steps": [
        { "action": "prepend", "options": { "text": ">> " } },
        { "action": "run_mog", "options": { "file": "child.mog" } }
    ] }"#;
    let mog = parse_mog(parent).unwrap();

    let out = execute_at(&mog, "hello", Some(dir.path())).unwrap();
    assert_eq!(out, ">> HELLO");
}

#[test]
fn run_mog_with_passes_constants_to_child() {
    let dir = tempdir().unwrap();
    // The child declares a default `marker` constant; a caller can override it.
    let child = r##"{ "constants": { "marker": "# " },
        "steps": [ { "action": "prefix_lines", "options": { "text": "{{marker}}" } } ] }"##;
    fs::write(dir.path().join("child.mog"), child).unwrap();

    // No `with`: the child uses its own constant.
    let p_default =
        r#"{ "steps": [ { "action": "run_mog", "options": { "file": "child.mog" } } ] }"#;
    let out = execute_at(&parse_mog(p_default).unwrap(), "a\nb", Some(dir.path())).unwrap();
    assert_eq!(out, "# a\n# b");

    // `with` overrides the child's constant for this call.
    let p_with = r#"{ "steps": [ { "action": "run_mog",
        "options": { "file": "child.mog", "with": { "marker": ">> " } } } ] }"#;
    let out = execute_at(&parse_mog(p_with).unwrap(), "a\nb", Some(dir.path())).unwrap();
    assert_eq!(out, ">> a\n>> b");
}

#[test]
fn run_mog_without_base_dir_errors() {
    let mog =
        parse_mog(r#"{ "steps": [ { "action": "run_mog", "options": { "file": "x.mog" } } ] }"#)
            .unwrap();
    let err = execute_at(&mog, "hi", None).unwrap_err();
    assert!(err.to_string().contains("base directory"));
}

#[test]
fn run_mog_recursion_limit_is_enforced() {
    // A .mog that runs itself must hit the depth guard rather than looping.
    let dir = tempdir().unwrap();
    let self_ref = r#"{ "steps": [ { "action": "run_mog", "options": { "file": "loop.mog" } } ] }"#;
    fs::write(dir.path().join("loop.mog"), self_ref).unwrap();

    let mog = parse_mog(self_ref).unwrap();
    let err = execute_at(&mog, "hi", Some(dir.path())).unwrap_err();
    assert!(err.to_string().contains("recursion limit"));
}
