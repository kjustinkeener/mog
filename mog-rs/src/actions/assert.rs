//! Assertion (post-condition) action: validate the working text and fail the run
//! if a condition is not met, otherwise pass the text through unchanged. This
//! turns a transform into an enforced guarantee (e.g. a redaction that fails if a
//! secret survived) and gives CI a content gate. All checks are optional; an
//! assert with none is a pass-through.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;

use crate::line_text::split;
use crate::model::Step;

fn compile(pattern: &str, ignore_case: bool) -> Result<Regex> {
    let full = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.to_string()
    };
    Regex::new(&full).map_err(|e| anyhow!("assert: invalid pattern '{pattern}': {e}"))
}

/// `assert`: fail the pipeline unless every configured condition holds.
/// Options: `not_matches` (fail if the regex is found), `matches` (fail if it is
/// not), `line_count` / `min_lines` / `max_lines` (line-count bounds),
/// `ignore_case`, and `message` (a custom failure message). Returns the input
/// unchanged on success.
pub fn assert(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let custom = step.get_string("message");
    let fail = |detail: String| -> anyhow::Error {
        match &custom {
            Some(m) => anyhow!("{m}"),
            None => anyhow!("assertion failed: {detail}"),
        }
    };

    if let Some(pat) = step.get_string("not_matches") {
        if compile(&pat, ignore_case)?
            .is_match(input)
            .map_err(|e| anyhow!(e))?
        {
            return Err(fail(format!("output still matches /{pat}/")));
        }
    }
    if let Some(pat) = step.get_string("matches") {
        if !compile(&pat, ignore_case)?
            .is_match(input)
            .map_err(|e| anyhow!(e))?
        {
            return Err(fail(format!("output does not match /{pat}/")));
        }
    }

    let lines = split(input).lines.len() as i64;
    // Strict reads: a malformed bound must error, never silently coerce to a value
    // that trivially passes (which would disarm the gate this action exists to be).
    if step.has_key("line_count") {
        let want = step.get_i64_strict("line_count", "assert")?;
        if lines != want {
            return Err(fail(format!("expected {want} lines, found {lines}")));
        }
    }
    if step.has_key("min_lines") {
        let min = step.get_i64_strict("min_lines", "assert")?;
        if lines < min {
            return Err(fail(format!(
                "expected at least {min} lines, found {lines}"
            )));
        }
    }
    if step.has_key("max_lines") {
        let max = step.get_i64_strict("max_lines", "assert")?;
        if lines > max {
            return Err(fail(format!("expected at most {max} lines, found {lines}")));
        }
    }
    Ok(input.to_string())
}

#[cfg(test)]
mod tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> Result<String, anyhow::Error> {
        crate::execute(&parse_mog(json).unwrap(), input)
    }

    #[test]
    fn malformed_line_count_errors_instead_of_disarming_the_gate() {
        // A non-integer line_count used to fall back to the actual count, making the
        // assertion trivially pass (silently disarming the gate). It must error.
        let e = run(
            r#"{"steps":[{"action":"assert","options":{"line_count":"abc"}}]}"#,
            "one\ntwo\n",
        )
        .unwrap_err()
        .to_string();
        assert!(e.contains("must be an integer"), "{e}");
    }

    #[test]
    fn passes_through_when_ok() {
        let out = run(
            r#"{"steps":[{"action":"assert","options":{"not_matches":"SECRET"}}]}"#,
            "all clean here",
        )
        .unwrap();
        assert_eq!(out, "all clean here");
    }

    #[test]
    fn fails_when_forbidden_pattern_present() {
        let err = run(
            r#"{"steps":[{"action":"assert","options":{"not_matches":"sk_live_","message":"secret survived redaction"}}]}"#,
            "token=sk_live_abc",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("secret survived redaction"), "got: {err}");
    }

    #[test]
    fn fails_when_required_pattern_absent() {
        assert!(run(
            r#"{"steps":[{"action":"assert","options":{"matches":"^BEGIN"}}]}"#,
            "no header",
        )
        .is_err());
    }

    #[test]
    fn line_count_bounds() {
        assert!(run(
            r#"{"steps":[{"action":"assert","options":{"line_count":2}}]}"#,
            "a\nb",
        )
        .is_ok());
        assert!(run(
            r#"{"steps":[{"action":"assert","options":{"max_lines":1}}]}"#,
            "a\nb",
        )
        .is_err());
    }

    #[test]
    fn asserts_as_a_gate_after_a_transform() {
        // A redaction step followed by an assert that no secret survived.
        let out = run(
            r#"{"steps":[
                {"action":"replace_regex","options":{"find":"sk_live_\\w+","replace_with":"[redacted]"}},
                {"action":"assert","options":{"not_matches":"sk_live_"}}
            ]}"#,
            "key=sk_live_abc123",
        )
        .unwrap();
        assert_eq!(out, "key=[redacted]");
    }
}
