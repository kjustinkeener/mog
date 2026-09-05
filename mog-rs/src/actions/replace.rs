//! Replace actions: literal `replace` and regex `replace_regex` /
//! `replace_regex_multiline`. Mirrors .NET `ReplaceActions.cs`.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;

use crate::model::Step;

/// Escape a literal string for use inside a regex pattern.
fn escape_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\.+*?()|[]{}^$#-".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn require_find(action: &str, step: &Step) -> Result<String> {
    step.get_string("find")
        .ok_or_else(|| anyhow!("Action '{action}' requires a 'find' option."))
}

/// Perform a LITERAL find/replace on `input`. Case-sensitive uses a plain
/// string replace; case-insensitive uses a regex with an escaped find and a
/// literal replacement (no `$`-expansion). Shared by `replace` and
/// `replace_extended`.
pub(crate) fn literal_replace(
    input: &str,
    find: &str,
    replace_with: &str,
    ignore_case: bool,
) -> Result<String> {
    if !ignore_case {
        return Ok(input.replace(find, replace_with));
    }

    let pattern = format!("(?i){}", escape_literal(find));
    let re = Regex::new(&pattern).map_err(|e| anyhow!("Invalid pattern built from 'find': {e}"))?;
    // Closure replacer => replacement text is treated literally (no $-expansion).
    let replacement = replace_with.to_string();
    let out = re.replace_all(input, |_: &fancy_regex::Captures| replacement.clone());
    Ok(out.into_owned())
}

/// Decode Notepad++ "Extended" mode escape sequences into real characters.
///
/// Supported: `\n` `\r` `\t` `\0` `\\` `\xHH` (2 hex digits) and `\uHHHH`
/// (4 hex digits, a Unicode scalar). Any other escape (including an invalid
/// `\x`/`\u`, an unknown letter, or a lone trailing backslash) is left literal:
/// the backslash and the following character are kept unchanged.
pub(crate) fn decode_extended(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    // Iterate over chars so multi-byte input is handled; use a manual index so
    // we can look ahead for hex digits.
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let n = chars.len();
    while i < n {
        let c = chars[i];
        if c != '\\' {
            out.push(c);
            i += 1;
            continue;
        }
        // We are at a backslash. Look at the next char (if any).
        if i + 1 >= n {
            // Trailing lone backslash: keep literal.
            out.push('\\');
            i += 1;
            continue;
        }
        let next = chars[i + 1];
        match next {
            'n' => {
                out.push('\n');
                i += 2;
            }
            'r' => {
                out.push('\r');
                i += 2;
            }
            't' => {
                out.push('\t');
                i += 2;
            }
            '0' => {
                out.push('\0');
                i += 2;
            }
            '\\' => {
                out.push('\\');
                i += 2;
            }
            'x' => {
                // \xHH: exactly two hex digits.
                if i + 3 < n {
                    let h1 = chars[i + 2];
                    let h2 = chars[i + 3];
                    if let (Some(d1), Some(d2)) = (h1.to_digit(16), h2.to_digit(16)) {
                        let code = (d1 * 16 + d2) as u8;
                        out.push(code as char);
                        i += 4;
                        continue;
                    }
                }
                // Not a valid \xHH: leave literal.
                out.push('\\');
                out.push('x');
                i += 2;
            }
            'u' => {
                // \uHHHH: exactly four hex digits forming a valid scalar.
                if i + 5 < n {
                    let mut code: u32 = 0;
                    let mut ok = true;
                    for k in 0..4 {
                        match chars[i + 2 + k].to_digit(16) {
                            Some(d) => code = code * 16 + d,
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                            i += 6;
                            continue;
                        }
                    }
                }
                // Not a valid \uHHHH: leave literal.
                out.push('\\');
                out.push('u');
                i += 2;
            }
            other => {
                // Unknown escape: keep the backslash AND the following char.
                out.push('\\');
                out.push(other);
                i += 2;
            }
        }
    }
    out
}

/// Literal `replace` / `r`. Case-sensitive uses a plain string replace;
/// case-insensitive uses a regex with escaped find and a literal replacement.
/// With `whole_word: true`, the find is bounded by `\b...\b` so it only matches
/// a standalone token (e.g. `datetime` but not inside `sysdatetime`).
pub fn replace(input: &str, step: &Step) -> Result<String> {
    let find = require_find("replace", step)?;
    let replace_with = step.get_string_or("replace_with", "");
    let ignore_case = step.get_bool("ignore_case", false)?;
    let whole_word = step.get_bool("whole_word", false)?;

    if find.is_empty() {
        return Err(anyhow!("Action 'replace' requires a non-empty 'find'."));
    }

    if whole_word {
        return whole_word_replace(input, &find, &replace_with, ignore_case);
    }
    literal_replace(input, &find, &replace_with, ignore_case)
}

/// Literal replace bounded by word boundaries (`\b...\b`). The find is escaped
/// (matched literally) and the replacement is inserted literally (no
/// `$`-expansion). Best for word-like tokens; a find that starts or ends with a
/// non-word character gets a `\b` in an odd spot, so prefer regex there.
fn whole_word_replace(
    input: &str,
    find: &str,
    replace_with: &str,
    ignore_case: bool,
) -> Result<String> {
    let prefix = if ignore_case { "(?i)" } else { "" };
    let pattern = format!("{prefix}\\b{}\\b", escape_literal(find));
    let re = Regex::new(&pattern)
        .map_err(|e| anyhow!("Invalid whole-word pattern built from 'find': {e}"))?;
    let replacement = replace_with.to_string();
    let out = re.replace_all(input, |_: &fancy_regex::Captures| replacement.clone());
    Ok(out.into_owned())
}

/// `replace_extended`: Notepad++ "Extended" search mode. Decodes escape
/// sequences in BOTH `find` and `replace_with`, then performs a literal
/// replace (honoring `ignore_case`) via [`literal_replace`].
pub fn replace_extended(input: &str, step: &Step) -> Result<String> {
    let find = require_find("replace_extended", step)?;
    let replace_with = step.get_string_or("replace_with", "");
    let ignore_case = step.get_bool("ignore_case", false)?;

    if find.is_empty() {
        return Err(anyhow!(
            "Action 'replace_extended' requires a non-empty 'find'."
        ));
    }

    let decoded_find = decode_extended(&find);
    let decoded_replace = decode_extended(&replace_with);

    if decoded_find.is_empty() {
        return Err(anyhow!(
            "Action 'replace_extended' requires a non-empty 'find' after decoding escapes."
        ));
    }

    literal_replace(input, &decoded_find, &decoded_replace, ignore_case)
}

/// `replace_regex` / `rr`. With `decode_replacement: true`, escape sequences in
/// `replace_with` (`\n \r \t \0 \\ \xHH \uHHHH`) are decoded to real characters
/// before `$`-expansion, so a regex find can pair with a newline in the output.
pub fn replace_regex(input: &str, step: &Step) -> Result<String> {
    replace_regex_seeded(input, step, None)
}

/// Seed-aware `replace_regex`: `seed` pins the per-occurrence `$uuid` token's RNG
/// stream (from the run's `--pin-seed` / `mog --test`, threaded via the engine's
/// exec context). `None` = a fresh random uuid per match. All other behavior is
/// identical to [`replace_regex`].
pub fn replace_regex_seeded(input: &str, step: &Step, seed: Option<u64>) -> Result<String> {
    let find = require_find("replace_regex", step)?;
    let replace_with = replacement_of(step)?;
    let ignore_case = step.get_bool("ignore_case", false)?;
    run_regex(input, &find, &replace_with, ignore_case, false, seed)
}

/// `replace_regex_multiline` / `rrm`. Also honors `decode_replacement`.
pub fn replace_regex_multiline(input: &str, step: &Step) -> Result<String> {
    replace_regex_multiline_seeded(input, step, None)
}

/// Seed-aware `replace_regex_multiline` (see [`replace_regex_seeded`]).
pub fn replace_regex_multiline_seeded(
    input: &str,
    step: &Step,
    seed: Option<u64>,
) -> Result<String> {
    let find = require_find("replace_regex_multiline", step)?;
    let replace_with = replacement_of(step)?;
    let ignore_case = step.get_bool("ignore_case", false)?;
    run_regex(input, &find, &replace_with, ignore_case, true, seed)
}

/// The `replace_with` option, with escapes decoded when `decode_replacement` is
/// set. Decoding happens before `$`-expansion, and never touches `$`.
fn replacement_of(step: &Step) -> Result<String> {
    let raw = step.get_string_or("replace_with", "");
    if step.get_bool("decode_replacement", false)? {
        Ok(decode_extended(&raw))
    } else {
        Ok(raw)
    }
}

/// Build the effective pattern `run_regex` compiles: the `find` body with `(?i)`
/// and/or `(?m)` prepended for the ignore_case / multiline options.
pub(crate) fn build_pattern(find: &str, ignore_case: bool, multiline: bool) -> String {
    let mut prefix = String::new();
    if ignore_case {
        prefix.push_str("(?i)");
    }
    if multiline {
        prefix.push_str("(?m)");
    }
    format!("{prefix}{find}")
}

/// True when a pattern can run ONLY on the backtracking engine -- it uses
/// backreferences or lookaround, which the linear-time `regex` crate rejects but
/// fancy-regex accepts. Such patterns fall back to fancy-regex in `run_regex` and
/// are potentially O(n^2) on large input. Drives the CLI pre-flight perf warning
/// and the descriptor complexity classification. A pattern that is simply invalid
/// (both engines reject) is not flagged here.
pub fn pattern_needs_backtracking(find: &str, ignore_case: bool, multiline: bool) -> bool {
    let pattern = build_pattern(find, ignore_case, multiline);
    regex::Regex::new(&pattern).is_err() && Regex::new(&pattern).is_ok()
}

fn run_regex(
    input: &str,
    find: &str,
    replace_with: &str,
    ignore_case: bool,
    multiline: bool,
    seed: Option<u64>,
) -> Result<String> {
    let pattern = build_pattern(find, ignore_case, multiline);
    // Per-occurrence tokens ($# = 1-based match number, $lineno = 1-based line of
    // the match, $uuid = a fresh uuid per match) require a per-match replacer that
    // carries running state; only pay that cost when the replacement uses one. The
    // substring probes over-trigger on an escaped `$$uuid`/`$$#`, which is harmless
    // (the slow path still emits the literal correctly).
    let needs_uuid = replace_with.contains("$uuid");
    let needs_tokens =
        replace_with.contains("$#") || replace_with.contains("$lineno") || needs_uuid;
    // Fast path: if the linear-time `regex` crate accepts this pattern, it uses
    // no backreferences or lookaround, so its leftmost-first match and capture
    // semantics equal fancy-regex's, and both expand $1 / ${name} / $$ identically
    // (pinned by the byte-identity fuzz in the tests). fancy-regex's backtracking
    // `replace_all` is O(n^2) on large input; this keeps the common case linear.
    if let Ok(re) = regex::Regex::new(&pattern) {
        if !needs_tokens {
            return Ok(re.replace_all(input, replace_with).into_owned());
        }
        let mut occ: u64 = 0;
        let mut line_acc: usize = 0;
        let mut last: usize = 0;
        let mut uuid_rng = needs_uuid.then(|| crate::builtins::uuid_stream(seed));
        let out = re.replace_all(input, |caps: &regex::Captures| {
            occ += 1;
            let start = caps.get(0).map(|m| m.start()).unwrap_or(last);
            line_acc += input.get(last..start).map(count_newlines).unwrap_or(0);
            last = start;
            let uuid = uuid_rng.as_mut().map(crate::builtins::uuid_from_rng);
            let tmpl = subst_occurrence_tokens(replace_with, occ, line_acc + 1, uuid.as_deref());
            let mut dst = String::new();
            caps.expand(&tmpl, &mut dst);
            dst
        });
        return Ok(out.into_owned());
    }
    // Fancy features (backreferences / lookaround): fall back to fancy-regex.
    // &str replacer => $1 / ${name} / $$ expansion (matches .NET dollar syntax).
    let re = Regex::new(&pattern).map_err(|e| anyhow!("Invalid regex '{find}': {e}"))?;
    if !needs_tokens {
        return Ok(re.replace_all(input, replace_with).into_owned());
    }
    let mut occ: u64 = 0;
    let mut line_acc: usize = 0;
    let mut last: usize = 0;
    let mut uuid_rng = needs_uuid.then(|| crate::builtins::uuid_stream(seed));
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        occ += 1;
        let start = caps.get(0).map(|m| m.start()).unwrap_or(last);
        line_acc += input.get(last..start).map(count_newlines).unwrap_or(0);
        last = start;
        let uuid = uuid_rng.as_mut().map(crate::builtins::uuid_from_rng);
        let tmpl = subst_occurrence_tokens(replace_with, occ, line_acc + 1, uuid.as_deref());
        let mut dst = String::new();
        caps.expand(&tmpl, &mut dst);
        dst
    });
    Ok(out.into_owned())
}

fn count_newlines(s: &str) -> usize {
    s.matches('\n').count()
}

/// Substitute the per-occurrence tokens `$#` (occurrence number), `$lineno` (line
/// number), and `$uuid` (a per-match uuid, when `uuid` is `Some`) into a regex
/// replacement template, leaving every other `$` construct for the engine's own
/// expansion. `$$` is preserved verbatim so the engine still emits a literal `$`
/// (and `$$#` / `$$uuid` stay literal). `$uuid` is matched as an exact prefix, so
/// `$user` / `$url` and other `$u...` text pass through untouched.
fn subst_occurrence_tokens(template: &str, occ: u64, lineno: usize, uuid: Option<&str>) -> String {
    let bytes = template.as_bytes();
    let mut out = String::with_capacity(template.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            // Preserve the literal-dollar escape so the engine handles it.
            if bytes.get(i + 1) == Some(&b'$') {
                out.push_str("$$");
                i += 2;
                continue;
            }
            if let Some(u) = uuid {
                if template[i..].starts_with("$uuid") {
                    out.push_str(u);
                    i += "$uuid".len();
                    continue;
                }
            }
            if template[i..].starts_with("$lineno") {
                out.push_str(&lineno.to_string());
                i += "$lineno".len();
                continue;
            }
            if template[i..].starts_with("$#") {
                out.push_str(&occ.to_string());
                i += 2;
                continue;
            }
        }
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Extract (find, replace) pairs from an object-valued option (e.g. `map`).
/// Non-string values are stringified. Absent/non-object -> empty.
fn pairs_from_map(step: &Step, key: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(serde_json::Value::Object(m)) = step.options.get(key) {
        for (k, v) in m {
            let rep = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            out.push((k.clone(), rep));
        }
    }
    out
}

/// `replace_map`: apply many literal find->replace pairs from `map` in a SINGLE
/// pass (longest find wins), so replacements never chain (a->b then b->c does not
/// turn a into c). Options `ignore_case` and `whole_word` route through a regex
/// alternation; the plain path uses the shared single-pass replacer.
pub fn replace_map(input: &str, step: &Step) -> Result<String> {
    let mut pairs = pairs_from_map(step, "map");
    if pairs.is_empty() {
        return Ok(input.to_string());
    }
    let ignore_case = step.get_bool("ignore_case", false)?;
    let whole_word = step.get_bool("whole_word", false)?;
    // Longest find first so a longer key wins over a shorter prefix.
    pairs.sort_by_key(|(f, _)| std::cmp::Reverse(f.chars().count()));

    if !ignore_case && !whole_word {
        return Ok(crate::actions::text::single_pass_replace(input, &pairs));
    }

    let alts: Vec<String> = pairs
        .iter()
        .filter(|(f, _)| !f.is_empty())
        .map(|(f, _)| escape_literal(f))
        .collect();
    if alts.is_empty() {
        return Ok(input.to_string());
    }
    let mut pat = String::new();
    if ignore_case {
        pat.push_str("(?i)");
    }
    pat.push_str(if whole_word { "\\b(?:" } else { "(?:" });
    pat.push_str(&alts.join("|"));
    pat.push_str(if whole_word { ")\\b" } else { ")" });
    let re = Regex::new(&pat).map_err(|e| anyhow!("replace_map: invalid pattern: {e}"))?;

    let lookup: std::collections::HashMap<String, String> = pairs
        .iter()
        .map(|(f, r)| {
            (
                if ignore_case {
                    f.to_lowercase()
                } else {
                    f.clone()
                },
                r.clone(),
            )
        })
        .collect();
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        let m = caps.get(0).map(|x| x.as_str()).unwrap_or("");
        let key = if ignore_case {
            m.to_lowercase()
        } else {
            m.to_string()
        };
        lookup.get(&key).cloned().unwrap_or_else(|| m.to_string())
    });
    Ok(out.into_owned())
}

/// `lookup_replace`: for each match of `pattern`, take capture `group` (default 1;
/// 0 = whole match), look it up in `map`, and replace the WHOLE match with the
/// mapped value. `on_miss`: leave (default) / blank / key. Option `ignore_case`.
pub fn lookup_replace(input: &str, step: &Step) -> Result<String> {
    let pattern = step
        .get_string("pattern")
        .ok_or_else(|| anyhow!("lookup_replace requires a 'pattern' option"))?;
    let group = step.get_usize("group", 1)?;
    let on_miss = step.get_string_or("on_miss", "leave");
    let ignore_case = step.get_bool("ignore_case", false)?;
    let lookup: std::collections::HashMap<String, String> = pairs_from_map(step, "map")
        .into_iter()
        .map(|(k, v)| (if ignore_case { k.to_lowercase() } else { k }, v))
        .collect();

    let pat = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.clone()
    };
    let re = Regex::new(&pat)
        .map_err(|e| anyhow!("lookup_replace: invalid pattern '{pattern}': {e}"))?;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        let whole = caps.get(0).map(|x| x.as_str()).unwrap_or("");
        let key = caps.get(group).map(|x| x.as_str()).unwrap_or(whole);
        let k = if ignore_case {
            key.to_lowercase()
        } else {
            key.to_string()
        };
        match lookup.get(&k) {
            Some(v) => v.clone(),
            None => match on_miss.as_str() {
                "blank" | "remove" => String::new(),
                "key" => key.to_string(),
                _ => whole.to_string(),
            },
        }
    });
    Ok(out.into_owned())
}

/// `smart_case_replace`: literal find/replace that preserves the case pattern of
/// each occurrence. Matching is case-insensitive; if the matched text is ALL-CAPS
/// the replacement is upper-cased, all-lower -> lower, Title -> capitalized, mixed
/// -> replacement as written.
pub fn smart_case_replace(input: &str, step: &Step) -> Result<String> {
    let find = require_find("smart_case_replace", step)?;
    if find.is_empty() {
        return Err(anyhow!(
            "Action 'smart_case_replace' requires a non-empty 'find'."
        ));
    }
    let replace_with = step.get_string_or("replace_with", "");
    let pattern = format!("(?i){}", escape_literal(&find));
    let re =
        Regex::new(&pattern).map_err(|e| anyhow!("smart_case_replace: invalid pattern: {e}"))?;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        let m = caps.get(0).map(|x| x.as_str()).unwrap_or("");
        apply_case_pattern(m, &replace_with)
    });
    Ok(out.into_owned())
}

/// Cast `replacement` into the case pattern observed in `matched`.
fn apply_case_pattern(matched: &str, replacement: &str) -> String {
    let letters: Vec<char> = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return replacement.to_string();
    }
    if letters.iter().all(|c| c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    if letters.iter().all(|c| c.is_lowercase()) {
        return replacement.to_lowercase();
    }
    let first_upper = letters.first().map(|c| c.is_uppercase()).unwrap_or(false);
    let rest_lower = letters.iter().skip(1).all(|c| c.is_lowercase());
    if first_upper && rest_lower {
        return capitalize_first(replacement);
    }
    replacement.to_string()
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => {
            let mut out: String = f.to_uppercase().collect();
            out.push_str(&chars.as_str().to_lowercase());
            out
        }
    }
}

/// `replace_nth`: replace ONLY the Nth occurrence of a regex `find` with
/// `replace_with` (supporting `$1` / `${name}` / `$$` backrefs). `n` is 1-based; a
/// NEGATIVE `n` counts from the end (`n=-1` is the last match). Occurrences are
/// counted across the whole input; when a `scope` restricts the region the engine
/// runs this action per span, so counting is naturally within that span. An
/// out-of-range `n` (|n| greater than the match count, or n=0) is a no-op: the
/// input is returned unchanged. Whole-file, backtracking-capable (fancy-regex).
pub fn replace_nth(input: &str, step: &Step) -> Result<String> {
    let n = step.get_i64("n", 1)?;
    replace_positional(input, step, "replace_nth", n)
}

/// `replace_first`: convenience for `replace_nth` with n=1 (the first occurrence).
pub fn replace_first(input: &str, step: &Step) -> Result<String> {
    replace_positional(input, step, "replace_first", 1)
}

/// `replace_last`: convenience for `replace_nth` with n=-1 (the last occurrence).
pub fn replace_last(input: &str, step: &Step) -> Result<String> {
    replace_positional(input, step, "replace_last", -1)
}

/// Shared core for the positional replaces. Counts non-overlapping matches of
/// `find` (with `ignore_case`), resolves the 1-based/negative `n` to a target
/// occurrence, and rewrites just that one via `$`-expansion; an out-of-range `n`
/// returns the input unchanged.
fn replace_positional(input: &str, step: &Step, action: &str, n: i64) -> Result<String> {
    let find = step
        .get_string("find")
        .ok_or_else(|| anyhow!("Action '{action}' requires a 'find' option."))?;
    if find.is_empty() {
        return Err(anyhow!("Action '{action}' requires a non-empty 'find'."));
    }
    let replace_with = step.get_string_or("replace_with", "");
    let ignore_case = step.get_bool("ignore_case", false)?;
    let pattern = build_pattern(&find, ignore_case, false);
    let re = Regex::new(&pattern).map_err(|e| anyhow!("Invalid regex '{find}': {e}"))?;

    // Count total non-overlapping matches (same left-to-right walk replace_all uses).
    let mut total: usize = 0;
    for m in re.find_iter(input) {
        m.map_err(|e| anyhow!("{action}: match error: {e}"))?;
        total += 1;
    }
    if total == 0 {
        return Ok(input.to_string());
    }

    // Resolve n (1-based; negative from the end) to a 0-based target index.
    let target0: Option<usize> = if n > 0 {
        let idx = (n - 1) as usize;
        (idx < total).then_some(idx)
    } else if n < 0 {
        let from_end = (-n) as usize; // 1 = last
        (from_end <= total).then(|| total - from_end)
    } else {
        None // n == 0 is out of range
    };
    let Some(target0) = target0 else {
        return Ok(input.to_string()); // out of range: no-op
    };
    let target_occ = target0 + 1; // 1-based occurrence to rewrite

    let mut occ: usize = 0;
    let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
        occ += 1;
        if occ == target_occ {
            let mut dst = String::new();
            caps.expand(&replace_with, &mut dst);
            dst
        } else {
            // Leave every other occurrence byte-for-byte unchanged.
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        }
    });
    Ok(out.into_owned())
}

#[cfg(test)]
mod tests {
    use super::decode_extended;

    #[test]
    fn decode_extended_basic_escapes() {
        assert_eq!(decode_extended(r"a\nb"), "a\nb");
        assert_eq!(decode_extended(r"a\rb"), "a\rb");
        assert_eq!(decode_extended(r"a\tb"), "a\tb");
        assert_eq!(decode_extended(r"a\0b"), "a\0b");
    }

    #[test]
    fn decode_extended_crlf() {
        assert_eq!(decode_extended(r"a\r\nb"), "a\r\nb");
    }

    #[test]
    fn decode_extended_backslash() {
        // Two-char `\\` collapses to one backslash.
        assert_eq!(decode_extended(r"\\"), "\\");
        assert_eq!(decode_extended(r"a\\b"), r"a\b");
    }

    #[test]
    fn decode_extended_hex() {
        assert_eq!(decode_extended(r"\x41"), "A");
        assert_eq!(decode_extended(r"x\x41y"), "xAy");
        // Not two hex digits => left literal.
        assert_eq!(decode_extended(r"\xZZ"), r"\xZZ");
        assert_eq!(decode_extended(r"\x4"), r"\x4");
    }

    #[test]
    fn decode_extended_unicode() {
        assert_eq!(decode_extended("\\u00e9"), "\u{00e9}");
        // Too short / invalid => left literal.
        assert_eq!(decode_extended("\\u00e"), "\\u00e");
        assert_eq!(decode_extended(r"\uZZZZ"), r"\uZZZZ");
    }

    #[test]
    fn decode_extended_unknown_escape_is_literal() {
        assert_eq!(decode_extended(r"\q"), r"\q");
        assert_eq!(decode_extended(r"a\qb"), r"a\qb");
    }

    #[test]
    fn decode_extended_trailing_backslash_is_literal() {
        assert_eq!(decode_extended(r"abc\"), r"abc\");
    }
}

#[cfg(test)]
mod map_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn occurrence_token_numbers_matches() {
        // $# expands to the 1-based match number across the whole input; $0 still
        // expands to the whole match.
        let out = run(
            r#"{"steps":[{"action":"replace_regex","options":{"find":"[a-z]+","replace_with":"[$#]$0"}}]}"#,
            "a b\nc",
        );
        assert_eq!(out, "[1]a [2]b\n[3]c");
    }

    #[test]
    fn lineno_token_reports_match_line() {
        // $lineno = 1-based line of the match; multiline ^ matches each line start.
        let out = run(
            r#"{"steps":[{"action":"replace_regex_multiline","options":{"find":"^","replace_with":"$lineno: "}}]}"#,
            "a\nb\nc",
        );
        assert_eq!(out, "1: a\n2: b\n3: c");
    }

    #[test]
    fn literal_dollar_before_token_is_preserved() {
        // $$# is a literal "$#" (the $$ escape), then $# is the occurrence number.
        let out = run(
            r#"{"steps":[{"action":"replace_regex","options":{"find":"x","replace_with":"$$#$#"}}]}"#,
            "x\nx",
        );
        assert_eq!(out, "$#1\n$#2");
    }

    #[test]
    fn tokens_work_on_fancy_backtracking_engine() {
        // A backreference forces the fancy-regex fallback; tokens must still expand.
        let out = run(
            r#"{"steps":[{"action":"replace_regex","options":{"find":"([a-z])\\1","replace_with":"$#"}}]}"#,
            "aa\nbb",
        );
        assert_eq!(out, "1\n2");
    }

    #[test]
    fn replace_map_longest_first_no_chaining() {
        // "New York" must beat "York"; and a->b, b->c must not chain a into c.
        let out = run(
            r#"{"steps":[{"action":"replace_map","options":{"map":{"a":"b","b":"c","New York":"NY","York":"Y"}}}]}"#,
            "a b New York York",
        );
        assert_eq!(out, "b c NY Y");
    }

    #[test]
    fn replace_map_whole_word() {
        let out = run(
            r#"{"steps":[{"action":"replace_map","options":{"map":{"in":"IN"},"whole_word":true}}]}"#,
            "in printing in",
        );
        assert_eq!(out, "IN printing IN");
    }

    #[test]
    fn lookup_replace_enriches_from_table() {
        let out = run(
            r#"{"steps":[{"action":"lookup_replace","options":{"pattern":"\\b(\\d{3})\\b","map":{"404":"Not Found","200":"OK"}}}]}"#,
            "got 404 then 200 and 500",
        );
        // 500 not in the map -> left as-is (default on_miss=leave).
        assert_eq!(out, "got Not Found then OK and 500");
    }

    #[test]
    fn smart_case_replace_preserves_case() {
        let out = run(
            r#"{"steps":[{"action":"smart_case_replace","options":{"find":"color","replace_with":"colour"}}]}"#,
            "color Color COLOR",
        );
        assert_eq!(out, "colour Colour COLOUR");
    }

    #[test]
    fn replace_nth_positive_targets_that_occurrence() {
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":2}}]}"#,
            "a a a a",
        );
        assert_eq!(out, "a X a a");
    }

    #[test]
    fn replace_nth_negative_targets_from_end() {
        // n=-1 is the last occurrence.
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":-1}}]}"#,
            "a a a a",
        );
        assert_eq!(out, "a a a X");
    }

    #[test]
    fn replace_nth_negative_two_from_end() {
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":-2}}]}"#,
            "a a a a",
        );
        assert_eq!(out, "a a X a");
    }

    #[test]
    fn replace_nth_out_of_range_is_noop() {
        let input = "a a a";
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":9}}]}"#,
            input,
        );
        assert_eq!(out, input);
        // Negative out of range too.
        let out2 = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":-9}}]}"#,
            input,
        );
        assert_eq!(out2, input);
        // n=0 is out of range (1-based).
        let out3 = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":0}}]}"#,
            input,
        );
        assert_eq!(out3, input);
    }

    #[test]
    fn replace_nth_default_n_is_first() {
        // No n -> defaults to 1, so replace_nth == replace_first.
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X"}}]}"#,
            "a a a",
        );
        assert_eq!(out, "X a a");
    }

    #[test]
    fn replace_first_equals_nth_n1() {
        let by_first = run(
            r#"{"steps":[{"action":"replace_first","options":{"find":"a","replace_with":"X"}}]}"#,
            "a a a",
        );
        let by_nth = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"a","replace_with":"X","n":1}}]}"#,
            "a a a",
        );
        assert_eq!(by_first, "X a a");
        assert_eq!(by_first, by_nth);
    }

    #[test]
    fn replace_last_targets_final_occurrence() {
        let out = run(
            r#"{"steps":[{"action":"replace_last","options":{"find":"a","replace_with":"X"}}]}"#,
            "a a a",
        );
        assert_eq!(out, "a a X");
    }

    #[test]
    fn replace_nth_ignore_case() {
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"foo","replace_with":"X","n":2,"ignore_case":true}}]}"#,
            "foo FOO Foo",
        );
        assert_eq!(out, "foo X Foo");
    }

    #[test]
    fn replace_nth_supports_backref_in_replacement() {
        // $1 backref: swap the matched digits of only the 2nd occurrence.
        let out = run(
            r#"{"steps":[{"action":"replace_nth","options":{"find":"(\\d)(\\d)","replace_with":"$2$1","n":2}}]}"#,
            "12 34 56",
        );
        assert_eq!(out, "12 43 56");
    }

    #[test]
    fn replace_nth_within_scope_counts_per_line() {
        // With only_lines_matching, the action runs per matching line, so n counts
        // within each line independently.
        let out = run(
            r#"{"steps":[{"action":"replace_nth","only_lines_matching":"x","options":{"find":"x","replace_with":"X","n":1}}]}"#,
            "x x x\nno match\nx x",
        );
        assert_eq!(out, "X x x\nno match\nX x");
    }
}

/// The per-occurrence `$uuid` replacement token: a fresh uuid v4 per regex match,
/// reproducible when the run's RNG seed is pinned (drawn from the same seeded
/// stream `{{@uuid}}` uses) and random otherwise. Exercised directly against
/// `run_regex` (the seed is what the engine threads in from `--pin-seed` /
/// `mog --test`); the engine-wiring path is covered in `engine::uuid_seed_tests`.
#[cfg(test)]
mod uuid_token_tests {
    use super::run_regex;

    /// A v4 uuid: 8-4-4-4-12 lowercase hex, version nibble 4, variant in [89ab].
    fn is_uuid_v4(s: &str) -> bool {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 5 {
            return false;
        }
        for (p, &l) in parts.iter().zip([8, 4, 4, 4, 12].iter()) {
            let ok = p.len() == l
                && p.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
            if !ok {
                return false;
            }
        }
        parts[2].starts_with('4') && matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b')
    }

    #[test]
    fn pinned_seed_is_byte_identical_across_runs() {
        let a = run_regex("x x x", "x", "$uuid", false, false, Some(0)).unwrap();
        let b = run_regex("x x x", "x", "$uuid", false, false, Some(0)).unwrap();
        assert_eq!(
            a, b,
            "pinned per-match uuids must be reproducible run-to-run"
        );
        let ids: Vec<&str> = a.split(' ').collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.iter().all(|s| is_uuid_v4(s)), "got: {a}");
    }

    #[test]
    fn matches_get_distinct_uuids() {
        let out = run_regex("x x x", "x", "$uuid", false, false, Some(0)).unwrap();
        let ids: std::collections::HashSet<&str> = out.split(' ').collect();
        assert_eq!(ids.len(), 3, "each match must get a fresh uuid: {out}");
    }

    #[test]
    fn first_pinned_uuid_equals_placeholder_uuid() {
        // The per-match stream reuses {{@uuid}}'s seeded RNG, so the first draw
        // equals make_uuid(Some(seed)) exactly (shared format + version/variant).
        let out = run_regex("x", "x", "$uuid", false, false, Some(0)).unwrap();
        assert_eq!(out, crate::builtins::make_uuid(Some(0)));
    }

    #[test]
    fn double_dollar_uuid_stays_literal() {
        let out = run_regex("x\nx", "x", "$$uuid", false, false, Some(0)).unwrap();
        assert_eq!(out, "$uuid\n$uuid");
    }

    #[test]
    fn uuid_works_on_fancy_backtracking_engine() {
        // A backreference forces the fancy-regex fallback; $uuid must still expand,
        // reproducibly, and distinctly per match.
        let out = run_regex("aa\nbb", "([a-z])\\1", "$uuid", false, true, Some(0)).unwrap();
        let ids: Vec<&str> = out.split('\n').collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.iter().all(|s| is_uuid_v4(s)), "got: {out}");
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn unpinned_uuid_is_valid_shape() {
        let out = run_regex("x", "x", "$uuid", false, false, None).unwrap();
        assert!(is_uuid_v4(&out), "got: {out}");
    }

    #[test]
    fn does_not_misfire_on_other_dollar_u_text() {
        // The template engages the token path via $uuid; an adjacent $user must NOT
        // be treated as $uuid -- it is left for the engine, which expands the unknown
        // capture group to empty.
        let out = run_regex("x", "x", "$uuid|$user", false, false, Some(0)).unwrap();
        let (id, rest) = out.split_once('|').unwrap();
        assert!(is_uuid_v4(id), "got: {out}");
        assert_eq!(rest, "", "$user must not become a uuid: {out}");
    }

    #[test]
    fn combines_with_occurrence_number_token() {
        // $# and $uuid coexist; $# is the match number, $uuid a fresh uuid per match.
        let out = run_regex("x x", "x", "$#:$uuid", false, false, Some(0)).unwrap();
        let parts: Vec<&str> = out.split(' ').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].starts_with("1:"));
        assert!(parts[1].starts_with("2:"));
        assert!(
            is_uuid_v4(&parts[0][2..]) && is_uuid_v4(&parts[1][2..]),
            "got: {out}"
        );
    }
}

/// The `regex`-crate fast path in `run_regex` must be byte-for-byte identical to
/// the old fancy-regex-only path for every pattern the fast path accepts. This
/// fuzzes the two against each other over a matrix of group/`$`-expansion-heavy
/// cases (named + numbered groups, `$$`, `$0`, out-of-range refs, ambiguous
/// `$1a`, trailing `$`, empty matches) crossed with ignore_case/multiline and
/// thousands of random inputs. Any divergence here means the fast path is unsafe
/// for that shape and must be excluded from it.
#[cfg(test)]
mod regex_equivalence {
    use fancy_regex::Regex as Fancy;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    /// Oracle: the pre-optimization path -- always fancy-regex, built with the same
    /// pattern prefix as `run_regex`. Returned compiled so callers hoist the (slow)
    /// compile out of the input loop.
    fn compiled_oracle(find: &str, ic: bool, ml: bool) -> Fancy {
        let mut prefix = String::new();
        if ic {
            prefix.push_str("(?i)");
        }
        if ml {
            prefix.push_str("(?m)");
        }
        Fancy::new(&format!("{prefix}{find}")).unwrap()
    }

    #[test]
    fn hybrid_replace_regex_byte_identical_to_fancy() {
        let cases: &[(&str, &str)] = &[
            (r"(\w+)=(\d+)", "$2:$1"),
            (r"(\w+)=(\d+)", "${2}-${1}"),
            (r"(?P<k>\w+)=(?P<v>\d+)", "${v}=${k}"),
            (r"(a)(b)?", "[$1$2]"),
            (r"(a)(b)?", "$1a"), // ambiguous: longest name is "1a"
            (r"x*", "<$0>"),     // empty matches + whole-match ref
            (r"\d", "#"),
            (r"(\d)(\d)(\d)", "$3$2$1"),
            (r"\b\w+\b", "($0)"),
            (r"a", "price $$"), // literal dollar
            (r"(a)", "$1 $9"),  // out-of-range group -> empty
            (r"(a)", "end$"),   // trailing literal $
            (r"(a)", "${1}z"),
            (r"foo", "BAR"), // no groups, plain literal
        ];
        let alphabet = ['a', 'b', '0', '1', '=', ' ', '\n', 'x', 'f', 'o'];
        let mut rng = StdRng::seed_from_u64(0x5EED_1234);
        let mut checked = 0u64;
        // Full (pattern, replacement) x flags matrix; random inputs per combo. The
        // oracle regex is compiled once per combo (compilation dominates runtime),
        // so this stays a few seconds. A one-off 40x-larger run also passed; this
        // committed guard is a regression net, not the original proof.
        for (find, repl) in cases {
            for ml in [false, true] {
                for ic in [false, true] {
                    let oracle = compiled_oracle(find, ic, ml);
                    for _ in 0..60 {
                        let len = rng.gen_range(0..30);
                        let input: String = (0..len)
                            .map(|_| alphabet[rng.gen_range(0..alphabet.len())])
                            .collect();
                        let got = super::run_regex(&input, find, repl, ic, ml, None).unwrap();
                        let want = oracle.replace_all(&input, *repl).into_owned();
                        assert_eq!(
                            got, want,
                            "diverged: find={find:?} repl={repl:?} ic={ic} ml={ml} input={input:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(
            checked >= 3_000,
            "fuzz should cover 3k+ cases, got {checked}"
        );
    }
}
