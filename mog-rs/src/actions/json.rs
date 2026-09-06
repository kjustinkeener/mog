//! JSON *reading* actions (serde-backed). mog is a line/regex engine, so these
//! read JSON as input and emit plain text/lines/CSV -- they never try to build
//! JSON with regex. Scope is deliberately bounded to reading/extraction/reformat,
//! NOT a query language: dotted paths + `[n]` index + `[*].field` array-collect,
//! plus a `||` fallback between paths. For predicates, joins, or nested transforms,
//! pipe to jq.
//!
//! JSONL is the sweet spot: one JSON value per line, so per-line actions are
//! linear and streamable. Whole-document actions (pretty/minify) buffer the input.

use anyhow::{anyhow, bail, Result};
use serde_json::{Map, Value};

use crate::model::Step;

/// One segment of an extraction path.
enum Seg {
    Key(String),
    Index(usize),
    /// `[*]`: iterate array elements, applying the remaining segments to each and
    /// collecting the results. On a non-array value it yields nothing (so a `||`
    /// fallback can take over).
    Wild,
}

/// Parse `a.b[0].c[*].d` into segments. Malformed brackets are skipped.
fn parse_path(s: &str) -> Vec<Seg> {
    let mut segs = Vec::new();
    let mut key = String::new();
    let mut chars = s.chars().peekable();
    let flush = |key: &mut String, segs: &mut Vec<Seg>| {
        if !key.is_empty() {
            segs.push(Seg::Key(std::mem::take(key)));
        }
    };
    while let Some(&c) = chars.peek() {
        match c {
            '.' => {
                chars.next();
                flush(&mut key, &mut segs);
            }
            '[' => {
                chars.next();
                flush(&mut key, &mut segs);
                let mut inner = String::new();
                while let Some(&d) = chars.peek() {
                    chars.next();
                    if d == ']' {
                        break;
                    }
                    inner.push(d);
                }
                if inner == "*" {
                    segs.push(Seg::Wild);
                } else if let Ok(n) = inner.parse::<usize>() {
                    segs.push(Seg::Index(n));
                }
            }
            _ => {
                key.push(c);
                chars.next();
            }
        }
    }
    flush(&mut key, &mut segs);
    segs
}

/// A leaf value as text. Only scalars count as leaves (string/number/bool); an
/// object or array yields nothing unless `raw` is set (then compact JSON). This is
/// what lets `content[*].text || content` drop tool-noise: a content array with no
/// text leaves is empty, so the fallback (a bare string content) wins, while a
/// content array of non-text blocks yields nothing at all.
fn leaf(v: &Value, raw: bool) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        other => {
            if raw {
                serde_json::to_string(other).ok()
            } else {
                None
            }
        }
    }
}

/// Walk `segs` from `v`, pushing each resolved leaf into `out`.
fn eval(v: &Value, segs: &[Seg], raw: bool, out: &mut Vec<String>) {
    match segs.split_first() {
        None => {
            if let Some(s) = leaf(v, raw) {
                out.push(s);
            }
        }
        Some((Seg::Key(k), rest)) => {
            if let Value::Object(m) = v {
                if let Some(child) = m.get(k) {
                    eval(child, rest, raw, out);
                }
            }
        }
        Some((Seg::Index(i), rest)) => {
            if let Value::Array(a) = v {
                if let Some(child) = a.get(*i) {
                    eval(child, rest, raw, out);
                }
            }
        }
        Some((Seg::Wild, rest)) => {
            if let Value::Array(a) = v {
                for el in a {
                    eval(el, rest, raw, out);
                }
            }
        }
    }
}

/// Evaluate a path expression against `root`, trying each `||`-separated fallback
/// in order and returning the first that yields at least one leaf (joined by `sep`).
fn extract(root: &Value, expr: &str, sep: &str, raw: bool) -> Option<String> {
    for alt in expr.split("||") {
        let segs = parse_path(alt.trim());
        let mut out = Vec::new();
        eval(root, &segs, raw, &mut out);
        if !out.is_empty() {
            return Some(out.join(sep));
        }
    }
    None
}

/// Fill `${path}` placeholders in a template with extracted values (missing ->
/// empty). `$$` is a literal `$`.
fn fill_template(root: &Value, template: &str, sep: &str, raw: bool) -> String {
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
                    let expr = &template[i + 2..i + 2 + close];
                    out.push_str(&extract(root, expr, sep, raw).unwrap_or_default());
                    i = i + 2 + close + 1;
                    continue;
                }
            }
        }
        // push one char (respecting UTF-8 boundaries)
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Run `f` over each line, dropping lines where it returns `None`, and preserve a
/// trailing newline. `f` gets the parsed JSON value, or an error string for a
/// non-JSON (non-blank) line.
fn map_json_lines<F>(input: &str, mut f: F) -> Result<String>
where
    F: FnMut(Result<Value>, &str) -> Option<String>,
{
    let mut out: Vec<String> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed = serde_json::from_str::<Value>(line).map_err(|e| anyhow!("{e}"));
        if let Some(emit) = f(parsed, line) {
            out.push(emit);
        }
    }
    let mut s = out.join("\n");
    if input.ends_with('\n') && !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// How to treat a line that is not valid JSON.
enum OnInvalid {
    Skip,
    Keep,
    Error,
}
fn on_invalid(step: &Step) -> Result<OnInvalid> {
    Ok(
        match step
            .get_enum("on_invalid", "json", "skip", &["skip", "keep", "error"])?
            .as_str()
        {
            "keep" => OnInvalid::Keep,
            "error" => OnInvalid::Error,
            _ => OnInvalid::Skip,
        },
    )
}

/// `json_extract`: pull scalar values out of each JSONL line. With `template`,
/// fill `${path}` placeholders; otherwise emit the single `path`. Lines that yield
/// nothing are dropped.
pub fn json_extract(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", "\n");
    let raw = step.get_bool("raw", false)?;
    let template = step.get_string("template");
    let path = step.get_string("path");
    if template.is_none() && path.is_none() {
        return Err(anyhow!("json_extract requires a 'path' or a 'template'"));
    }
    let inv = on_invalid(step)?;
    let mut err: Option<anyhow::Error> = None;
    let result = map_json_lines(input, |parsed, line| {
        let root = match parsed {
            Ok(v) => v,
            Err(e) => {
                return match inv {
                    OnInvalid::Keep => Some(line.to_string()),
                    OnInvalid::Error => {
                        err.get_or_insert_with(|| anyhow!("json_extract: invalid JSON line: {e}"));
                        None
                    }
                    OnInvalid::Skip => None,
                }
            }
        };
        if let Some(t) = &template {
            Some(fill_template(&root, t, &sep, raw))
        } else {
            extract(&root, path.as_ref().unwrap(), &sep, raw)
        }
    })?;
    if let Some(e) = err {
        return Err(e);
    }
    Ok(result)
}

/// `json_filter`: keep the JSONL lines where `path` matches. With `equals`, keep
/// where the extracted value equals it; with `matches`, where it matches the regex;
/// with neither, keep where the path resolves to any scalar (existence). `invert`
/// flips the decision. Non-JSON lines are dropped (or kept via on_invalid=keep).
pub fn json_filter(input: &str, step: &Step) -> Result<String> {
    let path = step
        .get_string("path")
        .ok_or_else(|| anyhow!("json_filter requires a 'path'"))?;
    let sep = step.get_string_or("separator", ",");
    let raw = step.get_bool("raw", false)?;
    let invert = step.get_bool("invert", false)?;
    let equals = step.get_string("equals");
    let re = match step.get_string("matches") {
        Some(p) => Some(
            regex::Regex::new(&p)
                .map_err(|e| anyhow!("json_filter: invalid 'matches' regex: {e}"))?,
        ),
        None => None,
    };
    let inv = on_invalid(step)?;
    map_json_lines(input, |parsed, line| {
        let root = match parsed {
            Ok(v) => v,
            Err(_) => {
                return match inv {
                    OnInvalid::Keep => Some(line.to_string()),
                    _ => None,
                }
            }
        };
        let got = extract(&root, &path, &sep, raw);
        let mut keep = if let Some(eq) = &equals {
            got.as_deref() == Some(eq.as_str())
        } else if let Some(re) = &re {
            re.is_match(got.as_deref().unwrap_or(""))
        } else {
            got.is_some()
        };
        if invert {
            keep = !keep;
        }
        if keep {
            Some(line.to_string())
        } else {
            None
        }
    })
}

/// `json_keys`: emit each JSONL object's keys, joined by `separator` (default ", ").
pub fn json_keys(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", ", ");
    let inv = on_invalid(step)?;
    map_json_lines(input, |parsed, line| match parsed {
        Ok(Value::Object(m)) => Some(m.keys().cloned().collect::<Vec<_>>().join(&sep)),
        Ok(_) => None,
        Err(_) => match inv {
            OnInvalid::Keep => Some(line.to_string()),
            _ => None,
        },
    })
}

/// `json_minify`: parse the whole input as one JSON value and re-serialize it
/// compactly. Valid by construction (serde round-trip), unlike a regex approach.
pub fn json_minify(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_minify: invalid JSON: {e}"))?;
    serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
}

/// `json_pretty`: parse the whole input as one JSON value and re-serialize it with
/// 2-space indentation.
pub fn json_pretty(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_pretty: invalid JSON: {e}"))?;
    serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
}

/// Join a notebook cell's `source` (a JSON string, or an array of line strings) into
/// one text block. Array elements already carry their own trailing newlines, so they
/// concatenate directly.
fn cell_source_text(src: Option<&Value>) -> String {
    match src {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(lines)) => lines.iter().filter_map(|l| l.as_str()).collect(),
        _ => String::new(),
    }
}

/// `ipynb_to_python`: convert a Jupyter notebook (.ipynb JSON) to a Python script.
/// Each code cell is emitted after a `# %%` cell marker (the Jupytext / VS Code
/// "percent" format); a markdown cell becomes a `# %% [markdown]` block with each
/// line commented, unless `markdown` is "skip". A cell's `source` may be a JSON
/// string or an array of line strings. Options: `cell_marker` (default `# %%`),
/// `markdown` (comment | skip, default comment). Whole-file.
pub fn ipynb_to_python(input: &str, step: &Step) -> Result<String> {
    let marker = step.get_string_or("cell_marker", "# %%");
    let skip_md = step.get_string_or("markdown", "comment") == "skip";
    let nb: Value = serde_json::from_str(input)
        .map_err(|e| anyhow!("ipynb_to_python: invalid notebook JSON: {e}"))?;
    let cells = nb
        .get("cells")
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow!("ipynb_to_python: no 'cells' array (not a Jupyter notebook?)"))?;

    let mut blocks: Vec<String> = Vec::new();
    for cell in cells {
        let ctype = cell.get("cell_type").and_then(|t| t.as_str()).unwrap_or("");
        let src = cell_source_text(cell.get("source"));
        let block = match ctype {
            "code" => {
                let mut b = format!("{marker}\n{src}");
                if !b.ends_with('\n') {
                    b.push('\n');
                }
                b
            }
            "markdown" | "raw" if !skip_md => {
                let mut b = format!("{marker} [markdown]\n");
                for line in src.trim_end_matches('\n').split('\n') {
                    if line.is_empty() {
                        b.push_str("#\n");
                    } else {
                        b.push_str("# ");
                        b.push_str(line);
                        b.push('\n');
                    }
                }
                b
            }
            _ => continue,
        };
        blocks.push(block);
    }
    // A blank line between blocks (each block already ends in a newline).
    Ok(blocks.join("\n"))
}

/// `json_unescape`: decode JSON string escapes (`\n \t \r \b \f \/ \" \\ \uXXXX`)
/// in the input text. The inverse of the json-escape mog.
pub fn json_unescape(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('/') => out.push('/'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('u') => {
                let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    Some(ch) => out.push(ch),
                    None => {
                        out.push('\\');
                        out.push('u');
                        out.push_str(&hex);
                    }
                }
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    Ok(out)
}

/// CSV-quote a field if it contains a comma, quote, CR, or LF.
fn csv_quote(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// `json_to_csv`: flatten a JSON array (or JSONL) of FLAT objects into CSV. Header
/// is the union of keys in first-seen order; nested values become compact JSON.
/// Missing keys are empty cells.
pub fn json_to_csv(input: &str, _step: &Step) -> Result<String> {
    // Accept either a single JSON array document or newline-delimited objects.
    let trimmed = input.trim_start();
    let rows: Vec<Value> = if trimmed.starts_with('[') {
        match serde_json::from_str(input)
            .map_err(|e| anyhow!("json_to_csv: invalid JSON array: {e}"))?
        {
            Value::Array(a) => a,
            other => vec![other],
        }
    } else {
        let mut rows = Vec::new();
        for line in input.lines() {
            if line.trim().is_empty() {
                continue;
            }
            rows.push(
                serde_json::from_str(line)
                    .map_err(|e| anyhow!("json_to_csv: invalid JSON line: {e}"))?,
            );
        }
        rows
    };

    let mut headers: Vec<String> = Vec::new();
    for r in &rows {
        if let Value::Object(m) = r {
            for k in m.keys() {
                if !headers.iter().any(|h| h == k) {
                    headers.push(k.clone());
                }
            }
        }
    }

    let mut out = String::new();
    out.push_str(
        &headers
            .iter()
            .map(|h| csv_quote(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    for r in &rows {
        out.push('\n');
        let cells: Vec<String> = headers
            .iter()
            .map(|h| {
                let cell = r.get(h).and_then(|v| leaf(v, true)).unwrap_or_default();
                csv_quote(&cell)
            })
            .collect();
        out.push_str(&cells.join(","));
    }
    out.push('\n');
    Ok(out)
}

/// Turn a CSV cell into a JSON value. Without `infer` everything is a string;
/// with it, `true`/`false` -> bool, an integer or float -> number, empty -> null.
fn cell_value(raw: &str, infer: bool) -> Value {
    if !infer {
        return Value::String(raw.to_string());
    }
    match raw {
        "" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if let Ok(i) = raw.parse::<i64>() {
                Value::Number(i.into())
            } else if let Some(n) = raw
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
            {
                Value::Number(n)
            } else {
                Value::String(raw.to_string())
            }
        }
    }
}

/// `csv_to_json`: read CSV (first line = header) into JSON objects keyed by header.
/// Emits JSONL (one object per line) by default, or a single JSON array with
/// `array`. `delimiter` (default ","), `infer` types off by default (all strings).
/// Quote-aware but single-line (no newlines inside quoted fields), like mog's other
/// CSV ops. The read counterpart to `json_to_csv`.
pub fn csv_to_json(input: &str, step: &Step) -> Result<String> {
    let delim = step
        .get_string_or("delimiter", ",")
        .chars()
        .next()
        .unwrap_or(',');
    let array = step.get_bool("array", false)?;
    let infer = step.get_bool("infer", false)?;

    let records = crate::actions::data::read_csv_records(input, delim as u8)?;
    let mut it = records.into_iter();
    let header = match it.next() {
        Some(h) => h,
        None => return Ok(String::new()),
    };

    let mut rows: Vec<Value> = Vec::new();
    for fields in it {
        let mut map = serde_json::Map::new();
        for (i, key) in header.iter().enumerate() {
            let raw = fields.get(i).map(String::as_str).unwrap_or("");
            map.insert(key.clone(), cell_value(raw, infer));
        }
        rows.push(Value::Object(map));
    }

    if array {
        let mut s = serde_json::to_string(&Value::Array(rows)).map_err(|e| anyhow!("{e}"))?;
        s.push('\n');
        Ok(s)
    } else {
        let mut s = rows
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");
        if input.ends_with('\n') && !s.is_empty() {
            s.push('\n');
        }
        Ok(s)
    }
}

/// `properties_to_json`: parse a Java `.properties` file (key=value or key:value,
/// `#`/`!` comments) into a flat JSON object of string values. Dotted keys are kept
/// as-is (flat).
pub fn properties_to_json(input: &str, step: &Step) -> Result<String> {
    let mut map = serde_json::Map::new();
    for line in input.lines() {
        let t = line.trim_start();
        if t.is_empty() || t.starts_with('#') || t.starts_with('!') {
            continue;
        }
        if let Some(idx) = t.find(['=', ':']) {
            let key = t[..idx].trim().to_string();
            if !key.is_empty() {
                map.insert(key, Value::String(t[idx + 1..].trim().to_string()));
            }
        }
    }
    let v = Value::Object(map);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_wrap`: nest the whole JSON value under a dotted `path`, creating the
/// intermediate objects (e.g. path `services.app.environment` wraps the input in
/// `{services:{app:{environment: <input>}}}`). Composition primitive for shaping.
pub fn json_wrap(input: &str, step: &Step) -> Result<String> {
    let path = step
        .get_string("path")
        .ok_or_else(|| anyhow!("json_wrap requires a 'path' option"))?;
    let mut cur: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_wrap: invalid JSON: {e}"))?;
    for seg in path.split('.').rev() {
        let mut m = serde_json::Map::new();
        m.insert(seg.to_string(), cur);
        cur = Value::Object(m);
    }
    if step.get_bool("compact", false)? {
        serde_json::to_string(&cur).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&cur).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_merge`: shallow-merge the JSON `object` option in front of the input
/// object (the object's keys come first; the input's keys are added after and win
/// on conflict). A shaping primitive: e.g. wrap env vars under `data`, then merge
/// `{apiVersion, kind, metadata}` to build a k8s manifest.
pub fn json_merge(input: &str, step: &Step) -> Result<String> {
    let obj_str = step
        .get_string("object")
        .ok_or_else(|| anyhow!("json_merge requires an 'object' (a JSON object to merge in)"))?;
    let extra: Value = serde_json::from_str(&obj_str)
        .map_err(|e| anyhow!("json_merge: invalid 'object' JSON: {e}"))?;
    let base: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_merge: invalid JSON input: {e}"))?;
    let (extra, base) = match (extra, base) {
        (Value::Object(e), Value::Object(b)) => (e, b),
        _ => {
            return Err(anyhow!(
                "json_merge: both the input and 'object' must be JSON objects"
            ))
        }
    };
    let mut out = serde_json::Map::new();
    for (k, v) in extra {
        out.insert(k, v);
    }
    for (k, v) in base {
        out.insert(k, v);
    }
    let v = Value::Object(out);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// Compact string form of a scalar JSON value (non-scalars -> compact JSON).
fn scalar_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Insert into `map`, turning a repeated key into an array.
fn insert_multi(map: &mut serde_json::Map<String, Value>, key: String, val: Value) {
    if let Some(existing) = map.get_mut(&key) {
        if let Value::Array(arr) = existing {
            arr.push(val);
        } else {
            let prev = existing.take();
            *existing = Value::Array(vec![prev, val]);
        }
    } else {
        map.insert(key, val);
    }
}

fn strip_quotes(s: &str) -> String {
    let b = s.as_bytes();
    if s.len() >= 2
        && ((b[0] == b'"' && b[s.len() - 1] == b'"') || (b[0] == b'\'' && b[s.len() - 1] == b'\''))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// `env_to_json`: parse a `.env` file (KEY=VALUE, `#` comments, optional `export`,
/// optional surrounding quotes) into a flat JSON object of string values.
pub fn env_to_json(input: &str, step: &Step) -> Result<String> {
    let mut map = serde_json::Map::new();
    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let t = t.strip_prefix("export ").unwrap_or(t);
        if let Some(eq) = t.find('=') {
            let key = t[..eq].trim().to_string();
            if key.is_empty() {
                continue;
            }
            map.insert(key, Value::String(strip_quotes(t[eq + 1..].trim())));
        }
    }
    let v = Value::Object(map);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_to_env`: flat JSON object -> `KEY=VALUE` lines. Values with spaces or
/// special characters are double-quoted; null is skipped; nested values become
/// compact JSON.
pub fn json_to_env(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_to_env: invalid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("json_to_env: top level must be a JSON object"))?;
    let mut out = String::new();
    for (k, val) in obj {
        if val.is_null() {
            continue;
        }
        let s = scalar_str(val);
        if s.is_empty() || s.contains([' ', '"', '\'', '#', '\n']) {
            out.push_str(&format!(
                "{k}=\"{}\"\n",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        } else {
            out.push_str(&format!("{k}={s}\n"));
        }
    }
    Ok(out)
}

fn urldecode(s: &str) -> String {
    // `+` means space in form-encoded query strings.
    urlencoding::decode(&s.replace('+', "%20"))
        .map(|c| c.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

/// `querystring_to_json`: `a=1&b=2` -> a JSON object (url-decoded; a repeated key
/// becomes an array).
pub fn querystring_to_json(input: &str, step: &Step) -> Result<String> {
    let s = input.trim();
    let s = s.strip_prefix('?').unwrap_or(s);
    let mut map = serde_json::Map::new();
    for pair in s.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        insert_multi(&mut map, urldecode(k), Value::String(urldecode(v)));
    }
    let v = Value::Object(map);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_to_querystring`: a JSON object -> `a=1&b=2` (url-encoded; an array value
/// repeats its key). Null values are skipped.
pub fn json_to_querystring(input: &str, _step: &Step) -> Result<String> {
    let v: Value = serde_json::from_str(input)
        .map_err(|e| anyhow!("json_to_querystring: invalid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("json_to_querystring: top level must be a JSON object"))?;
    let mut parts = Vec::new();
    for (k, val) in obj {
        let ek = urlencoding::encode(k);
        match val {
            Value::Null => {}
            Value::Array(a) => {
                for item in a {
                    parts.push(format!("{ek}={}", urlencoding::encode(&scalar_str(item))));
                }
            }
            other => parts.push(format!("{ek}={}", urlencoding::encode(&scalar_str(other)))),
        }
    }
    Ok(parts.join("&"))
}

/// Parse one logfmt line (`k=v k2="v 2" flag`) into a flat object. A bare word
/// with no `=` becomes `key=true`; bare and quoted values are strings.
fn parse_logfmt(line: &str) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    let mut chars = line.chars().peekable();
    loop {
        while matches!(chars.peek(), Some(' ')) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        let mut key = String::new();
        while let Some(&c) = chars.peek() {
            if c == '=' || c == ' ' {
                break;
            }
            key.push(c);
            chars.next();
        }
        if key.is_empty() {
            break;
        }
        if chars.peek() == Some(&'=') {
            chars.next();
            let val = if chars.peek() == Some(&'"') {
                chars.next();
                let mut v = String::new();
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        if let Some(n) = chars.next() {
                            v.push(match n {
                                'n' => '\n',
                                't' => '\t',
                                '"' => '"',
                                '\\' => '\\',
                                other => other,
                            });
                        }
                    } else if c == '"' {
                        break;
                    } else {
                        v.push(c);
                    }
                }
                Value::String(v)
            } else {
                let mut v = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' {
                        break;
                    }
                    v.push(c);
                    chars.next();
                }
                Value::String(v)
            };
            map.insert(key, val);
        } else {
            map.insert(key, Value::Bool(true));
        }
    }
    map
}

/// `logfmt_to_json`: convert each logfmt line to a flat JSON object (JSONL).
pub fn logfmt_to_json(input: &str, _step: &Step) -> Result<String> {
    let mut out: Vec<String> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(
            serde_json::to_string(&Value::Object(parse_logfmt(line)))
                .map_err(|e| anyhow!("{e}"))?,
        );
    }
    let mut s = out.join("\n");
    if input.ends_with('\n') && !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// Format one JSON value as a logfmt value, quoting when it holds a space, `=`, or
/// `"` (or is empty). Non-scalars become compact JSON, quoted.
fn logfmt_value(v: &Value) -> String {
    let raw = match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => return b.to_string(),
        Value::Number(n) => return n.to_string(),
        Value::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    };
    if raw.is_empty() || raw.contains([' ', '=', '"']) {
        format!("\"{}\"", raw.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        raw
    }
}

/// `json_to_logfmt`: convert each JSONL object to a logfmt line (`k=v ...`). Null
/// values are skipped. The read counterpart to logfmt_to_json.
pub fn json_to_logfmt(input: &str, step: &Step) -> Result<String> {
    let inv = on_invalid(step)?;
    map_json_lines(input, |parsed, line| {
        let obj = match parsed {
            Ok(Value::Object(m)) => m,
            Ok(_) => return None,
            Err(_) => {
                return match inv {
                    OnInvalid::Keep => Some(line.to_string()),
                    _ => None,
                }
            }
        };
        let pairs: Vec<String> = obj
            .iter()
            .filter(|(_, v)| !v.is_null())
            .map(|(k, v)| format!("{k}={}", logfmt_value(v)))
            .collect();
        Some(pairs.join(" "))
    })
}

/// `json_to_jsonl`: expand a JSON array into JSONL (one compact value per line). A
/// non-array document becomes a single line.
pub fn json_to_jsonl(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_to_jsonl: invalid JSON: {e}"))?;
    let items = match v {
        Value::Array(a) => a,
        other => vec![other],
    };
    let mut s = items
        .iter()
        .map(|e| serde_json::to_string(e).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    if !s.is_empty() {
        s.push('\n');
    }
    Ok(s)
}

/// `jsonl_to_json`: collect JSONL (one value per line) into a single JSON array
/// (pretty by default, `compact` for one line).
pub fn jsonl_to_json(input: &str, step: &Step) -> Result<String> {
    let mut items: Vec<Value> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str(line)
                .map_err(|e| anyhow!("jsonl_to_json: invalid JSON line: {e}"))?,
        );
    }
    let v = Value::Array(items);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `yaml_to_json`: parse one YAML document and emit it as JSON (pretty by default,
/// `compact` for one line). Key order is preserved.
pub fn yaml_to_json(input: &str, step: &Step) -> Result<String> {
    let v: Value =
        serde_yaml::from_str(input).map_err(|e| anyhow!("yaml_to_json: invalid YAML: {e}"))?;
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// Find the first balanced `{...}` or `[...]` in `s`, string-aware (brackets and the
/// close char inside a JSON string do not count).
fn extract_balanced(s: &str) -> Option<String> {
    let start = s.find(['{', '['])?;
    let open = s.as_bytes()[start] as char;
    let close = if open == '{' { '}' } else { ']' };
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (i, ch) in s[start..].char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                return Some(s[start..start + i + ch.len_utf8()].to_string());
            }
        }
    }
    None
}

/// `extract_json`: pull a JSON value out of chatty text (e.g. an LLM reply). Prefers
/// a fenced ```json block; otherwise takes the first balanced object/array. With
/// `pretty`, the result is validated and re-indented; otherwise it is returned as
/// found. Errors if no JSON is present.
pub fn extract_json(input: &str, step: &Step) -> Result<String> {
    let fence = regex::Regex::new(r"(?s)```(?:json|JSON)?\s*\r?\n(.*?)```")
        .map_err(|e| anyhow!("extract_json: {e}"))?;
    let found = if let Some(c) = fence.captures(input) {
        c[1].trim().to_string()
    } else {
        extract_balanced(input)
            .ok_or_else(|| anyhow!("extract_json: no JSON object or array found"))?
    };
    if step.get_bool("pretty", false)? {
        let v: Value = serde_json::from_str(&found)
            .map_err(|e| anyhow!("extract_json: extracted text is not valid JSON: {e}"))?;
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    } else {
        Ok(found)
    }
}

/// `json_to_yaml`: parse the whole input as one JSON value and emit it as YAML.
pub fn json_to_yaml(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_to_yaml: invalid JSON: {e}"))?;
    serde_yaml::to_string(&v).map_err(|e| anyhow!("{e}"))
}

// ---- Bounded JSON write (json_set / json_delete) --------------------------
// Deterministic, path-addressed edits over a parsed JSON value. This is NOT a
// query language: dotted keys + `[n]` index + `[*]` array-wildcard, no predicates,
// pipes, or arithmetic. Serde-backed so the output is always valid JSON.

/// One step of a JSON path.
enum PathSeg {
    Key(String),
    Index(usize),
    Wildcard,
}

/// Parse a path like `cells[*].outputs` or `a.b[0].c` into segments.
fn parse_json_path(path: &str) -> Result<Vec<PathSeg>> {
    let mut segs = Vec::new();
    let mut key = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '.' => {
                if !key.is_empty() {
                    segs.push(PathSeg::Key(std::mem::take(&mut key)));
                }
            }
            '[' => {
                if !key.is_empty() {
                    segs.push(PathSeg::Key(std::mem::take(&mut key)));
                }
                let mut inner = String::new();
                for ic in chars.by_ref() {
                    if ic == ']' {
                        break;
                    }
                    inner.push(ic);
                }
                if inner == "*" {
                    segs.push(PathSeg::Wildcard);
                } else {
                    let n = inner
                        .parse::<usize>()
                        .map_err(|_| anyhow!("bad array index '[{inner}]' in path '{path}'"))?;
                    segs.push(PathSeg::Index(n));
                }
            }
            _ => key.push(c),
        }
    }
    if !key.is_empty() {
        segs.push(PathSeg::Key(key));
    }
    if segs.is_empty() {
        bail!("empty path");
    }
    Ok(segs)
}

/// The write to perform at the addressed location(s).
enum JsonOp {
    Set(Value),
    Delete,
    Rename(String),
}

/// Apply `op` at the path `segs` within `node`. For `Set`, missing intermediate
/// object keys are created; a `[*]` fans out to every array element.
fn apply_json_op(node: &mut Value, segs: &[PathSeg], op: &JsonOp) -> Result<()> {
    let (seg, rest) = match segs.split_first() {
        Some(x) => x,
        None => return Ok(()),
    };
    if rest.is_empty() {
        // Final segment: perform the op on this child of `node`.
        match (seg, op) {
            (PathSeg::Key(k), JsonOp::Set(v)) => match node {
                Value::Object(m) => {
                    m.insert(k.clone(), v.clone());
                }
                _ => bail!("json_set: expected an object to set key '{k}'"),
            },
            (PathSeg::Key(k), JsonOp::Delete) => {
                if let Value::Object(m) = node {
                    m.remove(k);
                }
            }
            (PathSeg::Index(i), JsonOp::Set(v)) => match node {
                Value::Array(a) if *i < a.len() => a[*i] = v.clone(),
                Value::Array(_) => bail!("json_set: index {i} out of range"),
                _ => bail!("json_set: expected an array to set index {i}"),
            },
            (PathSeg::Index(i), JsonOp::Delete) => {
                if let Value::Array(a) = node {
                    if *i < a.len() {
                        a.remove(*i);
                    }
                }
            }
            (PathSeg::Wildcard, JsonOp::Set(v)) => match node {
                Value::Array(a) => a.iter_mut().for_each(|e| *e = v.clone()),
                _ => bail!("json_set: [*] expects an array"),
            },
            (PathSeg::Wildcard, JsonOp::Delete) => {
                if let Value::Array(a) = node {
                    a.clear();
                }
            }
            (PathSeg::Key(k), JsonOp::Rename(to)) => {
                if let Value::Object(m) = node {
                    if let Some(v) = m.remove(k) {
                        m.insert(to.clone(), v);
                    }
                }
            }
            (PathSeg::Index(_), JsonOp::Rename(_)) | (PathSeg::Wildcard, JsonOp::Rename(_)) => {
                bail!("json_rename: the path must end in a key name (not an array index or [*])")
            }
        }
        return Ok(());
    }
    // Descend one level.
    match seg {
        PathSeg::Key(k) => {
            if node.get(k).is_none() {
                if let JsonOp::Set(_) = op {
                    if let Value::Object(m) = node {
                        m.insert(k.clone(), Value::Object(Map::new()));
                    }
                }
            }
            if let Some(child) = node.get_mut(k) {
                apply_json_op(child, rest, op)?;
            }
        }
        PathSeg::Index(i) => {
            if let Some(child) = node.get_mut(*i) {
                apply_json_op(child, rest, op)?;
            }
        }
        PathSeg::Wildcard => {
            if let Value::Array(a) = node {
                for child in a.iter_mut() {
                    apply_json_op(child, rest, op)?;
                }
            }
        }
    }
    Ok(())
}

/// Interpret the `value` option as JSON if it parses, else as a plain string.
fn value_option(step: &Step) -> Value {
    let s = step.get_string_or("value", "");
    serde_json::from_str(&s).unwrap_or(Value::String(s))
}

/// Run a JSON write over the whole input, or per line when `jsonl` is set.
fn json_write(input: &str, step: &Step, action: &str, op: JsonOp) -> Result<String> {
    let segs = parse_json_path(
        &step
            .get_string("path")
            .ok_or_else(|| anyhow!("{action} requires a 'path' option"))?,
    )
    .map_err(|e| anyhow!("{action}: {e}"))?;
    if step.get_bool("jsonl", false)? {
        let mut out: Vec<String> = Vec::new();
        for line in input.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let mut v: Value = serde_json::from_str(line)
                .map_err(|e| anyhow!("{action}: invalid JSON line: {e}"))?;
            apply_json_op(&mut v, &segs, &op)?;
            out.push(serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))?);
        }
        let mut s = out.join("\n");
        if input.ends_with('\n') && !s.is_empty() {
            s.push('\n');
        }
        Ok(s)
    } else {
        let mut v: Value =
            serde_json::from_str(input).map_err(|e| anyhow!("{action}: invalid JSON: {e}"))?;
        apply_json_op(&mut v, &segs, &op)?;
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_set`: set the value at a dotted `path` (creating missing object keys).
/// `[n]` indexes an array element, `[*]` sets every element. `value` is parsed as
/// JSON if it can be, otherwise treated as a string. With `jsonl` set, applies to
/// each JSON object line. Bounded write, not a query language.
pub fn json_set(input: &str, step: &Step) -> Result<String> {
    let v = value_option(step);
    json_write(input, step, "json_set", JsonOp::Set(v))
}

/// `json_delete`: remove the key/element at a dotted `path` (`[n]` an array element,
/// `[*]` clears an array). Missing paths are a no-op. With `jsonl` set, applies to
/// each JSON object line.
pub fn json_delete(input: &str, step: &Step) -> Result<String> {
    json_write(input, step, "json_delete", JsonOp::Delete)
}

/// `json_rename`: rename the key at a dotted `path` to `to`, keeping its value. The
/// path must end in a key name (`[*]` earlier renames the key in every array
/// element, e.g. `messages[*].from` -> `role`). A missing key is a no-op. With
/// `jsonl` set, applies to each JSON object line.
pub fn json_rename(input: &str, step: &Step) -> Result<String> {
    let to = step
        .get_string("to")
        .ok_or_else(|| anyhow!("json_rename requires a 'to' option (the new key name)"))?;
    json_write(input, step, "json_rename", JsonOp::Rename(to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn json_set_creates_and_overwrites_by_path() {
        // Set a nested value, creating the intermediate object.
        let out = json_set(
            r#"{"server":{"port":80}}"#,
            &step(json!({"path": "server.host", "value": "localhost"})),
        )
        .unwrap();
        assert!(out.contains("\"host\": \"localhost\""));
        assert!(out.contains("\"port\": 80"));
        // A JSON-parseable value becomes that JSON type.
        let out2 = json_set(r#"{"a":1}"#, &step(json!({"path": "a", "value": "[1,2]"}))).unwrap();
        assert!(out2.contains("\"a\": [\n"));
    }

    #[test]
    fn json_set_wildcard_hits_every_element() {
        let out = json_set(
            r#"{"cells":[{"outputs":[1]},{"outputs":[2,3]}]}"#,
            &step(json!({"path": "cells[*].outputs", "value": "[]"})),
        )
        .unwrap();
        // Both cells' outputs are now empty.
        assert_eq!(out.matches("\"outputs\": []").count(), 2);
    }

    #[test]
    fn json_delete_removes_key_and_ignores_missing() {
        let out = json_delete(
            r#"{"keep":1,"secret":"x"}"#,
            &step(json!({"path": "secret"})),
        )
        .unwrap();
        assert!(!out.contains("secret"));
        assert!(out.contains("\"keep\": 1"));
        // Missing path is a no-op.
        let out2 = json_delete(r#"{"keep":1}"#, &step(json!({"path": "nope.gone"}))).unwrap();
        assert!(out2.contains("\"keep\": 1"));
    }

    #[test]
    fn json_rename_key_including_wildcard() {
        let out = json_rename(
            r#"{"conversations":[{"from":"human","value":"hi"}]}"#,
            &step(json!({"path": "conversations", "to": "messages"})),
        )
        .unwrap();
        assert!(out.contains("\"messages\""));
        assert!(!out.contains("conversations"));
        // Rename a key inside every array element.
        let out2 = json_rename(
            r#"{"messages":[{"from":"human"},{"from":"gpt"}]}"#,
            &step(json!({"path": "messages[*].from", "to": "role"})),
        )
        .unwrap();
        assert_eq!(out2.matches("\"role\"").count(), 2);
        assert!(!out2.contains("\"from\""));
    }

    #[test]
    fn json_set_jsonl_applies_per_line() {
        let out = json_set(
            "{\"id\":1,\"pw\":\"a\"}\n{\"id\":2,\"pw\":\"b\"}\n",
            &step(json!({"path": "pw", "value": "[REDACTED]", "jsonl": true})),
        )
        .unwrap();
        assert_eq!(
            out,
            "{\"id\":1,\"pw\":\"[REDACTED]\"}\n{\"id\":2,\"pw\":\"[REDACTED]\"}\n"
        );
    }

    #[test]
    fn ipynb_to_python_code_and_markdown_cells() {
        let nb = r##"{"cells":[
            {"cell_type":"markdown","source":["# Title\n","intro"]},
            {"cell_type":"code","source":["import os\n","print(os.getcwd())"]},
            {"cell_type":"code","source":"x = 1"}
        ],"metadata":{}}"##;
        let out = ipynb_to_python(nb, &step(json!({}))).unwrap();
        assert_eq!(
            out,
            "# %% [markdown]\n# # Title\n# intro\n\n# %%\nimport os\nprint(os.getcwd())\n\n# %%\nx = 1\n"
        );
    }

    #[test]
    fn ipynb_to_python_can_skip_markdown() {
        let nb = r#"{"cells":[
            {"cell_type":"markdown","source":["notes"]},
            {"cell_type":"code","source":["y = 2\n"]}
        ]}"#;
        let out = ipynb_to_python(nb, &step(json!({"markdown": "skip"}))).unwrap();
        assert_eq!(out, "# %%\ny = 2\n");
    }

    #[test]
    fn ipynb_to_python_rejects_non_notebook() {
        assert!(ipynb_to_python(r#"{"foo":1}"#, &step(json!({}))).is_err());
    }

    #[test]
    fn extract_dotted_and_wildcard() {
        // Assistant-style: content is an array of blocks; only text blocks have .text.
        let line = json!({"message":{"role":"assistant","content":[
            {"type":"thinking","thinking":"hmm"},
            {"type":"text","text":"hello"},
            {"type":"tool_use","name":"x"},
            {"type":"text","text":"world"}
        ]}})
        .to_string();
        let out = json_extract(
            &line,
            &step(json!({"path":"message.content[*].text","separator":" "})),
        )
        .unwrap();
        assert_eq!(out, "hello world");
    }

    #[test]
    fn extract_coalesce_string_or_blocks() {
        // The transcript projection: string content OR text blocks, dropping noise.
        let path = "message.content[*].text||message.content";
        let user = json!({"message":{"role":"user","content":"hi there"}}).to_string();
        let asst =
            json!({"message":{"role":"assistant","content":[{"type":"text","text":"reply"}]}})
                .to_string();
        let tool =
            json!({"message":{"role":"user","content":[{"type":"tool_result","content":"junk"}]}})
                .to_string();
        assert_eq!(
            json_extract(&user, &step(json!({"path":path}))).unwrap(),
            "hi there"
        );
        assert_eq!(
            json_extract(&asst, &step(json!({"path":path}))).unwrap(),
            "reply"
        );
        // tool-only line yields no scalar -> dropped (empty output).
        assert_eq!(
            json_extract(&tool, &step(json!({"path":path}))).unwrap(),
            ""
        );
    }

    #[test]
    fn extract_template() {
        let line = json!({"message":{"role":"assistant","content":[{"type":"text","text":"yo"}]}})
            .to_string();
        let out = json_extract(
            &line,
            &step(json!({"template":"## ${message.role}\n\n${message.content[*].text}"})),
        )
        .unwrap();
        assert_eq!(out, "## assistant\n\nyo");
    }

    #[test]
    fn filter_matches_and_existence() {
        let input = [
            json!({"type":"user","message":{"content":"a"}}).to_string(),
            json!({"type":"system"}).to_string(),
            json!({"type":"assistant","message":{"content":[{"type":"text","text":"b"}]}})
                .to_string(),
        ]
        .join("\n");
        let kept = json_filter(
            &input,
            &step(json!({"path":"type","matches":"^(user|assistant)$"})),
        )
        .unwrap();
        assert_eq!(kept.lines().count(), 2);
    }

    #[test]
    fn minify_pretty_roundtrip() {
        let pretty = json_pretty("{\"b\":1,\"a\":[1,2]}", &step(json!({}))).unwrap();
        assert!(pretty.contains('\n'));
        let mini = json_minify(&pretty, &step(json!({}))).unwrap();
        assert_eq!(mini, "{\"b\":1,\"a\":[1,2]}");
    }

    #[test]
    fn unescape_decodes() {
        assert_eq!(
            json_unescape("a\\nb\\t\\u0041", &step(json!({}))).unwrap(),
            "a\nb\tA"
        );
    }

    #[test]
    fn csv_to_json_jsonl_and_infer() {
        let input = "id,name,active\n1,Ada,true\n2,\"Bo, Jr\",false\n";
        // Default: everything a string.
        let jsonl = csv_to_json(input, &step(json!({}))).unwrap();
        assert_eq!(
            jsonl,
            "{\"id\":\"1\",\"name\":\"Ada\",\"active\":\"true\"}\n{\"id\":\"2\",\"name\":\"Bo, Jr\",\"active\":\"false\"}\n"
        );
        // infer: numbers and bools become typed.
        let typed = csv_to_json(input, &step(json!({"infer":true}))).unwrap();
        assert!(typed.contains("\"id\":1,"));
        assert!(typed.contains("\"active\":true"));
        // array mode wraps into one JSON document.
        let arr = csv_to_json(input, &step(json!({"array":true}))).unwrap();
        assert!(arr.starts_with("[{") && arr.trim_end().ends_with("}]"));
    }

    #[test]
    fn csv_to_json_multiline_quoted_field() {
        // A newline inside a quoted field: the RFC-4180 reader keeps it as one cell.
        let input = "id,note\n1,\"line one\nline two\"\n";
        let out = csv_to_json(input, &step(json!({}))).unwrap();
        assert_eq!(out, "{\"id\":\"1\",\"note\":\"line one\\nline two\"}\n");
    }

    #[test]
    fn yaml_json_roundtrip() {
        let yaml = "name: App\nport: 8080\ntags:\n  - a\n  - b\n";
        let j = yaml_to_json(yaml, &step(json!({"compact":true}))).unwrap();
        assert_eq!(j, r#"{"name":"App","port":8080,"tags":["a","b"]}"#);
        let y = json_to_yaml(&j, &step(json!({}))).unwrap();
        // Round-trips back to equivalent JSON.
        assert_eq!(yaml_to_json(&y, &step(json!({"compact":true}))).unwrap(), j);
    }

    #[test]
    fn env_roundtrip() {
        let env = "# c\nexport NAME=App\nURL=\"http://x y\"\nPORT=8080\n";
        let j = env_to_json(env, &step(json!({"compact":true}))).unwrap();
        assert_eq!(j, r#"{"NAME":"App","URL":"http://x y","PORT":"8080"}"#);
        let back = json_to_env(&j, &step(json!({}))).unwrap();
        assert_eq!(back, "NAME=App\nURL=\"http://x y\"\nPORT=8080\n");
    }

    #[test]
    fn querystring_roundtrip() {
        let qs = "a=1&b=hello+world&a=2";
        let j = querystring_to_json(qs, &step(json!({"compact":true}))).unwrap();
        assert_eq!(j, r#"{"a":["1","2"],"b":"hello world"}"#);
        let back = json_to_querystring(&j, &step(json!({}))).unwrap();
        assert_eq!(back, "a=1&a=2&b=hello%20world");
    }

    #[test]
    fn logfmt_roundtrip() {
        let line = r#"level=info msg="hello world" ok=true count=3"#;
        let j = logfmt_to_json(line, &step(json!({}))).unwrap();
        assert_eq!(
            j,
            r#"{"level":"info","msg":"hello world","ok":"true","count":"3"}"#
        );
        // json -> logfmt quotes the spaced value.
        let back = json_to_logfmt(&j, &step(json!({}))).unwrap();
        assert_eq!(back, r#"level=info msg="hello world" ok=true count=3"#);
    }

    #[test]
    fn to_csv_union_keys() {
        let input = [
            json!({"id":1,"name":"a"}).to_string(),
            json!({"id":2,"city":"x"}).to_string(),
        ]
        .join("\n");
        let csv = json_to_csv(&input, &step(json!({}))).unwrap();
        assert_eq!(csv, "id,name,city\n1,a,\n2,,x\n");
    }
}
