//! Whitespace actions. These operate on raw text (regex/string ops), NOT via
//! LineText. Mirrors .NET `WhitespaceActions.cs` + `MogStringExtensions.cs`.

use anyhow::{anyhow, Result};
use fancy_regex::Regex;

use crate::line_text::{join, split};
use crate::model::Step;

fn width_of(step: &Step) -> Result<usize> {
    let w = step.get_i64("width", 4)?;
    if w < 1 {
        Ok(4)
    } else {
        Ok(w as usize)
    }
}

/// `trim_whitespace_right` / `twr`: strip trailing spaces/tabs per line,
/// preserving a `\r` before the newline.
///
/// Byte-identical to the former regex `(?m)[ \t]+(\r?$) -> $1`, but O(n) instead
/// of O(n^2): fancy-regex is a backtracking engine, and its multiline `$` anchor
/// made whole-file trims quadratic (measured: ~69x for 8x input). Splitting on
/// `\n` gives each line's end position directly. Within a line the only line-end
/// is its trailing edge, so we strip trailing ` `/`\t`, keeping a lone trailing
/// `\r` (the CRLF carriage return, captured and re-emitted by the old `\r?`).
/// The equivalence is fuzzed against the reference regex in the tests below.
pub fn trim_right(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    for (i, part) in input.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if let Some(body) = part.strip_suffix('\r') {
            out.push_str(body.trim_end_matches([' ', '\t']));
            out.push('\r');
        } else {
            out.push_str(part.trim_end_matches([' ', '\t']));
        }
    }
    Ok(out)
}

/// `trim_whitespace_left` / `twl`: strip leading spaces/tabs per line.
///
/// Byte-identical to the former regex `(?m)^[ \t]* -> ``, O(n) (see `trim_right`
/// for why the regex form was quadratic). `^` matched at string start and after
/// every `\n`, i.e. at each line's start, so stripping leading ` `/`\t` per
/// `\n`-split segment is exactly equivalent.
pub fn trim_left(input: &str, _step: &Step) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    for (i, part) in input.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(part.trim_start_matches([' ', '\t']));
    }
    Ok(out)
}

/// `trim_whitespace` / `tw`: right then left.
pub fn trim_both(input: &str, step: &Step) -> Result<String> {
    let r = trim_right(input, step)?;
    trim_left(&r, step)
}

/// `tabs_to_spaces` / `t2s`: every tab -> `width` spaces (default 4, clamp <1 -> 4).
pub fn tabs_to_spaces(input: &str, step: &Step) -> Result<String> {
    let width = width_of(step)?;
    let spaces = " ".repeat(width);
    Ok(input.replace('\t', &spaces))
}

/// `spaces_to_tabs` / `s2t`: options width, leading_only.
pub fn spaces_to_tabs(input: &str, step: &Step) -> Result<String> {
    let width = width_of(step)?;
    let leading_only = step.get_bool("leading_only", false)?;

    if leading_only {
        // Replace leading run of spaces per line with tabs + remainder spaces.
        let re = Regex::new(r"(?m)^ +").map_err(|e| anyhow!(e))?;
        let out = re.replace_all(input, |caps: &fancy_regex::Captures| {
            let len = caps.get(0).map(|m| m.as_str().len()).unwrap_or(0);
            let tabs = len / width;
            let rem = len % width;
            format!("{}{}", "\t".repeat(tabs), " ".repeat(rem))
        });
        Ok(out.into_owned())
    } else {
        // Replace each exact run of `width` spaces with a tab.
        let pattern = format!("(?m) {{{width}}}");
        let re = Regex::new(&pattern).map_err(|e| anyhow!(e))?;
        Ok(re.replace_all(input, "\t").into_owned())
    }
}

/// `collapse_whitespace`: within each line, replace runs of spaces/tabs with a
/// single space. Newlines are left untouched.
pub fn collapse_whitespace(input: &str, _step: &Step) -> Result<String> {
    let re = Regex::new(r"[ \t]+").map_err(|e| anyhow!(e))?;
    Ok(re.replace_all(input, " ").into_owned())
}

/// `squeeze_spaces`: collapse runs of two or more spaces to a single space, but
/// preserve each line's leading indentation and leave tabs alone. Unlike
/// `collapse_whitespace` this keeps code/table indentation intact.
pub fn squeeze_spaces(input: &str, _step: &Step) -> Result<String> {
    let s = split(input);
    let out: Vec<String> = s.lines.iter().map(|l| squeeze_line(l)).collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

fn squeeze_line(line: &str) -> String {
    // Keep the leading run of spaces/tabs verbatim; squeeze only the body.
    let lead_end = line.find(|c| c != ' ' && c != '\t').unwrap_or(line.len());
    let (lead, rest) = line.split_at(lead_end);
    let mut out = String::with_capacity(line.len());
    out.push_str(lead);
    let mut prev_space = false;
    for c in rest.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(c);
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// `eol_to_space`: every line ending -> single space.
pub fn eol_to_space(input: &str, _step: &Step) -> Result<String> {
    let re = Regex::new(r"\r\n|\r|\n").map_err(|e| anyhow!(e))?;
    Ok(re.replace_all(input, " ").into_owned())
}

/// `trim_and_eol_to_space`: split lines, trim each (all Unicode whitespace),
/// join with single spaces. Trailing newline is discarded.
pub fn trim_and_eol_to_space(input: &str, _step: &Step) -> Result<String> {
    let s = split(input);
    let trimmed: Vec<String> = s.lines.iter().map(|l| l.trim().to_string()).collect();
    Ok(trimmed.join(" "))
}

/// Leading-whitespace prefix of a line (spaces and tabs).
fn leading_ws(line: &str) -> &str {
    let end = line
        .find(|c: char| c != ' ' && c != '\t')
        .unwrap_or(line.len());
    &line[..end]
}

/// `dedent`: remove the longest common leading-whitespace prefix shared by all
/// non-blank lines (Python `textwrap.dedent`). Blank lines are ignored when
/// computing the prefix and are left unchanged.
pub fn dedent(input: &str, _step: &Step) -> Result<String> {
    let s = split(input);
    let mut common: Option<String> = None;
    for l in &s.lines {
        if l.trim().is_empty() {
            continue;
        }
        let lead = leading_ws(l);
        common = Some(match common {
            None => lead.to_string(),
            Some(prev) => {
                let n = prev
                    .chars()
                    .zip(lead.chars())
                    .take_while(|(a, b)| a == b)
                    .count();
                prev.chars().take(n).collect()
            }
        });
        if common.as_deref() == Some("") {
            break;
        }
    }
    let prefix = common.unwrap_or_default();
    if prefix.is_empty() {
        return Ok(join(&s.lines, s.eol, s.trailing_eol));
    }
    let out: Vec<String> = s
        .lines
        .iter()
        .map(|l| l.strip_prefix(&prefix).unwrap_or(l).to_string())
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `align_columns`: align cells on a `separator` (default "|") across the block.
/// Each aligned line's cells are trimmed, padded to the per-column max width, and
/// rejoined with " <sep> ". Lines not containing the separator pass through.
pub fn align_columns(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", "|");
    if sep.is_empty() {
        return Ok(input.to_string());
    }
    let s = split(input);
    let rows: Vec<Option<Vec<String>>> = s
        .lines
        .iter()
        .map(|l| {
            if l.contains(sep.as_str()) {
                Some(
                    l.split(sep.as_str())
                        .map(|c| c.trim().to_string())
                        .collect(),
                )
            } else {
                None
            }
        })
        .collect();
    let mut widths: Vec<usize> = Vec::new();
    for row in rows.iter().flatten() {
        for (i, cell) in row.iter().enumerate() {
            let w = cell.chars().count();
            if i < widths.len() {
                widths[i] = widths[i].max(w);
            } else {
                widths.push(w);
            }
        }
    }
    let joiner = format!(" {sep} ");
    let out: Vec<String> = s
        .lines
        .iter()
        .zip(rows.iter())
        .map(|(orig, row)| match row {
            None => orig.clone(),
            Some(cells) => {
                let padded: Vec<String> = cells
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let w = widths.get(i).copied().unwrap_or(0);
                        let pad = w.saturating_sub(cell.chars().count());
                        format!("{cell}{}", " ".repeat(pad))
                    })
                    .collect();
                padded.join(&joiner).trim_end().to_string()
            }
        })
        .collect();
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Detect a leading quote/comment prefix (`> `, `# `, `// `, `-- `, possibly
/// repeated/nested) on a line. Returns the matched prefix slice, or "" if none.
fn leading_prefix(re: &Regex, line: &str) -> String {
    match re.find(line) {
        Ok(Some(m)) => line[..m.end()].to_string(),
        _ => String::new(),
    }
}

fn prefix_regex() -> Result<Regex> {
    Regex::new(r"^(?:[ \t]*(?:>|#|//|--)[ \t]?)+").map_err(|e| anyhow!(e))
}

/// Greedy first-fit line packing of `words` into lines of at most `width` display
/// columns, each carrying `prefix`. A word longer than the available width is left
/// intact unless `break_long` splits it at the width boundary. Deterministic.
fn wrap_words(out: &mut Vec<String>, words: &[&str], prefix: &str, width: usize, break_long: bool) {
    if words.is_empty() {
        return;
    }
    let content_width = {
        let w = width.saturating_sub(prefix.chars().count());
        if w == 0 {
            1
        } else {
            w
        }
    };
    let mut cur = String::new();
    let mut cur_len = 0usize;
    let start_word = |cur: &mut String, cur_len: &mut usize, out: &mut Vec<String>, word: &str| {
        let wl = word.chars().count();
        if wl > content_width && break_long {
            let mut chunk = String::new();
            let mut cl = 0usize;
            for ch in word.chars() {
                chunk.push(ch);
                cl += 1;
                if cl == content_width {
                    out.push(format!("{prefix}{chunk}"));
                    chunk.clear();
                    cl = 0;
                }
            }
            *cur = chunk;
            *cur_len = cl;
        } else {
            *cur = word.to_string();
            *cur_len = wl;
        }
    };
    for &word in words {
        let wl = word.chars().count();
        if cur.is_empty() {
            start_word(&mut cur, &mut cur_len, out, word);
        } else if cur_len + 1 + wl <= content_width {
            cur.push(' ');
            cur.push_str(word);
            cur_len += 1 + wl;
        } else {
            out.push(format!("{prefix}{cur}"));
            cur.clear();
            start_word(&mut cur, &mut cur_len, out, word);
        }
    }
    if !cur.is_empty() {
        out.push(format!("{prefix}{cur}"));
    }
}

/// `wrap_text`: reflow text to a target column `width` (greedy first-fit, like
/// Emacs fill-paragraph / Vim gq). Blank lines are hard breaks when
/// `preserve_paragraphs` (default true); a leading quote/comment prefix is carried
/// onto every wrapped line when `prefix_aware`. Fully deterministic (no Knuth-Plass).
pub fn wrap_text(input: &str, step: &Step) -> Result<String> {
    let width = {
        let w = step.get_i64("width", 80)?;
        if w < 1 {
            80
        } else {
            w as usize
        }
    };
    let preserve = step.get_bool("preserve_paragraphs", true)?;
    let prefix_aware = step.get_bool("prefix_aware", false)?;
    let break_long = step.get_bool("break_long_words", false)?;
    let re = prefix_regex()?;
    let s = split(input);
    let mut out: Vec<String> = Vec::new();

    // Gather the words of a paragraph (its lines, with any per-line prefix stripped
    // when prefix_aware) and append the wrapped result, using the prefix detected on
    // the paragraph's first line.
    let flush = |out: &mut Vec<String>, para: &[&str]| {
        if para.is_empty() {
            return;
        }
        let prefix = if prefix_aware {
            leading_prefix(&re, para[0])
        } else {
            String::new()
        };
        let mut words: Vec<&str> = Vec::new();
        for l in para {
            let body = if prefix_aware {
                let p = leading_prefix(&re, l);
                &l[p.len()..]
            } else {
                l
            };
            words.extend(body.split_whitespace());
        }
        wrap_words(out, &words, &prefix, width, break_long);
    };

    if !preserve {
        // Whole input is one paragraph; blank lines are dropped.
        let para: Vec<&str> = s.lines.iter().map(|l| l.as_str()).collect();
        flush(&mut out, &para);
    } else {
        let mut para: Vec<&str> = Vec::new();
        for l in &s.lines {
            if l.trim().is_empty() {
                flush(&mut out, &para);
                para.clear();
                out.push(l.clone()); // preserve the blank separator verbatim
            } else {
                para.push(l);
            }
        }
        flush(&mut out, &para);
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// `unwrap_text`: collapse the soft-wrapped lines of each paragraph back to a single
/// line, joined by `separator` (default one space). Blank lines are preserved as
/// paragraph breaks. With `prefix_aware`, a common `> `/`# ` prefix is stripped
/// before joining and re-emitted once. The inverse of `wrap_text`.
pub fn unwrap_text(input: &str, step: &Step) -> Result<String> {
    let sep = step.get_string_or("separator", " ");
    let prefix_aware = step.get_bool("prefix_aware", false)?;
    let re = prefix_regex()?;
    let s = split(input);
    let mut out: Vec<String> = Vec::new();

    let flush = |out: &mut Vec<String>, para: &[&str]| {
        if para.is_empty() {
            return;
        }
        if prefix_aware {
            let prefix = leading_prefix(&re, para[0]);
            let bodies: Vec<&str> = para
                .iter()
                .map(|l| {
                    let p = leading_prefix(&re, l);
                    &l[p.len()..]
                })
                .collect();
            out.push(format!("{prefix}{}", bodies.join(sep.as_str())));
        } else {
            out.push(para.join(sep.as_str()));
        }
    };

    let mut para: Vec<&str> = Vec::new();
    for l in &s.lines {
        if l.trim().is_empty() {
            flush(&mut out, &para);
            para.clear();
            out.push(l.clone());
        } else {
            para.push(l);
        }
    }
    flush(&mut out, &para);
    Ok(join(&out, s.eol, s.trailing_eol))
}

#[cfg(test)]
mod added_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn dedent_removes_common_indent() {
        let out = run(
            r#"{"steps":[{"action":"dedent"}]}"#,
            "    a\n      b\n    c",
        );
        assert_eq!(out, "a\n  b\nc");
    }

    #[test]
    fn align_columns_pipe() {
        let out = run(
            r#"{"steps":[{"action":"align_columns","options":{"separator":"|"}}]}"#,
            "a|bb|c\naaa|b|cc",
        );
        assert_eq!(out, "a   | bb | c\naaa | b  | cc");
    }

    /// The O(n) trim_right/trim_left must stay byte-for-byte identical to the
    /// backtracking-regex forms they replaced (which were quadratic). Fuzz both
    /// against the reference regex over random strings drawn from the exact
    /// alphabet that exercises the tricky cases: spaces, tabs, lone CR, LF, and
    /// ordinary chars (so CRLF vs lone-CR, EOF-with/without-newline, blank lines,
    /// and interior CR are all hit).
    #[test]
    fn trim_right_left_byte_identical_to_regex_reference() {
        use fancy_regex::Regex;
        use rand::{rngs::StdRng, Rng, SeedableRng};

        // The pre-optimization regex forms, kept only as the equivalence oracle.
        let re_r = Regex::new(r"(?m)[ \t]+(\r?$)").unwrap();
        let re_l = Regex::new(r"(?m)^[ \t]*").unwrap();
        // Both functions ignore their Step; any valid Step works.
        let step: crate::model::Step = serde_json::from_str(r#"{"action":"x"}"#).unwrap();

        let alphabet = [' ', '\t', '\r', '\n', 'a', 'b'];
        let mut rng = StdRng::seed_from_u64(0xB0BA_CAFE);
        for _ in 0..20_000 {
            let len = rng.gen_range(0..24);
            let s: String = (0..len)
                .map(|_| alphabet[rng.gen_range(0..alphabet.len())])
                .collect();

            let got_r = super::trim_right(&s, &step).unwrap();
            let want_r = re_r.replace_all(&s, "$1").into_owned();
            assert_eq!(got_r, want_r, "trim_right diverged on {s:?}");

            let got_l = super::trim_left(&s, &step).unwrap();
            let want_l = re_l.replace_all(&s, "").into_owned();
            assert_eq!(got_l, want_l, "trim_left diverged on {s:?}");
        }
    }

    #[test]
    fn wrap_text_greedy_fill() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":10}}]}"#,
            "the quick brown fox jumps",
        );
        assert_eq!(out, "the quick\nbrown fox\njumps");
    }

    #[test]
    fn wrap_text_preserves_paragraph_breaks() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":12}}]}"#,
            "one two three\n\nfour five six",
        );
        assert_eq!(out, "one two\nthree\n\nfour five\nsix");
    }

    #[test]
    fn wrap_text_no_preserve_is_one_paragraph() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":12,"preserve_paragraphs":false}}]}"#,
            "one two\n\nthree four",
        );
        assert_eq!(out, "one two\nthree four");
    }

    #[test]
    fn wrap_text_prefix_aware_carries_quote() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":10,"prefix_aware":true}}]}"#,
            "> alpha beta gamma delta",
        );
        assert_eq!(out, "> alpha\n> beta\n> gamma\n> delta");
    }

    #[test]
    fn wrap_text_long_word_intact_by_default() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":5}}]}"#,
            "supercalifragilistic ok",
        );
        assert_eq!(out, "supercalifragilistic\nok");
    }

    #[test]
    fn wrap_text_break_long_words() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":5,"break_long_words":true}}]}"#,
            "abcdefghij ok",
        );
        assert_eq!(out, "abcde\nfghij\nok");
    }

    #[test]
    fn wrap_text_preserves_trailing_newline() {
        let out = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":40}}]}"#,
            "short line\n",
        );
        assert_eq!(out, "short line\n");
    }

    #[test]
    fn unwrap_text_joins_paragraph() {
        let out = run(
            r#"{"steps":[{"action":"unwrap_text"}]}"#,
            "the quick\nbrown fox\n\nnext para\nline two",
        );
        assert_eq!(out, "the quick brown fox\n\nnext para line two");
    }

    #[test]
    fn unwrap_text_prefix_aware_roundtrip() {
        let wrapped = "> alpha\n> beta\n> gamma\n> delta";
        let out = run(
            r#"{"steps":[{"action":"unwrap_text","options":{"prefix_aware":true}}]}"#,
            wrapped,
        );
        assert_eq!(out, "> alpha beta gamma delta");
    }

    #[test]
    fn wrap_unwrap_roundtrip() {
        let original = "This is a fairly long paragraph that will be reflowed and then joined back together again.";
        let wrapped = run(
            r#"{"steps":[{"action":"wrap_text","options":{"width":20}}]}"#,
            original,
        );
        let unwrapped = run(r#"{"steps":[{"action":"unwrap_text"}]}"#, &wrapped);
        assert_eq!(unwrapped, original);
    }
}
