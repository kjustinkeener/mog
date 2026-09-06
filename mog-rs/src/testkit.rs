//! Shared discovery + comparison for a mogfile's torture fixtures.
//!
//! Each mog lives in its own directory: `<mog>/<mog>.mog` ships a
//! `<mog>/tests/input.<ext>` (an edge-heavy input) and a
//! `<mog>/tests/expected.<ext>` golden. Running the script over the input must
//! reproduce the golden (EOL-normalized). This module is the ONE source of truth
//! for that discovery + comparison, shared by the `tests/torture_standard.rs`
//! suite and the `mog --test` CLI verb. See docs/mogfile-standard.md.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use similar::TextDiff;

use crate::{execute_with_library_sources_observed, parse_mog_with_defines};

/// Normalize EOLs to LF, so a golden compares tolerantly across CRLF/LF/CR.
pub fn lf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Find a mog's fixture, if any. `infix` is the fixture role (`TestInput` /
/// `TestExpectedOutput`). The one-directory-per-mog layout keeps fixtures in a
/// `tests/` subdir under new stems (`input` / `expected`), i.e.
/// `<mog>/tests/input.<ext>` and `<mog>/tests/expected.<ext>`; that is
/// tried first. For backward compatibility with the older flat layout (a library
/// authored before the per-mog-directory move), a `<stem>.<infix>.<ext>` file
/// beside the `.mog` is used as a fallback. Returns the first matching file (any ext).
pub fn sibling(mog: &Path, infix: &str) -> Option<PathBuf> {
    let dir = mog.parent()?;

    // Preferred: <mog>/tests/input.* or <mog>/tests/expected.*
    let tests_stem = match infix {
        "TestInput" => "input",
        "TestExpectedOutput" => "expected",
        other => other,
    };
    if let Some(found) = first_with_prefix(&dir.join("tests"), &format!("{tests_stem}.")) {
        return Some(found);
    }

    // Fallback: the legacy flat sibling <stem>.<infix>.<ext> beside the .mog.
    let stem = mog.file_stem()?.to_str()?;
    first_with_prefix(dir, &format!("{stem}.{infix}."))
}

/// The first entry in `dir` whose file name starts with `prefix`, if any.
fn first_with_prefix(dir: &Path, prefix: &str) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(prefix))
        })
}

/// Collect every `.mog` at or under `path` (sorted). A file path that is itself a
/// `.mog` returns just itself; a directory is walked recursively.
pub fn find_mogs(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect(path, &mut out);
    out.sort();
    out.dedup();
    out
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|x| x == "mog") {
            out.push(path.to_path_buf());
        }
        return;
    }
    if let Ok(entries) = fs::read_dir(path) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect(&p, out);
            } else if p.extension().is_some_and(|x| x == "mog") {
                out.push(p);
            }
        }
    }
}

/// The result of running one mogfile against its sibling golden.
pub struct TestOutcome {
    /// The `.mog` script that was run.
    pub mog: PathBuf,
    /// True when the script's output matched its `TestExpectedOutput` golden.
    pub pass: bool,
    /// A unified diff (golden -> actual) on a content mismatch, else `None`.
    pub diff: Option<String>,
    /// A human-readable reason on failure (mismatch, missing fixture, run error).
    pub message: Option<String>,
}

/// Run `mog` over its `tests/input.<ext>`, compare (EOL-normalized) to
/// `tests/expected.<ext>`. Never panics: a missing fixture, parse error, or run
/// error becomes a failing [`TestOutcome`] with a `message`.
/// Run one mogfile's torture fixtures. `now`/`seed` override the fixed test pins
/// ([`crate::builtins::TEST_NOW`] / [`TEST_SEED`]) that make {{@now}}/{{@uuid}}
/// mogs deterministic in their goldens.
pub fn run_mog_test(mog: &Path, now: Option<&str>, seed: Option<u64>) -> TestOutcome {
    match run_inner(mog, now, seed) {
        Ok(None) => TestOutcome {
            mog: mog.to_path_buf(),
            pass: true,
            diff: None,
            message: None,
        },
        Ok(Some(diff)) => TestOutcome {
            mog: mog.to_path_buf(),
            pass: false,
            diff: Some(diff),
            message: Some("output does not match TestExpectedOutput".to_string()),
        },
        Err(e) => TestOutcome {
            mog: mog.to_path_buf(),
            pass: false,
            diff: None,
            message: Some(format!("{e:#}")),
        },
    }
}

/// Derive the enclosing library root for `mog` in the flat store layout
/// (`<root>/<name>/<name>.mog`): the mog directory's parent (the `.mog`'s
/// grandparent). Returns `None` only when no such ancestor exists.
///
/// With one directory per mog, a composed mog's `run_mog` fragment is no
/// longer a filesystem sibling; it is another mog elsewhere in the same
/// library. Providing this root lets [`crate::library::resolve_script`] find it
/// by name, matching how the same mog resolves when run from an installed
/// library.
fn enclosing_library_root(mog: &Path) -> Option<PathBuf> {
    // Work from an absolute path. With a relative input (e.g. `foo/foo.mog`, the
    // natural `mog --test foo` form) the mog dir's parent would be an empty
    // path, yielding an empty library root and breaking `run_mog` /
    // `for_each_block` fragment resolution. Joining onto the current dir first
    // gives the mog directory a real parent.
    let abs;
    let start = if mog.is_absolute() {
        mog
    } else {
        abs = std::env::current_dir().ok()?.join(mog);
        abs.as_path()
    };
    // Flat layout, one directory per mog: `<root>/<name>/<name>.mog`, so the
    // enclosing library root is the mog directory's parent (the .mog's
    // grandparent). Fragments then resolve by a recursive search of that root.
    start
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
}

/// Returns `Ok(None)` on a match, `Ok(Some(diff))` on a mismatch, `Err` when a
/// fixture is missing or the script fails to parse/run.
fn run_inner(mog: &Path, now: Option<&str>, seed: Option<u64>) -> Result<Option<String>> {
    let input_path = sibling(mog, "TestInput")
        .with_context(|| format!("{}: no tests/input.<ext> fixture", mog.display()))?;
    let expected_path = sibling(mog, "TestExpectedOutput")
        .with_context(|| format!("{}: no tests/expected.<ext> fixture", mog.display()))?;
    let script =
        fs::read_to_string(mog).with_context(|| format!("failed to read '{}'", mog.display()))?;
    let input = fs::read_to_string(&input_path)
        .with_context(|| format!("failed to read '{}'", input_path.display()))?;
    let expected = fs::read_to_string(&expected_path)
        .with_context(|| format!("failed to read '{}'", expected_path.display()))?;
    // Pin the clock/RNG to the fixed test reference (overridable) so mogs that
    // read {{@now}}/{{@year}}/{{@uuid}} produce a stable golden.
    let mut defines = std::collections::BTreeMap::new();
    crate::builtins::inject_builtins(
        &mut defines,
        Some(now.unwrap_or(crate::builtins::TEST_NOW)),
        Some(seed.unwrap_or(crate::builtins::TEST_SEED)),
    )?;
    let parsed = parse_mog_with_defines(&script, &defines)
        .with_context(|| format!("parse '{}'", mog.display()))?;
    // Pin the same effective RNG seed the builtins used, so the per-occurrence
    // `$uuid` replacement token (resolved at execution time, not load time) is
    // reproducible in a golden too.
    let pin = Some(seed.unwrap_or(crate::builtins::TEST_SEED));
    // A mog may declare `sources` (confined to its own directory) for
    // source-aware actions (fill_from_list, the two-input compare family); load
    // them relative to the .mog so a golden exercises the real threaded path.
    let sources = if parsed.sources.is_empty() {
        std::collections::BTreeMap::new()
    } else {
        crate::sources::load_mog_sources(&parsed.sources, mog.parent())
            .with_context(|| format!("load sources for '{}'", mog.display()))?
    };
    // Composed mogs reference their `run_mog` fragments by name; with one
    // directory per mog those fragments are no longer filesystem siblings, so
    // resolution falls back to the enclosing library (factory/user/community).
    let lib_root = enclosing_library_root(mog);
    let out = execute_with_library_sources_observed(
        &parsed,
        &input,
        mog.parent(),
        lib_root.as_deref(),
        std::sync::Arc::new(sources),
        None,
        pin,
    )
    .with_context(|| format!("run '{}'", mog.display()))?;
    let (got, want) = (lf(&out), lf(&expected));
    if got == want {
        Ok(None)
    } else {
        // Golden is the "before" side, so `-` lines are expected/missing and
        // `+` lines are what the run actually produced.
        Ok(Some(unified_diff(&mog.display().to_string(), &want, &got)))
    }
}

/// Build a plain unified diff (before -> after) labelled with `name`.
pub fn unified_diff(name: &str, before: &str, after: &str) -> String {
    let diff = TextDiff::from_lines(before, after);
    let mut ud = diff.unified_diff();
    ud.header(&format!("a/{name}"), &format!("b/{name}"));
    ud.to_string()
}
