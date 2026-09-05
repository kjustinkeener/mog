//! `flag_matching`: annotate lines that need human review with a canonical,
//! machine-detectable marker. The guaranteed format (`<TAG>(mog): <message>`) is
//! what lets tooling, the CLI report and the studio, find mog's flags apart
//! from a human's own TODO/FIXME comments.

use anyhow::{anyhow, bail, Result};
use fancy_regex::Regex;

use crate::line_text::{join, split};
use crate::model::Step;

/// The fixed set of tags (a closed dropdown). All are `(mog)`-scoped, so any of
/// them is detected by [`find_flags`].
pub const TAGS: &[&str] = &["FIXME", "TODO", "NOTE", "HACK", "XXX", "WARN"];

/// `flag_matching`: append (or insert) a `<comment> <TAG>(mog): <message>` marker
/// on every line matching `pattern`. `comment` (the line-comment prefix) and
/// `message` are required; `tag` defaults to FIXME; `position` is
/// inline (default), before, or after.
pub fn flag_matching(input: &str, step: &Step) -> Result<String> {
    let pattern = step
        .get_string("pattern")
        .ok_or_else(|| anyhow!("Action 'flag_matching' requires a 'pattern' option."))?;
    let message = step
        .get_string("message")
        .ok_or_else(|| anyhow!("Action 'flag_matching' requires a 'message' option."))?;
    let comment = step.get_string("comment").ok_or_else(|| {
        anyhow!("Action 'flag_matching' requires a 'comment' option (the line-comment prefix, e.g. // or -- or #).")
    })?;

    let tag = step
        .get_string("tag")
        .unwrap_or_else(|| "FIXME".to_string());
    let tag = tag.trim().to_ascii_uppercase();
    if !TAGS.contains(&tag.as_str()) {
        bail!("flag_matching 'tag' must be one of {TAGS:?}, got '{tag}'");
    }
    let position = step
        .get_string("position")
        .unwrap_or_else(|| "inline".to_string());
    let ignore_case = step.get_bool("ignore_case", false)?;

    let full = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.clone()
    };
    let re = Regex::new(&full).map_err(|e| anyhow!("Invalid pattern '{pattern}': {e}"))?;

    let marker = format!("{} {}(mog): {}", comment.trim_end(), tag, message);

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        let matched = re.is_match(line).map_err(|e| anyhow!(e))?;
        if !matched {
            out.push(line.clone());
            continue;
        }
        match position.as_str() {
            "before" => {
                out.push(marker.clone());
                out.push(line.clone());
            }
            "after" => {
                out.push(line.clone());
                out.push(marker.clone());
            }
            _ => out.push(format!("{line} {marker}")),
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// A flag found in text: 1-based line number, tag, and message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    pub line: usize,
    pub tag: String,
    pub message: String,
}

/// Scan `text` for mog flag markers (`<TAG>(mog): <message>`) and return them in
/// order. Used by the CLI (and studio) to report what needs review.
pub fn find_flags(text: &str) -> Vec<Flag> {
    let re = Regex::new(r"\b(FIXME|TODO|NOTE|HACK|XXX|WARN)\(mog\): (.*)$")
        .expect("static flag-marker regex");
    let mut flags = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if let Ok(Some(caps)) = re.captures(line) {
            flags.push(Flag {
                line: idx + 1,
                tag: caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                message: caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            });
        }
    }
    flags
}

/// Built-in secret patterns (kind, regex). Heuristic but tested; the value is a
/// vetted set a caller does not have to re-derive.
const SECRET_PATTERNS: &[(&str, &str)] = &[
    ("aws-key", r"AKIA[0-9A-Z]{16}"),
    ("google-key", r"AIza[0-9A-Za-z_\-]{35}"),
    ("stripe", r"[sr]k_live_[0-9A-Za-z]{10,}"),
    ("github-token", r"gh[pousr]_[0-9A-Za-z]{20,}"),
    ("slack-token", r"xox[baprs]-[0-9A-Za-z-]{10,}"),
    (
        "jwt",
        r"eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+",
    ),
    ("private-key", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    (
        "assignment",
        r"(?i)(?:api[_\-]?key|secret|token|password|passwd)\s*[:=]\s*\S{6,}",
    ),
];

/// Built-in PII patterns (kind, regex). Shaped PII only; names need NER and are
/// out of scope by design (flag, don't fake).
const PII_PATTERNS: &[(&str, &str)] = &[
    ("email", r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}"),
    ("ssn", r"\b\d{3}-\d{2}-\d{4}\b"),
    ("credit-card", r"\b\d{4}[ \-]?\d{4}[ \-]?\d{4}[ \-]?\d{4}\b"),
    (
        "phone",
        r"\b(?:\+?1[ .\-]?)?\(?\d{3}\)?[ .\-]?\d{3}[ .\-]?\d{4}\b",
    ),
    ("ipv4", r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
];

/// Shared detector: annotate each line matching any built-in pattern with a
/// `<comment> <TAG>(mog): possible <label> (<kinds>)` marker (picked up by the
/// CLI flag report / --check). Options: `comment` (default "#"), `tag` (default
/// WARN), `position` (inline/before/after).
fn detect(input: &str, step: &Step, patterns: &[(&str, &str)], label: &str) -> Result<String> {
    let comment = step.get_string_or("comment", "#");
    let tag = step.get_string("tag").unwrap_or_else(|| "WARN".to_string());
    let tag = tag.trim().to_ascii_uppercase();
    if !TAGS.contains(&tag.as_str()) {
        bail!("detect 'tag' must be one of {TAGS:?}, got '{tag}'");
    }
    let position = step
        .get_string("position")
        .unwrap_or_else(|| "inline".to_string());

    let compiled: Vec<(&str, Regex)> = patterns
        .iter()
        .map(|(k, p)| {
            Regex::new(p)
                .map(|re| (*k, re))
                .map_err(|e| anyhow!("detect: bad built-in pattern /{p}/: {e}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        let mut kinds: Vec<&str> = Vec::new();
        for (k, re) in &compiled {
            if re.is_match(line).map_err(|e| anyhow!(e))? {
                kinds.push(k);
            }
        }
        if kinds.is_empty() {
            out.push(line.clone());
            continue;
        }
        let marker = format!(
            "{} {}(mog): possible {label} ({})",
            comment.trim_end(),
            tag,
            kinds.join(", ")
        );
        match position.as_str() {
            "before" => {
                out.push(marker);
                out.push(line.clone());
            }
            "after" => {
                out.push(line.clone());
                out.push(marker);
            }
            _ => out.push(format!("{line} {marker}")),
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `detect_secrets`: flag lines that look like they contain a secret.
pub fn detect_secrets(input: &str, step: &Step) -> Result<String> {
    detect(input, step, SECRET_PATTERNS, "secret")
}

/// `detect_pii`: flag lines that look like they contain shaped PII (email, SSN,
/// card, phone, IP). Names are out of scope (they need NER).
pub fn detect_pii(input: &str, step: &Step) -> Result<String> {
    detect(input, step, PII_PATTERNS, "PII")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(opts: serde_json::Value, input: &str) -> Result<String> {
        let step: Step =
            serde_json::from_value(json!({ "action": "flag_matching", "options": opts })).unwrap();
        flag_matching(input, &step)
    }

    #[test]
    fn inline_flag_default_fixme() {
        let out = run(
            json!({ "pattern": "STORED", "message": "must be IMMUTABLE", "comment": "--" }),
            "a STORED,\nb int\n",
        )
        .unwrap();
        assert_eq!(out, "a STORED, -- FIXME(mog): must be IMMUTABLE\nb int\n");
    }

    #[test]
    fn tag_and_comment_and_position() {
        let out = run(
            json!({ "pattern": "^x", "message": "later", "comment": "//", "tag": "todo", "position": "after" }),
            "x = 1\ny = 2\n",
        )
        .unwrap();
        assert_eq!(out, "x = 1\n// TODO(mog): later\ny = 2\n");
    }

    #[test]
    fn comment_is_required() {
        assert!(run(json!({ "pattern": "x", "message": "m" }), "x\n").is_err());
    }

    #[test]
    fn invalid_tag_errors() {
        assert!(run(
            json!({ "pattern": "x", "message": "m", "comment": "#", "tag": "nope" }),
            "x\n"
        )
        .is_err());
    }

    #[test]
    fn find_flags_scans_markers() {
        let text = "a -- FIXME(mog): fix me\nb\n// TODO(mog): do it\n";
        let flags = find_flags(text);
        assert_eq!(flags.len(), 2);
        assert_eq!(
            flags[0],
            Flag {
                line: 1,
                tag: "FIXME".into(),
                message: "fix me".into()
            }
        );
        assert_eq!(
            flags[1],
            Flag {
                line: 3,
                tag: "TODO".into(),
                message: "do it".into()
            }
        );
    }

    #[test]
    fn detect_secrets_flags_a_key() {
        let step: Step =
            serde_json::from_value(json!({ "action": "detect_secrets", "options": {} })).unwrap();
        let out = detect_secrets("token=sk_live_abcdefghij123\nok line\n", &step).unwrap();
        assert!(out.contains("WARN(mog): possible secret"), "got: {out}");
        assert!(out.contains("ok line\n"));
        assert_eq!(find_flags(&out).len(), 1);
    }

    #[test]
    fn detect_pii_flags_email_and_ssn() {
        let step: Step =
            serde_json::from_value(json!({ "action": "detect_pii", "options": {} })).unwrap();
        let out = detect_pii("contact a@b.io\nssn 123-45-6789\nnothing\n", &step).unwrap();
        assert_eq!(find_flags(&out).len(), 2);
    }
}
