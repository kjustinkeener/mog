//! Per-line escaping: turn each line into a safely-escaped value for a target
//! context (regex, POSIX shell, SQL string literal, CSV field). Hand-escaping
//! these is easy to get subtly wrong; these actions are deterministic and
//! reviewable. Each line is escaped independently; blank lines pass through as
//! that context's empty value.

use anyhow::Result;

use crate::line_text::{join, split};
use crate::model::Step;

/// Apply `f` to each line, preserving the detected EOL and trailing terminator.
fn per_line(input: &str, mut f: impl FnMut(&str) -> String) -> String {
    let s = split(input);
    let mapped: Vec<String> = s.lines.iter().map(|l| f(l)).collect();
    join(&mapped, s.eol, s.trailing_eol)
}

/// `escape_regex`: escape regex metacharacters in each line so the text matches
/// literally when dropped into a pattern. Uses the same escaping as the `regex`
/// crate, so it is exact for mog's own regex actions.
pub fn escape_regex(input: &str, _step: &Step) -> Result<String> {
    Ok(per_line(input, regex::escape))
}

/// `escape_shell`: single-quote each line for safe use as one POSIX-shell word,
/// with any embedded `'` written as the `'\''` idiom. A blank line becomes `''`.
pub fn escape_shell(input: &str, _step: &Step) -> Result<String> {
    Ok(per_line(input, |l| {
        format!("'{}'", l.replace('\'', "'\\''"))
    }))
}

/// `escape_sql`: SQL string-literal escape each line (embedded `'` doubled to
/// `''`). With `quote` (default true) the whole value is wrapped in single quotes;
/// set it false to escape without wrapping (e.g. building a larger literal).
pub fn escape_sql(input: &str, step: &Step) -> Result<String> {
    let quote = step.get_bool("quote", true)?;
    Ok(per_line(input, |l| {
        let escaped = l.replace('\'', "''");
        if quote {
            format!("'{escaped}'")
        } else {
            escaped
        }
    }))
}

/// `escape_csv`: quote each line as one RFC-4180 CSV field. A field is quoted only
/// when it contains the delimiter, a double quote, or a CR/LF (embedded `"`
/// doubled); set `always` to quote every field. `delimiter` (default `,`) is the
/// delimiter to guard against.
pub fn escape_csv(input: &str, step: &Step) -> Result<String> {
    let delim = step
        .get_string("delimiter")
        .and_then(|d| d.chars().next())
        .unwrap_or(',');
    let always = step.get_bool("always", false)?;
    Ok(per_line(input, |l| {
        let needs =
            always || l.contains(delim) || l.contains('"') || l.contains('\r') || l.contains('\n');
        if needs {
            format!("\"{}\"", l.replace('"', "\"\""))
        } else {
            l.to_string()
        }
    }))
}

/// `escape_json`: escape each line as a JSON string value. Control characters
/// become their short escapes (`\n` `\t` `\r` `\b` `\f`) or `\u00XX`; `"` and `\`
/// are backslash-escaped. With `quote` (default true) the whole value is wrapped in
/// double quotes; set it false to escape without wrapping (building a larger
/// string). Non-ASCII characters pass through (JSON is UTF-8).
pub fn escape_json(input: &str, step: &Step) -> Result<String> {
    let quote = step.get_bool("quote", true)?;
    Ok(per_line(input, |l| {
        let mut e = String::with_capacity(l.len() + 2);
        if quote {
            e.push('"');
        }
        for c in l.chars() {
            match c {
                '"' => e.push_str("\\\""),
                '\\' => e.push_str("\\\\"),
                '\n' => e.push_str("\\n"),
                '\r' => e.push_str("\\r"),
                '\t' => e.push_str("\\t"),
                '\u{08}' => e.push_str("\\b"),
                '\u{0c}' => e.push_str("\\f"),
                c if (c as u32) < 0x20 => e.push_str(&format!("\\u{:04x}", c as u32)),
                c => e.push(c),
            }
        }
        if quote {
            e.push('"');
        }
        e
    }))
}

/// `escape_xml`: XML/HTML-escape each line, replacing `& < > " '` with their entity
/// references (`&amp; &lt; &gt; &quot; &apos;`). Safe for both element text and
/// attribute values. `&` is escaped first so existing text is not double-escaped
/// incorrectly.
pub fn escape_xml(input: &str, _step: &Step) -> Result<String> {
    Ok(per_line(input, |l| {
        l.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }))
}

/// `escape_c`: escape each line as a C/C++/Java string literal. `\` and `"` are
/// backslash-escaped, common control characters use their short escapes (`\t` `\r`
/// `\n`), and any other control character uses a 3-digit octal escape (`\ooo`,
/// which never runs into a following digit the way `\xHH` can). With `quote`
/// (default true) the value is wrapped in double quotes.
pub fn escape_c(input: &str, step: &Step) -> Result<String> {
    let quote = step.get_bool("quote", true)?;
    Ok(per_line(input, |l| {
        let mut e = String::with_capacity(l.len() + 2);
        if quote {
            e.push('"');
        }
        for c in l.chars() {
            match c {
                '"' => e.push_str("\\\""),
                '\\' => e.push_str("\\\\"),
                '\n' => e.push_str("\\n"),
                '\r' => e.push_str("\\r"),
                '\t' => e.push_str("\\t"),
                c if (c as u32) < 0x20 => e.push_str(&format!("\\{:03o}", c as u32)),
                c => e.push(c),
            }
        }
        if quote {
            e.push('"');
        }
        e
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

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
    fn regex_escapes_metacharacters() {
        assert_eq!(
            escape_regex("a.b*c+(d)\n", &step(json!({}))).unwrap(),
            "a\\.b\\*c\\+\\(d\\)\n"
        );
    }

    #[test]
    fn shell_quotes_and_handles_embedded_quote() {
        assert_eq!(
            escape_shell("it's fine\nplain\n", &step(json!({}))).unwrap(),
            "'it'\\''s fine'\n'plain'\n"
        );
    }

    #[test]
    fn sql_doubles_quotes_and_wraps_by_default() {
        assert_eq!(
            escape_sql("O'Brien\n", &step(json!({}))).unwrap(),
            "'O''Brien'\n"
        );
        assert_eq!(
            escape_sql("O'Brien\n", &step(json!({"quote": false}))).unwrap(),
            "O''Brien\n"
        );
    }

    #[test]
    fn csv_quotes_only_when_needed() {
        let out = escape_csv("plain\na,b\nsay \"hi\"\n", &step(json!({}))).unwrap();
        assert_eq!(out, "plain\n\"a,b\"\n\"say \"\"hi\"\"\"\n");
    }

    #[test]
    fn csv_always_quotes_when_requested() {
        assert_eq!(
            escape_csv("plain\n", &step(json!({"always": true}))).unwrap(),
            "\"plain\"\n"
        );
    }

    #[test]
    fn json_escapes_quotes_backslash_and_tab() {
        assert_eq!(
            escape_json("a\"b\\c\td\n", &step(json!({}))).unwrap(),
            "\"a\\\"b\\\\c\\td\"\n"
        );
        // Without wrapping quotes.
        assert_eq!(
            escape_json("a\"b\n", &step(json!({"quote": false}))).unwrap(),
            "a\\\"b\n"
        );
    }

    #[test]
    fn xml_escapes_the_five_entities() {
        assert_eq!(
            escape_xml("a & b < c > \"d\" 'e'\n", &step(json!({}))).unwrap(),
            "a &amp; b &lt; c &gt; &quot;d&quot; &apos;e&apos;\n"
        );
    }

    #[test]
    fn c_escapes_quote_backslash_and_tab() {
        assert_eq!(
            escape_c("say \"hi\"\\\tx\n", &step(json!({}))).unwrap(),
            "\"say \\\"hi\\\"\\\\\\tx\"\n"
        );
    }
}
