//! Character-level text cleanup actions: keep/remove characters by class, strip
//! control characters, normalize look-alike punctuation and odd whitespace, and
//! repair mojibake. These fix characters a caller often cannot see (zero-width,
//! control, look-alike quotes/dashes) or would get wrong by hand, which is the
//! cleanest "grep for edits" division of labor.

use anyhow::{anyhow, bail, Result};
use fancy_regex::Regex;

use crate::line_text::{join, split};
use crate::model::Step;

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

/// The line-comment marker for a named comment style.
fn comment_marker(style: &str) -> Result<&'static str> {
    Ok(match style {
        "hash" => "#",
        "slashes" => "//",
        "dash" => "--",
        "semicolon" => ";",
        other => bail!(
            "convert_comment_style: unknown style '{other}' \
             (expected hash, slashes, dash, semicolon)"
        ),
    })
}

/// `chunk_text`: split text into fixed-size chunks for RAG ingestion. Each chunk is
/// about `size` characters, snapped back to the nearest whitespace so words are not
/// cut; `overlap` characters are repeated at the start of the next chunk to preserve
/// context across boundaries. Chunks are emitted joined by `separator` (default a
/// `\n---\n` rule). Deterministic.
pub fn chunk_text(input: &str, step: &Step) -> Result<String> {
    let size = step.get_usize("size", 1000)?.max(1);
    let overlap = step.get_usize("overlap", 0)?.min(size - 1);
    let separator = step.get_string_or("separator", "\n---\n");
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut chunks: Vec<String> = Vec::new();
    let mut start = 0usize;
    while start < n {
        let mut end = (start + size).min(n);
        // Snap the end back to a whitespace boundary (unless we hit the input end).
        if end < n {
            let mut e = end;
            while e > start + 1 && !chars[e - 1].is_whitespace() {
                e -= 1;
            }
            if e > start + 1 {
                end = e;
            }
        }
        let chunk: String = chars[start..end].iter().collect();
        let chunk = chunk.trim().to_string();
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        if end >= n {
            break;
        }
        // Advance, carrying `overlap` chars; always make progress.
        start = end.saturating_sub(overlap).max(start + 1);
    }
    Ok(chunks.join(&separator))
}

/// `convert_comment_style`: swap the LINE-comment marker at the start of each
/// comment line from one style to another: hash (`#`), slashes (`//`), dash (`--`),
/// or semicolon (`;`). Only a line whose first non-whitespace is the `from` marker
/// is changed; leading indentation and the comment text after the marker are
/// preserved. `from` and `to` are required. Block comments (`/* */`) are not
/// handled.
pub fn convert_comment_style(input: &str, step: &Step) -> Result<String> {
    let from = step.get_string_or("from", "slashes");
    let to = step.get_string_or("to", "hash");
    let from_m = comment_marker(&from)?;
    let to_m = comment_marker(&to)?;
    let s = split(input);
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| {
            let ws_len = l.len() - l.trim_start().len();
            let (ws, rest) = l.split_at(ws_len);
            if let Some(stripped) = rest.strip_prefix(from_m) {
                format!("{ws}{to_m}{stripped}")
            } else {
                l.clone()
            }
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// The named character classes accepted by `keep_chars` / `remove_chars`.
const CHAR_CLASSES: &[&str] = &[
    "digits",
    "letters",
    "alnum",
    "ascii",
    "printable",
    "whitespace",
    "punctuation",
];

/// Classify a char against a named class. Callers validate the class name up front
/// (see [`ensure_known_class`]), so an unknown class here simply matches nothing.
fn in_class(c: char, class: &str) -> bool {
    match class {
        "digits" => c.is_ascii_digit(),
        "letters" => c.is_alphabetic(),
        "alnum" => c.is_alphanumeric(),
        "ascii" => c.is_ascii(),
        "printable" => !c.is_control(),
        "whitespace" => c.is_whitespace(),
        "punctuation" => c.is_ascii_punctuation(),
        _ => false,
    }
}

/// Reject an unrecognized character-class name instead of silently matching
/// nothing (which would make `keep_chars` wipe all content and `remove_chars` a
/// no-op). Only relevant when no explicit `set` is given.
fn ensure_known_class(action: &str, class: &str) -> Result<()> {
    if CHAR_CLASSES.contains(&class) {
        Ok(())
    } else {
        bail!(
            "{action}: unknown character class '{class}' (expected one of: {})",
            CHAR_CLASSES.join(", ")
        )
    }
}

/// `keep_chars`: keep only characters in `class` (or in the explicit `set`); drop
/// everything else. Newlines are kept by default so line structure survives.
pub fn keep_chars(input: &str, step: &Step) -> Result<String> {
    let set = step.get_string("set");
    let class = step.get_string_or("class", "alnum");
    let keep_nl = step.get_bool("keep_newlines", true)?;
    if set.is_none() {
        ensure_known_class("keep_chars", &class)?;
    }
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        let keep = if keep_nl && is_newline(c) {
            true
        } else if let Some(s) = &set {
            s.contains(c)
        } else {
            in_class(c, &class)
        };
        if keep {
            out.push(c);
        }
    }
    Ok(out)
}

/// `remove_chars`: drop characters in `class` (or in `set`); keep the rest.
/// Newlines are kept by default.
pub fn remove_chars(input: &str, step: &Step) -> Result<String> {
    let set = step.get_string("set");
    let class = step.get_string_or("class", "punctuation");
    let keep_nl = step.get_bool("keep_newlines", true)?;
    if set.is_none() {
        ensure_known_class("remove_chars", &class)?;
    }
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if keep_nl && is_newline(c) {
            out.push(c);
            continue;
        }
        let remove = if let Some(s) = &set {
            s.contains(c)
        } else {
            in_class(c, &class)
        };
        if !remove {
            out.push(c);
        }
    }
    Ok(out)
}

/// `strip_control_chars`: remove C0/C1 control characters, always keeping tab,
/// newline, and carriage return. Option `mode` = delete (default) or replace
/// (emit `replacement`, default a single space).
pub fn strip_control_chars(input: &str, step: &Step) -> Result<String> {
    let replace = step.get_enum(
        "mode",
        "strip_control_chars",
        "delete",
        &["delete", "replace"],
    )? == "replace";
    let replacement = step.get_string_or("replacement", " ");
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        let keep = matches!(c, '\t' | '\n' | '\r') || !c.is_control();
        if keep {
            out.push(c);
        } else if replace {
            out.push_str(&replacement);
        }
    }
    Ok(out)
}

/// `normalize`: clean up look-alike and invisible characters. Boolean flags:
/// `quotes` (curly -> straight), `dashes` (en/em/minus -> hyphen), `ellipsis`
/// (`…` -> `...`), `spaces` (odd Unicode spaces -> a normal space, zero-width
/// removed) all default ON; `accents` (de-accent common Latin letters -> ASCII)
/// defaults OFF. Full NFC/NFKC and confusable folding are intentionally out of
/// scope (they need a Unicode-data dependency; the engine stays small).
pub fn normalize(input: &str, step: &Step) -> Result<String> {
    let quotes = step.get_bool("quotes", true)?;
    let dashes = step.get_bool("dashes", true)?;
    let ellipsis = step.get_bool("ellipsis", true)?;
    let spaces = step.get_bool("spaces", true)?;
    let accents = step.get_bool("accents", false)?;

    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        // Zero-width and BOM-in-body: removed entirely.
        if spaces
            && matches!(
                c,
                '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}'
            )
        {
            continue;
        }
        if quotes
            && matches!(
                c,
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}'
            )
        {
            out.push('\'');
            continue;
        }
        if quotes
            && matches!(
                c,
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}'
            )
        {
            out.push('"');
            continue;
        }
        if dashes
            && matches!(
                c,
                '\u{2010}'
                    | '\u{2011}'
                    | '\u{2012}'
                    | '\u{2013}'
                    | '\u{2014}'
                    | '\u{2015}'
                    | '\u{2212}'
            )
        {
            out.push('-');
            continue;
        }
        if ellipsis && c == '\u{2026}' {
            out.push_str("...");
            continue;
        }
        // Any other Unicode whitespace (nbsp, thin/hair spaces, ideographic space)
        // collapses to a normal space; the ASCII whitespace we keep as-is.
        if spaces && c.is_whitespace() && !matches!(c, ' ' | '\t' | '\n' | '\r') {
            out.push(' ');
            continue;
        }
        if accents {
            if let Some(a) = deaccent(c) {
                out.push_str(a);
                continue;
            }
        }
        out.push(c);
    }
    Ok(out)
}

/// Map a common accented Latin letter to its ASCII base; `None` if not covered.
fn deaccent(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
        'è' | 'é' | 'ê' | 'ë' => "e",
        'ì' | 'í' | 'î' | 'ï' => "i",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => "o",
        'ù' | 'ú' | 'û' | 'ü' => "u",
        'ç' => "c",
        'ñ' => "n",
        'ý' | 'ÿ' => "y",
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => "A",
        'È' | 'É' | 'Ê' | 'Ë' => "E",
        'Ì' | 'Í' | 'Î' | 'Ï' => "I",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' => "O",
        'Ù' | 'Ú' | 'Û' | 'Ü' => "U",
        'Ç' => "C",
        'Ñ' => "N",
        'Ý' => "Y",
        'ß' => "ss",
        'æ' => "ae",
        'Æ' => "AE",
        'œ' => "oe",
        'Œ' => "OE",
        _ => return None,
    })
}

/// `fix_mojibake`: repair text that was UTF-8 decoded as Windows-1252 (the classic
/// "Ã©" for "é", "â€™" for "'"). The repair table is *derived* by re-encoding a set
/// of commonly-corrupted characters through the same mistake, so every sequence is
/// correct by construction rather than hand-typed.
pub fn fix_mojibake(input: &str, _step: &Step) -> Result<String> {
    Ok(single_pass_replace(input, &mojibake_table()))
}

/// Windows-1252 byte -> char. Bytes outside 0x80-0x9F map to the same code point
/// (CP1252 agrees with Latin-1 there); the 0x80-0x9F block is the special table.
/// The five unassigned slots (0x81/0x8D/0x8F/0x90/0x9D) fall through to identity.
fn cp1252(b: u8) -> char {
    match b {
        0x80 => '\u{20AC}',
        0x82 => '\u{201A}',
        0x83 => '\u{0192}',
        0x84 => '\u{201E}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02C6}',
        0x89 => '\u{2030}',
        0x8A => '\u{0160}',
        0x8B => '\u{2039}',
        0x8C => '\u{0152}',
        0x8E => '\u{017D}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201C}',
        0x94 => '\u{201D}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02DC}',
        0x99 => '\u{2122}',
        0x9A => '\u{0161}',
        0x9B => '\u{203A}',
        0x9C => '\u{0153}',
        0x9E => '\u{017E}',
        0x9F => '\u{0178}',
        other => other as char,
    }
}

/// Reproduce the corruption for one target string: encode it to UTF-8, then read
/// each byte back as Windows-1252. That is exactly the mistake that produced the
/// mojibake, so the result is the corrupted form we want to repair.
fn mojibake_of(target: &str) -> String {
    target.bytes().map(cp1252).collect()
}

/// (corrupted, correct) pairs, longest-corrupted-first so a prefix never wins over
/// a longer match. ASCII targets whose corruption equals themselves are dropped.
fn mojibake_table() -> Vec<(String, String)> {
    const TARGETS: &[&str] = &[
        "’", "‘", "“", "”", "–", "\u{2014}", "…", "•", "€", "™", "®", "©", "°", "±", "×", "·", "«",
        "»", "é", "è", "ê", "ë", "à", "â", "ä", "ç", "î", "ï", "ô", "ö", "ù", "û", "ü", "ñ", "á",
        "í", "ó", "ú", "ß", "É", "È", "À", "Ç", "Ü", "Ö", "Ä",
    ];
    let mut t: Vec<(String, String)> = TARGETS
        .iter()
        .map(|s| (mojibake_of(s), (*s).to_string()))
        .filter(|(m, tgt)| m != tgt)
        .collect();
    t.sort_by_key(|(m, _)| std::cmp::Reverse(m.len()));
    t
}

/// Replace using an ordered list of (find, replace) pairs in a single
/// left-to-right pass: at each position the first matching `find` (the list should
/// be ordered longest-first) is emitted as its `replace` and scanning resumes
/// after it, so replacements never chain into one another. Shared by
/// `fix_mojibake` and `replace_map`.
pub fn single_pass_replace(input: &str, pairs: &[(String, String)]) -> String {
    if pairs.is_empty() {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    'outer: while !rest.is_empty() {
        for (find, rep) in pairs {
            if !find.is_empty() && rest.starts_with(find.as_str()) {
                out.push_str(rep);
                rest = &rest[find.len()..];
                continue 'outer;
            }
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

/// De-accent every char of a string (multi-char expansions like ss/ae handled).
fn deaccent_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match deaccent(c) {
            Some(a) => out.push_str(a),
            None => out.push(c),
        }
    }
    out
}

/// `slugify`: turn each line into a URL/filename slug. De-accents to ASCII, keeps
/// alphanumerics, and collapses every other run into a single `separator`
/// (default "-"), trimming leading/trailing separators. `lowercase` (default
/// true).
pub fn slugify(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", "-");
    let lower = step.get_bool("lowercase", true)?;
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| slug_line(l, &sep, lower)).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

fn slug_line(line: &str, sep: &str, lower: bool) -> String {
    let src = deaccent_str(line);
    let mut out = String::with_capacity(src.len());
    let mut need_sep = false;
    for c in src.chars() {
        if c.is_ascii_alphanumeric() {
            if need_sep && !out.is_empty() {
                out.push_str(sep);
            }
            need_sep = false;
            out.push(if lower { c.to_ascii_lowercase() } else { c });
        } else if !out.is_empty() {
            need_sep = true;
        }
    }
    out
}

/// `strip_html_tags`: remove HTML/XML tags (`<...>`), leaving the text content.
/// Entities are left as-is (chain `html_decode` to decode them).
pub fn strip_html_tags(input: &str, _step: &Step) -> Result<String> {
    let re = Regex::new(r"<[^>]*>").map_err(|e| anyhow!(e))?;
    Ok(re.replace_all(input, "").into_owned())
}

/// `strip_markdown`: reduce common Markdown to plain text -- drop fence lines,
/// unwrap links/images to their text, and remove header/blockquote/list markers,
/// horizontal rules, and `*`/`**`/`` ` ``/`~~` emphasis. Underscore emphasis is
/// left alone so snake_case identifiers are not corrupted; for those, or nested
/// structure, this is a best-effort strip, not a Markdown parser.
pub fn strip_markdown(input: &str, _step: &Step) -> Result<String> {
    // (pattern, replacement) applied in order. Images before links.
    let rules: &[(&str, &str)] = &[
        (r"(?m)^\s*```.*$\n?", ""),                 // fenced code delimiters
        (r"!\[([^\]]*)\]\([^)]*\)", "$1"),          // image -> alt
        (r"\[([^\]]*)\]\([^)]*\)", "$1"),           // link -> text
        (r"(?m)^\s{0,3}#{1,6}\s+", ""),             // ATX headers
        (r"(?m)^\s{0,3}>\s?", ""),                  // blockquote
        (r"(?m)^(\s*)[-*+]\s+", "$1"),              // bullet markers
        (r"(?m)^(\s*)\d+\.\s+", "$1"),              // ordered markers
        (r"(?m)^\s{0,3}([-*_])( *\1){2,}\s*$", ""), // horizontal rules
        (r"\*\*([^*]+)\*\*", "$1"),                 // bold
        (r"\*([^*]+)\*", "$1"),                     // italic
        (r"~~([^~]+)~~", "$1"),                     // strikethrough
        (r"`([^`]+)`", "$1"),                       // inline code
    ];
    let mut s = input.to_string();
    for (pat, rep) in rules {
        let re = Regex::new(pat).map_err(|e| anyhow!(e))?;
        s = re.replace_all(&s, *rep).into_owned();
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        let m = parse_mog(json).unwrap();
        crate::execute(&m, input).unwrap()
    }

    fn err(json: &str, input: &str) -> String {
        let m = parse_mog(json).unwrap();
        crate::execute(&m, input).unwrap_err().to_string()
    }

    #[test]
    fn convert_comment_style_swaps_line_markers_and_keeps_indent() {
        let out = run(
            r#"{"steps":[{"action":"convert_comment_style","options":{"from":"slashes","to":"hash"}}]}"#,
            "  // note\ncode();\n//edge",
        );
        // Only comment lines change; indentation and text are preserved; code untouched.
        assert_eq!(out, "  # note\ncode();\n#edge");
    }

    #[test]
    fn keep_chars_digits_only_preserves_newlines() {
        let out = run(
            r#"{"steps":[{"action":"keep_chars","options":{"class":"digits"}}]}"#,
            "call 555-1234\nor 555.5678",
        );
        assert_eq!(out, "5551234\n5555678");
    }

    #[test]
    fn keep_chars_explicit_set() {
        let out = run(
            r#"{"steps":[{"action":"keep_chars","options":{"set":"abc"}}]}"#,
            "a1b2c3d",
        );
        assert_eq!(out, "abc");
    }

    #[test]
    fn keep_chars_unknown_class_errors_instead_of_silent_wipe() {
        // A typo'd class used to match nothing, so keep_chars silently kept only
        // newlines (all content gone, exit 0). It must error instead.
        let e = err(
            r#"{"steps":[{"action":"keep_chars","options":{"class":"digts"}}]}"#,
            "abc123",
        );
        assert!(e.contains("unknown character class 'digts'"), "{e}");
    }

    #[test]
    fn remove_chars_unknown_class_errors() {
        let e = err(
            r#"{"steps":[{"action":"remove_chars","options":{"class":"punct"}}]}"#,
            "a.b,c",
        );
        assert!(e.contains("unknown character class 'punct'"), "{e}");
    }

    #[test]
    fn keep_chars_bogus_class_ignored_when_set_given() {
        // An explicit set overrides class, so a bogus class is irrelevant here.
        let out = run(
            r#"{"steps":[{"action":"keep_chars","options":{"set":"abc","class":"bogus"}}]}"#,
            "a1b2c3",
        );
        assert_eq!(out, "abc");
    }

    #[test]
    fn strip_control_chars_unknown_mode_errors() {
        // A typo'd enum value used to silently take the default branch; now it errors.
        let e = err(
            r#"{"steps":[{"action":"strip_control_chars","options":{"mode":"replac"}}]}"#,
            "a\u{0007}b",
        );
        assert!(e.contains("invalid 'mode'"), "{e}");
    }

    #[test]
    fn remove_chars_punctuation_default() {
        let out = run(
            r#"{"steps":[{"action":"remove_chars"}]}"#,
            "hello, world! (yes)",
        );
        assert_eq!(out, "hello world yes");
    }

    #[test]
    fn strip_control_chars_keeps_tab_and_newline() {
        let out = run(
            r#"{"steps":[{"action":"strip_control_chars"}]}"#,
            "a\u{0000}b\tc\n",
        );
        assert_eq!(out, "ab\tc\n");
    }

    #[test]
    fn normalize_smart_punctuation_and_zero_width() {
        let out = run(
            r#"{"steps":[{"action":"normalize"}]}"#,
            "\u{201C}Hi\u{201D} \u{2014} it\u{2019}s\u{200B} fine\u{2026}\u{00A0}ok",
        );
        assert_eq!(out, "\"Hi\" - it's fine... ok");
    }

    #[test]
    fn normalize_accents_opt_in() {
        let out = run(
            r#"{"steps":[{"action":"normalize","options":{"accents":true}}]}"#,
            "café naïve über",
        );
        assert_eq!(out, "cafe naive uber");
    }

    #[test]
    fn normalize_accents_off_by_default() {
        let out = run(r#"{"steps":[{"action":"normalize"}]}"#, "café");
        assert_eq!(out, "café");
    }

    #[test]
    fn fix_mojibake_repairs_common_sequences() {
        // A phrase with an accent, an em dash, and a curly quote, corrupted
        // via UTF-8-read-as-CP1252, then repaired.
        let corrupted = mojibake_of("café \u{2014} it’s");
        let out = run(r#"{"steps":[{"action":"fix_mojibake"}]}"#, &corrupted);
        assert_eq!(out, "café \u{2014} it’s");
    }

    #[test]
    fn single_pass_does_not_chain() {
        // a->b then b->c must NOT turn a into c in one pass.
        let pairs = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        assert_eq!(single_pass_replace("ab", &pairs), "bc");
    }

    #[test]
    fn slugify_title() {
        let out = run(
            r#"{"steps":[{"action":"slugify"}]}"#,
            "Café con Leche!  Yes",
        );
        assert_eq!(out, "cafe-con-leche-yes");
    }

    #[test]
    fn strip_html_tags_removes_tags() {
        let out = run(
            r#"{"steps":[{"action":"strip_html_tags"}]}"#,
            "<p>Hi <b>there</b></p>",
        );
        assert_eq!(out, "Hi there");
    }
}
