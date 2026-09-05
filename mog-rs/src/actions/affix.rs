//! Affix actions. `prepend` / `append` operate on the whole document; the
//! per-line variants (`prefix_lines`, `suffix_lines`, `wrap_lines`, `indent`,
//! `outdent`) touch every line while preserving EOL style and trailing newline.

use anyhow::Result;

use crate::line_text::{join, split};
use crate::model::Step;

/// `prepend`. Option `text` (string, default "").
pub fn prepend(input: &str, step: &Step) -> Result<String> {
    let text = step.get_string_or("text", "");
    Ok(format!("{text}{input}"))
}

/// `append`. Option `text` (string, default "").
pub fn append(input: &str, step: &Step) -> Result<String> {
    let text = step.get_string_or("text", "");
    Ok(format!("{input}{text}"))
}

/// `insert_if_absent`: prepend (or append) `text` only when `marker` is not already
/// present, so re-running is idempotent (no double insertion). `marker` defaults to
/// `text`; set `regex` to treat it as a pattern. `position` = prepend (default) or
/// append. Ideal for stamping a license/banner header exactly once.
pub fn insert_if_absent(input: &str, step: &Step) -> Result<String> {
    let text = step.get_string_or("text", "");
    let marker = step.get_string("marker").unwrap_or_else(|| text.clone());
    let present = if step.get_bool("regex", false)? {
        fancy_regex::Regex::new(&marker)
            .map_err(|e| anyhow::anyhow!("insert_if_absent: pattern error: {e}"))?
            .is_match(input)
            .unwrap_or(false)
    } else {
        input.contains(marker.as_str())
    };
    if present {
        return Ok(input.to_string());
    }
    Ok(if step.get_string_or("position", "prepend") == "append" {
        format!("{input}{text}")
    } else {
        format!("{text}{input}")
    })
}

/// `prefix_lines`. Put option `text` at the start of every line.
pub fn prefix_lines(input: &str, step: &Step) -> Result<String> {
    let text = step.get_string_or("text", "");
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| format!("{text}{l}")).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `suffix_lines`. Put option `text` at the end of every line.
pub fn suffix_lines(input: &str, step: &Step) -> Result<String> {
    let text = step.get_string_or("text", "");
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| format!("{l}{text}")).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `wrap_lines`. Put option `prefix` before and option `suffix` after every line.
pub fn wrap_lines(input: &str, step: &Step) -> Result<String> {
    let prefix = step.get_string_or("prefix", "");
    let suffix = step.get_string_or("suffix", "");
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| format!("{prefix}{l}{suffix}"))
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `indent`. Add leading indentation to every line. Option `spaces` (integer,
/// default 4); if option `text` is given, that string is used instead.
pub fn indent(input: &str, step: &Step) -> Result<String> {
    let pad = match step.get_string("text") {
        Some(t) => t,
        None => " ".repeat(step.get_usize("spaces", 4)?),
    };
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| format!("{pad}{l}")).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `outdent`. Remove up to `spaces` (default 4) leading spaces from every line;
/// a single leading tab is also removed as one indent level.
pub fn outdent(input: &str, step: &Step) -> Result<String> {
    let spaces = step.get_usize("spaces", 4)?;
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| outdent_line(l, spaces)).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

fn outdent_line(line: &str, spaces: usize) -> String {
    // A leading tab counts as a full indent level.
    if let Some(rest) = line.strip_prefix('\t') {
        return rest.to_string();
    }
    let mut removed = 0;
    let mut idx = 0;
    for c in line.chars() {
        if c == ' ' && removed < spaces {
            removed += 1;
            idx += 1;
        } else {
            break;
        }
    }
    line[idx..].to_string()
}

/// `format_list`: turn the input lines into one structured snippet. Each line is
/// rendered through the `item` template (default "$0", where `$0` is the line),
/// items are joined by `separator`, and the whole is wrapped in `header` /
/// `footer`. Handles the comma-between-not-after-last case (SQL IN lists, JSON
/// arrays, HTML lists). `$$` yields a literal `$` in the item template.
pub fn format_list(input: &str, step: &Step) -> Result<String> {
    let header = step.get_string_or("header", "");
    let item = step.get_string_or("item", "$0");
    let separator = step.get_string_or("separator", "");
    let footer = step.get_string_or("footer", "");
    let s = split(input);
    let rendered: Vec<String> = s.lines.iter().map(|l| render_item(&item, l)).collect();
    Ok(format!("{header}{}{footer}", rendered.join(&separator)))
}

/// Render one item template: `$0` -> the line, `$$` -> a literal `$`.
fn render_item(template: &str, line: &str) -> String {
    let mut out = String::with_capacity(template.len() + line.len());
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            match chars.peek() {
                Some('0') => {
                    chars.next();
                    out.push_str(line);
                }
                Some('$') => {
                    chars.next();
                    out.push('$');
                }
                _ => out.push('$'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod added_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn format_list_sql_in_clause() {
        let out = run(
            r#"{"steps":[{"action":"format_list","options":{"header":"IN (","item":"'$0'","separator":", ","footer":")"}}]}"#,
            "a\nb\nc",
        );
        assert_eq!(out, "IN ('a', 'b', 'c')");
    }

    #[test]
    fn format_list_default_item_is_identity() {
        let out = run(
            r#"{"steps":[{"action":"format_list","options":{"separator":","}}]}"#,
            "x\ny",
        );
        assert_eq!(out, "x,y");
    }
}
