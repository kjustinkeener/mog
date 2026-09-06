//! Generate a per-mog `README.md` from the mog itself: its metadata
//! (name / summary / description / tags / steps) plus a real input -> output
//! example pulled from its golden fixture. The output is deterministic (derived
//! only from committed files, no clock or RNG), so a drift test can assert the
//! committed docs stay in sync with the mogs, exactly like a golden.
//!
//! A mog folder may also carry a hand-authored `GUIDE.md` for deeper reference
//! (mapping tables, caveats) that cannot come from metadata; when present, the
//! generated README links to it.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::model::Mog;
use crate::testkit;

/// Marker on every generated file so contributors edit the .mog, not the doc.
const MARKER: &str =
    "<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->";

/// How many fixture lines to show in the example before truncating.
const EXAMPLE_LINES: usize = 18;

/// Trim a fixture to at most `max` lines; returns the excerpt and how many lines
/// were dropped.
fn excerpt(s: &str, max: usize) -> (String, usize) {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= max {
        (s.trim_end_matches('\n').to_string(), 0)
    } else {
        (lines[..max].join("\n"), lines.len() - max)
    }
}

/// A fenced-code delimiter long enough to wrap `content` even if it contains
/// backtick runs of its own (at least three backticks).
fn fence(content: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in content.chars() {
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat(longest.max(2) + 1)
}

fn code_block(out: &mut String, body: &str) {
    let f = fence(body);
    out.push_str(&f);
    out.push('\n');
    out.push_str(body);
    out.push('\n');
    out.push_str(&f);
    out.push('\n');
}

/// Render the `README.md` markdown for one mog `.mog` at `mog_path`.
pub fn render_mog_doc(mog_path: &Path) -> Result<String> {
    let text = std::fs::read_to_string(mog_path)
        .with_context(|| format!("read '{}'", mog_path.display()))?;
    // Parse without interpolating constants, so a mog with unbound
    // `{{placeholders}}` still documents (mirrors the listing path).
    let mog: Mog =
        serde_json::from_str(&text).with_context(|| format!("parse '{}'", mog_path.display()))?;
    let stem = mog_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("recipe");
    let title = mog.name.clone().unwrap_or_else(|| stem.to_string());

    let mut md = String::new();
    md.push_str(&format!("# {title}\n\n"));
    if let Some(s) = mog.summary.as_deref().filter(|s| !s.is_empty()) {
        md.push_str(s);
        md.push_str("\n\n");
    }
    if let Some(d) = mog.description.as_deref().filter(|d| !d.is_empty()) {
        md.push_str(d);
        md.push_str("\n\n");
    }

    // Run line.
    md.push_str("## Run\n\n");
    code_block(&mut md, &format!("mog -m {stem} <file>"));
    md.push_str(
        "\nPreview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.\n\n",
    );

    // Example from the golden fixture (input -> expected output).
    if let (Some(inp), Some(exp)) = (
        testkit::sibling(mog_path, "TestInput"),
        testkit::sibling(mog_path, "TestExpectedOutput"),
    ) {
        let input = std::fs::read_to_string(&inp).unwrap_or_default();
        let expected = std::fs::read_to_string(&exp).unwrap_or_default();
        let (ie, more_in) = excerpt(&input, EXAMPLE_LINES);
        let (ee, more_out) = excerpt(&expected, EXAMPLE_LINES);
        md.push_str("## Example\n\nInput:\n\n");
        code_block(&mut md, &ie);
        if more_in > 0 {
            md.push_str(&format!("\n_(... {more_in} more line(s))_\n"));
        }
        md.push_str("\nOutput:\n\n");
        code_block(&mut md, &ee);
        if more_out > 0 {
            md.push_str(&format!("\n_(... {more_out} more line(s))_\n"));
        }
        md.push('\n');
    }

    // Pipeline: the enabled steps in order.
    let steps: Vec<&crate::model::Step> = mog.steps.iter().filter(|s| !s.disabled).collect();
    if !steps.is_empty() {
        md.push_str("## Pipeline\n\n");
        for step in steps {
            let action = step.action.as_deref().unwrap_or("(no action)");
            match step.description.as_deref().filter(|d| !d.is_empty()) {
                Some(d) => md.push_str(&format!("- `{action}`: {d}\n")),
                None => md.push_str(&format!("- `{action}`\n")),
            }
        }
        md.push('\n');
    }

    // Tags.
    if !mog.tags.is_empty() {
        md.push_str("## Tags\n\n");
        md.push_str(
            &mog.tags
                .iter()
                .map(|t| format!("`{t}`"))
                .collect::<Vec<_>>()
                .join(" "),
        );
        md.push_str("\n\n");
    }

    // Link to a hand-authored deep guide, if the mog ships one.
    if mog_path
        .parent()
        .is_some_and(|p| p.join("GUIDE.md").exists())
    {
        md.push_str("## Full guide\n\nSee [GUIDE.md](GUIDE.md) for the complete reference and known limitations.\n\n");
    }

    md.push_str("---\n");
    md.push_str(MARKER);
    md.push('\n');
    Ok(md)
}

/// LF-normalize so a comparison is not confused by CRLF checkout differences.
fn norm(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Generate (or, under `check`, verify) a `README.md` beside every `.mog` under
/// `dir`. In check mode this writes nothing and returns an error listing the
/// mogs whose committed doc drifted from what the mog would generate.
pub fn gen_docs(dir: &Path, check: bool) -> Result<i32> {
    let mogs = testkit::find_mogs(dir);
    if mogs.is_empty() {
        bail!("no mogs found under '{}'", dir.display());
    }
    let mut drifted = Vec::new();
    let mut wrote = 0usize;
    for mog in &mogs {
        let content = render_mog_doc(mog)?;
        let readme = mog
            .parent()
            .with_context(|| format!("'{}' has no parent dir", mog.display()))?
            .join("README.md");
        if check {
            let existing = std::fs::read_to_string(&readme).unwrap_or_default();
            if norm(&existing) != norm(&content) {
                drifted.push(readme);
            }
        } else {
            std::fs::write(&readme, &content)
                .with_context(|| format!("write '{}'", readme.display()))?;
            wrote += 1;
        }
    }
    if check {
        if drifted.is_empty() {
            println!("all {} mog docs are up to date", mogs.len());
            Ok(0)
        } else {
            for d in &drifted {
                println!("out of date: {}", d.display());
            }
            bail!(
                "{} mog doc(s) out of date; run `mog market gen-docs`",
                drifted.len()
            )
        }
    } else {
        println!("wrote {wrote} mog doc(s)");
        Ok(0)
    }
}
