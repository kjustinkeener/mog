//! Number / math actions: apply arithmetic to each numeric token in scope, round
//! decimals, and zero-pad integer runs. These genuinely cannot compose from the
//! text/replace actions (they need real numeric parsing + deterministic
//! formatting), so they are first-class actions. Every one operates token-by-token
//! over the whole input, which makes them scope-friendly for free: under a `field`
//! or `char_range` scope the engine hands the action just that slice, so
//! `arithmetic` under a `field` scope does column math.

use anyhow::{anyhow, bail, Result};
use regex::Regex;

use crate::model::Step;

/// The default token matcher: a standalone signed integer or decimal, e.g. `-3`,
/// `42`, `3.14`. A leading `-` directly before the digits is treated as a negative
/// sign (so negatives round-trip), and the whole match is parsed as the number.
const DEFAULT_NUMBER_PATTERN: &str = r"-?\d+(\.\d+)?";

/// The arithmetic operation applied to each matched number.
#[derive(Clone, Copy)]
enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn parse_op(s: &str) -> Result<Op> {
    Ok(match s {
        "add" => Op::Add,
        "subtract" => Op::Subtract,
        "multiply" => Op::Multiply,
        "divide" => Op::Divide,
        other => {
            bail!("arithmetic: unknown op '{other}' (expected add, subtract, multiply, divide)")
        }
    })
}

/// Deterministic number formatting rule shared by every number action:
///
/// * if `places` is `Some(p)` (p >= 0), ALWAYS format with exactly `p` decimals
///   (so rounding is explicit and stable), e.g. places=2 -> `3.00`;
/// * else emit the minimal representation: an integral result gets no decimal
///   point (`5`), a fractional result gets its shortest round-tripping decimal
///   with trailing zeros trimmed (`2.5`, not `2.50`).
///
/// Rust's `f64` `Display` already yields the shortest round-tripping decimal and
/// drops the point for integral values, so it is the minimal-representation path.
/// A `-0.0` result is normalized to `0` first so multiply-by-zero never prints a
/// stray minus sign.
fn format_number(val: f64, places: Option<i64>) -> String {
    let val = if val == 0.0 { 0.0 } else { val };
    match places {
        Some(p) if p >= 0 => format!("{:.*}", p as usize, val),
        _ => format!("{val}"),
    }
}

/// Shared core: replace each token matched by `pattern` (parsed as an f64) with
/// `op` applied against `by`, formatted per `format_number`. A matched token that
/// does not parse as a number is left verbatim. `by == 0` under divide is a clean
/// up-front error (the divisor is a single constant, so it is checked once).
fn transform_numbers(
    input: &str,
    pattern: &str,
    op: Op,
    by: f64,
    places: Option<i64>,
) -> Result<String> {
    if matches!(op, Op::Divide) && by == 0.0 {
        bail!("arithmetic: division by zero (by = 0)");
    }
    let re = Regex::new(pattern)
        .map_err(|e| anyhow!("arithmetic: invalid find pattern '{pattern}': {e}"))?;
    let out = re.replace_all(input, |caps: &regex::Captures| {
        let tok = &caps[0];
        match tok.parse::<f64>() {
            Ok(n) => {
                let r = match op {
                    Op::Add => n + by,
                    Op::Subtract => n - by,
                    Op::Multiply => n * by,
                    Op::Divide => n / by,
                };
                format_number(r, places)
            }
            Err(_) => tok.to_string(),
        }
    });
    Ok(out.into_owned())
}

/// `places` option as an explicit `Option`: present -> `Some`, absent -> `None`.
fn places_opt(step: &Step) -> Result<Option<i64>> {
    if step.has_key("places") {
        Ok(Some(step.get_i64("places", 0)?))
    } else {
        Ok(None)
    }
}

/// `arithmetic`: apply `op` (add|subtract|multiply|divide) with `by` to each
/// numeric token in scope. `find` overrides the default token pattern; `places`
/// rounds/formats the result to N decimals.
pub fn arithmetic(input: &str, step: &Step) -> Result<String> {
    let op = parse_op(&step.get_string_or("op", ""))?;
    if !step.has_key("by") {
        bail!("arithmetic: 'by' is required");
    }
    let by = step.get_f64("by", 0.0)?;
    let pattern = step.get_string_or("find", DEFAULT_NUMBER_PATTERN);
    transform_numbers(input, &pattern, op, by, places_opt(step)?)
}

/// `increment_numbers`: add `by` (default 1) to each matched number. A thin
/// convenience over `arithmetic` with op=add.
pub fn increment_numbers(input: &str, step: &Step) -> Result<String> {
    let by = step.get_f64("by", 1.0)?;
    let pattern = step.get_string_or("find", DEFAULT_NUMBER_PATTERN);
    transform_numbers(input, &pattern, Op::Add, by, places_opt(step)?)
}

/// `round_numbers`: round each matched number to `places` decimals (default 0).
/// Implemented as an add-zero through the shared core with `places` always set, so
/// the formatting is the single source of truth.
pub fn round_numbers(input: &str, step: &Step) -> Result<String> {
    let places = step.get_i64("places", 0)?;
    let pattern = step.get_string_or("find", DEFAULT_NUMBER_PATTERN);
    transform_numbers(input, &pattern, Op::Add, 0.0, Some(places))
}

/// `pad_numbers`: left-pad each integer run (`\d+`) to `width` using `pad`
/// (default "0"). A run already `>= width` characters is left unchanged. Only
/// integer runs are targeted, so a leading `-` sign and any decimal part stay
/// outside the padded digits.
pub fn pad_numbers(input: &str, step: &Step) -> Result<String> {
    if !step.has_key("width") {
        bail!("pad_numbers: 'width' is required");
    }
    let width = step.get_i64("width", 0)?;
    if width < 0 {
        bail!("pad_numbers: 'width' must be non-negative");
    }
    let width = width as usize;
    let pad = step.get_string_or("pad", "0");
    if pad.is_empty() {
        bail!("pad_numbers: 'pad' must not be empty");
    }
    let pad_chars: Vec<char> = pad.chars().collect();
    let re = Regex::new(r"\d+").map_err(|e| anyhow!(e))?;
    let out = re.replace_all(input, |caps: &regex::Captures| {
        let tok = &caps[0];
        let len = tok.chars().count();
        if len >= width {
            return tok.to_string();
        }
        let needed = width - len;
        let mut prefix = String::with_capacity(needed);
        for i in 0..needed {
            prefix.push(pad_chars[i % pad_chars.len()]);
        }
        prefix.push_str(tok);
        prefix
    });
    Ok(out.into_owned())
}

#[cfg(test)]
mod tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    fn err(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn arithmetic_add_integers_no_decimal_point() {
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"add","by":10}}]}"#,
            "port 8080\nport 9090",
        );
        assert_eq!(out, "port 8090\nport 9100");
    }

    #[test]
    fn arithmetic_handles_negative_and_decimal() {
        // -5 * 3 = -15 (integral -> no point); 2.5 * 3 = 7.5 (minimal decimal).
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"multiply","by":3}}]}"#,
            "-5 and 2.5",
        );
        assert_eq!(out, "-15 and 7.5");
    }

    #[test]
    fn arithmetic_multiply_by_zero_has_no_negative_zero() {
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"multiply","by":0}}]}"#,
            "-5",
        );
        assert_eq!(out, "0");
    }

    #[test]
    fn arithmetic_divide_with_places_rounds() {
        // 10/3 = 3.333..., rounded to 2 places.
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"divide","by":3,"places":2}}]}"#,
            "10",
        );
        assert_eq!(out, "3.33");
    }

    #[test]
    fn arithmetic_places_forces_trailing_zeros() {
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"add","by":0,"places":2}}]}"#,
            "3",
        );
        assert_eq!(out, "3.00");
    }

    #[test]
    fn arithmetic_divide_by_zero_errors() {
        let e = err(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"divide","by":0}}]}"#,
            "10",
        );
        assert!(e.contains("division by zero"), "got: {e}");
    }

    #[test]
    fn arithmetic_custom_find_pattern_limits_matches() {
        // find restricts the token set: only runs of 2+ digits are touched.
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"add","by":1,"find":"\\d{2,}"}}]}"#,
            "v3 keeps 99",
        );
        assert_eq!(out, "v3 keeps 100");
    }

    #[test]
    fn arithmetic_under_field_scope_does_column_math() {
        // Add 100 to only the 2nd comma field of each line.
        let out = run(
            r#"{"steps":[{"action":"arithmetic","options":{"op":"add","by":100},"scope":{"field":{"index":2}}}]}"#,
            "a,1,x\nb,2,y",
        );
        assert_eq!(out, "a,101,x\nb,102,y");
    }

    #[test]
    fn increment_numbers_default_by_one() {
        let out = run(
            r#"{"steps":[{"action":"increment_numbers"}]}"#,
            "item 1\nitem 9",
        );
        assert_eq!(out, "item 2\nitem 10");
    }

    #[test]
    fn round_numbers_default_zero_places() {
        // 2.7 -> 3, 2.4 -> 2 (no decimal point at 0 places).
        let out = run(r#"{"steps":[{"action":"round_numbers"}]}"#, "2.7 and 2.4");
        assert_eq!(out, "3 and 2");
    }

    #[test]
    fn round_numbers_to_places() {
        let out = run(
            r#"{"steps":[{"action":"round_numbers","options":{"places":1}}]}"#,
            "3.14159",
        );
        assert_eq!(out, "3.1");
    }

    #[test]
    fn pad_numbers_left_pads_to_width() {
        let out = run(
            r#"{"steps":[{"action":"pad_numbers","options":{"width":4}}]}"#,
            "7\n42\n12345",
        );
        // 12345 is already wider than 4, so it is unchanged.
        assert_eq!(out, "0007\n0042\n12345");
    }

    #[test]
    fn pad_numbers_custom_pad_char() {
        let out = run(
            r#"{"steps":[{"action":"pad_numbers","options":{"width":3,"pad":"x"}}]}"#,
            "5",
        );
        assert_eq!(out, "xx5");
    }

    #[test]
    fn pad_numbers_only_pads_digit_run_not_sign() {
        // The '-' is outside the \d+ run, so only the digits are padded.
        let out = run(
            r#"{"steps":[{"action":"pad_numbers","options":{"width":3}}]}"#,
            "-5",
        );
        assert_eq!(out, "-005");
    }
}
