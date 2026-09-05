//! Two-input actions: compare the pipeline's primary input against a second
//! reference set supplied by a named `source` (declared in the .mog's `sources`
//! or bound with `--source NAME=PATH`). This is the write-side complement to a
//! join/diff over two files, kept deterministic and reviewable.
//!
//! - `intersect` / `subtract`: line-set membership filters (keep lines that are /
//!   are not present in the reference), order- and duplicate-preserving.
//! - `diff`: a line diff of the reference (old) against the input (new).
//! - `reconcile`: a key-aware record compare (match rows by a key column, then
//!   annotate / select ADDED / REMOVED / CHANGED / SAME rows).
//!
//! These need `ctx.sources`, so the engine dispatches them here (like
//! `fill_from_list`) rather than through the plain `resolve` registry.

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{anyhow, bail, Result};
use similar::{ChangeTag, TextDiff};

use crate::actions::data::{csv_field, parse_csv_line};
use crate::line_text::{join, split};
use crate::model::Step;

/// Read the `ref` option and resolve it to a loaded source's lines.
fn ref_lines<'a>(
    step: &Step,
    sources: &'a BTreeMap<String, Vec<String>>,
    action: &str,
) -> Result<&'a Vec<String>> {
    let name = step
        .get_string("ref")
        .ok_or_else(|| anyhow!("{action} requires a 'ref' option (the name of a loaded source)"))?;
    sources.get(&name).ok_or_else(|| {
        anyhow!(
            "{action}: unknown source '{name}' \
             (bind it with --source {name}=<path> or a 'sources' entry)"
        )
    })
}

/// The comparison key for a whole line, honoring `trim` / `ignore_case`.
fn line_key(line: &str, trim: bool, ignore_case: bool) -> String {
    let s = if trim { line.trim() } else { line };
    if ignore_case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}

/// Build the set of reference line-keys.
fn ref_key_set(refl: &[String], trim: bool, ignore_case: bool) -> HashSet<String> {
    refl.iter()
        .map(|l| line_key(l, trim, ignore_case))
        .collect()
}

/// `intersect`: keep each primary line whose key also appears in the reference.
/// Order- and duplicate-preserving (a membership filter, not a set reduction).
pub fn intersect(
    input: &str,
    step: &Step,
    sources: &BTreeMap<String, Vec<String>>,
) -> Result<String> {
    membership_filter(input, step, sources, "intersect", true)
}

/// `subtract`: keep each primary line whose key does NOT appear in the reference.
pub fn subtract(
    input: &str,
    step: &Step,
    sources: &BTreeMap<String, Vec<String>>,
) -> Result<String> {
    membership_filter(input, step, sources, "subtract", false)
}

fn membership_filter(
    input: &str,
    step: &Step,
    sources: &BTreeMap<String, Vec<String>>,
    action: &str,
    keep_when_present: bool,
) -> Result<String> {
    let refl = ref_lines(step, sources, action)?;
    let trim = step.get_bool("trim", false)?;
    let ignore_case = step.get_bool("ignore_case", false)?;
    let set = ref_key_set(refl, trim, ignore_case);

    let s = split(input);
    let kept: Vec<String> = s
        .lines
        .iter()
        .filter(|l| set.contains(&line_key(l, trim, ignore_case)) == keep_when_present)
        .cloned()
        .collect();
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `diff`: emit a line diff of the reference (old side) against the input (new
/// side). `format` selects the shape: `marker` (default; git-style `+`/`-`/` `
/// prefixes), `unified` (a unified diff), `only_added` (input lines not in the
/// reference), `only_removed` (reference lines not in the input).
pub fn diff(input: &str, step: &Step, sources: &BTreeMap<String, Vec<String>>) -> Result<String> {
    let refl = ref_lines(step, sources, "diff")?;
    let format = step.get_string_or("format", "marker");

    let s = split(input);
    // Terminate every line uniformly (including the last) so the final line is
    // never treated as different from a mid-list line with the same text.
    let to_block = |lines: &[String]| -> String {
        if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        }
    };
    let old_text = to_block(refl);
    let new_text = to_block(&s.lines);
    let td = TextDiff::from_lines(&old_text, &new_text);

    let out = match format.as_str() {
        "unified" => {
            let mut ud = td.unified_diff();
            ud.header("reference", "input");
            ud.to_string()
        }
        "marker" => {
            let mut lines: Vec<String> = Vec::new();
            for change in td.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "- ",
                    ChangeTag::Insert => "+ ",
                    ChangeTag::Equal => "  ",
                };
                lines.push(format!("{sign}{}", change.value().trim_end_matches('\n')));
            }
            join(&lines, s.eol, s.trailing_eol)
        }
        "only_added" | "only_removed" => {
            let want = if format == "only_added" {
                ChangeTag::Insert
            } else {
                ChangeTag::Delete
            };
            let lines: Vec<String> = td
                .iter_all_changes()
                .filter(|c| c.tag() == want)
                .map(|c| c.value().trim_end_matches('\n').to_string())
                .collect();
            join(&lines, s.eol, s.trailing_eol)
        }
        other => bail!(
            "diff: unknown format '{other}' (expected marker, unified, only_added, only_removed)"
        ),
    };
    Ok(out)
}

/// `reconcile`: match input rows against reference rows by a `key` column and
/// report the differences. `report` selects the output:
/// - `annotated` (default): every input row prefixed with a status field
///   (`ADDED` / `CHANGED` / `SAME`), then every reference-only row as `REMOVED`.
/// - `added`: rows whose key is in the input but not the reference.
/// - `removed`: rows whose key is in the reference but not the input.
/// - `changed`: rows whose key is in both but whose full row differs (input side).
/// - `common`: rows whose key is in both (input side), changed or not.
///
/// `key` is a 1-based column index, or a header name when `has_header` is set.
pub fn reconcile(
    input: &str,
    step: &Step,
    sources: &BTreeMap<String, Vec<String>>,
) -> Result<String> {
    let refl = ref_lines(step, sources, "reconcile")?;
    let key_opt = step.get_string("key").ok_or_else(|| {
        anyhow!("reconcile requires a 'key' option (column index or header name)")
    })?;
    let delim = step.get_string_or("delimiter", ",");
    let delim = delim.chars().next().unwrap_or(',');
    let has_header = step.get_bool("has_header", false)?;
    let report = step.get_string_or("report", "annotated");

    let s = split(input);
    let (in_header, in_rows) = header_split(&s.lines, has_header);
    let (_ref_header, ref_rows) = header_split(refl, has_header);

    // Resolve the key column index for each side (a name may sit at different
    // positions in two exports; an index is shared).
    let in_key_idx = resolve_key(&key_opt, in_header, delim, "reconcile")?;
    let ref_key_idx = resolve_key(&key_opt, _ref_header, delim, "reconcile")?;

    // Reference rows by key (last one wins on a dup key).
    let mut ref_by_key: HashMap<String, &String> = HashMap::new();
    for row in ref_rows {
        if let Some(k) = field_at(row, ref_key_idx, delim) {
            ref_by_key.insert(k, row);
        }
    }

    let mut out: Vec<String> = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    if report == "annotated" {
        if let Some(h) = in_header {
            out.push(format!("status{delim}{h}"));
        }
    }

    for row in in_rows {
        let key = match field_at(row, in_key_idx, delim) {
            Some(k) => k,
            None => continue,
        };
        seen_keys.insert(key.clone());
        let status = match ref_by_key.get(&key) {
            None => "ADDED",
            Some(rref) => {
                if rref.as_str() == row.as_str() {
                    "SAME"
                } else {
                    "CHANGED"
                }
            }
        };
        let emit = match report.as_str() {
            "annotated" => Some(format!("{status}{delim}{row}")),
            "added" => (status == "ADDED").then(|| row.clone()),
            "changed" => (status == "CHANGED").then(|| row.clone()),
            "common" => (status != "ADDED").then(|| row.clone()),
            "removed" => None,
            other => bail!(
                "reconcile: unknown report '{other}' \
                 (expected annotated, added, removed, changed, common)"
            ),
        };
        if let Some(line) = emit {
            out.push(line);
        }
    }

    // Reference-only rows (REMOVED). Emit in reference order for determinism.
    if report == "annotated" || report == "removed" {
        for row in ref_rows {
            let key = match field_at(row, ref_key_idx, delim) {
                Some(k) => k,
                None => continue,
            };
            if seen_keys.contains(&key) {
                continue;
            }
            match report.as_str() {
                "annotated" => out.push(format!("REMOVED{delim}{row}")),
                "removed" => out.push(row.clone()),
                _ => {}
            }
        }
    }

    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Parse the `value` option (comma-separated 1-based indices or header names) into
/// 0-based reference column indices. When absent, defaults to every reference
/// column except the key column (a VLOOKUP that brings in the rest of the row).
fn resolve_value_cols(
    value: Option<&str>,
    ref_header: Option<&String>,
    delim: char,
    ref_key_idx: usize,
    ref_ncols: usize,
) -> Result<Vec<usize>> {
    match value {
        Some(spec) => {
            let mut cols = Vec::new();
            for part in spec.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                cols.push(resolve_key(part, ref_header, delim, "lookup")?);
            }
            if cols.is_empty() {
                bail!("lookup: 'value' listed no columns");
            }
            Ok(cols)
        }
        None => Ok((0..ref_ncols).filter(|&c| c != ref_key_idx).collect()),
    }
}

/// `lookup`: enrich each input row with value column(s) pulled from a reference
/// source, matched by a key column (a keyed join / VLOOKUP -- the write-side of
/// "look this key up in that table"). `ref` names the reference source; `key` is
/// the input's key column (1-based index, or a header name when `has_header` is
/// set); `ref_key` is the reference's key column (defaults to `key`); `value`
/// picks which reference column(s) to append (comma-separated indices/names;
/// default = every reference column except the key). On a key with no match,
/// `on_miss` = blank (append empty cells, the default), leave (row unchanged),
/// drop (omit the row), or error. The first matching reference row wins on a
/// duplicate key. Options: `delimiter` (default ","), `has_header` (default false).
pub fn lookup(input: &str, step: &Step, sources: &BTreeMap<String, Vec<String>>) -> Result<String> {
    let refl = ref_lines(step, sources, "lookup")?;
    let key_opt = step
        .get_string("key")
        .ok_or_else(|| anyhow!("lookup requires a 'key' option (column index or header name)"))?;
    let ref_key_opt = step
        .get_string("ref_key")
        .unwrap_or_else(|| key_opt.clone());
    let delim = step.get_string_or("delimiter", ",");
    let delim = delim.chars().next().unwrap_or(',');
    let has_header = step.get_bool("has_header", false)?;
    let on_miss = step.get_string_or("on_miss", "blank");
    let value_opt = step.get_string("value");

    let s = split(input);
    let (in_header, in_rows) = header_split(&s.lines, has_header);
    let (ref_header, ref_rows) = header_split(refl, has_header);

    let in_key_idx = resolve_key(&key_opt, in_header, delim, "lookup")?;
    let ref_key_idx = resolve_key(&ref_key_opt, ref_header, delim, "lookup")?;

    let ref_ncols = refl
        .iter()
        .map(|l| parse_csv_line(l, delim).len())
        .max()
        .unwrap_or(0);
    let value_cols = resolve_value_cols(
        value_opt.as_deref(),
        ref_header,
        delim,
        ref_key_idx,
        ref_ncols,
    )?;

    // Reference rows by key (the FIRST match wins, the VLOOKUP convention).
    let mut ref_by_key: HashMap<String, &String> = HashMap::new();
    for row in ref_rows {
        if let Some(k) = field_at(row, ref_key_idx, delim) {
            ref_by_key.entry(k).or_insert(row);
        }
    }

    let ds = delim.to_string();
    let n_val = value_cols.len();
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());

    // Header: pass the input header through, appending the reference column names.
    if let Some(h) = in_header {
        let mut row = h.clone();
        for &vc in &value_cols {
            let name = ref_header
                .and_then(|rh| field_at(rh, vc, delim))
                .unwrap_or_else(|| format!("col{}", vc + 1));
            row.push(delim);
            row.push_str(&csv_field(&name, &ds));
        }
        out.push(row);
    }

    for row in in_rows {
        let key = field_at(row, in_key_idx, delim);
        let matched = key.as_ref().and_then(|k| ref_by_key.get(k).copied());
        match matched {
            Some(rref) => {
                let mut line = row.clone();
                for &vc in &value_cols {
                    let v = field_at(rref, vc, delim).unwrap_or_default();
                    line.push(delim);
                    line.push_str(&csv_field(&v, &ds));
                }
                out.push(line);
            }
            None => match on_miss.as_str() {
                "leave" => out.push(row.clone()),
                "drop" => {}
                "blank" => {
                    let mut line = row.clone();
                    for _ in 0..n_val {
                        line.push(delim);
                    }
                    out.push(line);
                }
                "error" => bail!(
                    "lookup: no reference match for key '{}' (on_miss=error)",
                    key.unwrap_or_default()
                ),
                other => {
                    bail!("lookup: unknown on_miss '{other}' (expected blank, leave, drop, error)")
                }
            },
        }
    }

    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Split lines into an optional header line and the data rows.
fn header_split(lines: &[String], has_header: bool) -> (Option<&String>, &[String]) {
    if has_header && !lines.is_empty() {
        (Some(&lines[0]), &lines[1..])
    } else {
        (None, lines)
    }
}

/// Resolve `key` (a 1-based index string, or a header name when a header is
/// present) to a 0-based column index. `action` names the caller for errors.
fn resolve_key(key: &str, header: Option<&String>, delim: char, action: &str) -> Result<usize> {
    if let Ok(n) = key.trim().parse::<usize>() {
        if n == 0 {
            bail!("{action}: column index is 1-based (got 0)");
        }
        return Ok(n - 1);
    }
    // Non-numeric: must be a header name.
    let header = header.ok_or_else(|| {
        anyhow!("{action}: key '{key}' is a name but has_header is not set (use a 1-based index)")
    })?;
    let fields = parse_csv_line(header, delim);
    fields
        .iter()
        .position(|f| f == key)
        .ok_or_else(|| anyhow!("{action}: column '{key}' not found in the header row"))
}

/// The `idx`-th field of a row (quote-aware), or None if the row is too short.
fn field_at(row: &str, idx: usize, delim: char) -> Option<String> {
    let fields = parse_csv_line(row, delim);
    fields.get(idx).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn step(opts: serde_json::Value) -> Step {
        Step {
            description: None,
            section: None,
            action: None,
            disabled: false,
            only_lines_matching: None,
            except_lines_matching: None,
            match_ignore_case: false,
            scope: None,
            options: opts.as_object().cloned().unwrap_or_default(),
        }
    }

    fn srcs(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn intersect_keeps_present_lines_in_order() {
        let s = srcs(&[("ref", &["b", "d", "a"])]);
        let st = step(json!({"ref": "ref"}));
        assert_eq!(intersect("a\nb\nc\nd\n", &st, &s).unwrap(), "a\nb\nd\n");
    }

    #[test]
    fn subtract_drops_present_lines() {
        let s = srcs(&[("ref", &["b", "d"])]);
        let st = step(json!({"ref": "ref"}));
        assert_eq!(subtract("a\nb\nc\nd\n", &st, &s).unwrap(), "a\nc\n");
    }

    #[test]
    fn membership_honors_trim_and_case() {
        let s = srcs(&[("ref", &["  A  "])]);
        let st = step(json!({"ref": "ref", "trim": true, "ignore_case": true}));
        assert_eq!(intersect("a\nb\n", &st, &s).unwrap(), "a\n");
    }

    #[test]
    fn intersect_preserves_duplicates() {
        let s = srcs(&[("ref", &["a"])]);
        let st = step(json!({"ref": "ref"}));
        assert_eq!(intersect("a\na\nb\n", &st, &s).unwrap(), "a\na\n");
    }

    #[test]
    fn diff_marker_shows_added_and_removed() {
        let s = srcs(&[("ref", &["a", "b", "c"])]);
        let st = step(json!({"ref": "ref", "format": "marker"}));
        // input drops b, adds d.
        assert_eq!(diff("a\nc\nd\n", &st, &s).unwrap(), "  a\n- b\n  c\n+ d\n");
    }

    #[test]
    fn diff_only_added_and_removed() {
        let s = srcs(&[("ref", &["a", "b", "c"])]);
        let added = step(json!({"ref": "ref", "format": "only_added"}));
        assert_eq!(diff("a\nc\nd\n", &added, &s).unwrap(), "d\n");
        let removed = step(json!({"ref": "ref", "format": "only_removed"}));
        assert_eq!(diff("a\nc\nd\n", &removed, &s).unwrap(), "b\n");
    }

    #[test]
    fn reconcile_annotated_by_index() {
        // key = column 1. ref: id 1 (old name), 2 (same), 3 (removed).
        let s = srcs(&[("old", &["1,alice", "2,bob", "3,carol"])]);
        let st = step(json!({"ref": "old", "key": "1"}));
        // input: 1 changed, 2 same, 4 added; 3 removed.
        let got = reconcile("1,alicia\n2,bob\n4,dan\n", &st, &s).unwrap();
        assert_eq!(
            got,
            "CHANGED,1,alicia\nSAME,2,bob\nADDED,4,dan\nREMOVED,3,carol\n"
        );
    }

    #[test]
    fn reconcile_by_header_name() {
        let s = srcs(&[("old", &["id,name", "1,alice", "2,bob"])]);
        let st = step(json!({"ref": "old", "key": "id", "has_header": true}));
        let got = reconcile("id,name\n1,alicia\n3,dan\n", &st, &s).unwrap();
        assert_eq!(
            got,
            "status,id,name\nCHANGED,1,alicia\nADDED,3,dan\nREMOVED,2,bob\n"
        );
    }

    #[test]
    fn reconcile_report_added_only() {
        let s = srcs(&[("old", &["1,a", "2,b"])]);
        let st = step(json!({"ref": "old", "key": "1", "report": "added"}));
        assert_eq!(reconcile("1,a\n3,c\n", &st, &s).unwrap(), "3,c\n");
    }

    #[test]
    fn lookup_appends_the_rest_of_the_matched_row() {
        // ref: id -> (name, role). Default value = every ref col except the key.
        let s = srcs(&[("dir", &["1,Alice,admin", "2,Bob,editor"])]);
        let st = step(json!({"ref": "dir", "key": "1"}));
        assert_eq!(
            lookup("1,x\n2,y\n", &st, &s).unwrap(),
            "1,x,Alice,admin\n2,y,Bob,editor\n"
        );
    }

    #[test]
    fn lookup_selected_value_column_and_header() {
        let s = srcs(&[("dir", &["id,name,role", "1,Alice,admin", "2,Bob,editor"])]);
        let st = step(json!({"ref": "dir", "key": "id", "value": "name", "has_header": true}));
        assert_eq!(
            lookup("id,note\n1,hi\n2,yo\n", &st, &s).unwrap(),
            "id,note,name\n1,hi,Alice\n2,yo,Bob\n"
        );
    }

    #[test]
    fn lookup_on_miss_modes() {
        let s = srcs(&[("dir", &["1,Alice"])]);
        let blank = step(json!({"ref": "dir", "key": "1"}));
        assert_eq!(
            lookup("1,x\n9,z\n", &blank, &s).unwrap(),
            "1,x,Alice\n9,z,\n"
        );
        let leave = step(json!({"ref": "dir", "key": "1", "on_miss": "leave"}));
        assert_eq!(
            lookup("1,x\n9,z\n", &leave, &s).unwrap(),
            "1,x,Alice\n9,z\n"
        );
        let drop = step(json!({"ref": "dir", "key": "1", "on_miss": "drop"}));
        assert_eq!(lookup("1,x\n9,z\n", &drop, &s).unwrap(), "1,x,Alice\n");
        let err = step(json!({"ref": "dir", "key": "1", "on_miss": "error"}));
        assert!(lookup("9,z\n", &err, &s).is_err());
    }

    #[test]
    fn lookup_ref_key_differs_and_first_match_wins() {
        // input key col 1, ref key col 2; duplicate ref key -> first row wins.
        let s = srcs(&[("dir", &["Alice,1", "Alice2,1", "Bob,2"])]);
        let st = step(json!({"ref": "dir", "key": "1", "ref_key": "2", "value": "1"}));
        assert_eq!(lookup("1\n2\n", &st, &s).unwrap(), "1,Alice\n2,Bob\n");
    }

    #[test]
    fn lookup_quotes_appended_values_with_the_delimiter() {
        let s = srcs(&[("dir", &["1,\"Last, First\""])]);
        let st = step(json!({"ref": "dir", "key": "1"}));
        assert_eq!(lookup("1\n", &st, &s).unwrap(), "1,\"Last, First\"\n");
    }
}
