//! Line-oriented actions. All preserve EOL style and trailing-newline state via
//! the LineText helper. Mirrors .NET `LineActions.cs`.

use anyhow::{anyhow, bail, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;

use crate::line_text::{join, split};
use crate::model::Step;

fn casefold(s: &str, ignore_case: bool) -> String {
    if ignore_case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}

/// `remove_empty_lines` / `rel`. Option `include_whitespace` (default false).
pub fn remove_empty_lines(input: &str, step: &Step) -> Result<String> {
    let include_ws = step.get_bool("include_whitespace", false)?;
    let s = split(input);
    let kept: Vec<String> = s
        .lines
        .into_iter()
        .filter(|l| {
            if include_ws {
                !l.trim().is_empty()
            } else {
                !l.is_empty()
            }
        })
        .collect();
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `remove_duplicate_lines` / `dedupe`. Keeps first occurrence file-wide.
pub fn remove_duplicate_lines(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let s = split(input);
    let mut seen = std::collections::HashSet::new();
    let kept: Vec<String> = s
        .lines
        .into_iter()
        .filter(|l| seen.insert(casefold(l, ignore_case)))
        .collect();
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `remove_consecutive_duplicate_lines` / `uniq`. Collapses adjacent equals.
pub fn remove_consecutive_duplicate_lines(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let s = split(input);
    let mut kept: Vec<String> = Vec::new();
    let mut prev: Option<String> = None;
    for l in s.lines {
        let key = casefold(&l, ignore_case);
        if prev.as_deref() != Some(key.as_str()) {
            kept.push(l);
            prev = Some(key);
        }
    }
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `reverse_lines` / `reverse`.
pub fn reverse_lines(input: &str, _step: &Step) -> Result<String> {
    let s = split(input);
    let mut lines = s.lines;
    lines.reverse();
    Ok(join(&lines, s.eol, s.trailing_eol))
}

/// `shuffle_lines` / `shuffle`. Fisher-Yates; seeded when `seed` present.
pub fn shuffle_lines(input: &str, step: &Step) -> Result<String> {
    let s = split(input);
    let mut lines = s.lines;
    if step.has_key("seed") {
        let seed = step.get_i64("seed", 0)?;
        let mut rng = StdRng::seed_from_u64(seed as u64);
        fisher_yates(&mut lines, &mut rng);
    } else {
        let mut rng = rand::thread_rng();
        fisher_yates(&mut lines, &mut rng);
    }
    Ok(join(&lines, s.eol, s.trailing_eol))
}

fn fisher_yates<R: Rng>(lines: &mut [String], rng: &mut R) {
    let n = lines.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        let j = rng.gen_range(0..=i);
        lines.swap(i, j);
    }
}

/// `join_lines` / `join`. Option `separator` (default "").
pub fn join_lines(input: &str, step: &Step) -> Result<String> {
    let separator = step.get_string_or("separator", "");
    let s = split(input);
    let joined = s.lines.join(&separator);
    if s.trailing_eol && !joined.is_empty() {
        Ok(format!("{joined}{}", s.eol))
    } else {
        Ok(joined)
    }
}

/// Resolve a 1-based (negative-from-end) index against a count to a 0-based index
/// in range, or None when out of range. Local mirror of the engine helper.
fn resolve_1based(index: i64, n: usize) -> Option<usize> {
    if index == 0 || n == 0 {
        return None;
    }
    let ni = n as i64;
    let zero = if index < 0 { ni + index } else { index - 1 };
    if zero < 0 || zero >= ni {
        None
    } else {
        Some(zero as usize)
    }
}

/// `sort_lines` / `sort`. Options: order (asc|desc), ignore_case, numeric,
/// natural (version/number-aware), by_length, plus a chosen SORT KEY:
/// `key_field` (Nth `key_delimiter`-split field, 1-based, TAB default) or
/// `key_regex` (the regex's match, or capture group 1 when present); the two are
/// mutually exclusive. `by_frequency` instead orders lines by how often their key
/// occurs (descending by default; `order:"asc"` flips to ascending), ties keeping
/// first-seen order. With no key option the whole line is the key, preserving the
/// original behavior byte-identically.
pub fn sort_lines(input: &str, step: &Step) -> Result<String> {
    let order = step.get_string_or("order", "asc");
    let order_set = step.has_key("order");
    let descending = order.eq_ignore_ascii_case("desc");
    let ignore_case = step.get_bool("ignore_case", false)?;
    let numeric = step.get_bool("numeric", false)?;
    let natural = step.get_bool("natural", false)?;
    let by_length = step.get_bool("by_length", false)?;
    let by_frequency = step.get_bool("by_frequency", false)?;

    // Sort-key selector: key_field XOR key_regex (or neither = whole line).
    let has_field = step.has_key("key_field");
    let has_regex = step.has_key("key_regex");
    if has_field && has_regex {
        bail!("sort_lines: set only one of 'key_field' / 'key_regex'");
    }
    let key_re = if has_regex {
        let pat = step.get_string_or("key_regex", "");
        Some(
            regex::Regex::new(&pat)
                .map_err(|e| anyhow!("sort_lines: invalid key_regex '{pat}': {e}"))?,
        )
    } else {
        None
    };
    let field_index = step.get_i64("key_field", 0)?;
    let delim = step.get_string_or("key_delimiter", "\t");
    if has_field && delim.is_empty() {
        bail!("sort_lines: 'key_delimiter' must not be empty");
    }

    // Extract the sort key for a line. Regex: capture group 1 if the pattern has
    // one, else the whole match; no match -> empty key. Field: the 1-based field
    // (negatives count from the end); a missing field -> empty key.
    let key_of = |line: &str| -> String {
        if let Some(re) = &key_re {
            match re.captures(line) {
                Some(c) => {
                    let m = if c.len() > 1 { c.get(1) } else { c.get(0) };
                    m.map(|m| m.as_str()).unwrap_or("").to_string()
                }
                None => String::new(),
            }
        } else if has_field {
            let parts: Vec<&str> = line.split(delim.as_str()).collect();
            match resolve_1based(field_index, parts.len()) {
                Some(i) => parts[i].to_string(),
                None => String::new(),
            }
        } else {
            line.to_string()
        }
    };

    let s = split(input);
    let mut lines = s.lines;

    if by_frequency {
        // Count occurrences per (case-folded) key, then stable-sort by count.
        // Descending by default; an explicit order:"asc" flips to ascending.
        let norm = |k: String| if ignore_case { k.to_lowercase() } else { k };
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for l in &lines {
            *counts.entry(norm(key_of(l))).or_insert(0) += 1;
        }
        let asc_freq = order_set && order.eq_ignore_ascii_case("asc");
        lines.sort_by(|a, b| {
            let ca = counts[&norm(key_of(a))];
            let cb = counts[&norm(key_of(b))];
            if asc_freq {
                ca.cmp(&cb)
            } else {
                cb.cmp(&ca)
            }
        });
        return Ok(join(&lines, s.eol, s.trailing_eol));
    }

    // Base ascending comparator over the extracted key. Precedence: numeric >
    // natural > by_length > ignore_case > ordinal.
    let base = |a: &String, b: &String| -> Ordering {
        let ka = key_of(a);
        let kb = key_of(b);
        if numeric {
            numeric_cmp(&ka, &kb)
        } else if natural {
            natural_cmp(&ka, &kb, ignore_case)
        } else if by_length {
            ka.chars()
                .count()
                .cmp(&kb.chars().count())
                .then_with(|| ka.cmp(&kb))
        } else if ignore_case {
            ka.to_lowercase().cmp(&kb.to_lowercase())
        } else {
            ka.cmp(&kb)
        }
    };

    // Stable sort (Rust sort_by is stable). For descending we reverse the
    // comparator so equal keys keep original order, matching .NET's stable
    // OrderByDescending.
    if descending {
        lines.sort_by(|a, b| base(b, a));
    } else {
        lines.sort_by(|a, b| base(a, b));
    }

    Ok(join(&lines, s.eol, s.trailing_eol))
}

/// `sort_ip`: sort lines by IP address value (IPv4 before IPv6, numeric order,
/// not lexical). The IP is the first whitespace-separated token of each line, so
/// both bare IP lists and "IP rest-of-line" logs sort correctly. Lines whose
/// first token is not a valid IP keep their order at the end. Options: `order`
/// (asc|desc), `unique` (drop adjacent duplicate lines after sorting).
pub fn sort_ip(input: &str, step: &Step) -> Result<String> {
    sort_by_key_last(input, step, |line| {
        first_token(line).and_then(|t| t.parse::<std::net::IpAddr>().ok())
    })
}

/// `sort_versions`: sort lines by version-number value, like `sort -V` but with
/// semver pre-release rules (1.0.0-rc.1 sorts before 1.0.0). The version is the
/// first token of each line (an optional leading `v` is ignored). Lines whose
/// first token is not version-like keep their order at the end. Options: `order`
/// (asc|desc), `unique`.
pub fn sort_versions(input: &str, step: &Step) -> Result<String> {
    sort_by_key_last(input, step, |line| {
        first_token(line).and_then(parse_version)
    })
}

/// Stable sort by an `Ord` key extracted per line; lines with no key (`None`)
/// keep their original order after all keyed lines. Shared by `sort_ip` /
/// `sort_versions`. Honors `order` (asc|desc) and `unique`.
fn sort_by_key_last<K: Ord>(
    input: &str,
    step: &Step,
    key_of: impl Fn(&str) -> Option<K>,
) -> Result<String> {
    let descending = step
        .get_string_or("order", "asc")
        .eq_ignore_ascii_case("desc");
    let unique = step.get_bool("unique", false)?;
    let s = split(input);
    let mut lines = s.lines;

    // Precompute keys so the comparator does not re-parse each line.
    let mut keyed: Vec<(Option<K>, String)> = lines.drain(..).map(|l| (key_of(&l), l)).collect();
    keyed.sort_by(|a, b| {
        let ord = match (&a.0, &b.0) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => Ordering::Less, // keyed lines come first
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal, // stable: keep original order
        };
        if descending {
            ord.reverse()
        } else {
            ord
        }
    });
    let mut out: Vec<String> = keyed.into_iter().map(|(_, l)| l).collect();
    if unique {
        out.dedup();
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// The first whitespace-separated token of a line (trimmed), if any.
fn first_token(line: &str) -> Option<&str> {
    line.split_whitespace().next()
}

/// Default matcher for an import/include/use line across common languages.
const DEFAULT_IMPORT_PATTERN: &str =
    r"^\s*(?:import|from|#include|#import|using|use|require|@import)\b";

/// `sort_imports`: sort each maximal run of consecutive import lines in place,
/// leaving every other line (blank lines, code) as an anchor between runs. Which
/// lines count as imports is the `pattern` regex (default matches import / from /
/// #include / using / use / require / @import). Each run is sorted lexically;
/// `ignore_case` folds case and `dedupe` drops exact duplicates within a run.
/// Because it targets consecutive runs, it never reorders imports across a blank
/// line or a statement, so grouped/sectioned import blocks stay separate.
pub fn sort_imports(input: &str, step: &Step) -> Result<String> {
    let pattern = step.get_string_or("pattern", DEFAULT_IMPORT_PATTERN);
    let ignore_case = step.get_bool("ignore_case", false)?;
    let dedupe = step.get_bool("dedupe", false)?;
    let full = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.clone()
    };
    let re = regex::Regex::new(&full)
        .map_err(|e| anyhow::anyhow!("sort_imports: invalid pattern '{pattern}': {e}"))?;

    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    let mut i = 0;
    while i < s.lines.len() {
        if re.is_match(&s.lines[i]) {
            let start = i;
            while i < s.lines.len() && re.is_match(&s.lines[i]) {
                i += 1;
            }
            let mut run: Vec<String> = s.lines[start..i].to_vec();
            if ignore_case {
                run.sort_by(|a, b| {
                    a.to_lowercase()
                        .cmp(&b.to_lowercase())
                        .then_with(|| a.cmp(b))
                });
            } else {
                run.sort();
            }
            if dedupe {
                run.dedup();
            }
            out.extend(run);
        } else {
            out.push(s.lines[i].clone());
            i += 1;
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// A parsed version key: numeric release segments plus an optional pre-release
/// tag. `Ord` gives semver-style ordering (release sorts after its pre-releases).
#[derive(PartialEq, Eq)]
struct Version {
    release: Vec<u64>,
    pre: Vec<PreId>,
    has_pre: bool,
}

#[derive(PartialEq, Eq)]
enum PreId {
    Num(u64),
    Text(String),
}

impl Ord for PreId {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PreId::Num(a), PreId::Num(b)) => a.cmp(b),
            (PreId::Num(_), PreId::Text(_)) => Ordering::Less, // numeric < alphanumeric
            (PreId::Text(_), PreId::Num(_)) => Ordering::Greater,
            (PreId::Text(a), PreId::Text(b)) => a.cmp(b),
        }
    }
}
impl PartialOrd for PreId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare release segments element-wise, missing segments treated as 0.
        let n = self.release.len().max(other.release.len());
        for i in 0..n {
            let a = self.release.get(i).copied().unwrap_or(0);
            let b = other.release.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => {}
                ord => return ord,
            }
        }
        // A version WITH a pre-release sorts before the same version without one.
        match (self.has_pre, other.has_pre) {
            (false, false) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (true, true) => {
                // Compare pre-release identifiers; a shorter set is smaller when
                // all preceding identifiers are equal.
                for (x, y) in self.pre.iter().zip(other.pre.iter()) {
                    match x.cmp(y) {
                        Ordering::Equal => {}
                        ord => return ord,
                    }
                }
                self.pre.len().cmp(&other.pre.len())
            }
        }
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Parse a version-like token (`v?N(.N)*(-pre)?`). Returns None when the release
/// core is not a dotted run of integers, so non-version lines sort to the end.
fn parse_version(tok: &str) -> Option<Version> {
    let t = tok.trim();
    let t = t
        .strip_prefix('v')
        .or_else(|| t.strip_prefix('V'))
        .unwrap_or(t);
    let (core, pre_str) = match t.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (t, None),
    };
    if core.is_empty() {
        return None;
    }
    let mut release = Vec::new();
    for seg in core.split('.') {
        release.push(seg.parse::<u64>().ok()?);
    }
    let pre: Vec<PreId> = pre_str
        .map(|p| {
            p.split('.')
                .map(|id| match id.parse::<u64>() {
                    Ok(n) => PreId::Num(n),
                    Err(_) => PreId::Text(id.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Version {
        release,
        pre,
        has_pre: pre_str.is_some(),
    })
}

/// `number_lines`. Prefix each line with an incrementing number. Options:
/// `start` (integer, default 1), `separator` (string, default ". "). Numbers are
/// right-aligned/padded to equal width so the columns line up.
pub fn number_lines(input: &str, step: &Step) -> Result<String> {
    let start = step.get_i64("start", 1)?;
    let separator = step.get_string_or("separator", ". ");
    let s = split(input);
    let n = s.lines.len();
    if n == 0 {
        return Ok(join(&s.lines, s.eol, s.trailing_eol));
    }
    let last = start + (n as i64) - 1;
    let width = start.to_string().len().max(last.to_string().len());
    let out: Vec<String> = s
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let num = start + i as i64;
            format!("{num:>width$}{separator}{l}")
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `squeeze_blank_lines`. Collapse runs of consecutive blank lines down to a
/// single blank line. Option `include_whitespace` treats whitespace-only lines
/// as blank.
pub fn squeeze_blank_lines(input: &str, step: &Step) -> Result<String> {
    let include_ws = step.get_bool("include_whitespace", false)?;
    let is_blank = |l: &str| {
        if include_ws {
            l.trim().is_empty()
        } else {
            l.is_empty()
        }
    };
    let s = split(input);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    let mut prev_blank = false;
    for l in &s.lines {
        let blank = is_blank(l);
        if blank && prev_blank {
            continue;
        }
        out.push(l.clone());
        prev_blank = blank;
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Numeric line comparison mirroring .NET NumericLineComparer. Shared with the
/// record (`sort_blocks`) sorter so both use identical key semantics.
pub(crate) fn numeric_cmp(a: &String, b: &String) -> Ordering {
    let pa = parse_num(a);
    let pb = parse_num(b);
    match (pa, pb) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.cmp(b),
    }
}

fn parse_num(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

/// Natural / version-aware comparison: digit runs compare by numeric value
/// (leading zeros ignored) so "file2" sorts before "file10". Shared with the
/// record (`sort_blocks`) sorter.
pub(crate) fn natural_cmp(a: &str, b: &str, ignore_case: bool) -> Ordering {
    let sa = casefold(a, ignore_case);
    let sb = casefold(b, ignore_case);
    let mut ca = sa.chars().peekable();
    let mut cb = sb.chars().peekable();
    loop {
        match (ca.peek().copied(), cb.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                if x.is_ascii_digit() && y.is_ascii_digit() {
                    let na = take_digits(&mut ca);
                    let nb = take_digits(&mut cb);
                    let ta = na.trim_start_matches('0');
                    let tb = nb.trim_start_matches('0');
                    let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    // Equal numeric value: fewer leading zeros first for stability.
                    let ord = na.len().cmp(&nb.len());
                    if ord != Ordering::Equal {
                        return ord;
                    }
                } else {
                    let ord = x.cmp(&y);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    ca.next();
                    cb.next();
                }
            }
        }
    }
}

fn take_digits(it: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            it.next();
        } else {
            break;
        }
    }
    s
}

/// `split_lines`: split each line on a literal `separator` (default ","), emitting
/// every piece as its own line. Option `trim` trims each piece. Inverse of
/// `join_lines`.
pub fn split_lines(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", ",");
    if sep.is_empty() {
        return Ok(input.to_string());
    }
    let trim = step.get_bool("trim", false)?;
    let s = split(input);
    let mut out: Vec<String> = Vec::new();
    for line in &s.lines {
        for piece in line.split(sep.as_str()) {
            out.push(if trim {
                piece.trim().to_string()
            } else {
                piece.to_string()
            });
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `keep_duplicate_lines`: keep only lines that occur more than once (the inverse
/// of dedupe). By default one copy of each duplicated line, in first-seen order;
/// `all` keeps every occurrence. Option `ignore_case`.
pub fn keep_duplicate_lines(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let all = step.get_bool("all", false)?;
    let s = split(input);
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for l in &s.lines {
        *counts.entry(casefold(l, ignore_case)).or_insert(0) += 1;
    }
    let mut emitted: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut kept: Vec<String> = Vec::new();
    for l in &s.lines {
        let key = casefold(l, ignore_case);
        let is_dup = counts.get(&key).copied().unwrap_or(0) > 1;
        // `all` keeps every occurrence; otherwise keep the first of each duplicate.
        if is_dup && (all || emitted.insert(key)) {
            kept.push(l.clone());
        }
    }
    Ok(join(&kept, s.eol, s.trailing_eol))
}

/// `unique_with_count`: collapse to distinct lines, each prefixed with a
/// right-aligned occurrence count (like `sort | uniq -c`), in first-seen order.
pub fn unique_with_count(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let s = split(input);
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for l in &s.lines {
        let key = casefold(l, ignore_case);
        *counts.entry(key.clone()).or_insert(0) += 1;
        if seen.insert(key) {
            order.push(l.clone());
        }
    }
    let max = counts.values().copied().max().unwrap_or(0);
    let width = max.to_string().len();
    let out: Vec<String> = order
        .iter()
        .map(|l| {
            let c = counts.get(&casefold(l, ignore_case)).copied().unwrap_or(0);
            format!("{c:>width$} {l}")
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `stamp_sequence`: replace each occurrence of a literal marker (`find`, default
/// "#") with an incrementing number. Options: `start` (default 1), `step`
/// (default 1), `width` (zero-pad width, default 0 = none). Good for sequential
/// IDs / Bates-style numbering.
pub fn stamp_sequence(input: &str, step: &Step) -> Result<String> {
    let marker = step.get_string_or("find", "#");
    if marker.is_empty() {
        return Ok(input.to_string());
    }
    let start = step.get_i64("start", 1)?;
    let stride = step.get_i64("step", 1)?;
    let width = step.get_usize("width", 0)?;
    let mut n = start;
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(idx) = rest.find(marker.as_str()) {
        out.push_str(&rest[..idx]);
        if width > 0 {
            out.push_str(&format!("{n:0width$}"));
        } else {
            out.push_str(&n.to_string());
        }
        rest = &rest[idx + marker.len()..];
        n += stride;
    }
    out.push_str(rest);
    Ok(out)
}

/// `dedupe_by`: drop duplicate LINES by an EXTRACTED key, keeping the first
/// occurrence of each key. Unlike `remove_consecutive_duplicate_lines` this works
/// on non-adjacent duplicates, and unlike `remove_duplicate_lines` it compares an
/// extracted key rather than the whole line. Key selector (mutually exclusive):
/// `key_regex` (the regex's match, or capture group 1 when the pattern has one) OR
/// `key_field` (the Nth `key_delimiter`-split field, 1-based, TAB default). Option
/// `ignore_case` folds the key. A line whose key does not match / whose field is
/// missing uses the WHOLE LINE as its key, so such lines are only dropped when they
/// are byte-identical (never collapsed together just because they share "no key").
pub fn dedupe_by(input: &str, step: &Step) -> Result<String> {
    let ignore_case = step.get_bool("ignore_case", false)?;
    let has_field = step.has_key("key_field");
    let has_regex = step.has_key("key_regex");
    if has_field && has_regex {
        bail!("dedupe_by: set only one of 'key_field' / 'key_regex'");
    }
    if !has_field && !has_regex {
        bail!("dedupe_by: requires a 'key_regex' or 'key_field' option");
    }
    let key_re = if has_regex {
        let pat = step.get_string_or("key_regex", "");
        Some(
            regex::Regex::new(&pat)
                .map_err(|e| anyhow!("dedupe_by: invalid key_regex '{pat}': {e}"))?,
        )
    } else {
        None
    };
    let field_index = step.get_i64("key_field", 0)?;
    let delim = step.get_string_or("key_delimiter", "\t");
    if has_field && delim.is_empty() {
        bail!("dedupe_by: 'key_delimiter' must not be empty");
    }

    // Extract the dedupe key for a line. Regex: capture group 1 if present, else
    // the whole match; NO match -> the whole line (so unmatched lines dedupe only
    // when identical). Field: the 1-based field; a MISSING field -> the whole line.
    let key_of = |line: &str| -> String {
        let raw = if let Some(re) = &key_re {
            match re.captures(line) {
                Some(c) => {
                    let m = if c.len() > 1 { c.get(1) } else { c.get(0) };
                    match m {
                        Some(m) => m.as_str().to_string(),
                        None => line.to_string(),
                    }
                }
                None => line.to_string(),
            }
        } else {
            let parts: Vec<&str> = line.split(delim.as_str()).collect();
            match resolve_1based(field_index, parts.len()) {
                Some(i) => parts[i].to_string(),
                None => line.to_string(),
            }
        };
        casefold(&raw, ignore_case)
    };

    let s = split(input);
    let mut seen = std::collections::HashSet::new();
    let kept: Vec<String> = s
        .lines
        .into_iter()
        .filter(|l| seen.insert(key_of(l)))
        .collect();
    Ok(join(&kept, s.eol, s.trailing_eol))
}

#[cfg(test)]
mod added_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn sort_ip_orders_numerically_invalid_last() {
        let out = run(
            r#"{"steps":[{"action":"sort_ip"}]}"#,
            "10.0.0.10\n10.0.0.2\n2001:db8::1\nnope\n10.0.0.1",
        );
        assert_eq!(out, "10.0.0.1\n10.0.0.2\n10.0.0.10\n2001:db8::1\nnope");
    }

    #[test]
    fn sort_ip_extracts_leading_token() {
        let out = run(
            r#"{"steps":[{"action":"sort_ip"}]}"#,
            "192.168.0.20 GET /b\n192.168.0.3 GET /a",
        );
        assert_eq!(out, "192.168.0.3 GET /a\n192.168.0.20 GET /b");
    }

    #[test]
    fn sort_versions_semver_prerelease_order() {
        let out = run(
            r#"{"steps":[{"action":"sort_versions"}]}"#,
            "v1.10.0\n1.9.0\n1.0.0\n1.0.0-rc.1\n1.0.0-alpha\ntip",
        );
        assert_eq!(out, "1.0.0-alpha\n1.0.0-rc.1\n1.0.0\n1.9.0\nv1.10.0\ntip");
    }

    #[test]
    fn sort_versions_desc_and_unique() {
        let out = run(
            r#"{"steps":[{"action":"sort_versions","options":{"order":"desc","unique":true}}]}"#,
            "1.2.0\n1.10.0\n1.2.0\n1.1.0",
        );
        assert_eq!(out, "1.10.0\n1.2.0\n1.1.0");
    }

    #[test]
    fn sort_imports_sorts_each_run_between_anchors() {
        // Two runs separated by a blank line and code; each sorted independently.
        let out = run(
            r#"{"steps":[{"action":"sort_imports"}]}"#,
            "import os\nimport abc\nfrom x import y\n\nimport later\ncode()",
        );
        assert_eq!(
            out,
            "from x import y\nimport abc\nimport os\n\nimport later\ncode()"
        );
    }

    #[test]
    fn sort_imports_dedupe_within_run() {
        let out = run(
            r#"{"steps":[{"action":"sort_imports","options":{"dedupe":true}}]}"#,
            "import b\nimport a\nimport b",
        );
        assert_eq!(out, "import a\nimport b");
    }

    #[test]
    fn split_lines_on_comma_trims() {
        let out = run(
            r#"{"steps":[{"action":"split_lines","options":{"separator":",","trim":true}}]}"#,
            "a, b ,c\nd,e",
        );
        assert_eq!(out, "a\nb\nc\nd\ne");
    }

    #[test]
    fn sort_natural_orders_versions() {
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"natural":true}}]}"#,
            "file10\nfile2\nfile1",
        );
        assert_eq!(out, "file1\nfile2\nfile10");
    }

    #[test]
    fn sort_by_length() {
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"by_length":true}}]}"#,
            "ccc\na\nbb",
        );
        assert_eq!(out, "a\nbb\nccc");
    }

    #[test]
    fn sort_by_key_field() {
        // Sort by the 2nd comma field, not the whole line.
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"key_field":2,"key_delimiter":","}}]}"#,
            "z,1\na,3\nm,2",
        );
        assert_eq!(out, "z,1\nm,2\na,3");
    }

    #[test]
    fn sort_by_key_field_numeric() {
        // Numeric comparison on the extracted field (10 after 2).
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"key_field":2,"key_delimiter":",","numeric":true}}]}"#,
            "a,10\nb,2\nc,1",
        );
        assert_eq!(out, "c,1\nb,2\na,10");
    }

    #[test]
    fn sort_by_key_regex_whole_match() {
        // No capture group: the whole match is the key (the 3-digit code).
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"key_regex":"\\d{3}"}}]}"#,
            "err 300\nwarn 100\ninfo 200",
        );
        assert_eq!(out, "warn 100\ninfo 200\nerr 300");
    }

    #[test]
    fn sort_by_key_regex_capture_group() {
        // With a capture group, group 1 is the key (the name after 'user=').
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"key_regex":"user=(\\w+)"}}]}"#,
            "id1 user=carol\nid2 user=alice\nid3 user=bob",
        );
        assert_eq!(out, "id2 user=alice\nid3 user=bob\nid1 user=carol");
    }

    #[test]
    fn sort_by_frequency_groups_most_common_first() {
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"by_frequency":true}}]}"#,
            "b\na\nb\nc\nb\na",
        );
        assert_eq!(out, "b\nb\nb\na\na\nc");
    }

    #[test]
    fn sort_by_frequency_ascending_with_explicit_order() {
        let out = run(
            r#"{"steps":[{"action":"sort_lines","options":{"by_frequency":true,"order":"asc"}}]}"#,
            "b\na\nb\nc\nb\na",
        );
        assert_eq!(out, "c\na\na\nb\nb\nb");
    }

    #[test]
    fn sort_key_field_and_regex_are_mutually_exclusive() {
        let e = crate::execute(
            &parse_mog(
                r#"{"steps":[{"action":"sort_lines","options":{"key_field":1,"key_regex":"x"}}]}"#,
            )
            .unwrap(),
            "a\nb",
        )
        .unwrap_err()
        .to_string();
        assert!(
            e.contains("only one of 'key_field' / 'key_regex'"),
            "got: {e}"
        );
    }

    #[test]
    fn keep_duplicate_lines_one_each() {
        let out = run(
            r#"{"steps":[{"action":"keep_duplicate_lines"}]}"#,
            "a\nb\na\nc\nb\na",
        );
        assert_eq!(out, "a\nb");
    }

    #[test]
    fn unique_with_count_counts() {
        let out = run(r#"{"steps":[{"action":"unique_with_count"}]}"#, "a\nb\na");
        assert_eq!(out, "2 a\n1 b");
    }

    #[test]
    fn stamp_sequence_zero_padded() {
        let out = run(
            r#"{"steps":[{"action":"stamp_sequence","options":{"find":"<n>","start":1,"width":3}}]}"#,
            "DOC-<n>\nDOC-<n>\nDOC-<n>",
        );
        assert_eq!(out, "DOC-001\nDOC-002\nDOC-003");
    }

    #[test]
    fn dedupe_by_regex_capture_group_non_adjacent() {
        // Key = the id after 'id='; non-adjacent duplicate ids are dropped, first kept.
        let out = run(
            r#"{"steps":[{"action":"dedupe_by","options":{"key_regex":"id=(\\w+)"}}]}"#,
            "id=1 alice\nid=2 bob\nid=1 alice-again\nid=3 carol\nid=2 bob-again",
        );
        assert_eq!(out, "id=1 alice\nid=2 bob\nid=3 carol");
    }

    #[test]
    fn dedupe_by_regex_whole_match_no_group() {
        // No capture group: the whole match (the 3-digit code) is the key.
        let out = run(
            r#"{"steps":[{"action":"dedupe_by","options":{"key_regex":"\\d{3}"}}]}"#,
            "err 500 a\nwarn 404 b\ninfo 500 c\ndbg 404 d",
        );
        assert_eq!(out, "err 500 a\nwarn 404 b");
    }

    #[test]
    fn dedupe_by_key_field() {
        // Key = 1st comma field; later rows with the same first field are dropped.
        let out = run(
            r#"{"steps":[{"action":"dedupe_by","options":{"key_field":1,"key_delimiter":","}}]}"#,
            "acme,100\nbeta,200\nacme,999\ngamma,300",
        );
        assert_eq!(out, "acme,100\nbeta,200\ngamma,300");
    }

    #[test]
    fn dedupe_by_ignore_case_key() {
        let out = run(
            r#"{"steps":[{"action":"dedupe_by","options":{"key_field":1,"key_delimiter":",","ignore_case":true}}]}"#,
            "Acme,1\nACME,2\nbeta,3",
        );
        assert_eq!(out, "Acme,1\nbeta,3");
    }

    #[test]
    fn dedupe_by_no_key_match_uses_whole_line() {
        // Lines with no key match keep the whole line as key: two DISTINCT unmatched
        // lines both survive, but an identical unmatched line is dropped.
        let out = run(
            r#"{"steps":[{"action":"dedupe_by","options":{"key_regex":"id=(\\w+)"}}]}"#,
            "id=1 a\nheader line\nother line\nheader line\nid=1 b",
        );
        assert_eq!(out, "id=1 a\nheader line\nother line");
    }

    #[test]
    fn dedupe_by_field_and_regex_are_mutually_exclusive() {
        let e = crate::execute(
            &parse_mog(
                r#"{"steps":[{"action":"dedupe_by","options":{"key_field":1,"key_regex":"x"}}]}"#,
            )
            .unwrap(),
            "a\nb",
        )
        .unwrap_err()
        .to_string();
        assert!(
            e.contains("only one of 'key_field' / 'key_regex'"),
            "got: {e}"
        );
    }

    #[test]
    fn dedupe_by_requires_a_key_selector() {
        let e = crate::execute(
            &parse_mog(r#"{"steps":[{"action":"dedupe_by"}]}"#).unwrap(),
            "a\nb",
        )
        .unwrap_err()
        .to_string();
        assert!(
            e.contains("requires a 'key_regex' or 'key_field'"),
            "got: {e}"
        );
    }
}
