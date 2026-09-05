//! Exact Rust-flavor regex matcher for Mog Studio's live regex tester.
//!
//! Mirrors mog's `run_regex` engine selection (mog-rs `src/actions/replace.rs`):
//! try the linear `regex` crate first; if it rejects the pattern, fall back to the
//! backtracking `fancy-regex`. A pattern only `fancy-regex` accepts uses
//! backreferences / lookaround and runs on the O(n^2) engine -- the same
//! classification mog surfaces as its complexity warning. Because we use the same
//! crates at the same versions and the same `(?i)`/`(?m)` prefixing, the match
//! spans reported here are what `mog.exe` will actually act on -- no JS-flavor drift.

use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Cap on matches returned to the UI, so a pathological pattern on a large input
/// can't flood the highlighter. Signalled to the caller via `truncated`.
const MATCH_CAP: usize = 5000;

#[derive(Serialize)]
struct Group {
    /// Named-capture name, if this group is named.
    name: Option<String>,
    /// UTF-16 offset into the haystack (JS string index), or null when the group
    /// did not participate in the match.
    start: Option<usize>,
    end: Option<usize>,
}

#[derive(Serialize)]
struct MatchSpan {
    /// UTF-16 offsets of the whole match (group 0).
    start: usize,
    end: usize,
    /// Capture groups; index 0 is the whole match.
    groups: Vec<Group>,
}

#[derive(Serialize)]
struct Analysis {
    /// True when the pattern compiled (on either engine).
    ok: bool,
    /// "linear" (regex crate), "backtracking" (fancy-regex), "invalid" (neither),
    /// or "empty" (no pattern typed yet).
    engine: &'static str,
    /// Compile error text (from fancy-regex, matching what mog reports) when invalid.
    error: Option<String>,
    /// Capture-group names by index (index 0 = whole match, always None), for the
    /// group legend in the UI.
    group_names: Vec<Option<String>>,
    matches: Vec<MatchSpan>,
    /// True when matching stopped at MATCH_CAP.
    truncated: bool,
}

/// Prepend the inline `(?i)` / `(?m)` flags exactly as mog's `build_pattern` does,
/// so the compiled pattern is identical to the one mog runs.
fn build_pattern(find: &str, ignore_case: bool, multiline: bool) -> String {
    let mut p = String::new();
    if ignore_case {
        p.push_str("(?i)");
    }
    if multiline {
        p.push_str("(?m)");
    }
    p.push_str(find);
    p
}

/// Map a set of UTF-8 byte offsets to UTF-16 code-unit offsets (JS string indices)
/// in a single linear scan. Rust regex spans are byte offsets; JS strings are
/// UTF-16, so highlighting non-ASCII text needs this conversion.
fn utf16_offsets(haystack: &str, wanted: &[usize]) -> HashMap<usize, usize> {
    let mut w: Vec<usize> = wanted.to_vec();
    w.sort_unstable();
    w.dedup();
    let mut map = HashMap::with_capacity(w.len());
    let mut wi = 0usize;
    let mut u16 = 0usize;
    for (b, ch) in haystack.char_indices() {
        while wi < w.len() && w[wi] == b {
            map.insert(b, u16);
            wi += 1;
        }
        u16 += ch.len_utf16();
    }
    // Offsets landing at end-of-string (e.g. a match ending at the last char).
    let end = haystack.len();
    while wi < w.len() && w[wi] == end {
        map.insert(end, u16);
        wi += 1;
    }
    map
}

/// Raw per-match group byte-spans (index 0 = whole match); None = group absent.
type RawMatches = Vec<Vec<Option<(usize, usize)>>>;

fn run_linear(re: &regex::Regex, haystack: &str) -> (RawMatches, bool) {
    let mut raw = Vec::new();
    let mut truncated = false;
    for caps in re.captures_iter(haystack) {
        if raw.len() >= MATCH_CAP {
            truncated = true;
            break;
        }
        let mut groups = Vec::with_capacity(caps.len());
        for i in 0..caps.len() {
            groups.push(caps.get(i).map(|m| (m.start(), m.end())));
        }
        raw.push(groups);
    }
    (raw, truncated)
}

fn run_fancy(re: &fancy_regex::Regex, haystack: &str) -> (RawMatches, bool) {
    let mut raw = Vec::new();
    let mut truncated = false;
    for caps in re.captures_iter(haystack) {
        if raw.len() >= MATCH_CAP {
            truncated = true;
            break;
        }
        // fancy-regex yields Result per step; stop cleanly on a runtime error.
        let caps = match caps {
            Ok(c) => c,
            Err(_) => break,
        };
        let mut groups = Vec::with_capacity(caps.len());
        for i in 0..caps.len() {
            groups.push(caps.get(i).map(|m| (m.start(), m.end())));
        }
        raw.push(groups);
    }
    (raw, truncated)
}

fn to_spans(haystack: &str, raw: RawMatches, names: &[Option<String>]) -> Vec<MatchSpan> {
    // Gather every byte offset we need, convert them all in one pass.
    let mut offs = Vec::new();
    for m in &raw {
        for g in m.iter().flatten() {
            offs.push(g.0);
            offs.push(g.1);
        }
    }
    let map = utf16_offsets(haystack, &offs);
    raw.into_iter()
        .map(|m| {
            let groups: Vec<Group> = m
                .iter()
                .enumerate()
                .map(|(i, g)| {
                    let name = names.get(i).cloned().flatten();
                    match g {
                        Some((s, e)) => Group {
                            name,
                            start: Some(map[s]),
                            end: Some(map[e]),
                        },
                        None => Group {
                            name,
                            start: None,
                            end: None,
                        },
                    }
                })
                .collect();
            // Group 0 always participates in a match.
            let (ws, we) = m[0].expect("group 0 present in every match");
            MatchSpan {
                start: map[&ws],
                end: map[&we],
                groups,
            }
        })
        .collect()
}

fn analyze_inner(find: &str, haystack: &str, ignore_case: bool, multiline: bool) -> Analysis {
    if find.is_empty() {
        return Analysis {
            ok: true,
            engine: "empty",
            error: None,
            group_names: vec![],
            matches: vec![],
            truncated: false,
        };
    }
    let pattern = build_pattern(find, ignore_case, multiline);

    // Fast path: the linear engine. If it compiles, mog uses it too.
    if let Ok(re) = regex::Regex::new(&pattern) {
        let names: Vec<Option<String>> = re
            .capture_names()
            .map(|o| o.map(|s| s.to_string()))
            .collect();
        let (raw, truncated) = run_linear(&re, haystack);
        let matches = to_spans(haystack, raw, &names);
        return Analysis {
            ok: true,
            engine: "linear",
            error: None,
            group_names: names,
            matches,
            truncated,
        };
    }

    // Fallback: the backtracking engine (backreferences / lookaround).
    match fancy_regex::Regex::new(&pattern) {
        Ok(re) => {
            let names: Vec<Option<String>> = re
                .capture_names()
                .map(|o| o.map(|s| s.to_string()))
                .collect();
            let (raw, truncated) = run_fancy(&re, haystack);
            let matches = to_spans(haystack, raw, &names);
            Analysis {
                ok: true,
                engine: "backtracking",
                error: None,
                group_names: names,
                matches,
                truncated,
            }
        }
        Err(e) => Analysis {
            ok: false,
            engine: "invalid",
            error: Some(e.to_string()),
            group_names: vec![],
            matches: vec![],
            truncated: false,
        },
    }
}

#[derive(Serialize)]
struct Substitution {
    ok: bool,
    engine: &'static str,
    error: Option<String>,
    /// The substituted text (unchanged input on error / empty pattern).
    output: String,
    /// True when the substitution altered the input.
    changed: bool,
}

/// Apply `replace` to every match of `find`, mirroring mog's `run_regex` replace
/// path exactly: the linear `regex` crate when it accepts the pattern, else
/// `fancy-regex`. Both expand `$1` / `${name}` / `$$` in the replacement
/// identically, so the output is byte-identical to what `mog.exe` produces for
/// `replace_regex` (this does NOT apply the `decode_replacement` option; the
/// tester previews the default, undecoded replacement).
fn substitute_inner(
    find: &str,
    replace: &str,
    haystack: &str,
    ignore_case: bool,
    multiline: bool,
) -> Substitution {
    if find.is_empty() {
        return Substitution {
            ok: true,
            engine: "empty",
            error: None,
            output: haystack.to_string(),
            changed: false,
        };
    }
    let pattern = build_pattern(find, ignore_case, multiline);

    if let Ok(re) = regex::Regex::new(&pattern) {
        let out = re.replace_all(haystack, replace).into_owned();
        let changed = out != haystack;
        return Substitution {
            ok: true,
            engine: "linear",
            error: None,
            changed,
            output: out,
        };
    }
    match fancy_regex::Regex::new(&pattern) {
        Ok(re) => {
            let out = re.replace_all(haystack, replace).into_owned();
            let changed = out != haystack;
            Substitution {
                ok: true,
                engine: "backtracking",
                error: None,
                changed,
                output: out,
            }
        }
        Err(e) => Substitution {
            ok: false,
            engine: "invalid",
            error: Some(e.to_string()),
            output: haystack.to_string(),
            changed: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_linear() {
        let s = substitute_inner("l+", "L", "hello", false, false);
        assert_eq!(s.engine, "linear");
        assert_eq!(s.output, "heLo");
        assert!(s.changed);
    }

    #[test]
    fn substitute_expands_group_refs() {
        // $2.$1 dollar-syntax expansion, same as mog.
        let s = substitute_inner(r"(\w+)@(\w+)", "$2.$1", "a@b", false, false);
        assert_eq!(s.output, "b.a");
    }

    #[test]
    fn substitute_backtracking_engine() {
        let s = substitute_inner(r"(a)\1", "X", "aa", false, false);
        assert_eq!(s.engine, "backtracking");
        assert_eq!(s.output, "X");
    }

    #[test]
    fn substitute_invalid_leaves_input() {
        let s = substitute_inner("(unclosed", "X", "aa", false, false);
        assert_eq!(s.engine, "invalid");
        assert_eq!(s.output, "aa");
        assert!(!s.changed);
    }

    #[test]
    fn linear_engine_counts_matches() {
        let a = analyze_inner("l+", "hello llama", false, false);
        assert_eq!(a.engine, "linear");
        assert!(a.ok);
        // "ll" in hello, "ll" in llama -> 2 matches.
        assert_eq!(a.matches.len(), 2);
        assert_eq!((a.matches[0].start, a.matches[0].end), (2, 4));
    }

    #[test]
    fn ignore_case_flag() {
        assert_eq!(analyze_inner("hello", "HELLO", false, false).matches.len(), 0);
        assert_eq!(analyze_inner("hello", "HELLO", true, false).matches.len(), 1);
    }

    #[test]
    fn capture_groups_and_names() {
        let a = analyze_inner(r"(?<word>\w+)@(\w+)", "a@b", false, false);
        assert_eq!(a.engine, "linear");
        assert_eq!(a.matches.len(), 1);
        let m = &a.matches[0];
        // group 0 (whole) + 2 captures.
        assert_eq!(m.groups.len(), 3);
        assert_eq!(a.group_names[1].as_deref(), Some("word"));
        assert_eq!((m.groups[1].start, m.groups[1].end), (Some(0), Some(1)));
        assert_eq!((m.groups[2].start, m.groups[2].end), (Some(2), Some(3)));
    }

    #[test]
    fn utf16_offsets_for_non_ascii() {
        // "café" is 5 bytes but 4 UTF-16 units; "𝔸" is 4 bytes and 2 UTF-16 units.
        // A match after them must report JS-string (UTF-16) offsets, not byte offsets.
        let a = analyze_inner("X", "café𝔸X", false, false);
        assert_eq!(a.matches.len(), 1);
        // c,a,f,é = 4 units; 𝔸 = 2 units; X starts at UTF-16 index 6.
        assert_eq!((a.matches[0].start, a.matches[0].end), (6, 7));
    }

    #[test]
    fn backreference_uses_backtracking_engine() {
        // A backreference is rejected by the linear `regex` crate but accepted by
        // fancy-regex -> mog's backtracking classification.
        let a = analyze_inner(r"(a)\1", "aa", false, false);
        assert_eq!(a.engine, "backtracking");
        assert!(a.ok);
        assert_eq!(a.matches.len(), 1);
    }

    #[test]
    fn invalid_pattern_reports_error() {
        let a = analyze_inner("(unclosed", "text", false, false);
        assert_eq!(a.engine, "invalid");
        assert!(!a.ok);
        assert!(a.error.is_some());
    }

    #[test]
    fn empty_pattern_is_inert() {
        let a = analyze_inner("", "anything", false, false);
        assert_eq!(a.engine, "empty");
        assert!(a.matches.is_empty());
    }

    #[test]
    fn multiline_anchors() {
        // Without (?m), $ matches only end-of-text; with it, each line end.
        assert_eq!(analyze_inner("a$", "a\na\na", false, false).matches.len(), 1);
        assert_eq!(analyze_inner("a$", "a\na\na", false, true).matches.len(), 3);
    }
}

/// Analyze `find` against `haystack` and return the result as a JSON string.
/// `ignore_case` / `multiline` correspond to mog's `replace_regex` `ignore_case`
/// option and the `replace_regex_multiline` variant.
#[wasm_bindgen]
pub fn analyze(find: &str, haystack: &str, ignore_case: bool, multiline: bool) -> String {
    let a = analyze_inner(find, haystack, ignore_case, multiline);
    serde_json::to_string(&a).unwrap_or_else(|_| {
        String::from(
            r#"{"ok":false,"engine":"invalid","error":"serialization failed","group_names":[],"matches":[],"truncated":false}"#,
        )
    })
}

/// Preview the result of replacing every match of `find` with `replace`, as a JSON
/// string. Byte-identical to mog's `replace_regex` (see `substitute_inner`).
#[wasm_bindgen]
pub fn substitute(
    find: &str,
    replace: &str,
    haystack: &str,
    ignore_case: bool,
    multiline: bool,
) -> String {
    let s = substitute_inner(find, replace, haystack, ignore_case, multiline);
    serde_json::to_string(&s).unwrap_or_else(|_| {
        String::from(
            r#"{"ok":false,"engine":"invalid","error":"serialization failed","output":"","changed":false}"#,
        )
    })
}
