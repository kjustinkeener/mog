//! Phase 4 streaming: which pipelines can run in bounded memory by processing the
//! input in large blocks split at newline boundaries and concatenating the
//! per-block outputs.
//!
//! Concatenating per-block outputs equals the whole-file output ONLY for actions
//! that derive each line's output from that line's bytes alone AND preserve every
//! terminator byte-for-byte, so a block boundary never changes the result. That
//! set is pinned by the chunk-concat-safety test in `tests/streaming.rs`: do not
//! add a name here without a passing case there.

use anyhow::Result;
use rayon::prelude::*;

use crate::model::Mog;

/// Split `text` into at most `n` line-aligned pieces: each piece ends at a `\n`
/// (except the last), so `pieces.concat() == text` exactly and every boundary is a
/// real line boundary. Fewer than `n` pieces are returned when the text is short
/// or has too few newlines.
fn split_line_aligned(text: &str, n: usize) -> Vec<&str> {
    if n <= 1 || text.len() < 2 {
        return vec![text];
    }
    let bytes = text.as_bytes();
    let mut bounds = Vec::with_capacity(n + 1);
    bounds.push(0usize);
    let step = text.len() / n;
    for k in 1..n {
        // Aim near an even split, then snap forward to just past the next newline.
        let mut p = (step * k).min(bytes.len());
        while p < bytes.len() && bytes[p] != b'\n' {
            p += 1;
        }
        if p < bytes.len() {
            p += 1; // include the newline in the earlier piece
        }
        if p > *bounds.last().unwrap() && p < text.len() {
            bounds.push(p);
        }
    }
    bounds.push(text.len());
    bounds.windows(2).map(|w| &text[w[0]..w[1]]).collect()
}

/// Transform `input` by splitting it into up to `chunks` line-aligned pieces,
/// running the whole pipeline on each in parallel (rayon), and concatenating the
/// results in order. Byte-identical to sequential [`crate::execute`] ONLY for
/// streamable pipelines -- the caller MUST gate on [`pipeline_is_streamable`].
/// A pure speed optimization for a large single file; memory is still O(file).
pub fn execute_parallel(mog: &Mog, input: &str, chunks: usize) -> Result<String> {
    let pieces = split_line_aligned(input, chunks.max(1));
    if pieces.len() == 1 {
        return crate::engine::execute(mog, input);
    }
    let outs: Result<Vec<String>> = pieces
        .par_iter()
        .map(|p| crate::engine::execute(mog, p))
        .collect();
    let outs = outs?;
    let mut joined = String::with_capacity(outs.iter().map(String::len).sum());
    for s in outs {
        joined.push_str(&s);
    }
    Ok(joined)
}

/// True if `name` (canonical or alias) is a chunk-concat-safe action.
///
/// Excluded and why: actions built on `line_text::split`+`join` (prefix_lines,
/// indent, the per-line filters, detectors, to_sentence, ...) -- `join`
/// normalizes every terminator to the first-detected EOL, so their output depends
/// on the whole input's first line; whole-string pattern actions (`replace`,
/// `replace_regex`) -- a pattern can match across a `\n`; and anything cross-line,
/// whole-document, or stateful (sort, dedupe, number_lines, base64, ...).
pub fn action_is_streamable(name: &str) -> bool {
    // Resolve aliases (e.g. "upper", "tw", "eol_unix") to the canonical name so
    // the check is written once against canonical names.
    let canonical = crate::descriptors::find_descriptor(name)
        .map(|d| d.name)
        .unwrap_or(name);
    is_streamable_canonical(canonical)
}

/// The streamable check on an already-canonical name, WITHOUT alias resolution.
/// Used by `descriptors::d()` to stamp each descriptor's `streamable` flag; going
/// through `action_is_streamable` there would recurse (descriptors -> find ->
/// descriptors).
pub(crate) fn is_streamable_canonical(canonical: &str) -> bool {
    matches!(
        canonical,
        "to_upper"
            | "to_lower"
            | "to_proper"
            | "invert_case"
            | "trim_whitespace"
            | "trim_whitespace_left"
            | "trim_whitespace_right"
            | "tabs_to_spaces"
            | "spaces_to_tabs"
            | "collapse_whitespace"
            | "keep_chars"
            | "remove_chars"
            | "strip_control_chars"
            | "eol_lf"
            | "eol_crlf"
            | "eol_cr"
    )
}

/// True if `name` (canonical or alias) ALWAYS preserves every input `\n` byte, so
/// a chunk boundary (which sits at a `\n`) stays valid after the action and any
/// later `\n`-dependent step is still chunk-safe.
///
/// Excluded (they can alter/drop `\n`, so they are only safe as the LAST step):
/// `eol_cr` (rewrites `\n`->`\r`); `keep_chars`/`remove_chars` (drop `\n` when
/// `keep_newlines` is false). Verified in `tests/streaming.rs`.
pub fn action_preserves_newlines(name: &str) -> bool {
    let canonical = crate::descriptors::find_descriptor(name)
        .map(|d| d.name)
        .unwrap_or(name);
    matches!(
        canonical,
        "to_upper"
            | "to_lower"
            | "to_proper"
            | "invert_case"
            | "trim_whitespace"
            | "trim_whitespace_left"
            | "trim_whitespace_right"
            | "tabs_to_spaces"
            | "spaces_to_tabs"
            | "collapse_whitespace"
            | "strip_control_chars"
            | "eol_lf"
            | "eol_crlf"
    )
}

/// True if the whole pipeline can be streamed: at least one step, no scope
/// (scoped/`only_lines_matching`/`except_lines_matching` steps re-split+join
/// lines), every step is a streamable action, AND every step except the LAST
/// preserves `\n` boundaries.
///
/// The last condition is the multi-step guard: single-action concat-safety is
/// necessary but NOT sufficient. If an early step damages the `\n` structure a
/// later step depends on (e.g. `[eol_cr, trim_whitespace_left]`), chunking at `\n`
/// diverges from the whole-file result. A `\n`-altering action is fine only when
/// nothing after it depends on `\n`, i.e. when it is last.
pub fn pipeline_is_streamable(mog: &Mog) -> bool {
    if mog.steps.is_empty() {
        return false;
    }
    let last = mog.steps.len() - 1;
    for (i, s) in mog.steps.iter().enumerate() {
        if s.only_lines_matching.is_some() || s.except_lines_matching.is_some() || s.scope.is_some()
        {
            return false;
        }
        let action = match s.action.as_deref() {
            Some(a) => a,
            None => return false,
        };
        if !action_is_streamable(action) {
            return false;
        }
        if i != last && !action_preserves_newlines(action) {
            return false;
        }
    }
    true
}
