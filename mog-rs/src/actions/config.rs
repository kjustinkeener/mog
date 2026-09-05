//! Config-format conversions between INI and TOML. Uses the bundled `toml` crate
//! to read TOML robustly; INI is parsed with a small line reader (sections +
//! key=value + `;`/`#` comments). Best-effort for the common flat/one-level shape:
//! TOML arrays flatten to comma-separated INI values, and INI value types are
//! inferred back on the way in. Deeply nested TOML degrades to dotted sections.

use anyhow::{anyhow, Result};

use crate::model::Step;

/// Render a TOML scalar (or array) as an INI value.
fn ini_scalar(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Datetime(d) => d.to_string(),
        toml::Value::Array(a) => a.iter().map(ini_scalar).collect::<Vec<_>>().join(", "),
        toml::Value::Table(_) => String::new(),
    }
}

fn emit_ini_section(out: &mut String, name: &str, table: &toml::map::Map<String, toml::Value>) {
    out.push_str(&format!("\n[{name}]\n"));
    for (k, v) in table {
        if !v.is_table() {
            out.push_str(&format!("{k} = {}\n", ini_scalar(v)));
        }
    }
    for (k, v) in table {
        if let Some(sub) = v.as_table() {
            emit_ini_section(out, &format!("{name}.{k}"), sub);
        }
    }
}

/// `toml_to_ini`: convert TOML to INI. Top-level scalars come first, then each
/// table becomes a `[section]` (nested tables become dotted sections). Arrays are
/// comma-joined.
pub fn toml_to_ini(input: &str, _step: &Step) -> Result<String> {
    let value: toml::Value =
        toml::from_str(input).map_err(|e| anyhow!("toml_to_ini: invalid TOML: {e}"))?;
    let table = value
        .as_table()
        .ok_or_else(|| anyhow!("toml_to_ini: top level is not a table"))?;

    let mut out = String::new();
    for (k, v) in table {
        if !v.is_table() {
            out.push_str(&format!("{k} = {}\n", ini_scalar(v)));
        }
    }
    for (k, v) in table {
        if let Some(sub) = v.as_table() {
            emit_ini_section(&mut out, k, sub);
        }
    }
    Ok(out)
}

/// Render a raw INI value as a TOML value literal: bare for bool/int/float, else a
/// quoted string (existing surrounding quotes are unwrapped first).
fn toml_scalar(raw: &str) -> String {
    if raw == "true" || raw == "false" {
        return raw.to_string();
    }
    if raw.parse::<i64>().is_ok() {
        return raw.to_string();
    }
    if raw.chars().any(|c| c == '.' || c == 'e' || c == 'E') && raw.parse::<f64>().is_ok() {
        return raw.to_string();
    }
    let inner = raw
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(raw);
    format!("\"{}\"", inner.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `ini_to_toml`: convert INI to TOML. `[section]` headers pass through, `key =
/// value` lines get a type-inferred TOML value, and `;`/`#` comment lines become
/// TOML `#` comments. Blank lines are preserved.
pub fn ini_to_toml(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::new();
    for line in input.lines() {
        let t = line.trim();
        if t.is_empty() {
            out.push('\n');
        } else if let Some(rest) = t.strip_prefix(';').or_else(|| t.strip_prefix('#')) {
            out.push_str(&format!("#{rest}\n"));
        } else if t.starts_with('[') && t.ends_with(']') {
            out.push_str(&format!("[{}]\n", t[1..t.len() - 1].trim()));
        } else if let Some(eq) = t.find('=') {
            let key = t[..eq].trim();
            let val = t[eq + 1..].trim();
            out.push_str(&format!("{key} = {}\n", toml_scalar(val)));
        }
    }
    Ok(out)
}

/// `toml_to_json`: parse TOML and emit it as JSON (pretty by default, `compact`
/// for one line). Key order is preserved.
pub fn toml_to_json(input: &str, step: &Step) -> Result<String> {
    let v: serde_json::Value =
        toml::from_str(input).map_err(|e| anyhow!("toml_to_json: invalid TOML: {e}"))?;
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

/// `json_to_toml`: parse a JSON object and emit it as TOML. The top level must be
/// an object and JSON `null` has no TOML equivalent (both are clear errors).
pub fn json_to_toml(input: &str, _step: &Step) -> Result<String> {
    let v: serde_json::Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_to_toml: invalid JSON: {e}"))?;
    toml::to_string_pretty(&v).map_err(|e| {
        anyhow!("json_to_toml: cannot represent this JSON as TOML ({e}); TOML needs a top-level object and has no null")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn step() -> Step {
        Step {
            description: None,
            section: None,
            action: None,
            disabled: false,
            only_lines_matching: None,
            except_lines_matching: None,
            match_ignore_case: false,
            scope: None,
            options: json!({}).as_object().cloned().unwrap_or_default(),
        }
    }

    #[test]
    fn toml_to_ini_sections() {
        let input = "title = \"App\"\nport = 8080\n\n[db]\nhost = \"localhost\"\npool = 5\n";
        let ini = toml_to_ini(input, &step()).unwrap();
        assert!(ini.contains("title = App"));
        assert!(ini.contains("port = 8080"));
        assert!(ini.contains("[db]"));
        assert!(ini.contains("host = localhost"));
    }

    #[test]
    fn toml_json_roundtrip() {
        let toml_in = "title = \"App\"\nport = 8080\n";
        let j = toml_to_json(toml_in, &step()).unwrap();
        assert!(j.contains("\"title\": \"App\""));
        assert!(j.contains("\"port\": 8080"));
        let back = json_to_toml(&j, &step()).unwrap();
        assert!(back.contains("title = \"App\""));
        assert!(back.contains("port = 8080"));
    }

    #[test]
    fn ini_to_toml_infers_types() {
        let input =
            "; a comment\ntitle = App\nport = 8080\n\n[db]\nhost = localhost\ndebug = true\n";
        let toml_out = ini_to_toml(input, &step()).unwrap();
        assert!(toml_out.contains("# a comment"));
        assert!(toml_out.contains("title = \"App\""));
        assert!(toml_out.contains("port = 8080"));
        assert!(toml_out.contains("[db]"));
        assert!(toml_out.contains("debug = true"));
        // The emitted TOML must re-parse.
        toml::from_str::<toml::Value>(&toml_out).expect("emitted TOML parses");
    }
}
