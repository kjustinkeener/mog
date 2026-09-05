//! Tauri commands: the studio's thin bridge to the `mog` CLI. Each command
//! serializes the current pipeline to a temp `.mog`, runs `mog` as a subprocess,
//! and returns its output. The engine is never linked as a library.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde_json::Value;

/// Locate the `mog` binary. Resolution order:
///   1. `MOG_BIN` (explicit override / dev),
///   2. the bundled sidecar next to the app executable (`mog.exe`),
///   3. the documented dev install `%LOCALAPPDATA%\mog\mog.exe`,
///   4. `mog` on `PATH`.
fn mog_bin() -> OsString {
    if let Some(p) = std::env::var_os("MOG_BIN") {
        if !p.is_empty() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join(if cfg!(windows) { "mog.exe" } else { "mog" });
            if cand.is_file() {
                return cand.into_os_string();
            }
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let cand = PathBuf::from(local).join("mog").join("mog.exe");
        if cand.is_file() {
            return cand.into_os_string();
        }
    }
    OsString::from("mog")
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique path in the OS temp dir for a short-lived file.
fn unique_temp(ext: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("mogstudio-{pid}-{nanos}-{n}.{ext}"))
}

/// A fresh, unique directory in the OS temp dir (for `mog --test`, which needs a
/// `.mog` beside its `TestInput` / `TestExpectedOutput` siblings). Caller cleans up.
fn unique_temp_dir(tag: &str) -> Result<PathBuf, String> {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mogstudio-{tag}-{pid}-{nanos}-{n}"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn stderr_message(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes).trim().to_string();
    if s.is_empty() {
        "mog exited with an error".to_string()
    } else {
        s
    }
}

/// Truncate a pipeline to its first `stop_after + 1` steps (by array index),
/// for "preview up to this step". Returns the pipeline unchanged if `None`.
fn maybe_truncate(pipeline: &str, stop_after: Option<usize>) -> Result<String, String> {
    let Some(n) = stop_after else {
        return Ok(pipeline.to_string());
    };
    let mut v: Value =
        serde_json::from_str(pipeline).map_err(|e| format!("invalid pipeline JSON: {e}"))?;
    if let Some(steps) = v.get_mut("steps").and_then(|s| s.as_array_mut()) {
        steps.truncate(n.saturating_add(1));
    }
    serde_json::to_string(&v).map_err(|e| e.to_string())
}

/// The action descriptor table (`mog --list-actions --json`), returned verbatim.
#[tauri::command]
pub fn list_actions() -> Result<Value, String> {
    let out = Command::new(mog_bin())
        .arg("--list-actions")
        .arg("--json")
        .output()
        .map_err(|e| format!("failed to launch mog ({e}). Set MOG_BIN or install the sidecar."))?;
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("could not parse --list-actions JSON: {e}"))
}

/// The ordered, labeled category registry (`mog --list-categories --json`), returned
/// verbatim: `[{ "name", "label", "rank" }, ...]`. The studio uses it to order and
/// label the palette. Older engines lack this flag, so callers must degrade
/// gracefully (the palette falls back to descriptor order + title-cased names).
#[tauri::command]
pub fn list_categories() -> Result<Value, String> {
    let out = Command::new(mog_bin())
        .arg("--list-categories")
        .arg("--json")
        .output()
        .map_err(|e| format!("failed to launch mog ({e}). Set MOG_BIN or install the sidecar."))?;
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("could not parse --list-categories JSON: {e}"))
}

/// The store root the engine uses: `MOG_HOME` > `%APPDATA%\mog` (Windows) >
/// `$HOME/.mog`. Mirrors `mog::library::default_root`; the engine is shelled out
/// to, not linked, so it is reimplemented here.
fn store_root() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("MOG_HOME") {
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("mog"))
    } else {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".mog"))
    }
}

/// The default full path to save `filename` into, so the Save As dialog lands in a
/// sensible place. User-authored recipes live in the store's user dir,
/// `<root>/mogs/user`, which the engine ensures exists but never scans, manages,
/// resolves from, or overwrites. This is only the save-dialog default. The dir is
/// best-effort created so the dialog opens in it. Returns null if no store root
/// can be resolved (caller falls back to the bare name).
#[tauri::command]
pub fn user_recipe_path(filename: String) -> Option<String> {
    let dir = store_root()?.join("mogs").join("user");
    let _ = std::fs::create_dir_all(&dir);
    Some(dir.join(filename).to_string_lossy().into_owned())
}

#[derive(Serialize)]
pub struct TransformResult {
    /// The transformed text, or (when `is_diff`) a unified diff.
    pub output: String,
    pub is_diff: bool,
    pub changed: bool,
}

/// Run the pipeline over `input`. With `diff`, returns a unified diff instead of
/// the transformed text. With `stop_after`, runs only the first N+1 steps.
#[tauri::command]
pub fn run_transform(
    pipeline: String,
    input: String,
    diff: bool,
    stop_after: Option<usize>,
) -> Result<TransformResult, String> {
    let pipeline = maybe_truncate(&pipeline, stop_after)?;

    let mog_path = unique_temp("mog");
    let in_path = unique_temp("txt");
    std::fs::write(&mog_path, pipeline.as_bytes()).map_err(|e| e.to_string())?;
    std::fs::write(&in_path, input.as_bytes()).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(mog_bin());
    cmd.arg("-m").arg(&mog_path).arg(&in_path);
    if diff {
        cmd.arg("--diff");
    }
    let result = cmd.output();

    let _ = std::fs::remove_file(&mog_path);
    let _ = std::fs::remove_file(&in_path);

    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    let output = String::from_utf8_lossy(&out.stdout).into_owned();
    let changed = if diff {
        !output.trim().is_empty()
    } else {
        output != input
    };
    Ok(TransformResult {
        output,
        is_diff: diff,
        changed,
    })
}

#[derive(Serialize)]
pub struct BatchResult {
    /// Per-file summary lines plus the final total, as printed by the CLI.
    pub lines: Vec<String>,
    /// Lines emitted on stderr (errors).
    pub errors: Vec<String>,
    pub ok: bool,
}

/// Batch-run the pipeline over files/globs, mirroring the CLI flags. Output is
/// returned as summary lines (non-streaming for v1).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn run_batch(
    pipeline: String,
    inputs: Vec<String>,
    in_place: bool,
    out_dir: Option<String>,
    dry_run: bool,
    output_encoding: String,
    backup: Option<String>,
    overwrite: bool,
    excludes: Vec<String>,
    jobs: Option<usize>,
) -> Result<BatchResult, String> {
    if inputs.is_empty() {
        return Err("no inputs given".to_string());
    }
    let mog_path = unique_temp("mog");
    std::fs::write(&mog_path, pipeline.as_bytes()).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(mog_bin());
    cmd.arg("-m").arg(&mog_path);
    for i in &inputs {
        cmd.arg(i);
    }
    if in_place {
        cmd.arg("--in-place");
    }
    if let Some(dir) = &out_dir {
        cmd.arg("--out-dir").arg(dir);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }
    if !output_encoding.is_empty() && output_encoding != "preserve" {
        cmd.arg("--output-encoding").arg(&output_encoding);
    }
    if let Some(sfx) = &backup {
        cmd.arg("--backup").arg(sfx);
    }
    if overwrite {
        cmd.arg("--overwrite");
    }
    for e in &excludes {
        cmd.arg("--exclude").arg(e);
    }
    if let Some(j) = jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }
    let result = cmd.output();
    let _ = std::fs::remove_file(&mog_path);

    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    let lines: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let errors: Vec<String> = String::from_utf8_lossy(&out.stderr)
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .collect();
    Ok(BatchResult {
        lines,
        errors,
        ok: out.status.success(),
    })
}

/// Path of the studio's optional debug log: `MOG_STUDIO_LOG` if set, else
/// `mog-studio.log` beside the working directory. Written only when the user turns
/// file logging on in Settings.
fn log_file_path() -> PathBuf {
    std::env::var_os("MOG_STUDIO_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join("mog-studio.log")
        })
}

/// Append one line to the debug log (see `log_file_path`). Creates the file if
/// needed. Called by the frontend logger only while file logging is enabled.
#[tauri::command]
pub fn append_log(line: String) -> Result<(), String> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path())
        .map_err(|e| e.to_string())?;
    writeln!(f, "{line}").map_err(|e| e.to_string())?;
    Ok(())
}

/// The absolute path the debug log writes to, for display in Settings.
#[tauri::command]
pub fn log_path() -> String {
    log_file_path().to_string_lossy().into_owned()
}

/// Ranked recipe search for the library typeahead. An empty query lists the whole
/// catalog (featured first). Returns the CLI's `{count, results:[...]}` verbatim.
#[tauri::command]
pub fn market_search(query: String) -> Result<Value, String> {
    let t = std::time::Instant::now();
    let verb = if query.trim().is_empty() {
        "list"
    } else {
        "search"
    };
    let mut cmd = Command::new(mog_bin());
    if query.trim().is_empty() {
        cmd.arg("market").arg("list").arg("--json");
    } else {
        cmd.arg("market").arg("search").arg(&query).arg("--json");
    }
    let out = cmd
        .output()
        .map_err(|e| format!("failed to launch mog ({e}). Set MOG_BIN or install the sidecar."))?;
    // Temporary: Marketplace initial-load profiling (goes to the app's stderr).
    eprintln!(
        "[mkt-rs] market_search({verb}) subprocess {}ms, {} KiB out",
        t.elapsed().as_millis(),
        out.stdout.len() / 1024
    );
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("could not parse market JSON: {e}"))
}

/// One recipe's detail, including its `.mog` `content` (as a JSON string) and its
/// resolvable name. Returns the CLI's `market show --json` object verbatim.
#[tauri::command]
pub fn market_show(name: String) -> Result<Value, String> {
    let t = std::time::Instant::now();
    let out = Command::new(mog_bin())
        .arg("market")
        .arg("show")
        .arg(&name)
        .arg("--json")
        .output()
        .map_err(|e| format!("failed to launch mog ({e})."))?;
    // Temporary: Marketplace profiling (goes to the app's stderr).
    eprintln!(
        "[mkt-rs] market_show({name}) subprocess {}ms, {} KiB out",
        t.elapsed().as_millis(),
        out.stdout.len() / 1024
    );
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    let mut v: Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("could not parse market show JSON: {e}"))?;
    // The engine reports the fixture's TestInput as a file path; read its content so
    // the studio can drop the recipe's sample input into the preview on open.
    let input_path = v
        .get("fixture")
        .and_then(|f| f.get("input"))
        .and_then(|p| p.as_str())
        .map(|s| s.to_string());
    let content = input_path.and_then(|p| std::fs::read_to_string(p).ok());
    if let Value::Object(map) = &mut v {
        map.insert(
            "fixture_input".to_string(),
            content.map(Value::String).unwrap_or(Value::Null),
        );
    }
    Ok(v)
}

/// Bring the machine current with the registry: `mog update`. Syncs marketplace
/// recipes by content hash and, unless `recipes_only`, self-replaces the engine
/// binary when a newer signed build exists. `check` reports without writing. The
/// binary this Studio shells out to updates on disk immediately, but a running
/// sidecar keeps its loaded image until the app restarts. Returns the CLI's
/// `{recipes, engine, ...}` JSON verbatim.
#[tauri::command]
pub fn mog_update(check: bool, recipes_only: bool, prune: bool) -> Result<Value, String> {
    let mut cmd = Command::new(mog_bin());
    cmd.arg("update");
    if check {
        cmd.arg("--check");
    }
    if recipes_only {
        cmd.arg("--no-engine");
    }
    if prune {
        cmd.arg("--prune");
    }
    cmd.arg("--json");
    let out = cmd
        .output()
        .map_err(|e| format!("failed to launch mog ({e}). Set MOG_BIN or install the sidecar."))?;
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("could not parse update JSON: {e}"))
}

// -- Convergence panels (5c): test / flag-check / batch-impact -----------------

#[derive(Serialize)]
pub struct TestOutcome {
    pub pass: bool,
    /// Unified diff (expected -> actual) on a mismatch, else null.
    pub diff: Option<String>,
    pub message: Option<String>,
}

/// Run the current recipe as a `mog --test` case: the pipeline plus `input`
/// (TestInput) and `expected` (TestExpectedOutput) are written as a sibling triple
/// in a throwaway temp dir, and `mog --test <dir> --json` compares them. The clock
/// and RNG are pinned by `--test`, so `{{@now}}`/`{{@uuid}}` recipes are stable.
#[tauri::command]
pub fn run_recipe_test(
    pipeline: String,
    input: String,
    expected: String,
) -> Result<TestOutcome, String> {
    let dir = unique_temp_dir("test")?;
    let mog = dir.join("case.mog");
    let tin = dir.join("case.TestInput.txt");
    let tout = dir.join("case.TestExpectedOutput.txt");
    let written = std::fs::write(&mog, pipeline.as_bytes())
        .and_then(|_| std::fs::write(&tin, input.as_bytes()))
        .and_then(|_| std::fs::write(&tout, expected.as_bytes()));
    if let Err(e) = written {
        let _ = std::fs::remove_dir_all(&dir);
        return Err(e.to_string());
    }
    let result = Command::new(mog_bin())
        .arg("--test")
        .arg(&dir)
        .arg("--json")
        .output();
    let _ = std::fs::remove_dir_all(&dir);

    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    // `--test` exits 1 when a test FAILS -- a valid outcome, not a launch error. The
    // JSON report is still on stdout; only a non-JSON stdout is a real failure.
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| stderr_message(&out.stderr))?;
    let first = v
        .get("tests")
        .and_then(|t| t.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "mog --test returned no result".to_string())?;
    Ok(TestOutcome {
        pass: first.get("pass").and_then(|b| b.as_bool()).unwrap_or(false),
        diff: first
            .get("diff")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        message: first
            .get("message")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
    })
}

#[derive(Serialize)]
pub struct FlagRow {
    pub tag: String,
    pub line: u64,
    pub message: String,
}

#[derive(Serialize)]
pub struct FlagCheck {
    /// True when the input would not change AND carries no flag (the `--check` gate).
    pub clean: bool,
    /// True when the pipeline would change the input.
    pub would_change: bool,
    pub flags: Vec<FlagRow>,
    /// (tag, count) pairs.
    pub counts: Vec<(String, u64)>,
    /// A per-file processing error, if any.
    pub error: Option<String>,
}

/// Run the recipe over the current input under `--json --check`, returning the flag
/// records (`detect_secrets` / `detect_pii` / `flag_matching`) and the gate verdict.
#[tauri::command]
pub fn flag_check(pipeline: String, input: String) -> Result<FlagCheck, String> {
    let mog_path = unique_temp("mog");
    let in_path = unique_temp("txt");
    std::fs::write(&mog_path, pipeline.as_bytes()).map_err(|e| e.to_string())?;
    std::fs::write(&in_path, input.as_bytes()).map_err(|e| e.to_string())?;

    let result = Command::new(mog_bin())
        .arg("-m")
        .arg(&mog_path)
        .arg(&in_path)
        .arg("--json")
        .arg("--check")
        .output();
    let _ = std::fs::remove_file(&mog_path);
    let _ = std::fs::remove_file(&in_path);

    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    // `--check` exits 1 (would change / flagged) or 2 (error); the report is still on
    // stdout, so parse regardless of the exit code.
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| stderr_message(&out.stderr))?;
    let flags = v
        .get("flags")
        .and_then(|f| f.as_array())
        .map(|a| {
            a.iter()
                .map(|r| FlagRow {
                    tag: r
                        .get("tag")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    line: r.get("line").and_then(|n| n.as_u64()).unwrap_or(0),
                    message: r
                        .get("message")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let counts = v
        .get("flag_counts")
        .and_then(|c| c.as_object())
        .map(|o| {
            o.iter()
                .map(|(k, val)| (k.clone(), val.as_u64().unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    let clean = v
        .get("check")
        .and_then(|c| c.get("clean"))
        .and_then(|b| b.as_bool())
        .unwrap_or(true);
    let would_change = v
        .get("summary")
        .and_then(|s| s.get("changed"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0)
        > 0;
    let error = v.get("files").and_then(|f| f.as_array()).and_then(|a| {
        a.iter().find_map(|f| {
            f.get("error")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string())
        })
    });
    Ok(FlagCheck {
        clean,
        would_change,
        flags,
        counts,
        error,
    })
}

#[derive(Serialize)]
pub struct FileImpact {
    pub path: String,
    pub changed: bool,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub diff: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct BatchImpact {
    pub total: u64,
    pub changed: u64,
    pub errors: u64,
    pub files: Vec<FileImpact>,
}

/// Dry-run the recipe over `inputs` and return the per-file impact (changed flag,
/// +/- line counts, and a unified diff) plus the totals. Writes nothing.
#[tauri::command]
pub fn run_batch_report(
    pipeline: String,
    inputs: Vec<String>,
    excludes: Vec<String>,
    jobs: Option<usize>,
) -> Result<BatchImpact, String> {
    if inputs.is_empty() {
        return Err("no inputs given".to_string());
    }
    let mog_path = unique_temp("mog");
    std::fs::write(&mog_path, pipeline.as_bytes()).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(mog_bin());
    cmd.arg("-m").arg(&mog_path);
    for i in &inputs {
        cmd.arg(i);
    }
    cmd.arg("--json").arg("--dry-run").arg("--diff");
    for e in &excludes {
        cmd.arg("--exclude").arg(e);
    }
    if let Some(j) = jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }
    let result = cmd.output();
    let _ = std::fs::remove_file(&mog_path);

    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    let v: Value = serde_json::from_slice(&out.stdout).map_err(|_| stderr_message(&out.stderr))?;
    let files: Vec<FileImpact> = v
        .get("files")
        .and_then(|f| f.as_array())
        .map(|a| {
            a.iter()
                .map(|f| FileImpact {
                    path: f
                        .get("path")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    changed: f.get("changed").and_then(|b| b.as_bool()).unwrap_or(false),
                    lines_added: f.get("lines_added").and_then(|n| n.as_u64()).unwrap_or(0),
                    lines_removed: f.get("lines_removed").and_then(|n| n.as_u64()).unwrap_or(0),
                    diff: f
                        .get("diff")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string()),
                    error: f
                        .get("error")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string()),
                })
                .collect()
        })
        .unwrap_or_default();
    let total = v
        .get("summary")
        .and_then(|s| s.get("files"))
        .and_then(|n| n.as_u64())
        .unwrap_or(files.len() as u64);
    let changed = v
        .get("summary")
        .and_then(|s| s.get("changed"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let errors = v
        .get("summary")
        .and_then(|s| s.get("errors"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    Ok(BatchImpact {
        total,
        changed,
        errors,
        files,
    })
}

/// The unified diff for one file on disk under the current recipe (`mog -m <mog>
/// <path> --diff`). Used by the impact view to show a file's diff on demand,
/// because the `--json` report carries per-file counts but not the diffs.
#[tauri::command]
pub fn diff_file(pipeline: String, path: String) -> Result<String, String> {
    let mog_path = unique_temp("mog");
    std::fs::write(&mog_path, pipeline.as_bytes()).map_err(|e| e.to_string())?;
    let result = Command::new(mog_bin())
        .arg("-m")
        .arg(&mog_path)
        .arg(&path)
        .arg("--diff")
        .output();
    let _ = std::fs::remove_file(&mog_path);
    let out = result.map_err(|e| format!("failed to launch mog ({e})."))?;
    if !out.status.success() {
        return Err(stderr_message(&out.stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
