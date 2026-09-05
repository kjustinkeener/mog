//! Phase 4 streaming: chunk-concat-safety classification.
//!
//! The streaming executor processes a file in large blocks split at newline
//! boundaries and concatenates the per-block outputs. That is byte-identical to
//! the whole-file path ONLY for actions where, for every split of the input at a
//! `\n` boundary into (a, b), `f(a) ++ f(b) == f(a ++ b)`. This test is the
//! arbiter of which actions get that property: the STREAMABLE list here must
//! match exactly what `mog::stream::action_is_streamable` allows, and the NEGATIVE
//! controls confirm the property really discriminates (they must NOT be safe).

use mog::{execute, parse_mog};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Run a single-step pipeline (given as a JSON step object) over `input`.
fn out(step_json: &str, input: &str) -> String {
    let mog = parse_mog(&format!(r#"{{"steps":[{step_json}]}}"#)).expect("step parses");
    execute(&mog, input).expect("step runs")
}

/// True iff `f(a) ++ f(b) == f(a ++ b)` for EVERY split of `input` at a `\n`
/// boundary (the byte just after each newline). Empty-split edges are covered by
/// the boundaries at the string ends implicitly.
fn concat_safe_on(step_json: &str, input: &str) -> bool {
    let whole = out(step_json, input);
    for (i, &byte) in input.as_bytes().iter().enumerate() {
        if byte == b'\n' {
            let (a, b) = input.split_at(i + 1);
            if out(step_json, a) + &out(step_json, b) != whole {
                return false;
            }
        }
    }
    true
}

/// Run a full pipeline (a JSON array of step objects) over `input`.
fn out_pipe(steps_json: &str, input: &str) -> String {
    let mog = parse_mog(&format!(r#"{{"steps":{steps_json}}}"#)).expect("pipeline parses");
    execute(&mog, input).expect("pipeline runs")
}

/// concat-safety for a WHOLE pipeline (not just one action). Single-action safety
/// is necessary but NOT sufficient: an early step that rewrites line terminators
/// (eol_cr: \n -> \r) or drops \n (keep/remove_chars, strip_control_chars) can
/// break a later \n-dependent step at the chunk boundary.
fn pipe_concat_safe(steps_json: &str, input: &str) -> bool {
    let whole = out_pipe(steps_json, input);
    for (i, &byte) in input.as_bytes().iter().enumerate() {
        if byte == b'\n' {
            let (a, b) = input.split_at(i + 1);
            if out_pipe(steps_json, a) + &out_pipe(steps_json, b) != whole {
                return false;
            }
        }
    }
    true
}

/// A batch of random inputs drawn from an alphabet rich in the boundary-relevant
/// bytes: newlines, CR, spaces, tabs, and a few content chars (letters, digits,
/// punctuation, an HTML angle, a multi-byte char).
fn corpus(seed: u64, n: usize) -> Vec<String> {
    let alphabet = [
        'a', 'B', 'c', '1', ' ', '\t', '\n', '\r', '.', '<', '>', '=', 'é',
    ];
    let mut rng = StdRng::seed_from_u64(seed);
    (0..n)
        .map(|_| {
            let len = rng.gen_range(0..40);
            (0..len)
                .map(|_| alphabet[rng.gen_range(0..alphabet.len())])
                .collect()
        })
        .collect()
}

/// Concrete single-step .mog snippets for actions that ARE chunk-concat safe:
/// they transform each line's content from ONLY that line's bytes AND preserve
/// every original line terminator byte-for-byte. Empirically verified below over
/// `\r`-heavy inputs. NOTE what is deliberately ABSENT and why:
///   - Actions using `line_text::split`+`join` (prefix_lines, indent, the per-line
///     filters, detectors, ...) are excluded: `join` normalizes ALL terminators to
///     the FIRST-detected EOL, making them whole-document EOL operations whose
///     output depends on the whole input's first line -- unsafe to chunk.
///   - Whole-string pattern actions (`replace`, `replace_regex`) are excluded: a
///     pattern can match across a `\n`.
///
/// The engine's `mog::stream::action_is_streamable` must equal exactly this set.
const STREAMABLE_STEPS: &[&str] = &[
    r#"{"action":"to_upper"}"#,
    r#"{"action":"to_lower"}"#,
    r#"{"action":"to_proper"}"#,
    r#"{"action":"invert_case"}"#,
    r#"{"action":"trim_whitespace"}"#,
    r#"{"action":"trim_whitespace_left"}"#,
    r#"{"action":"trim_whitespace_right"}"#,
    r#"{"action":"tabs_to_spaces"}"#,
    r#"{"action":"spaces_to_tabs"}"#,
    r#"{"action":"collapse_whitespace"}"#,
    r#"{"action":"keep_chars","options":{"set":"abc","keep_newlines":true}}"#,
    r#"{"action":"remove_chars","options":{"set":"x","keep_newlines":true}}"#,
    r#"{"action":"strip_control_chars"}"#,
    r#"{"action":"eol_lf"}"#,
    r#"{"action":"eol_crlf"}"#,
    r#"{"action":"eol_cr"}"#,
];

/// Action NAMES the engine classifies streamable, extracted from the steps above.
const STREAMABLE_NAMES: &[&str] = &[
    "to_upper",
    "to_lower",
    "to_proper",
    "invert_case",
    "trim_whitespace",
    "trim_whitespace_left",
    "trim_whitespace_right",
    "tabs_to_spaces",
    "spaces_to_tabs",
    "collapse_whitespace",
    "keep_chars",
    "remove_chars",
    "strip_control_chars",
    "eol_lf",
    "eol_crlf",
    "eol_cr",
];

/// Actions that must NOT be classified streamable: cross-line, whole-document,
/// EOL-normalizing (line_text::join), or stateful-across-lines. Discriminating
/// controls -- each must be caught as unsafe by some input.
const NON_STREAMABLE_STEPS: &[&str] = &[
    r#"{"action":"sort_lines"}"#,
    r#"{"action":"remove_duplicate_lines"}"#,
    r#"{"action":"reverse_lines"}"#,
    r#"{"action":"number_lines"}"#,   // running counter across lines
    r#"{"action":"squeeze_spaces"}"#, // empirically cross-boundary
    r#"{"action":"to_sentence"}"#,    // sentence state / EOL-normalizing
    r#"{"action":"prefix_lines","options":{"text":"> "}}"#, // line_text::join normalizes EOL
    r#"{"action":"indent","options":{"spaces":2}}"#, // line_text::join normalizes EOL
    r#"{"action":"remove_lines_matching","options":{"pattern":"a"}}"#, // line_text::join
    r#"{"action":"prepend","options":{"text":"HEAD\n"}}"#, // document head only
    r#"{"action":"append","options":{"text":"\nTAIL"}}"#, // document tail only
    r#"{"action":"base64_encode"}"#,  // whole-document codec
    r#"{"action":"strip_html_tags"}"#, // a tag can span a newline
    // A regex whose match spans the newline: chunking at \n splits the match.
    r#"{"action":"replace_regex","options":{"find":"a\\nb","replace_with":"X"}}"#,
];

/// Crafted inputs that stress the boundary directly: text that spans a `\n`,
/// blank/whitespace-only lines, CRLF, and no-trailing-newline. Random sampling
/// alone can miss a specific adjacency like "a\nb", so these are always included.
const ADVERSARIAL: &[&str] = &[
    "a\nb",
    "a\nb\n",
    "a a\n a a",
    "  \n  ",
    "\n\n\n",
    "a\r\nb\r\n",
    "HEAD\nx",
    "x\nHEAD",
    "<a\nhref>",
    "aXb\naYb",
    "\t\na\t",
    "abc",
    "",
];

/// The full input pool: random corpus plus the crafted adversarial cases.
fn input_pool(seed: u64, n: usize) -> Vec<String> {
    let mut v = corpus(seed, n);
    v.extend(ADVERSARIAL.iter().map(|s| s.to_string()));
    v
}

#[test]
fn streamable_steps_are_chunk_concat_safe() {
    let inputs = input_pool(0xC0FFEE, 400);
    let mut bad: Vec<String> = Vec::new();
    for step in STREAMABLE_STEPS {
        if let Some(input) = inputs.iter().find(|i| !concat_safe_on(step, i)) {
            bad.push(format!("  {step}  broke on {input:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "these were expected streamable but are concat-unsafe:\n{}",
        bad.join("\n")
    );
}

#[test]
fn engine_classification_matches_verified_set() {
    // Every name proven concat-safe above must be classified streamable...
    for name in STREAMABLE_NAMES {
        assert!(
            mog::stream::action_is_streamable(name),
            "{name} is concat-safe but action_is_streamable() rejects it"
        );
    }
    // ...and every unsafe control must be classified NOT streamable. Extract the
    // action name from each control step's JSON.
    for step in NON_STREAMABLE_STEPS {
        let v: serde_json::Value = serde_json::from_str(step).unwrap();
        let name = v["action"].as_str().unwrap();
        assert!(
            !mog::stream::action_is_streamable(name),
            "{name} is concat-UNSAFE but action_is_streamable() accepts it"
        );
    }
}

/// Actions that ALWAYS preserve every input `\n` byte, so a chunk boundary stays
/// valid after them and a later \n-dependent step is still safe. Must equal
/// `mog::stream::action_preserves_newlines`.
const NEWLINE_PRESERVING: &[&str] = &[
    "to_upper",
    "to_lower",
    "to_proper",
    "invert_case",
    "trim_whitespace",
    "trim_whitespace_left",
    "trim_whitespace_right",
    "tabs_to_spaces",
    "spaces_to_tabs",
    "collapse_whitespace",
    "strip_control_chars",
    "eol_lf",
    "eol_crlf",
];

#[test]
fn multistep_concat_safety_matches_expectation() {
    let pool = input_pool(0xABCD, 400);
    // A canary later step that depends on \n boundaries: if the preceding step
    // damaged the \n structure, [step, twl] stops being concat-safe.
    let twl = r#"{"action":"trim_whitespace_left"}"#;

    // Every NEWLINE_PRESERVING action keeps [action, twl] concat-safe...
    for name in NEWLINE_PRESERVING {
        let pipe = format!(r#"[{{"action":"{name}"}},{twl}]"#);
        assert!(
            pool.iter().all(|i| pipe_concat_safe(&pipe, i)),
            "{name} should preserve \\n boundaries but [{name}, twl] is concat-unsafe"
        );
    }

    // ...and the \n-altering streamable actions do NOT (worst-case options): eol_cr
    // rewrites \n->\r; keep_chars/remove_chars can drop \n.
    let breakers: &[&str] = &[
        r#"[{"action":"eol_cr"},{"action":"trim_whitespace_left"}]"#,
        r#"[{"action":"keep_chars","options":{"set":"abc ","keep_newlines":false}},{"action":"trim_whitespace_left"}]"#,
        r#"[{"action":"remove_chars","options":{"set":"\n","keep_newlines":false}},{"action":"trim_whitespace_left"}]"#,
    ];
    for pipe in breakers {
        assert!(
            !pool.iter().all(|i| pipe_concat_safe(pipe, i)),
            "expected concat-UNSAFE as a non-last step: {pipe}"
        );
    }
}

#[test]
fn execute_parallel_is_byte_identical_to_sequential() {
    let pipelines = [
        r#"[{"action":"to_upper"}]"#,
        r#"[{"action":"trim_whitespace"},{"action":"to_upper"},{"action":"tabs_to_spaces"}]"#,
        r#"[{"action":"collapse_whitespace"},{"action":"eol_crlf"}]"#,
    ];
    // Small random inputs plus one larger multi-line input so several chunks are
    // actually produced.
    let mut pool = input_pool(0x1234, 300);
    pool.push("row\n".repeat(500) + "tail no newline");

    for steps in pipelines {
        let mog = parse_mog(&format!(r#"{{"steps":{steps}}}"#)).unwrap();
        for input in &pool {
            let seq = execute(&mog, input).unwrap();
            for chunks in [1usize, 2, 3, 7, 16] {
                let par = mog::stream::execute_parallel(&mog, input, chunks).unwrap();
                assert_eq!(
                    par,
                    seq,
                    "parallel != sequential: steps={steps} chunks={chunks} input_len={}",
                    input.len()
                );
            }
        }
    }
}

#[test]
fn pipeline_streamability_respects_newline_preservation() {
    // NEWLINE_PRESERVING must equal action_preserves_newlines.
    for name in NEWLINE_PRESERVING {
        assert!(
            mog::stream::action_preserves_newlines(name),
            "{name} listed preserving but action_preserves_newlines rejects it"
        );
    }
    for name in ["eol_cr", "keep_chars", "remove_chars"] {
        assert!(
            !mog::stream::action_preserves_newlines(name),
            "{name} can alter \\n; must not be classified newline-preserving"
        );
    }

    // Whole-pipeline gate: a \n-altering step is allowed only as the last step.
    let cases: &[(&str, bool)] = &[
        (
            r#"[{"action":"trim_whitespace"},{"action":"to_upper"}]"#,
            true,
        ),
        (r#"[{"action":"to_upper"},{"action":"eol_cr"}]"#, true), // altering step last: ok
        (
            r#"[{"action":"eol_cr"},{"action":"trim_whitespace_left"}]"#,
            false, // altering step not last: rejected
        ),
        (
            r#"[{"action":"keep_chars","options":{"set":"a","keep_newlines":false}},{"action":"to_upper"}]"#,
            false,
        ),
        (
            r#"[{"action":"to_upper"},{"action":"keep_chars","options":{"set":"a"}}]"#,
            true, // keep_chars last: ok
        ),
        (r#"[{"action":"sort_lines"}]"#, false), // not streamable at all
    ];
    for (steps, expected) in cases {
        let mog = parse_mog(&format!(r#"{{"steps":{steps}}}"#)).unwrap();
        assert_eq!(
            mog::stream::pipeline_is_streamable(&mog),
            *expected,
            "pipeline_is_streamable mismatch: {steps}"
        );
    }
}

#[test]
fn non_streamable_steps_are_detected_as_unsafe() {
    // Each control must be caught as unsafe by SOME input, proving the property
    // discriminates rather than trivially passing everything.
    for step in NON_STREAMABLE_STEPS {
        let caught = input_pool(0xBEEF, 400)
            .iter()
            .any(|input| !concat_safe_on(step, input));
        assert!(
            caught,
            "control should be unsafe but looked safe: step={step}"
        );
    }
}
