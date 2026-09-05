//! Line-filtering actions: keep or drop lines by a regex pattern. Both preserve
//! EOL style and trailing-newline state via the LineText helper.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;

use crate::line_text::{join, split};
use crate::model::Step;

/// Build the line-matching regex from the `pattern` option, honoring
/// `ignore_case` (default false).
fn build_regex(action: &str, step: &Step) -> Result<Regex> {
    let pattern = step
        .get_string("pattern")
        .ok_or_else(|| anyhow!("Action '{action}' requires a 'pattern' option."))?;
    let ignore_case = step.get_bool("ignore_case", false)?;
    let full = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.clone()
    };
    Regex::new(&full).map_err(|e| anyhow!("Invalid pattern '{pattern}': {e}"))
}

/// `keep_lines_matching`: keep only lines whose text matches `pattern`, plus
/// any surrounding lines asked for by `before` / `after` (`context` sets both,
/// the way grep's -C does). Context lines are kept in their original order and
/// each line is kept once, so overlapping windows do not duplicate a line.
pub fn keep_lines_matching(input: &str, step: &Step) -> Result<String> {
    let re = build_regex("keep_lines_matching", step)?;
    let context = step.get_usize("context", 0)?;
    let before = step.get_usize("before", context)?;
    let after = step.get_usize("after", context)?;
    let s = split(input);
    let mut keep = vec![false; s.lines.len()];
    for (i, line) in s.lines.iter().enumerate() {
        if re.is_match(line).map_err(|e| anyhow!(e))? {
            let lo = i.saturating_sub(before);
            let hi = (i + after).min(s.lines.len().saturating_sub(1));
            for slot in keep.iter_mut().take(hi + 1).skip(lo) {
                *slot = true;
            }
        }
    }
    let kept: Vec<String> = s
        .lines
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(l, _)| l.clone())
        .collect();
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `remove_lines_matching`: drop lines whose text matches `pattern`.
pub fn remove_lines_matching(input: &str, step: &Step) -> Result<String> {
    let re = build_regex("remove_lines_matching", step)?;
    let s = split(input);
    let mut kept = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        if !re.is_match(line).map_err(|e| anyhow!(e))? {
            kept.push(line.clone());
        }
    }
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `insert_before_matching`: before every line matching `pattern`, insert a new
/// line holding the `text` option.
pub fn insert_before_matching(input: &str, step: &Step) -> Result<String> {
    insert_matching(input, step, "insert_before_matching", false)
}

/// `insert_after_matching`: after every line matching `pattern`, insert a new
/// line holding the `text` option.
pub fn insert_after_matching(input: &str, step: &Step) -> Result<String> {
    insert_matching(input, step, "insert_after_matching", true)
}

fn insert_matching(input: &str, step: &Step, action: &str, after: bool) -> Result<String> {
    let re = build_regex(action, step)?;
    let text = step.get_string_or("text", "");
    let s = split(input);
    let mut out = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        let matched = re.is_match(line).map_err(|e| anyhow!(e))?;
        if matched && !after {
            out.push(text.clone());
        }
        out.push(line.clone());
        if matched && after {
            out.push(text.clone());
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}
