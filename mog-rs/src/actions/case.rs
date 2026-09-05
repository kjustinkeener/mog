//! Case transformation actions. Mirrors .NET `CaseActions.cs`.
//! Note: .NET uses CurrentCulture casing; this port uses Unicode default
//! (invariant-like) casing, which matches en-US behavior for typical input.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::model::Step;

/// `to_upper` / `upper`.
pub fn to_upper(input: &str, _step: &Step) -> Result<String> {
    Ok(input.to_uppercase())
}

/// `to_lower` / `lower`.
pub fn to_lower(input: &str, _step: &Step) -> Result<String> {
    Ok(input.to_lowercase())
}

fn upper_first_lower_rest(word: &str, blend: bool) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            let rest: String = chars.collect();
            if blend {
                out.push_str(&rest);
            } else {
                out.push_str(&rest.to_lowercase());
            }
            out
        }
    }
}

/// `to_proper` / `proper`. Title-case each `\w+`; `blend` keeps the remainder.
pub fn to_proper(input: &str, step: &Step) -> Result<String> {
    let blend = step.get_bool("blend", false)?;
    let re = Regex::new(r"\w+").map_err(|e| anyhow!(e))?;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        upper_first_lower_rest(caps.get(0).map(|m| m.as_str()).unwrap_or(""), blend)
    });
    Ok(out.into_owned())
}

/// `to_sentence` / `sentence`. Capitalize the first letter after `. ! ?`.
/// `blend=false` lowercases the whole text first; `blend=true` keeps remainder.
pub fn to_sentence(input: &str, step: &Step) -> Result<String> {
    let blend = step.get_bool("blend", false)?;
    let base = if blend {
        input.to_string()
    } else {
        input.to_lowercase()
    };

    let mut out = String::with_capacity(base.len());
    let mut at_sentence_start = true;
    for c in base.chars() {
        if c.is_alphabetic() && at_sentence_start {
            for u in c.to_uppercase() {
                out.push(u);
            }
            at_sentence_start = false;
        } else {
            out.push(c);
            if c == '.' || c == '!' || c == '?' {
                at_sentence_start = true;
            }
        }
    }
    Ok(out)
}

/// Capitalize one word: first letter upper, remainder lower. Unicode-aware.
fn cap_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            out.push_str(&chars.as_str().to_lowercase());
            out
        }
    }
}

/// Split a delimiter-free part into words on case boundaries:
/// `getUserId` -> [get, User, Id]; `HTTPServer` -> [HTTP, Server].
fn split_case_runs(part: &str, out: &mut Vec<String>) {
    let chars: Vec<char> = part.chars().collect();
    if chars.is_empty() {
        return;
    }
    let mut start = 0usize;
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let cur = chars[i];
        // Boundary before an uppercase that follows a non-uppercase (aB, 2B),
        // or the last uppercase of a run that precedes a lowercase (HTTPServer -> HTTP|Server).
        let boundary = (!prev.is_uppercase() && cur.is_uppercase())
            || (prev.is_uppercase()
                && cur.is_uppercase()
                && i + 1 < chars.len()
                && chars[i + 1].is_lowercase());
        if boundary {
            out.push(chars[start..i].iter().collect());
            start = i;
        }
    }
    out.push(chars[start..].iter().collect());
}

/// Break an identifier run into its component words, splitting on `_`/`-`
/// delimiters and on internal case boundaries.
fn ident_words(ident: &str) -> Vec<String> {
    let mut words = Vec::new();
    for part in ident.split(['_', '-']) {
        if !part.is_empty() {
            split_case_runs(part, &mut words);
        }
    }
    words
}

// Matches an identifier: alphanumeric runs joined by single `_`/`-` separators.
// Starting/ending on an alphanumeric leaves any surrounding separators (leading
// `_`, dunders) untouched, so `__init__` recases to `init` in place.
fn ident_regex() -> Result<Regex> {
    Regex::new(r"[A-Za-z0-9]+(?:[_-]+[A-Za-z0-9]+)*").map_err(|e| anyhow!(e))
}

/// `to_camel` / `camel`. Recase each identifier to camelCase (`user_id` ->
/// `userId`). First word lowercased, later words title-cased; delimiters dropped.
pub fn to_camel(input: &str, _step: &Step) -> Result<String> {
    let re = ident_regex()?;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        let words = ident_words(caps.get(0).map(|m| m.as_str()).unwrap_or(""));
        let mut s = String::new();
        for (i, w) in words.iter().enumerate() {
            if i == 0 {
                s.push_str(&w.to_lowercase());
            } else {
                s.push_str(&cap_word(w));
            }
        }
        s
    });
    Ok(out.into_owned())
}

/// `to_pascal` / `pascal`. Recase each identifier to PascalCase (`user_id` ->
/// `UserId`). Every word title-cased; delimiters dropped.
pub fn to_pascal(input: &str, _step: &Step) -> Result<String> {
    let re = ident_regex()?;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        ident_words(caps.get(0).map(|m| m.as_str()).unwrap_or(""))
            .iter()
            .map(|w| cap_word(w))
            .collect::<String>()
    });
    Ok(out.into_owned())
}

/// `invert_case`. Swap upper<->lower per char.
pub fn invert_case(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_uppercase() {
            for l in c.to_lowercase() {
                out.push(l);
            }
        } else if c.is_lowercase() {
            for u in c.to_uppercase() {
                out.push(u);
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// `random_case`. Per letter, randomly upper/lower; seeded when `seed` present.
pub fn random_case(input: &str, step: &Step) -> Result<String> {
    if step.has_key("seed") {
        let seed = step.get_i64("seed", 0)?;
        let mut rng = StdRng::seed_from_u64(seed as u64);
        Ok(apply_random_case(input, &mut rng))
    } else {
        let mut rng = rand::thread_rng();
        Ok(apply_random_case(input, &mut rng))
    }
}

fn apply_random_case<R: Rng>(input: &str, rng: &mut R) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_alphabetic() {
            if rng.gen_range(0..2) == 0 {
                for u in c.to_uppercase() {
                    out.push(u);
                }
            } else {
                for l in c.to_lowercase() {
                    out.push(l);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn to_camel_from_snake_and_kebab() {
        let out = run(
            r#"{"steps":[{"action":"to_camel"}]}"#,
            "user_id\nfirst-name\nHTTP_PROXY",
        );
        assert_eq!(out, "userId\nfirstName\nhttpProxy");
    }

    #[test]
    fn to_camel_splits_existing_case_boundaries() {
        // Already-camel and Pascal input normalizes; acronym run splits.
        let out = run(
            r#"{"steps":[{"action":"to_camel"}]}"#,
            "getUserId\nHTTPServer",
        );
        assert_eq!(out, "getUserId\nhttpServer");
    }

    #[test]
    fn to_camel_preserves_surrounding_underscores() {
        let out = run(r#"{"steps":[{"action":"camel"}]}"#, "__init_val__");
        assert_eq!(out, "__initVal__");
    }

    #[test]
    fn to_pascal_from_snake() {
        let out = run(
            r#"{"steps":[{"action":"to_pascal"}]}"#,
            "user_id\nfirst-name",
        );
        assert_eq!(out, "UserId\nFirstName");
    }

    #[test]
    fn to_pascal_alias_and_single_word() {
        let out = run(r#"{"steps":[{"action":"pascal"}]}"#, "value");
        assert_eq!(out, "Value");
    }

    #[test]
    fn to_camel_leaves_non_identifier_punctuation() {
        // Delimiters between words are dropped, but a `=` and spaces are kept.
        let out = run(
            r#"{"steps":[{"action":"to_camel"}]}"#,
            "max_retries = default_value",
        );
        assert_eq!(out, "maxRetries = defaultValue");
    }
}
