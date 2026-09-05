//! Tabular / data-reshape actions. `records_to_columns` parses repeated
//! `key: value` stanzas (records separated by a blank line) into a CSV table:
//! the header is the union of keys in first-seen order, one row per record, with
//! empty cells for absent keys. Covers vCard, iCal, email headers, `docker
//! inspect`-style output, and LDAP dumps.

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{anyhow, bail, Result};
use base64::Engine;
use sha2::{Digest, Sha256, Sha512};

use crate::line_text::{join, split};
use crate::model::Step;

/// `paste_column`: append (or prepend) a column from a named source, pairing the
/// input's line `i` with the source's line `i` (Unix `paste`). `source` names a
/// loaded source; `delimiter` (default a tab) joins the two; `side` = right
/// (default, append) or left (prepend); `on_exhausted` = blank (default, empty
/// cell), skip (leave the line as-is), or error when the source has fewer lines
/// than the input. Source-aware, so the engine dispatches it directly.
pub fn paste_column(
    input: &str,
    step: &Step,
    sources: &BTreeMap<String, Vec<String>>,
) -> Result<String> {
    let name = step
        .get_string("source")
        .ok_or_else(|| anyhow!("paste_column requires a 'source' option"))?;
    let values = sources.get(&name).ok_or_else(|| {
        anyhow!(
            "paste_column: unknown source '{name}' \
             (bind it with --source {name}=<path> or a 'sources' entry)"
        )
    })?;
    let delim = step.get_string_or("delimiter", "\t");
    let left = step
        .get_string_or("side", "right")
        .eq_ignore_ascii_case("left");
    let on_exhausted = step.get_enum(
        "on_exhausted",
        "paste_column",
        "blank",
        &["blank", "skip", "error"],
    )?;

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for (i, line) in s.lines.iter().enumerate() {
        let val: Option<&str> = if i < values.len() {
            Some(values[i].as_str())
        } else {
            match on_exhausted.as_str() {
                "skip" => None,
                "error" => bail!(
                    "paste_column: source '{name}' exhausted after {} value(s), \
                     but the input has {} line(s)",
                    values.len(),
                    s.lines.len()
                ),
                _ => Some(""), // "blank": append an empty cell
            }
        };
        let combined = match val {
            Some(v) if left => format!("{v}{delim}{line}"),
            Some(v) => format!("{line}{delim}{v}"),
            None => line.clone(),
        };
        out.push(combined);
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// The (thousands, decimal) separator pair for a named number convention.
fn convention_seps(name: &str) -> Result<(char, char)> {
    match name {
        "us" => Ok((',', '.')),          // 1,234.56
        "eu" => Ok(('.', ',')),          // 1.234,56
        "swiss" => Ok(('\'', '.')),      // 1'234.56
        "space_comma" => Ok((' ', ',')), // 1 234,56 (SI-ish, comma decimal)
        "space_dot" => Ok((' ', '.')),   // 1 234.56 (SI-ish, dot decimal)
        other => bail!(
            "decimal_separator_normalize: unknown convention '{other}' \
             (expected us, eu, swiss, space_comma, space_dot)"
        ),
    }
}

/// `decimal_separator_normalize`: rewrite numbers from one decimal/thousands
/// convention to another (e.g. European `1.234,56` -> US `1234.56`). Only tokens
/// that actually carry a separator are touched, so bare integers like a year
/// (`2024`) are never regrouped. `from` / `to` name the conventions (us, eu,
/// swiss, space_comma, space_dot); `grouping` (default true) controls whether the
/// output emits thousands separators.
pub fn decimal_separator_normalize(input: &str, step: &Step) -> Result<String> {
    let from = step.get_string_or("from", "eu");
    let to = step.get_string_or("to", "us");
    let grouping = step.get_bool("grouping", true)?;
    let (ft, fd) = convention_seps(&from)?;
    let (tt, td) = convention_seps(&to)?;

    // A number token in the FROM convention that carries at least one separator:
    // either a grouped integer (>=1 thousands group) with an optional fraction, or
    // a plain integer with a fraction. Bare integers (no separator) are excluded.
    let t = regex::escape(&ft.to_string());
    let d = regex::escape(&fd.to_string());
    let pattern = format!(r"[+-]?(?:\d{{1,3}}(?:{t}\d{{3}})+(?:{d}\d+)?|\d+{d}\d+)");
    let re = regex::Regex::new(&pattern)
        .map_err(|e| anyhow!("decimal_separator_normalize: pattern error: {e}"))?;

    let out = re.replace_all(input, |caps: &regex::Captures| {
        let tok = &caps[0];
        reformat_number(tok, ft, fd, tt, td, grouping)
    });
    Ok(out.into_owned())
}

/// Reparse one number token (in the from convention) and re-emit it in the to
/// convention. `tok` is guaranteed to match the number pattern.
fn reformat_number(tok: &str, ft: char, fd: char, tt: char, td: char, grouping: bool) -> String {
    let sign = if tok.starts_with('-') {
        "-"
    } else if tok.starts_with('+') {
        "+"
    } else {
        ""
    };
    let rest = tok.trim_start_matches(['+', '-']);
    // Split into integer and fraction on the from-decimal separator.
    let (int_part, frac_part) = match rest.split_once(fd) {
        Some((i, f)) => (i, Some(f)),
        None => (rest, None),
    };
    // Strip from-thousands separators from the integer digits.
    let digits: String = int_part.chars().filter(|c| *c != ft).collect();
    let grouped = if grouping {
        group_thousands(&digits, tt)
    } else {
        digits
    };
    match frac_part {
        Some(f) => format!("{sign}{grouped}{td}{f}"),
        None => format!("{sign}{grouped}"),
    }
}

/// Insert `sep` every three digits from the right of a run of digits.
fn group_thousands(digits: &str, sep: char) -> String {
    if digits.len() <= 3 || sep == '\0' {
        return digits.to_string();
    }
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let first = digits.len() % 3;
    let first = if first == 0 { 3 } else { first };
    out.push_str(&digits[..first]);
    let mut idx = first;
    while idx < bytes.len() {
        out.push(sep);
        out.push_str(&digits[idx..idx + 3]);
        idx += 3;
    }
    out
}

/// Quote a CSV field if it contains the delimiter, a quote, or a newline.
pub(crate) fn csv_field(s: &str, delim: &str) -> String {
    if s.contains(delim) || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse a comma-separated list of 1-based column indices into 0-based indices.
/// Blank / non-numeric / zero entries are skipped.
fn parse_index_list(s: &str) -> Vec<usize> {
    s.split(',')
        .filter_map(|t| t.trim().parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .map(|n| n - 1)
        .collect()
}

/// The `idx`-th cell of a row as a `&str`, or "" when the row is too short.
fn cell(row: &[String], idx: usize) -> &str {
    row.get(idx).map(|s| s.as_str()).unwrap_or("")
}

/// `transpose`: swap the rows and columns of a delimited table. Jagged rows are
/// padded to the widest row. Quote-aware (RFC-4180, so a quoted field may span
/// lines); fully-blank rows are dropped. Whole-file. Options: `delimiter` (`,`).
pub fn transpose(input: &str, step: &Step) -> Result<String> {
    let delim = first_char(&step.get_string_or("delimiter", ","), ',');
    let ds = delim.to_string();
    let rows = read_csv_records(input, delim as u8)?;
    if rows.is_empty() {
        return Ok(String::new());
    }
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(ncols);
    for c in 0..ncols {
        let line: Vec<String> = rows.iter().map(|r| csv_field(cell(r, c), &ds)).collect();
        out.push(line.join(&ds));
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `unpivot`: reshape a wide table to long (id, key, value) form. The `id_columns`
/// (1-based, default "1") are kept; every other column becomes a (key, value) pair,
/// one output row each. With `has_header` (default true) the former column name is
/// the key; otherwise the 1-based column number is. Options: `key_name` ("key"),
/// `value_name` ("value"), `delimiter` (`,`). Whole-file.
pub fn unpivot(input: &str, step: &Step) -> Result<String> {
    let delim = first_char(&step.get_string_or("delimiter", ","), ',');
    let ds = delim.to_string();
    let has_header = step.get_bool("has_header", true)?;
    let id_cols = parse_index_list(&step.get_string_or("id_columns", "1"));
    let key_name = step.get_string_or("key_name", "key");
    let value_name = step.get_string_or("value_name", "value");

    let rows = read_csv_records(input, delim as u8)?;
    if rows.is_empty() {
        return Ok(String::new());
    }
    let (header, data): (Option<&Vec<String>>, &[Vec<String>]) = if has_header {
        (Some(&rows[0]), &rows[1..])
    } else {
        (None, &rows[..])
    };
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let value_cols: Vec<usize> = (0..ncols).filter(|c| !id_cols.contains(c)).collect();

    let s = split(input);
    let mut out: Vec<String> = Vec::new();
    if let Some(h) = header {
        let mut head: Vec<String> = id_cols
            .iter()
            .map(|&c| csv_field(cell(h, c), &ds))
            .collect();
        head.push(csv_field(&key_name, &ds));
        head.push(csv_field(&value_name, &ds));
        out.push(head.join(&ds));
    }
    for row in data {
        for &vc in &value_cols {
            let key = match header {
                Some(h) => cell(h, vc).to_string(),
                None => (vc + 1).to_string(),
            };
            let mut r: Vec<String> = id_cols
                .iter()
                .map(|&c| csv_field(cell(row, c), &ds))
                .collect();
            r.push(csv_field(&key, &ds));
            r.push(csv_field(cell(row, vc), &ds));
            out.push(r.join(&ds));
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `pivot`: reshape a long table to wide. The distinct values of `key_column`
/// (1-based, required) become columns; `value_column` (1-based, required) fills the
/// cells; rows are grouped by `id_columns` (1-based; default = every column except
/// the key and value). `aggregate` picks the value when a (group, key) pair repeats:
/// first (default) / last / error. GUARDRAIL: `max_columns` (default 1000) refuses a
/// runaway grid when the key column has too many distinct values -- set it to 0 to
/// lift the cap. With `has_header` (default true) the first row names the columns.
/// Whole-file; cost and output are O(groups x distinct-keys), which can exceed the
/// input for sparse data.
pub fn pivot(input: &str, step: &Step) -> Result<String> {
    let delim = first_char(&step.get_string_or("delimiter", ","), ',');
    let ds = delim.to_string();
    let has_header = step.get_bool("has_header", true)?;
    let key_i = one_based_req(step, "key_column")?;
    let val_i = one_based_req(step, "value_column")?;
    let aggregate = step.get_enum("aggregate", "pivot", "first", &["first", "last", "error"])?;
    let max_columns = step.get_usize("max_columns", 1000)?;

    let rows = read_csv_records(input, delim as u8)?;
    if rows.is_empty() {
        return Ok(String::new());
    }
    let (header, data): (Option<&Vec<String>>, &[Vec<String>]) = if has_header {
        (Some(&rows[0]), &rows[1..])
    } else {
        (None, &rows[..])
    };
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let id_cols: Vec<usize> = match step.get_string("id_columns") {
        Some(s) => parse_index_list(&s),
        None => (0..ncols).filter(|&c| c != key_i && c != val_i).collect(),
    };

    // Distinct keys (new columns) in first-seen order, plus groups by id-tuple.
    let mut keys: Vec<String> = Vec::new();
    let mut key_pos: HashMap<String, usize> = HashMap::new();
    let mut group_pos: HashMap<Vec<String>, usize> = HashMap::new();
    let mut groups: Vec<(Vec<String>, HashMap<usize, String>)> = Vec::new();

    for row in data {
        let key = cell(row, key_i).to_string();
        if !key_pos.contains_key(&key) {
            key_pos.insert(key.clone(), keys.len());
            keys.push(key.clone());
            if max_columns != 0 && keys.len() > max_columns {
                bail!(
                    "pivot: the key column has more than {max_columns} distinct values \
                     (the max_columns guardrail). Raise max_columns, or set it to 0 to lift \
                     the cap, if you really want that many columns."
                );
            }
        }
        let ki = key_pos[&key];
        let id_tuple: Vec<String> = id_cols.iter().map(|&c| cell(row, c).to_string()).collect();
        let gi = *group_pos.entry(id_tuple.clone()).or_insert_with(|| {
            groups.push((id_tuple, HashMap::new()));
            groups.len() - 1
        });
        let value = cell(row, val_i).to_string();
        let slot = groups[gi].1.entry(ki);
        match slot {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(value);
            }
            std::collections::hash_map::Entry::Occupied(mut o) => match aggregate.as_str() {
                "last" => {
                    o.insert(value);
                }
                "error" => bail!(
                    "pivot: duplicate value for the same group and key '{}' (aggregate=error)",
                    cell(row, key_i)
                ),
                _ => {} // "first": keep the existing value
            },
        }
    }

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(groups.len() + 1);
    // Header: id column names (from the source header, or col numbers) + the keys.
    let mut head: Vec<String> = id_cols
        .iter()
        .map(|&c| match header {
            Some(h) => csv_field(cell(h, c), &ds),
            None => csv_field(&(c + 1).to_string(), &ds),
        })
        .collect();
    head.extend(keys.iter().map(|k| csv_field(k, &ds)));
    out.push(head.join(&ds));

    for (id_tuple, cells) in &groups {
        let mut r: Vec<String> = id_tuple.iter().map(|v| csv_field(v, &ds)).collect();
        for ki in 0..keys.len() {
            r.push(csv_field(
                cells.get(&ki).map(|s| s.as_str()).unwrap_or(""),
                &ds,
            ));
        }
        out.push(r.join(&ds));
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Read a required 1-based column option and return it as a 0-based index.
fn one_based_req(step: &Step, key: &str) -> Result<usize> {
    let n = step.get_usize(key, 0)?;
    if n == 0 {
        bail!("pivot requires a 1-based '{key}' option");
    }
    Ok(n - 1)
}

/// `records_to_columns`: repeated `key<sep>value` stanzas (blank-line separated)
/// -> a CSV table. Options: `separator` (key/value split, default ":"),
/// `delimiter` (output CSV delimiter, default ","), `header` (emit a header row,
/// default true). Lines without the separator are ignored.
pub fn records_to_columns(input: &str, step: &Step) -> Result<String> {
    let kv_sep = step.get_string_or("separator", ":");
    let delim = step.get_string_or("delimiter", ",");
    let emit_header = step.get_bool("header", true)?;

    let s = split(input);
    // Group non-blank lines into records; a blank line ends the current record.
    let mut records: Vec<Vec<(String, String)>> = Vec::new();
    let mut current: Vec<(String, String)> = Vec::new();
    for line in &s.lines {
        if line.trim().is_empty() {
            if !current.is_empty() {
                records.push(std::mem::take(&mut current));
            }
            continue;
        }
        if let Some((k, v)) = line.split_once(kv_sep.as_str()) {
            current.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    if !current.is_empty() {
        records.push(current);
    }

    // Union of keys in first-seen order.
    let mut keys: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for rec in &records {
        for (k, _) in rec {
            if seen.insert(k.clone()) {
                keys.push(k.clone());
            }
        }
    }

    let mut out: Vec<String> = Vec::new();
    if emit_header {
        out.push(
            keys.iter()
                .map(|k| csv_field(k, &delim))
                .collect::<Vec<_>>()
                .join(&delim),
        );
    }
    for rec in &records {
        // Last value wins if a key repeats within a record.
        let map: HashMap<&str, &str> = rec.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let row: Vec<String> = keys
            .iter()
            .map(|k| csv_field(map.get(k.as_str()).copied().unwrap_or(""), &delim))
            .collect();
        out.push(row.join(&delim));
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Parse one CSV line into fields (quote-aware, RFC-4180-ish). Single-line: an
/// embedded newline inside a quoted field is not supported (each line is parsed
/// on its own). Shared with `json::csv_to_json`.
pub(crate) fn parse_csv_line(line: &str, delim: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delim {
            fields.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    fields.push(cur);
    fields
}

fn first_char(s: &str, default: char) -> char {
    s.chars().next().unwrap_or(default)
}

/// Read a whole CSV document into records using the `csv` crate: full RFC-4180,
/// including newlines inside quoted fields (which the per-line parser cannot do).
/// Fully-blank records are skipped. Shared by the CSV document readers.
pub(crate) fn read_csv_records(input: &str, delim: u8) -> Result<Vec<Vec<String>>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(input.as_bytes());
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| anyhow!("CSV parse error: {e}"))?;
        let fields: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
        if fields.iter().all(|f| f.is_empty()) {
            continue;
        }
        out.push(fields);
    }
    Ok(out)
}

/// Split a Markdown table row on unescaped `|`, dropping the empty cells the outer
/// pipes produce (`| a | b |` -> ["a", "b"]). `\|` becomes a literal `|`.
fn split_md_row(row: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = row.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') {
            cur.push('|');
            chars.next();
        } else if c == '|' {
            cells.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    cells.push(cur);
    if cells.first().map(|s| s.trim().is_empty()).unwrap_or(false) {
        cells.remove(0);
    }
    if cells.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        cells.pop();
    }
    cells
}

/// `csv_to_markdown`: render CSV (first line = header) as a GitHub-flavored
/// Markdown table. Quote-aware input; `|` inside a cell is escaped; ragged rows
/// are padded to the header width.
pub fn csv_to_markdown(input: &str, step: &Step) -> Result<String> {
    let delim = first_char(&step.get_string_or("delimiter", ","), ',') as u8;
    let records = read_csv_records(input, delim)?;
    let mut it = records.into_iter();
    let header = match it.next() {
        Some(h) => h,
        None => return Ok(String::new()),
    };
    let esc = |s: &str| s.replace('|', "\\|").trim().to_string();
    let row = |cells: &[String]| {
        format!(
            "| {} |\n",
            cells.iter().map(|c| esc(c)).collect::<Vec<_>>().join(" | ")
        )
    };

    let mut out = String::new();
    out.push_str(&row(&header));
    let sep: Vec<String> = header.iter().map(|_| "---".to_string()).collect();
    out.push_str(&format!("| {} |\n", sep.join(" | ")));
    for mut fields in it {
        fields.resize(header.len(), String::new());
        out.push_str(&row(&fields));
    }
    Ok(out)
}

/// Format a CSV cell as a SQL literal: NULL for empty, bare for int/float, else a
/// single-quoted string with `'` doubled.
fn sql_value(raw: &str) -> String {
    if raw.is_empty() {
        return "NULL".to_string();
    }
    if raw.parse::<i64>().is_ok() {
        return raw.to_string();
    }
    if raw.chars().any(|c| c == '.' || c == 'e' || c == 'E') && raw.parse::<f64>().is_ok() {
        return raw.to_string();
    }
    format!("'{}'", raw.replace('\'', "''"))
}

/// `csv_to_sql`: CSV (first line = header) into `INSERT INTO <table> (...) VALUES
/// (...);` statements. Numbers are bare, empty cells become NULL, everything else
/// is a single-quoted (escaped) string.
pub fn csv_to_sql(input: &str, step: &Step) -> Result<String> {
    let table = step
        .get_string("table")
        .ok_or_else(|| anyhow!("csv_to_sql requires a 'table' option"))?;
    let delim = first_char(&step.get_string_or("delimiter", ","), ',') as u8;
    let records = read_csv_records(input, delim)?;
    let mut it = records.into_iter();
    let header = match it.next() {
        Some(h) => h,
        None => return Ok(String::new()),
    };
    let cols = header.join(", ");
    let mut out = String::new();
    for fields in it {
        let vals: Vec<String> = (0..header.len())
            .map(|i| sql_value(fields.get(i).map(String::as_str).unwrap_or("")))
            .collect();
        out.push_str(&format!(
            "INSERT INTO {table} ({cols}) VALUES ({});\n",
            vals.join(", ")
        ));
    }
    Ok(out)
}

/// `csv_to_html`: CSV (first line = header) into an HTML `<table>`.
pub fn csv_to_html(input: &str, step: &Step) -> Result<String> {
    let delim = first_char(&step.get_string_or("delimiter", ","), ',') as u8;
    let records = read_csv_records(input, delim)?;
    let mut it = records.into_iter();
    let header = match it.next() {
        Some(h) => h,
        None => return Ok(String::new()),
    };
    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    let mut out = String::from("<table>\n  <thead>\n    <tr>");
    for h in &header {
        out.push_str(&format!("<th>{}</th>", esc(h)));
    }
    out.push_str("</tr>\n  </thead>\n  <tbody>\n");
    for mut fields in it {
        fields.resize(header.len(), String::new());
        out.push_str("    <tr>");
        for f in &fields {
            out.push_str(&format!("<td>{}</td>", esc(f)));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("  </tbody>\n</table>\n");
    Ok(out)
}

/// `html_table_to_csv`: extract an HTML `<table>` into CSV (regex-based, for simple
/// tables). Cell tags are stripped and a few entities decoded.
pub fn html_table_to_csv(input: &str, step: &Step) -> Result<String> {
    let delim = step.get_string_or("delimiter", ",");
    let row_re = regex::Regex::new(r"(?is)<tr[^>]*>(.*?)</tr>").unwrap();
    let cell_re = regex::Regex::new(r"(?is)<t[dh][^>]*>(.*?)</t[dh]>").unwrap();
    let tag_re = regex::Regex::new(r"(?s)<[^>]*>").unwrap();
    let mut out: Vec<String> = Vec::new();
    for row in row_re.captures_iter(input) {
        let cells: Vec<String> = cell_re
            .captures_iter(&row[1])
            .map(|c| {
                let text = tag_re.replace_all(&c[1], "");
                let text = text
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&");
                csv_field(text.trim(), &delim)
            })
            .collect();
        if !cells.is_empty() {
            out.push(cells.join(&delim));
        }
    }
    let mut s = out.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// `fixed_width_to_csv`: split each line into fields by fixed column `widths`
/// (comma-separated), trimming each field. Trailing characters past the last width
/// are dropped.
pub fn fixed_width_to_csv(input: &str, step: &Step) -> Result<String> {
    let widths: Vec<usize> = step
        .get_string("widths")
        .ok_or_else(|| {
            anyhow!("fixed_width_to_csv requires 'widths' (comma-separated column widths)")
        })?
        .split(',')
        .filter_map(|w| w.trim().parse().ok())
        .collect();
    let delim = step.get_string_or("delimiter", ",");
    let mut out: Vec<String> = Vec::new();
    for line in input.lines() {
        if line.is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut pos = 0;
        let mut cells = Vec::new();
        for w in &widths {
            let end = (pos + w).min(chars.len());
            let cell: String = chars[pos..end].iter().collect();
            cells.push(csv_field(cell.trim(), &delim));
            pos = end;
        }
        out.push(cells.join(&delim));
    }
    let mut s = out.join("\n");
    if input.ends_with('\n') && !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// `access_log_to_csv`: parse Common/Combined access-log lines into CSV with columns
/// ip,timestamp,method,path,protocol,status,size,referer,user_agent. Lines that do
/// not match are skipped.
pub fn access_log_to_csv(input: &str, step: &Step) -> Result<String> {
    let delim = step.get_string_or("delimiter", ",");
    let re = regex::Regex::new(
        r#"^(\S+) \S+ \S+ \[([^\]]+)\] "(\S+) (\S+) (\S+)" (\d+) (\S+)(?: "([^"]*)" "([^"]*)")?"#,
    )
    .unwrap();
    let cols = [
        "ip",
        "timestamp",
        "method",
        "path",
        "protocol",
        "status",
        "size",
        "referer",
        "user_agent",
    ];
    let mut out: Vec<String> = vec![cols.join(&delim)];
    for line in input.lines() {
        if let Some(c) = re.captures(line) {
            let fields: Vec<String> = (1..=9)
                .map(|i| csv_field(c.get(i).map_or("", |m| m.as_str()), &delim))
                .collect();
            out.push(fields.join(&delim));
        }
    }
    let mut s = out.join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// `markdown_table_to_csv`: parse a Markdown table into CSV, dropping the `---`
/// separator row. Cells are trimmed and CSV-quoted as needed. Non-table lines are
/// ignored.
pub fn markdown_table_to_csv(input: &str, step: &Step) -> Result<String> {
    let delim = step.get_string_or("delimiter", ",");
    let mut out: Vec<String> = Vec::new();
    for line in input.lines() {
        let t = line.trim();
        if !t.contains('|') {
            continue; // not a table row
        }
        let cells = split_md_row(t);
        // Separator row: every cell is only dashes/colons/spaces.
        let is_sep = !cells.is_empty()
            && cells.iter().all(|c| {
                let c = c.trim();
                !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')
            });
        if is_sep {
            continue;
        }
        let csv = cells
            .iter()
            .map(|c| csv_field(c.trim(), &delim))
            .collect::<Vec<_>>()
            .join(&delim);
        out.push(csv);
    }
    let mut s = out.join("\n");
    if input.ends_with('\n') && !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// `change_delimiter`: re-delimit each line, quote-aware. Splits on `from`
/// (default ",") and re-joins with `to` (default a tab), re-quoting fields that
/// need it. Multi-line quoted fields are not supported.
pub fn change_delimiter(input: &str, step: &Step) -> Result<String> {
    let from = first_char(&step.get_string_or("from", ","), ',');
    let to = step.get_string_or("to", "\t");
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| {
            parse_csv_line(l, from)
                .iter()
                .map(|f| csv_field(f, &to))
                .collect::<Vec<_>>()
                .join(&to)
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `fill_down`: fill each blank cell with the last non-blank value seen in that
/// column (the classic merged-cell export fix). Blank lines pass through. Option
/// `delimiter` (default ",").
pub fn fill_down(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let s = split(input);
    let mut last: Vec<String> = Vec::new();
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                return l.clone();
            }
            let mut fields = parse_csv_line(l, delim);
            for (i, f) in fields.iter_mut().enumerate() {
                if f.trim().is_empty() {
                    if let Some(prev) = last.get(i) {
                        *f = prev.clone();
                    }
                } else {
                    while last.len() <= i {
                        last.push(String::new());
                    }
                    last[i] = f.clone();
                }
            }
            fields
                .iter()
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s)
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Mask the middle of a value, keeping `keep_start` leading and `keep_end`
/// trailing characters. Values too short to mask are returned unchanged.
fn mask_value(v: &str, keep_start: usize, keep_end: usize, mc: char) -> String {
    let chars: Vec<char> = v.chars().collect();
    let n = chars.len();
    if n <= keep_start + keep_end {
        return v.to_string();
    }
    let mut out = String::with_capacity(n);
    out.extend(&chars[..keep_start]);
    for _ in 0..(n - keep_start - keep_end) {
        out.push(mc);
    }
    out.extend(&chars[n - keep_end..]);
    out
}

/// `mask_field`: mask the middle of one delimited field, keeping `keep_start`
/// leading and `keep_end` (default 4) trailing characters. Options: `field`
/// (1-based, default 1), `delimiter` (default ","), `mask_char` (default "*").
pub fn mask_field(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let field = step.get_usize("field", 1)?;
    let keep_start = step.get_usize("keep_start", 0)?;
    let keep_end = step.get_usize("keep_end", 4)?;
    let mc = first_char(&step.get_string_or("mask_char", "*"), '*');
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| {
            let mut fields = parse_csv_line(l, delim);
            if field >= 1 && field <= fields.len() {
                fields[field - 1] = mask_value(&fields[field - 1], keep_start, keep_end, mc);
            }
            fields
                .iter()
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s)
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Digest `data` with the named algorithm, returning the raw hash bytes.
fn digest_bytes(algorithm: &str, data: &[u8]) -> Result<Vec<u8>> {
    match algorithm {
        "sha256" => {
            let mut h = Sha256::new();
            h.update(data);
            Ok(h.finalize().to_vec())
        }
        "sha512" => {
            let mut h = Sha512::new();
            h.update(data);
            Ok(h.finalize().to_vec())
        }
        other => bail!("hash_field: unknown algorithm '{other}' (expected sha256, sha512)"),
    }
}

/// Encode digest bytes as a token string. `hex` (lowercase) or `base64`
/// (URL-safe, no padding: only [A-Za-z0-9_-], so the token is safe as an
/// identifier or filename).
fn encode_digest(encoding: &str, bytes: &[u8]) -> Result<String> {
    match encoding {
        "hex" => Ok(hex::encode(bytes)),
        "base64" => Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)),
        other => bail!("hash_field: unknown encoding '{other}' (expected hex, base64)"),
    }
}

/// Hash one value into a stable pseudonymous token. `salt` (which may be empty)
/// is mixed in before the value; the result is encoded, optionally truncated to
/// `length` chars, then given the literal `prefix`.
#[allow(clippy::too_many_arguments)]
fn hash_token(
    value: &str,
    algorithm: &str,
    salt: &str,
    encoding: &str,
    length: usize,
    prefix: &str,
) -> Result<String> {
    let mut data = Vec::with_capacity(salt.len() + value.len());
    data.extend_from_slice(salt.as_bytes());
    data.extend_from_slice(value.as_bytes());
    let digest = digest_bytes(algorithm, &data)?;
    let mut tok = encode_digest(encoding, &digest)?;
    if length > 0 && length < tok.len() {
        tok.truncate(length); // hex/base64 are ASCII, so a byte truncation is a char truncation
    }
    Ok(format!("{prefix}{tok}"))
}

/// `hash_field`: replace a field (or the whole line) with a STABLE hash, so the
/// data stays joinable across runs but the original value is pseudonymized.
/// Deterministic by design (same input -> same token), so it pairs with
/// `detect_pii` ("detect, then hash"). `field` is 1-based (default 1); `field` 0
/// hashes the whole line. `algorithm` (sha256 default / sha512), `encoding` (hex
/// default / base64 URL-safe), `length` (truncate the token to N chars; 0 =
/// full), `salt` (an explicit, non-random salt mixed in before hashing -- kept
/// explicit so tokens stay stable and testable), and `prefix` (a literal prefix
/// on the token, e.g. "user_") shape the output. Empty cells and blank lines are
/// left untouched. Options: `delimiter` (default ","). Applies to every line, so
/// scope the step (or drop the header first) if the input carries a header row.
pub fn hash_field(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let field = step.get_usize("field", 1)?;
    let algorithm = step.get_string_or("algorithm", "sha256");
    let encoding = step.get_string_or("encoding", "hex");
    let length = step.get_usize("length", 0)?;
    let salt = step.get_string_or("salt", "");
    let prefix = step.get_string_or("prefix", "");

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        if field == 0 {
            // Whole-line mode: hash the raw line and replace it. Blank lines pass
            // through (nothing to pseudonymize, and a hashed empty is a tell).
            if line.is_empty() {
                out.push(line.clone());
            } else {
                out.push(hash_token(
                    line, &algorithm, &salt, &encoding, length, &prefix,
                )?);
            }
            continue;
        }
        let mut fields = parse_csv_line(line, delim);
        if field >= 1 && field <= fields.len() && !fields[field - 1].is_empty() {
            fields[field - 1] = hash_token(
                &fields[field - 1],
                &algorithm,
                &salt,
                &encoding,
                length,
                &prefix,
            )?;
        }
        out.push(
            fields
                .iter()
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s),
        );
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `cut_fields`: select and/or reorder columns by 1-based index (quote-aware,
/// like `cut`/`awk` but honoring quotes). `fields` is a comma-separated index
/// list (e.g. "1,3,2"); out-of-range indices yield an empty cell. `delimiter`
/// default ",".
pub fn cut_fields(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let indices: Vec<usize> = step
        .get_string_or("fields", "")
        .split(',')
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .collect();
    if indices.is_empty() {
        return Ok(input.to_string());
    }
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| {
            let fields = parse_csv_line(l, delim);
            indices
                .iter()
                .map(|&i| {
                    fields
                        .get(i.saturating_sub(1))
                        .map(|c| c.as_str())
                        .unwrap_or("")
                })
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s)
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `explode_field`: split a field's inner list into multiple rows, one per value
/// (the rest of the row repeated). Options: `field` (1-based, default 1),
/// `delimiter` (row delimiter, default ","), `separator` (inner list separator,
/// default ";"). Rows whose field has no separator pass through.
pub fn explode_field(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let field = step.get_usize("field", 1)?;
    let sep = step.get_string_or("separator", ";");
    if sep.is_empty() {
        return Ok(input.to_string());
    }
    let s = split(input);
    let mut out: Vec<String> = Vec::new();
    for l in &s.lines {
        let fields = parse_csv_line(l, delim);
        if field < 1 || field > fields.len() {
            out.push(l.clone());
            continue;
        }
        let idx = field - 1;
        let values: Vec<&str> = fields[idx].split(sep.as_str()).collect();
        if values.len() <= 1 {
            out.push(l.clone());
            continue;
        }
        for v in values {
            let mut row = fields.clone();
            row[idx] = v.trim().to_string();
            out.push(
                row.iter()
                    .map(|f| csv_field(f, &delim_s))
                    .collect::<Vec<_>>()
                    .join(&delim_s),
            );
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Convert one epoch-seconds (or -millis) token to an ISO string, or None if it is
/// not an integer.
fn epoch_token_to_iso(v: &str, want_date: bool, millis: bool) -> Option<String> {
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    let n: i64 = t.parse().ok()?;
    let secs = if millis { n.div_euclid(1000) } else { n };
    Some(if want_date {
        crate::datetime::format_date(secs)
    } else {
        crate::datetime::format_datetime(secs)
    })
}

/// `epoch_to_iso`: convert a Unix-epoch number to an ISO date/time. `field` is
/// 1-based (0 = the whole line); `format` = datetime (default, `YYYY-MM-DDTHH:MM:SSZ`)
/// or date (`YYYY-MM-DD`); `unit` = seconds (default) or millis. Values that are not
/// an integer, and empty cells, pass through unchanged. Option `delimiter` (`,`).
pub fn epoch_to_iso(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let field = step.get_usize("field", 1)?;
    let want_date = step.get_string_or("format", "datetime") == "date";
    let millis = step.get_string_or("unit", "seconds") == "millis";
    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        if field == 0 {
            out.push(epoch_token_to_iso(line, want_date, millis).unwrap_or_else(|| line.clone()));
            continue;
        }
        let mut fields = parse_csv_line(line, delim);
        if field >= 1 && field <= fields.len() {
            if let Some(iso) = epoch_token_to_iso(&fields[field - 1], want_date, millis) {
                fields[field - 1] = iso;
            }
        }
        out.push(
            fields
                .iter()
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s),
        );
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Convert one ISO date/datetime token to an epoch number, or None if it does not
/// parse.
fn iso_token_to_epoch(v: &str, millis: bool) -> Option<String> {
    let t = v.trim();
    if t.is_empty() {
        return None;
    }
    let secs = crate::datetime::parse_now(t).ok()?;
    let n = if millis {
        secs.saturating_mul(1000)
    } else {
        secs
    };
    Some(n.to_string())
}

/// `iso_to_epoch`: convert an ISO date/datetime (`YYYY-MM-DD` or
/// `YYYY-MM-DDTHH:MM:SS[Z]`, UTC) to a Unix-epoch number. `field` is 1-based (0 =
/// the whole line); `unit` = seconds (default) or millis. Values that do not parse,
/// and empty cells, pass through unchanged. Option `delimiter` (`,`).
pub fn iso_to_epoch(input: &str, step: &Step) -> Result<String> {
    let delim_s = step.get_string_or("delimiter", ",");
    let delim = first_char(&delim_s, ',');
    let field = step.get_usize("field", 1)?;
    let millis = step.get_string_or("unit", "seconds") == "millis";
    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        if field == 0 {
            out.push(iso_token_to_epoch(line, millis).unwrap_or_else(|| line.clone()));
            continue;
        }
        let mut fields = parse_csv_line(line, delim);
        if field >= 1 && field <= fields.len() {
            if let Some(ep) = iso_token_to_epoch(&fields[field - 1], millis) {
                fields[field - 1] = ep;
            }
        }
        out.push(
            fields
                .iter()
                .map(|f| csv_field(f, &delim_s))
                .collect::<Vec<_>>()
                .join(&delim_s),
        );
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Resolve one `${...}` token against a row: an integer is a 1-based column, a name
/// is a header column (when a header is present). Unknown -> empty.
fn template_col(expr: &str, fields: &[String], header: Option<&[String]>) -> String {
    if let Ok(n) = expr.parse::<usize>() {
        if n >= 1 {
            return fields.get(n - 1).cloned().unwrap_or_default();
        }
    }
    if let Some(h) = header {
        if let Some(pos) = h.iter().position(|c| c == expr) {
            return fields.get(pos).cloned().unwrap_or_default();
        }
    }
    String::new()
}

/// Render one row through the template. `${N}` / `${name}` are replaced by the
/// row's fields; `$$` is a literal `$`.
fn render_row(template: &str, fields: &[String], header: Option<&[String]>) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < template.len() {
        if bytes[i] == b'$' && i + 1 < template.len() {
            if bytes[i + 1] == b'$' {
                out.push('$');
                i += 2;
                continue;
            }
            if bytes[i + 1] == b'{' {
                if let Some(close) = template[i + 2..].find('}') {
                    let expr = template[i + 2..i + 2 + close].trim();
                    out.push_str(&template_col(expr, fields, header));
                    i = i + 2 + close + 1;
                    continue;
                }
            }
        }
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// `shift_dates`: offset every ISO date (`YYYY-MM-DD`) in the text by a fixed
/// number of `days` (may be negative), preserving the interval between dates. Useful
/// for anonymizing test data or shifting a schedule while keeping relative timing.
/// Only whole `YYYY-MM-DD` tokens are touched; unparseable ones pass through.
pub fn shift_dates(input: &str, step: &Step) -> Result<String> {
    let days = step.get_i64("days", 0)?;
    let re = regex::Regex::new(r"\b\d{4}-\d{2}-\d{2}\b")
        .map_err(|e| anyhow!("shift_dates: pattern error: {e}"))?;
    let out = re.replace_all(input, |caps: &regex::Captures| {
        let tok = &caps[0];
        match crate::datetime::parse_now(tok) {
            Ok(epoch) => crate::datetime::format_date(epoch + days * 86_400),
            Err(_) => tok.to_string(),
        }
    });
    Ok(out.into_owned())
}

/// `row_to_template`: render each delimited row through a `template` string,
/// replacing `${N}` (1-based column) and, with `has_header`, `${name}` placeholders
/// with that row's fields (`$$` is a literal `$`). Unlocks generating SQL, config,
/// HTML, YAML, or form text from a table. Blank rows are skipped; with `has_header`
/// the first row supplies names and is not emitted. Option `delimiter` (`,`).
pub fn row_to_template(input: &str, step: &Step) -> Result<String> {
    let template = step
        .get_string("template")
        .ok_or_else(|| anyhow!("row_to_template requires a 'template' option"))?;
    let delim = first_char(&step.get_string_or("delimiter", ","), ',');
    let has_header = step.get_bool("has_header", false)?;
    let s = split(input);
    let (header, data): (Option<Vec<String>>, &[String]) = if has_header && !s.lines.is_empty() {
        (Some(parse_csv_line(&s.lines[0], delim)), &s.lines[1..])
    } else {
        (None, &s.lines[..])
    };
    let mut out: Vec<String> = Vec::new();
    for line in data {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line, delim);
        out.push(render_row(&template, &fields, header.as_deref()));
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

#[cfg(test)]
mod tests {
    use super::paste_column;
    use crate::model::Step;
    use crate::parse_mog;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    fn step(opts: Value) -> Step {
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

    fn srcs(name: &str, vals: &[&str]) -> BTreeMap<String, Vec<String>> {
        let mut m = BTreeMap::new();
        m.insert(
            name.to_string(),
            vals.iter().map(|s| s.to_string()).collect(),
        );
        m
    }

    #[test]
    fn transpose_swaps_rows_and_columns() {
        let out = run(
            r#"{"steps":[{"action":"transpose"}]}"#,
            "id,name\n1,Alice\n2,Bob",
        );
        assert_eq!(out, "id,1,2\nname,Alice,Bob");
    }

    #[test]
    fn unpivot_wide_to_long() {
        let out = run(
            r#"{"steps":[{"action":"unpivot","options":{"id_columns":"1"}}]}"#,
            "id,a,b\n1,x,y\n2,p,q",
        );
        assert_eq!(out, "id,key,value\n1,a,x\n1,b,y\n2,a,p\n2,b,q");
    }

    #[test]
    fn pivot_long_to_wide() {
        let out = run(
            r#"{"steps":[{"action":"pivot","options":{"key_column":"2","value_column":"3","id_columns":"1"}}]}"#,
            "id,attr,val\n1,color,red\n1,size,M\n2,color,blue",
        );
        // Missing (2, size) is a blank cell.
        assert_eq!(out, "id,color,size\n1,red,M\n2,blue,");
    }

    #[test]
    fn pivot_guardrail_refuses_then_override_lifts_it() {
        let m = crate::parse_mog(
            r#"{"steps":[{"action":"pivot","options":{"key_column":"2","value_column":"3","id_columns":"1","max_columns":1}}]}"#,
        )
        .unwrap();
        assert!(crate::execute(&m, "id,k,v\n1,a,x\n1,b,y").is_err());
        // max_columns:0 lifts the cap (the override).
        let out = run(
            r#"{"steps":[{"action":"pivot","options":{"key_column":"2","value_column":"3","id_columns":"1","max_columns":0}}]}"#,
            "id,k,v\n1,a,x\n1,b,y",
        );
        assert_eq!(out, "id,a,b\n1,x,y");
    }

    #[test]
    fn pivot_aggregate_last_and_error() {
        let last = run(
            r#"{"steps":[{"action":"pivot","options":{"key_column":"2","value_column":"3","id_columns":"1","aggregate":"last"}}]}"#,
            "id,k,v\n1,a,first\n1,a,second",
        );
        assert_eq!(last, "id,a\n1,second");
        let m = crate::parse_mog(
            r#"{"steps":[{"action":"pivot","options":{"key_column":"2","value_column":"3","id_columns":"1","aggregate":"error"}}]}"#,
        )
        .unwrap();
        assert!(crate::execute(&m, "id,k,v\n1,a,first\n1,a,second").is_err());
    }

    #[test]
    fn decimal_eu_to_us_and_preserves_bare_integers() {
        let out = run(
            r#"{"steps":[{"action":"decimal_separator_normalize","options":{"from":"eu","to":"us"}}]}"#,
            "Total 1.234.567,89 and 12,50 in 2024",
        );
        assert_eq!(out, "Total 1,234,567.89 and 12.50 in 2024");
    }

    #[test]
    fn decimal_us_to_eu_without_grouping() {
        let out = run(
            r#"{"steps":[{"action":"decimal_separator_normalize","options":{"from":"us","to":"eu","grouping":false}}]}"#,
            "1,234,567.89",
        );
        assert_eq!(out, "1234567,89");
    }

    #[test]
    fn decimal_handles_sign_and_decimal_only() {
        // us -> eu with default grouping: the 4-digit integer part is regrouped.
        let out = run(
            r#"{"steps":[{"action":"decimal_separator_normalize","options":{"from":"us","to":"eu"}}]}"#,
            "-1234.5",
        );
        assert_eq!(out, "-1.234,5");
    }

    #[test]
    fn paste_column_appends_by_index() {
        let s = srcs("ids", &["1", "2", "3"]);
        let st = step(json!({"source": "ids", "delimiter": ","}));
        assert_eq!(
            paste_column("a\nb\nc\n", &st, &s).unwrap(),
            "a,1\nb,2\nc,3\n"
        );
    }

    #[test]
    fn paste_column_left_side_and_blank_when_exhausted() {
        let s = srcs("ids", &["1"]);
        let st = step(json!({"source": "ids", "delimiter": ",", "side": "left"}));
        assert_eq!(paste_column("a\nb\n", &st, &s).unwrap(), "1,a\n,b\n");
    }

    #[test]
    fn paste_column_skip_leaves_exhausted_lines() {
        let s = srcs("ids", &["1"]);
        let st = step(json!({"source": "ids", "delimiter": ",", "on_exhausted": "skip"}));
        assert_eq!(paste_column("a\nb\n", &st, &s).unwrap(), "a,1\nb\n");
    }

    #[test]
    fn paste_column_error_when_source_short() {
        let s = srcs("ids", &["1"]);
        let st = step(json!({"source": "ids", "on_exhausted": "error"}));
        assert!(paste_column("a\nb\n", &st, &s).is_err());
    }

    #[test]
    fn stanzas_to_csv_with_header() {
        let out = run(
            r#"{"steps":[{"action":"records_to_columns"}]}"#,
            "name: Ada\nemail: ada@x.io\n\nname: Bob\nemail: bob@y.io\nphone: 555",
        );
        assert_eq!(out, "name,email,phone\nAda,ada@x.io,\nBob,bob@y.io,555");
    }

    #[test]
    fn quotes_fields_containing_the_delimiter() {
        let out = run(
            r#"{"steps":[{"action":"records_to_columns","options":{"header":false}}]}"#,
            "name: Last, First",
        );
        assert_eq!(out, "\"Last, First\"");
    }

    #[test]
    fn change_delimiter_is_quote_aware() {
        let out = run(
            r#"{"steps":[{"action":"change_delimiter","options":{"from":",","to":"|"}}]}"#,
            "a,\"b,c\",d",
        );
        assert_eq!(out, "a|b,c|d");
    }

    #[test]
    fn fill_down_propagates_last_nonblank() {
        let out = run(
            r#"{"steps":[{"action":"fill_down"}]}"#,
            "2024,Q1,10\n,,20\n2025,Q2,30",
        );
        assert_eq!(out, "2024,Q1,10\n2024,Q1,20\n2025,Q2,30");
    }

    #[test]
    fn mask_field_keeps_last_four() {
        let out = run(
            r#"{"steps":[{"action":"mask_field","options":{"field":2,"keep_end":4}}]}"#,
            "name,4111111111111234",
        );
        assert_eq!(out, "name,************1234");
    }

    #[test]
    fn hash_field_is_stable_and_prefixed() {
        // sha256("bob@x.io") truncated to 12 hex chars, with a "u_" prefix.
        let out = run(
            r#"{"steps":[{"action":"hash_field","options":{"field":1,"length":12,"prefix":"u_"}}]}"#,
            "bob@x.io,login\nada@x.io,login\nbob@x.io,logout",
        );
        let lines: Vec<&str> = out.lines().collect();
        // Same input value -> same token (rows 1 and 3), different from row 2.
        let t1 = lines[0].split(',').next().unwrap();
        let t2 = lines[1].split(',').next().unwrap();
        let t3 = lines[2].split(',').next().unwrap();
        assert_eq!(t1, t3);
        assert_ne!(t1, t2);
        assert!(t1.starts_with("u_"));
        assert_eq!(t1.len(), 2 + 12); // prefix + truncated token
                                      // The second field is untouched.
        assert!(lines[0].ends_with(",login"));
    }

    #[test]
    fn hash_field_leaves_empty_cells() {
        let out = run(
            r#"{"steps":[{"action":"hash_field","options":{"field":2}}]}"#,
            "a,,c",
        );
        assert_eq!(out, "a,,c");
    }

    #[test]
    fn hash_field_salt_changes_the_token() {
        let plain = run(
            r#"{"steps":[{"action":"hash_field","options":{"field":1,"length":16}}]}"#,
            "secret",
        );
        let salted = run(
            r#"{"steps":[{"action":"hash_field","options":{"field":1,"length":16,"salt":"pepper"}}]}"#,
            "secret",
        );
        assert_ne!(plain, salted);
    }

    #[test]
    fn hash_field_whole_line_base64() {
        // field 0 hashes the whole line; base64 tokens are URL-safe (no + / =).
        let out = run(
            r#"{"steps":[{"action":"hash_field","options":{"field":0,"encoding":"base64","length":20}}]}"#,
            "one,two,three",
        );
        let tok = out.trim();
        assert_eq!(tok.len(), 20);
        assert!(tok
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn cut_fields_selects_and_reorders() {
        let out = run(
            r#"{"steps":[{"action":"cut_fields","options":{"fields":"3,1"}}]}"#,
            "a,b,c\nx,y,z",
        );
        assert_eq!(out, "c,a\nz,x");
    }

    #[test]
    fn explode_field_one_row_to_many() {
        let out = run(
            r#"{"steps":[{"action":"explode_field","options":{"field":2,"separator":";"}}]}"#,
            "ada,red;green;blue",
        );
        assert_eq!(out, "ada,red\nada,green\nada,blue");
    }

    #[test]
    fn epoch_to_iso_converts_a_column() {
        let out = run(
            r#"{"steps":[{"action":"epoch_to_iso","options":{"field":2}}]}"#,
            "login,1700000000,ok\nnope,x,ok",
        );
        // Second column becomes an ISO datetime; a non-integer passes through.
        assert_eq!(out, "login,2023-11-14T22:13:20Z,ok\nnope,x,ok");
    }

    #[test]
    fn epoch_to_iso_date_and_millis_whole_line() {
        let out = run(
            r#"{"steps":[{"action":"epoch_to_iso","options":{"field":0,"format":"date","unit":"millis"}}]}"#,
            "1700000000000",
        );
        assert_eq!(out, "2023-11-14");
    }

    #[test]
    fn iso_to_epoch_roundtrips_with_epoch_to_iso() {
        let out = run(
            r#"{"steps":[{"action":"iso_to_epoch","options":{"field":1}}]}"#,
            "2023-11-14T22:13:20Z,event",
        );
        assert_eq!(out, "1700000000,event");
    }

    #[test]
    fn row_to_template_generates_sql_by_index() {
        let out = run(
            r#"{"steps":[{"action":"row_to_template","options":{"template":"INSERT INTO t(id,name) VALUES (${1}, '${2}');"}}]}"#,
            "1,Ada\n2,Bob",
        );
        assert_eq!(
            out,
            "INSERT INTO t(id,name) VALUES (1, 'Ada');\nINSERT INTO t(id,name) VALUES (2, 'Bob');"
        );
    }

    #[test]
    fn row_to_template_uses_header_names_and_literal_dollar() {
        let out = run(
            r#"{"steps":[{"action":"row_to_template","options":{"template":"${name} costs $$${price}","has_header":true}}]}"#,
            "name,price\nWidget,9\nGadget,12",
        );
        assert_eq!(out, "Widget costs $9\nGadget costs $12");
    }
}
