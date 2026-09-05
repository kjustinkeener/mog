//! Region-aware actions: transform a bounded block of lines while carrying
//! context captured from the block's header line.
//!
//! `hoist_from_block` is the primitive that overcomes mog's usual
//! statelessness: it lifts matching lines *out* of a block and re-emits them as
//! new statements before/after the block, interpolating both the block header's
//! captures (`${bN}`) and the extracted line's captures (`${N}`). The motivating
//! case is MySQL's inline `KEY name (cols)` index definitions, which PostgreSQL
//! needs as separate `CREATE INDEX name ON <table> (cols);` statements after the
//! table, where `<table>` comes from the `CREATE TABLE` header line.

use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use fancy_regex::{Captures, Regex};

use crate::line_text::{join, split};
use crate::model::Step;

fn compile_required(step: &Step, key: &str, ignore_case: bool) -> Result<Regex> {
    let pat = step
        .get_string(key)
        .ok_or_else(|| anyhow!("Action 'hoist_from_block' requires a '{key}' option."))?;
    let full = if ignore_case {
        format!("(?i){pat}")
    } else {
        pat.clone()
    };
    Regex::new(&full).map_err(|e| anyhow!("Invalid '{key}' pattern '{pat}': {e}"))
}

/// Collect capture groups 1..N of `caps` as owned strings (missing groups empty).
pub(crate) fn group_values(caps: &Captures) -> Vec<String> {
    (1..caps.len())
        .map(|g| {
            caps.get(g)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        })
        .collect()
}

/// Expand `${bN}` (header captures) and `${N}` (item captures) in a template.
fn render(template: &str, header: &[String], item: &[String]) -> String {
    // A single pass handles both scopes; the `b` prefix selects header captures.
    let re = Regex::new(r"\$\{(b?)(\d+)\}").expect("static template regex");
    re.replace_all(template, |c: &Captures| {
        let is_header = c.get(1).map(|m| m.as_str() == "b").unwrap_or(false);
        let n: usize = c.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        let src = if is_header { header } else { item };
        if n >= 1 && n <= src.len() {
            src[n - 1].clone()
        } else {
            String::new()
        }
    })
    .into_owned()
}

/// Expand `${N}` in `template` from a single list of captures. Used by
/// `for_each_block` to build its `bind` constant values from the header match.
pub(crate) fn expand_caps(template: &str, caps: &[String]) -> String {
    let re = Regex::new(r"\$\{(\d+)\}").expect("static template regex");
    re.replace_all(template, |c: &Captures| {
        let n: usize = c.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
        if n >= 1 && n <= caps.len() {
            caps[n - 1].clone()
        } else {
            String::new()
        }
    })
    .into_owned()
}

/// If the last kept body line ends with a comma (which happens when the original
/// trailing item was extracted), drop that dangling comma.
fn fix_trailing_comma(body: &mut [String]) {
    if let Some(last) = body.iter_mut().rev().find(|l| !l.trim().is_empty()) {
        let trimmed_end = last.trim_end();
        if let Some(without) = trimmed_end.strip_suffix(',') {
            *last = without.to_string();
        }
    }
}

/// `hoist_from_block`. Options: `block_start`, `block_end`, `extract` (regexes,
/// required), `emit_before` and/or `emit_after` (templates), `ignore_case`.
pub fn hoist_from_block(input: &str, step: &Step) -> Result<String> {
    let ic = step.get_bool("ignore_case", false)?;
    let start_re = compile_required(step, "block_start", ic)?;
    let end_re = compile_required(step, "block_end", ic)?;
    let extract_re = compile_required(step, "extract", ic)?;
    let emit_before = step.get_string("emit_before");
    let emit_after = step.get_string("emit_after");

    let s = split(input);
    let lines = &s.lines;
    let mut out: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let start_caps = start_re.captures(&lines[i]).map_err(|e| anyhow!(e))?;
        let Some(caps) = start_caps else {
            out.push(lines[i].clone());
            i += 1;
            continue;
        };
        // Find the block end at a later line.
        let mut end = None;
        let mut k = i + 1;
        while k < lines.len() {
            if end_re.is_match(&lines[k]).map_err(|e| anyhow!(e))? {
                end = Some(k);
                break;
            }
            k += 1;
        }
        let Some(endk) = end else {
            // No end found: leave the line as-is and move on.
            out.push(lines[i].clone());
            i += 1;
            continue;
        };

        let header = group_values(&caps);
        let mut before_stmts: Vec<String> = Vec::new();
        let mut after_stmts: Vec<String> = Vec::new();
        let mut kept_body: Vec<String> = Vec::new();
        let mut extracted = false;

        for line in &lines[(i + 1)..endk] {
            if let Some(item_caps) = extract_re.captures(line).map_err(|e| anyhow!(e))? {
                extracted = true;
                let item = group_values(&item_caps);
                if let Some(t) = &emit_before {
                    before_stmts.push(render(t, &header, &item));
                }
                if let Some(t) = &emit_after {
                    after_stmts.push(render(t, &header, &item));
                }
            } else {
                kept_body.push(line.clone());
            }
        }

        if extracted {
            fix_trailing_comma(&mut kept_body);
        }

        out.extend(before_stmts);
        out.push(lines[i].clone());
        out.extend(kept_body);
        out.push(lines[endk].clone());
        out.extend(after_stmts);
        i = endk + 1;
    }

    Ok(join(&out, s.eol, s.trailing_eol))
}

// -- Record (multi-line block) ops --------------------------------------------
//
// A "record" here is a paragraph: a maximal run of consecutive NON-blank lines,
// with records separated by one or more blank lines (a blank line is one that is
// empty or whitespace-only). This reuses the same blank-line record convention as
// `records_to_columns`. It is deliberately DISTINCT from mog's regex-delimited
// block machinery (`for_each_block` / the `in_block` scope), which bounds a region
// by explicit start/end patterns; whole-record sort/dedupe wants the paragraph
// convention instead. On output, records are re-joined with a single blank line,
// preserving the file's EOL style and final-newline state.

/// True for a blank (empty or whitespace-only) line -- the record separator.
fn is_record_separator(line: &str) -> bool {
    line.trim().is_empty()
}

/// Split `input` into paragraph records (each a `Vec` of its lines), returning the
/// detected EOL and trailing-newline state so the output can preserve them.
fn split_records(input: &str) -> (Vec<Vec<String>>, &'static str, bool) {
    let s = split(input);
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in s.lines {
        if is_record_separator(&line) {
            if !current.is_empty() {
                records.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        records.push(current);
    }
    (records, s.eol, s.trailing_eol)
}

/// Re-join records with a single blank separator line between them, preserving the
/// EOL style and whether the input ended with a terminator.
fn join_records(records: &[Vec<String>], eol: &str, trailing_eol: bool) -> String {
    let mut lines: Vec<String> = Vec::new();
    for (i, rec) in records.iter().enumerate() {
        if i > 0 {
            lines.push(String::new()); // one blank line separates records
        }
        lines.extend(rec.iter().cloned());
    }
    join(&lines, eol, trailing_eol)
}

/// The record text used for whole-record comparison / dedupe: the record's lines
/// joined with `\n` (a canonical separator, independent of the file's EOL).
fn record_text(rec: &[String]) -> String {
    rec.join("\n")
}

/// How a record's key is chosen for `sort_blocks` / `dedupe_blocks`.
enum RecordKey {
    WholeBlock,
    FirstLine,
    KeyRegex(regex::Regex),
}

/// Resolve the `by` option (+ `key_regex`) into a [`RecordKey`] for `action`.
fn record_key_of(action: &str, step: &Step) -> Result<RecordKey> {
    let by = step.get_string_or("by", "whole_block");
    match by.as_str() {
        "whole_block" => Ok(RecordKey::WholeBlock),
        "first_line" => Ok(RecordKey::FirstLine),
        "key_regex" => {
            let pat = step.get_string("key_regex").ok_or_else(|| {
                anyhow!("{action}: by=\"key_regex\" requires a 'key_regex' option")
            })?;
            let re = regex::Regex::new(&pat)
                .map_err(|e| anyhow!("{action}: invalid key_regex '{pat}': {e}"))?;
            Ok(RecordKey::KeyRegex(re))
        }
        other => Err(anyhow!(
            "{action}: 'by' must be whole_block, first_line, or key_regex (got '{other}')"
        )),
    }
}

impl RecordKey {
    /// Extract this key from a record. For `key_regex`: capture group 1 if the
    /// pattern has one, else the whole match; no match -> empty string.
    fn extract(&self, rec: &[String]) -> String {
        match self {
            RecordKey::WholeBlock => record_text(rec),
            RecordKey::FirstLine => rec.first().cloned().unwrap_or_default(),
            RecordKey::KeyRegex(re) => {
                let hay = record_text(rec);
                match re.captures(&hay) {
                    Some(c) => {
                        let m = if c.len() > 1 { c.get(1) } else { c.get(0) };
                        m.map(|m| m.as_str()).unwrap_or("").to_string()
                    }
                    None => String::new(),
                }
            }
        }
    }
}

/// `sort_blocks`: sort whole paragraph records as units (stable). Options: `by`
/// (whole_block | first_line | key_regex), `key_regex` (when by=key_regex), `order`
/// (asc|desc), `ignore_case`, `numeric`, `natural` -- the key semantics mirror
/// `sort_lines`.
pub fn sort_blocks(input: &str, step: &Step) -> Result<String> {
    let key = record_key_of("sort_blocks", step)?;
    let order = step.get_string_or("order", "asc");
    let descending = order.eq_ignore_ascii_case("desc");
    let ignore_case = step.get_bool("ignore_case", false)?;
    let numeric = step.get_bool("numeric", false)?;
    let natural = step.get_bool("natural", false)?;

    let (mut records, eol, trailing_eol) = split_records(input);

    // Base ascending comparator over the extracted key. Precedence numeric >
    // natural > ignore_case > ordinal, matching sort_lines; reuses its comparators.
    let base = |a: &Vec<String>, b: &Vec<String>| -> Ordering {
        let ka = key.extract(a);
        let kb = key.extract(b);
        if numeric {
            crate::actions::line::numeric_cmp(&ka, &kb)
        } else if natural {
            crate::actions::line::natural_cmp(&ka, &kb, ignore_case)
        } else if ignore_case {
            ka.to_lowercase().cmp(&kb.to_lowercase())
        } else {
            ka.cmp(&kb)
        }
    };

    // Stable sort (Rust sort_by is stable). Descending reverses the comparator so
    // equal keys keep their original order.
    if descending {
        records.sort_by(|a, b| base(b, a));
    } else {
        records.sort_by(|a, b| base(a, b));
    }

    Ok(join_records(&records, eol, trailing_eol))
}

/// `dedupe_blocks`: drop duplicate paragraph records, keeping the first occurrence
/// (works on non-adjacent duplicates). Options: `by` (whole_block | first_line |
/// key_regex), `key_regex` (when by=key_regex), `ignore_case`.
pub fn dedupe_blocks(input: &str, step: &Step) -> Result<String> {
    let key = record_key_of("dedupe_blocks", step)?;
    let ignore_case = step.get_bool("ignore_case", false)?;
    let (records, eol, trailing_eol) = split_records(input);

    let mut seen = std::collections::HashSet::new();
    let kept: Vec<Vec<String>> = records
        .into_iter()
        .filter(|rec| {
            let k = key.extract(rec);
            let k = if ignore_case { k.to_lowercase() } else { k };
            seen.insert(k)
        })
        .collect();
    Ok(join_records(&kept, eol, trailing_eol))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(opts: serde_json::Value, input: &str) -> String {
        let step: Step = serde_json::from_value(json!({
            "action": "hoist_from_block",
            "options": opts,
        }))
        .unwrap();
        hoist_from_block(input, &step).unwrap()
    }

    #[test]
    fn lifts_inline_indexes_to_create_index() {
        let input = "CREATE TABLE customers (\n\
                     \tid bigint NOT NULL,\n\
                     \tPRIMARY KEY (id),\n\
                     \tKEY idx_status (status)\n\
                     ) ENGINE=InnoDB;\n\
                     next;\n";
        let out = run(
            json!({
                "block_start": "CREATE TABLE (\\w+) \\(",
                "block_end": "^\\) ENGINE",
                "extract": "\\s*KEY (\\w+) \\(([^)]*)\\),?",
                "emit_after": "CREATE INDEX ${1} ON ${b1} (${2});"
            }),
            input,
        );
        // KEY line removed, its comma-less predecessor keeps its comma stripped,
        // and a CREATE INDEX is emitted after the table using the table name.
        assert_eq!(
            out,
            "CREATE TABLE customers (\n\
             \tid bigint NOT NULL,\n\
             \tPRIMARY KEY (id)\n\
             ) ENGINE=InnoDB;\n\
             CREATE INDEX idx_status ON customers (status);\n\
             next;\n"
        );
    }

    #[test]
    fn no_match_passes_through() {
        let input = "SELECT 1;\nSELECT 2;\n";
        let out = run(
            json!({
                "block_start": "CREATE TABLE (\\w+) \\(",
                "block_end": "^\\)",
                "extract": "KEY (\\w+)",
                "emit_after": "X ${b1} ${1}"
            }),
            input,
        );
        assert_eq!(out, input);
    }

    #[test]
    fn emit_before_hoists_upward() {
        let input = "CREATE TABLE t (\n\tc int,\n\tE foo\n);\n";
        let out = run(
            json!({
                "block_start": "CREATE TABLE (\\w+) \\(",
                "block_end": "^\\)",
                "extract": "\\s*E (\\w+)",
                "emit_before": "PRE ${b1} ${1};"
            }),
            input,
        );
        assert_eq!(out, "PRE t foo;\nCREATE TABLE t (\n\tc int\n);\n");
    }
}

#[cfg(test)]
mod record_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn sort_blocks_whole_block_default() {
        // Two multi-line records separated by a blank line; sorted as whole units.
        let out = run(
            r#"{"steps":[{"action":"sort_blocks"}]}"#,
            "banana\nyellow\n\napple\nred",
        );
        assert_eq!(out, "apple\nred\n\nbanana\nyellow");
    }

    #[test]
    fn sort_blocks_by_first_line() {
        // Key is the first line only; bodies travel with their header.
        let out = run(
            r#"{"steps":[{"action":"sort_blocks","options":{"by":"first_line"}}]}"#,
            "charlie\nz-body\n\nalpha\na-body\n\nbravo\nb-body",
        );
        assert_eq!(out, "alpha\na-body\n\nbravo\nb-body\n\ncharlie\nz-body");
    }

    #[test]
    fn sort_blocks_by_key_regex_capture_group() {
        // Sort vCard-like records by the FN capture, not by the first line.
        let out = run(
            r#"{"steps":[{"action":"sort_blocks","options":{"by":"key_regex","key_regex":"FN:(\\w+)"}}]}"#,
            "BEGIN\nFN:Zoe\nEND\n\nBEGIN\nFN:Amy\nEND",
        );
        assert_eq!(out, "BEGIN\nFN:Amy\nEND\n\nBEGIN\nFN:Zoe\nEND");
    }

    #[test]
    fn sort_blocks_numeric_by_key_regex() {
        // Numeric key: 10 sorts after 2 (not lexically).
        let out = run(
            r#"{"steps":[{"action":"sort_blocks","options":{"by":"key_regex","key_regex":"n=(\\d+)","numeric":true}}]}"#,
            "n=10\nx\n\nn=2\ny\n\nn=1\nz",
        );
        assert_eq!(out, "n=1\nz\n\nn=2\ny\n\nn=10\nx");
    }

    #[test]
    fn sort_blocks_ignore_case_and_desc() {
        let out = run(
            r#"{"steps":[{"action":"sort_blocks","options":{"by":"first_line","ignore_case":true,"order":"desc"}}]}"#,
            "apple\n1\n\nBanana\n2\n\ncherry\n3",
        );
        assert_eq!(out, "cherry\n3\n\nBanana\n2\n\napple\n1");
    }

    #[test]
    fn sort_blocks_is_stable_on_equal_keys() {
        // Equal first-line keys keep their original relative order.
        let out = run(
            r#"{"steps":[{"action":"sort_blocks","options":{"by":"first_line"}}]}"#,
            "a\nfirst\n\na\nsecond\n\na\nthird",
        );
        assert_eq!(out, "a\nfirst\n\na\nsecond\n\na\nthird");
    }

    #[test]
    fn sort_blocks_no_trailing_final_newline_preserved() {
        // Input has no final newline: output must not gain one.
        let out = run(r#"{"steps":[{"action":"sort_blocks"}]}"#, "b\nb2\n\na\na2");
        assert_eq!(out, "a\na2\n\nb\nb2");
        assert!(!out.ends_with('\n'));
    }

    #[test]
    fn sort_blocks_trailing_final_newline_preserved() {
        let out = run(r#"{"steps":[{"action":"sort_blocks"}]}"#, "b\n\na\n");
        assert_eq!(out, "a\n\nb\n");
    }

    #[test]
    fn dedupe_blocks_whole_block_non_adjacent() {
        // Identical multi-line records dropped, keeping the first; non-adjacent.
        let out = run(
            r#"{"steps":[{"action":"dedupe_blocks"}]}"#,
            "a\n1\n\nb\n2\n\na\n1\n\nc\n3",
        );
        assert_eq!(out, "a\n1\n\nb\n2\n\nc\n3");
    }

    #[test]
    fn dedupe_blocks_by_first_line() {
        // Records sharing a first line collapse to the first record with that line.
        let out = run(
            r#"{"steps":[{"action":"dedupe_blocks","options":{"by":"first_line"}}]}"#,
            "id1\nkeep\n\nid2\nx\n\nid1\ndropped",
        );
        assert_eq!(out, "id1\nkeep\n\nid2\nx");
    }

    #[test]
    fn dedupe_blocks_by_key_regex_ignore_case() {
        let out = run(
            r#"{"steps":[{"action":"dedupe_blocks","options":{"by":"key_regex","key_regex":"KEY=(\\w+)","ignore_case":true}}]}"#,
            "KEY=Abc\none\n\nKEY=abc\ntwo\n\nKEY=xyz\nthree",
        );
        assert_eq!(out, "KEY=Abc\none\n\nKEY=xyz\nthree");
    }

    #[test]
    fn blocks_collapse_multiple_blank_separators() {
        // Several blank lines between records act as one separator; output uses one.
        let out = run(r#"{"steps":[{"action":"sort_blocks"}]}"#, "b\n\n\n\na");
        assert_eq!(out, "a\n\nb");
    }
}
