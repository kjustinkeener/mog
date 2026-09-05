//! The nine Mog MCP tool handlers.
//!
//! Ported 1:1 from the retired Node server (`mcp/src/tools.ts`). Each handler
//! reproduces the same argument shaping and result envelope, but instead of a
//! Node process shelling out to `mog.exe`, the server IS `mog.exe`: every call
//! runs the engine via [`run_engine`], which re-invokes the current binary with
//! the exact CLI args the Node adapter used. That keeps byte-for-byte parity
//! with the old server while collapsing the whole thing into one Rust binary and
//! no Node runtime. The single subprocess boundary ([`run_engine`]) is also the
//! one place a future in-process dispatch would replace.

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{json, Value};

/// The outcome of one engine invocation.
struct ExecResult {
    code: i32,
    stdout: String,
    stderr: String,
    /// Set when the process could not be spawned at all (mirrors the Node
    /// `spawnError`): a hard failure distinct from a nonzero exit.
    spawn_error: Option<String>,
}

/// Re-invoke the running `mog` binary with `args`, optionally feeding `stdin`,
/// and capture its output. This replaces the Node server's `execMog`: the server
/// self-locates via `current_exe`, so there is no `MOG_BIN`. The child inherits
/// this process's environment (so `MOG_HOME` / `MOG_REPORT` flow through).
fn run_engine(args: &[String], stdin: Option<&str>) -> ExecResult {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return ExecResult {
                code: -1,
                stdout: String::new(),
                stderr: String::new(),
                spawn_error: Some(format!("could not locate the running mog binary: {e}")),
            }
        }
    };
    let mut child = match Command::new(&exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return ExecResult {
                code: -1,
                stdout: String::new(),
                stderr: String::new(),
                spawn_error: Some(e.to_string()),
            }
        }
    };

    // Write stdin from a thread so a large stdin can't deadlock against the
    // child filling its stdout pipe. When no stdin is supplied we still close
    // the handle (drop it) so a filter-mode engine sees EOF instead of hanging.
    if let Some(mut sink) = child.stdin.take() {
        let data = stdin.unwrap_or("").as_bytes().to_vec();
        std::thread::spawn(move || {
            let _ = sink.write_all(&data);
            // sink drops here -> EOF for the child.
        });
    }

    match child.wait_with_output() {
        Ok(out) => ExecResult {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            spawn_error: None,
        },
        Err(e) => ExecResult {
            code: -1,
            stdout: String::new(),
            stderr: String::new(),
            spawn_error: Some(e.to_string()),
        },
    }
}

/// Wrap a JSON value as a single text-content tool result.
fn text_result(value: Value, is_error: bool) -> Value {
    json!({
        "isError": is_error,
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }],
    })
}

/// Parse engine stdout as JSON; `None` if it is not valid JSON.
fn try_parse(s: &str) -> Option<Value> {
    serde_json::from_str(s).ok()
}

/// The engine's structured-error envelope: `{ "error": true, ... }`.
fn is_mog_error(v: &Value) -> bool {
    v.get("error").and_then(Value::as_bool).unwrap_or(false)
}

/// A uniform error result for a spawn failure (mirrors Node's `execFailure`).
fn spawn_failure(r: &ExecResult, what: &str) -> Option<Value> {
    r.spawn_error.as_ref().map(|e| {
        text_result(
            json!({
                "error": true,
                "message": format!("failed to run mog ({what}): {e}"),
            }),
            true,
        )
    })
}

// ===========================================================================
// A temp-file scratch for the `.mog` scripts that several tools pass as `-m`.
// ===========================================================================

/// Write `contents` to a uniquely named temp `.mog`, run `f` with its path, then
/// delete it. Mirrors the Node `withTempMog`.
fn with_temp_mog<F: FnOnce(&str) -> Value>(contents: &str, f: F) -> Value {
    let mut path = std::env::temp_dir();
    // A collision-resistant name without needing a rng crate: pid + a process-
    // local counter. Determinism is irrelevant here (temp scratch only).
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!("mog-mcp-{}-{}.mog", std::process::id(), n));
    if let Err(e) = std::fs::write(&path, contents) {
        return text_result(
            json!({ "error": true, "message": format!("could not write temp .mog: {e}") }),
            true,
        );
    }
    let result = f(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    result
}

/// Resolve the script for a run: either inline `mog` JSON (written to a temp
/// file) or an installed recipe named by `recipe`, which the engine resolves
/// from the library itself. Exactly one of the two must be given.
fn with_script<F: FnOnce(&str) -> Value>(input: &Value, f: F) -> Value {
    let mog = input.get("mog").and_then(Value::as_str).unwrap_or("");
    let recipe = input
        .get("recipe")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    match (mog.trim().is_empty(), recipe) {
        (true, Some(name)) => f(name),
        (false, None) => with_temp_mog(mog, f),
        (false, Some(_)) => text_result(
            json!({ "error": true, "message": "pass either 'mog' (inline script) or 'recipe' (an installed recipe name), not both" }),
            true,
        ),
        (true, None) => text_result(
            json!({ "error": true, "message": "no script: pass 'mog' (inline script JSON) or 'recipe' (an installed recipe name; search mog_market to find one)" }),
            true,
        ),
    }
}

// ===========================================================================
// Run-dashboard (opt-out) plumbing: report resolution + resource_link.
// ===========================================================================

/// Decide whether a run emits an HTML dashboard, per the precedence: per-call
/// `report` arg > persisted config `reports` > `MOG_REPORT` env > default true.
fn resolve_report(per_call: Option<bool>) -> bool {
    if let Some(v) = per_call {
        return v;
    }
    let r = run_engine(&["config".into(), "get".into(), "--json".into()], None);
    if r.spawn_error.is_none() && r.code == 0 {
        if let Some(cfg) = try_parse(&r.stdout) {
            if let Some(b) = cfg.get("reports").and_then(Value::as_bool) {
                return b;
            }
        }
    }
    if let Ok(env) = std::env::var("MOG_REPORT") {
        match env.trim().to_lowercase().as_str() {
            "off" | "0" | "false" | "no" => return false,
            "on" | "1" | "true" | "yes" => return true,
            _ => {}
        }
    }
    true
}

/// Pull the dashboard path from the engine's `report: wrote <PATH>` stderr line
/// (last one wins).
fn extract_report_path(stderr: &str) -> Option<String> {
    let mut found = None;
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("report: wrote ") {
            found = Some(rest.trim().to_string());
        }
    }
    found
}

/// A terse one-line run summary for the resource_link description.
fn run_summary_line(parsed: &Value) -> String {
    let mode = parsed.get("mode").and_then(Value::as_str).unwrap_or("run");
    if let Some(s) = parsed.get("summary") {
        if s.is_object() {
            let changed = s.get("changed").and_then(Value::as_i64).unwrap_or(0);
            let files = s.get("files").and_then(Value::as_i64).unwrap_or(0);
            let mut parts = vec![format!("{changed}/{files} files changed")];
            let added = s.get("lines_added").and_then(Value::as_i64);
            let removed = s.get("lines_removed").and_then(Value::as_i64);
            if added.is_some() || removed.is_some() {
                parts.push(format!("+{} -{}", added.unwrap_or(0), removed.unwrap_or(0)));
            }
            if let Some(errors) = s.get("errors").and_then(Value::as_i64) {
                if errors != 0 {
                    parts.push(format!("{errors} errors"));
                }
            }
            return format!("{mode}: {}", parts.join(", "));
        }
    }
    mode.to_string()
}

/// Convert a filesystem path to a `file://` URI for the resource_link.
fn path_to_file_uri(path: &str) -> String {
    // A pragmatic file URI: absolute path, backslashes to forward slashes, and a
    // leading slash for a Windows drive path (C:\ -> file:///C:/).
    let norm = path.replace('\\', "/");
    if norm.starts_with('/') {
        format!("file://{norm}")
    } else {
        format!("file:///{norm}")
    }
}

/// Build a run result: the full run JSON, plus a visible path line and a
/// `resource_link` when a dashboard was produced.
fn with_report_link(parsed: &Value, report_path: Option<String>, is_error: bool) -> Value {
    let mut content = vec![json!({
        "type": "text",
        "text": serde_json::to_string_pretty(parsed).unwrap_or_default(),
    })];
    if let Some(p) = report_path {
        let summary = run_summary_line(parsed);
        content.push(json!({ "type": "text", "text": format!("Run dashboard: {p}") }));
        content.push(json!({
            "type": "resource_link",
            "uri": path_to_file_uri(&p),
            "name": "Mog run dashboard",
            "description": summary,
            "mimeType": "text/html",
        }));
    }
    json!({ "isError": is_error, "content": content })
}

// ===========================================================================
// Ranking: tokenized, synonym-aware search (ported from tools.ts).
// ===========================================================================

const SYNONYM_GROUPS: &[&[&str]] = &[
    &[
        "whitespace",
        "space",
        "spaces",
        "tab",
        "tabs",
        "indent",
        "indentation",
        "blank",
        "empty",
    ],
    &["trim", "strip", "clean", "cleanup", "tidy"],
    &[
        "collapse", "squeeze", "reduce", "dedupe", "dedup", "compact",
    ],
    &[
        "eol",
        "newline",
        "newlines",
        "linebreak",
        "crlf",
        "lf",
        "ending",
        "endings",
    ],
    &[
        "case",
        "upper",
        "uppercase",
        "lower",
        "lowercase",
        "capitalize",
        "title",
    ],
    &["prefix", "suffix", "affix", "prepend", "append"],
    &["encode", "decode", "escape", "unescape", "base64"],
    &["line", "lines", "row", "rows"],
    &["sort", "order", "alphabetize"],
    &["replace", "substitute", "swap", "regex"],
    &[
        "redact", "mask", "sanitize", "sanitise", "scrub", "censor", "secret", "secrets", "pii",
    ],
    &[
        "ansi",
        "color",
        "colors",
        "colour",
        "colours",
        "terminal",
        "decolorize",
        "vt100",
    ],
    &["log", "logs", "output", "console", "stdout"],
];

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Expand query tokens with their synonyms: literal tokens weigh 2, synonyms
/// pulled in by a group weigh 1.
fn expand_query(tokens: &[String]) -> Vec<(String, i64)> {
    let mut terms: Vec<(String, i64)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for t in tokens {
        if seen.insert(t.clone()) {
            terms.push((t.clone(), 2));
        }
    }
    for t in tokens {
        for group in SYNONYM_GROUPS {
            if group.contains(&t.as_str()) {
                for g in *group {
                    if seen.insert(g.to_string()) {
                        terms.push((g.to_string(), 1));
                    }
                }
            }
        }
    }
    terms
}

/// Rank a list of objects. `strong_fields`/`weak_fields` name the string (or
/// string-array) keys that form the strong/weak haystacks. A strong hit counts
/// double a weak hit; term weight (literal 2 / synonym 1) multiplies it. Ties
/// keep the original order. Mirrors rankActions / rankScripts.
fn rank(list: &[Value], query: &str, strong_fields: &[&str], weak_fields: &[&str]) -> Vec<Value> {
    let collect = |d: &Value, fields: &[&str]| -> String {
        let mut buf = String::new();
        for f in fields {
            match d.get(*f) {
                Some(Value::String(s)) => {
                    buf.push_str(s);
                    buf.push('\n');
                }
                Some(Value::Array(a)) => {
                    for item in a {
                        if let Some(s) = item.as_str() {
                            buf.push_str(s);
                            buf.push('\n');
                        }
                    }
                }
                _ => {}
            }
        }
        buf.to_lowercase()
    };

    let tokens = tokenize(query);
    if tokens.is_empty() {
        // Degenerate query (all punctuation): raw substring over all fields.
        let q = query.to_lowercase();
        let all: Vec<&str> = strong_fields.iter().chain(weak_fields).copied().collect();
        return list
            .iter()
            .filter(|d| collect(d, &all).contains(&q))
            .cloned()
            .collect();
    }
    let terms = expand_query(&tokens);
    let mut scored: Vec<(usize, i64, &Value)> = list
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            let strong = collect(d, strong_fields);
            let weak = collect(d, weak_fields);
            let mut score = 0i64;
            for (term, weight) in &terms {
                let in_strong = strong.contains(term.as_str());
                let in_weak = !in_strong && weak.contains(term.as_str());
                if !in_strong && !in_weak {
                    continue;
                }
                score += weight * if in_strong { 2 } else { 1 };
            }
            if score > 0 {
                Some((i, score, d))
            } else {
                None
            }
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().map(|(_, _, d)| d.clone()).collect()
}

// ===========================================================================
// The nine tools.
// ===========================================================================

/// `mog_actions`: progressive-disclosure over the action vocabulary.
pub fn mog_actions(input: &Value) -> Value {
    let name = input.get("name").and_then(Value::as_str);
    let query = input.get("query").and_then(Value::as_str);
    let tier = input.get("tier").and_then(Value::as_str);
    let category = input.get("category").and_then(Value::as_str);

    // A specific action -> its full descriptor.
    if let Some(name) = name {
        let r = run_engine(&["--describe".into(), name.into(), "--json".into()], None);
        if let Some(f) = spawn_failure(&r, "--describe") {
            return f;
        }
        match try_parse(&r.stdout) {
            Some(parsed) => return text_result(parsed.clone(), is_mog_error(&parsed)),
            None => {
                let msg = if !r.stderr.is_empty() {
                    &r.stderr
                } else if !r.stdout.is_empty() {
                    &r.stdout
                } else {
                    "no output"
                };
                return text_result(json!({ "error": true, "message": msg }), true);
            }
        }
    }

    // Otherwise the compact index, filtered in-process.
    let r = run_engine(
        &["--list-actions".into(), "--compact".into(), "--json".into()],
        None,
    );
    if let Some(f) = spawn_failure(&r, "--list-actions --compact") {
        return f;
    }
    let parsed = match try_parse(&r.stdout) {
        Some(Value::Array(a)) => a,
        _ => {
            let msg = if !r.stderr.is_empty() {
                r.stderr.clone()
            } else {
                "unexpected --list-actions output".to_string()
            };
            return text_result(json!({ "error": true, "message": msg }), true);
        }
    };
    let mut list: Vec<Value> = parsed;
    if let Some(q) = query {
        // An explicit search bypasses the core-tier default (searches all tiers).
        list = rank(&list, q, &["name", "aliases"], &["summary", "category"]);
    } else {
        let tier = tier.unwrap_or("core");
        if tier == "core" {
            list.retain(|d| d.get("tier").and_then(Value::as_str) == Some("core"));
        }
    }
    if let Some(cat) = category {
        list.retain(|d| d.get("category").and_then(Value::as_str) == Some(cat));
    }
    text_result(Value::Array(list), false)
}

/// `mog_validate`: parse a `.mog` without touching files.
pub fn mog_validate(input: &Value) -> Value {
    let mog = input.get("mog").and_then(Value::as_str).unwrap_or("");
    with_temp_mog(mog, |file| {
        let r = run_engine(
            &[
                "-m".into(),
                file.into(),
                "--dry-run".into(),
                "--json".into(),
            ],
            Some(""),
        );
        if let Some(f) = spawn_failure(&r, "validate") {
            return f;
        }
        let parsed = try_parse(&r.stdout);
        if let Some(ref p) = parsed {
            if is_mog_error(p) {
                return text_result(p.clone(), false);
            }
        }
        if r.code == 0 && parsed.is_some() {
            return text_result(json!({ "ok": true }), false);
        }
        let msg = if !r.stderr.is_empty() {
            &r.stderr
        } else if !r.stdout.is_empty() {
            &r.stdout
        } else {
            "validation failed"
        };
        text_result(json!({ "error": true, "message": msg }), true)
    })
}

/// `mog_preview`: read-only impact report (dry-run), with an optional dashboard.
pub fn mog_preview(input: &Value) -> Value {
    let inputs: Vec<String> = input
        .get("inputs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let text = input.get("text").and_then(Value::as_str);
    let report_on = resolve_report(input.get("report").and_then(Value::as_bool));

    with_script(input, |file| {
        let mut args: Vec<String> = vec!["-m".into(), file.into()];
        let r = if !inputs.is_empty() {
            args.extend(inputs.iter().cloned());
            args.push("--dry-run".into());
            if report_on {
                args.push("--report".into());
            }
            args.push("--json".into());
            run_engine(&args, None)
        } else {
            args.push("--dry-run".into());
            if report_on {
                args.push("--report".into());
            }
            args.push("--json".into());
            run_engine(&args, Some(text.unwrap_or("")))
        };
        if let Some(f) = spawn_failure(&r, "preview") {
            return f;
        }
        let parsed = match try_parse(&r.stdout) {
            Some(p) => p,
            None => {
                let msg = if !r.stderr.is_empty() {
                    &r.stderr
                } else if !r.stdout.is_empty() {
                    &r.stdout
                } else {
                    "no output"
                };
                return text_result(json!({ "error": true, "message": msg }), true);
            }
        };
        let report_path = if report_on {
            extract_report_path(&r.stderr)
        } else {
            None
        };
        with_report_link(&parsed, report_path, is_mog_error(&parsed))
    })
}

/// `mog_apply`: the mutating tool. Reversible output modes only.
pub fn mog_apply(input: &Value) -> Value {
    let inputs: Vec<String> = input
        .get("inputs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    if inputs.is_empty() {
        return text_result(
            json!({ "error": true, "message": "mog_apply requires at least one input file/glob" }),
            true,
        );
    }

    let out = input.get("output");
    let mut write_args: Vec<String> = Vec::new();
    match out.and_then(|o| o.get("mode")).and_then(Value::as_str) {
        Some("out_dir") => {
            let dir = out.and_then(|o| o.get("dir")).and_then(Value::as_str);
            match dir {
                Some(dir) if !dir.is_empty() => {
                    write_args.push("-o".into());
                    write_args.push(dir.into());
                    if out
                        .and_then(|o| o.get("overwrite"))
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        write_args.push("--overwrite".into());
                    }
                }
                _ => {
                    return text_result(
                        json!({ "error": true, "message": "out_dir mode requires 'dir'" }),
                        true,
                    )
                }
            }
        }
        Some("in_place_backup") => {
            let suffix = out
                .and_then(|o| o.get("backup_suffix"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if suffix.trim().is_empty() {
                return text_result(
                    json!({ "error": true, "message": "in_place_backup mode requires a non-empty 'backup_suffix' (bare in-place is refused)" }),
                    true,
                );
            }
            write_args.push("-i".into());
            write_args.push("--backup".into());
            write_args.push(suffix.into());
        }
        _ => {
            return text_result(
                json!({ "error": true, "message": "output must be {mode:'out_dir',dir} or {mode:'in_place_backup',backup_suffix}" }),
                true,
            )
        }
    }

    let report_on = resolve_report(input.get("report").and_then(Value::as_bool));

    with_script(input, |file| {
        let mut args: Vec<String> = vec!["-m".into(), file.into()];
        args.extend(inputs.iter().cloned());
        args.extend(write_args.iter().cloned());
        if report_on {
            args.push("--report".into());
        }
        args.push("--json".into());
        let r = run_engine(&args, None);
        if let Some(f) = spawn_failure(&r, "apply") {
            return f;
        }
        let parsed = match try_parse(&r.stdout) {
            Some(p) => p,
            None => {
                let msg = if !r.stderr.is_empty() {
                    &r.stderr
                } else if !r.stdout.is_empty() {
                    &r.stdout
                } else {
                    "no output"
                };
                return text_result(json!({ "error": true, "message": msg }), true);
            }
        };
        let failed = r.code != 0 || is_mog_error(&parsed);
        let report_path = if report_on {
            extract_report_path(&r.stderr)
        } else {
            None
        };
        with_report_link(&parsed, report_path, failed)
    })
}

/// `mog_test`: run a `.mog`'s sibling torture fixtures.
pub fn mog_test(input: &Value) -> Value {
    let path = input.get("path").and_then(Value::as_str).unwrap_or("");
    if path.is_empty() {
        return text_result(
            json!({ "error": true, "message": "mog_test requires 'path'" }),
            true,
        );
    }
    let r = run_engine(&["--test".into(), path.into(), "--json".into()], None);
    if let Some(f) = spawn_failure(&r, "test") {
        return f;
    }
    match try_parse(&r.stdout) {
        Some(parsed) => text_result(parsed, r.code != 0),
        None => {
            let msg = if !r.stderr.is_empty() {
                &r.stderr
            } else if !r.stdout.is_empty() {
                &r.stdout
            } else {
                "no output"
            };
            text_result(json!({ "error": true, "message": msg }), true)
        }
    }
}

/// `mog_check`: author-by-example. Run the candidate on inline input/expected
/// pairs and report per-example pass/fail.
pub fn mog_check(input: &Value) -> Value {
    let mog = input.get("mog").and_then(Value::as_str).unwrap_or("");
    let examples: Vec<Value> = input
        .get("examples")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if examples.is_empty() {
        return text_result(
            json!({ "error": true, "message": "mog_check requires at least one example { input, expected }" }),
            true,
        );
    }
    with_temp_mog(mog, |file| {
        let mut results: Vec<Value> = Vec::new();
        for (i, ex) in examples.iter().enumerate() {
            let ex_input = ex.get("input").and_then(Value::as_str).unwrap_or("");
            let expected = ex.get("expected").and_then(Value::as_str).unwrap_or("");
            let r = run_engine(&["-m".into(), file.into()], Some(ex_input));
            if r.spawn_error.is_some() || (r.code != 0 && r.stdout.is_empty()) {
                return text_result(
                    json!({
                        "error": true,
                        "message": if !r.stderr.is_empty() { r.stderr.clone() } else { "mog failed to run the candidate .mog".to_string() },
                        "failing_example": i,
                    }),
                    true,
                );
            }
            let actual = r.stdout;
            let pass = actual == expected;
            if pass {
                results.push(json!({ "index": i, "pass": true }));
            } else {
                results.push(
                    json!({ "index": i, "pass": false, "expected": expected, "actual": actual }),
                );
            }
        }
        let passed = results
            .iter()
            .filter(|x| x.get("pass").and_then(Value::as_bool) == Some(true))
            .count();
        let total = results.len();
        text_result(
            json!({ "passed": passed, "total": total, "all_pass": passed == total, "results": results }),
            passed != total,
        )
    })
}

/// `mog_market`: search / list / show / install recipes.
pub fn mog_market(input: &Value) -> Value {
    let op = input.get("op").and_then(Value::as_str);
    let name = input.get("name").and_then(Value::as_str);
    let query = input.get("query").and_then(Value::as_str);

    let op = op.unwrap_or_else(|| {
        if name.is_some() {
            "show"
        } else if query.is_some() {
            "search"
        } else {
            "list"
        }
    });
    let mut args: Vec<String> = vec!["market".into(), op.into()];
    match op {
        "search" => match query {
            Some(q) => args.push(q.into()),
            None => {
                return text_result(
                    json!({ "error": true, "message": "market search needs a 'query'." }),
                    true,
                )
            }
        },
        "show" | "install" => match name {
            Some(n) => args.push(n.into()),
            None => {
                return text_result(
                    json!({ "error": true, "message": format!("market {op} needs a 'name'.") }),
                    true,
                )
            }
        },
        _ => {}
    }
    args.push("--json".into());

    let r = run_engine(&args, None);
    if let Some(f) = spawn_failure(&r, &format!("market {op}")) {
        return f;
    }
    match try_parse(&r.stdout) {
        Some(parsed) => {
            let is_err = r.code != 0 || is_mog_error(&parsed);
            text_result(parsed, is_err)
        }
        None => {
            let msg = if !r.stderr.is_empty() {
                &r.stderr
            } else if !r.stdout.is_empty() {
                &r.stdout
            } else {
                "no output"
            };
            text_result(json!({ "error": true, "message": msg }), true)
        }
    }
}

/// `mog_update`: sync recipes (content-hash diff) + self-replace the engine.
pub fn mog_update(input: &Value) -> Value {
    let mut args: Vec<String> = vec!["update".into()];
    if input.get("check").and_then(Value::as_bool).unwrap_or(false) {
        args.push("--check".into());
    }
    if input
        .get("recipesOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        args.push("--no-engine".into());
    }
    if input.get("prune").and_then(Value::as_bool).unwrap_or(false) {
        args.push("--prune".into());
    }
    args.push("--json".into());

    let r = run_engine(&args, None);
    if let Some(f) = spawn_failure(&r, "update") {
        return f;
    }
    match try_parse(&r.stdout) {
        Some(parsed) => {
            let is_err = r.code != 0 || is_mog_error(&parsed);
            text_result(parsed, is_err)
        }
        None => {
            let msg = if !r.stderr.is_empty() {
                &r.stderr
            } else if !r.stdout.is_empty() {
                &r.stdout
            } else {
                "no output"
            };
            text_result(json!({ "error": true, "message": msg }), true)
        }
    }
}

/// `mog_config`: read/write persisted preferences.
pub fn mog_config(input: &Value) -> Value {
    let mut set_keys: Vec<String> = Vec::new();

    let do_set = |key: &str, value: &str| -> Result<(), Value> {
        let r = run_engine(
            &[
                "config".into(),
                "set".into(),
                key.into(),
                value.into(),
                "--json".into(),
            ],
            None,
        );
        if let Some(f) = spawn_failure(&r, &format!("config set {key}")) {
            return Err(f);
        }
        if r.code != 0 {
            let msg = if !r.stderr.is_empty() {
                r.stderr.clone()
            } else if !r.stdout.is_empty() {
                r.stdout.clone()
            } else {
                format!("failed to set {key}")
            };
            return Err(text_result(json!({ "error": true, "message": msg }), true));
        }
        Ok(())
    };

    if let Some(reports) = input.get("reports").and_then(Value::as_bool) {
        let v = if reports { "on" } else { "off" };
        if let Err(e) = do_set("reports", v) {
            return e;
        }
        set_keys.push(format!("reports={v}"));
    }
    if let Some(template) = input.get("template").and_then(Value::as_str) {
        let t = template.trim();
        if !t.is_empty() {
            if let Err(e) = do_set("template", t) {
                return e;
            }
            set_keys.push(format!("template={t}"));
        }
    }

    if set_keys.is_empty() {
        let r = run_engine(&["config".into(), "get".into(), "--json".into()], None);
        if let Some(f) = spawn_failure(&r, "config get") {
            return f;
        }
        if r.code != 0 {
            let msg = if !r.stderr.is_empty() {
                &r.stderr
            } else if !r.stdout.is_empty() {
                &r.stdout
            } else {
                "config get failed"
            };
            return text_result(json!({ "error": true, "message": msg }), true);
        }
        let parsed = try_parse(&r.stdout).unwrap_or_else(|| json!({}));
        return text_result(json!({ "config": parsed }), false);
    }

    text_result(
        json!({ "ok": true, "set": set_keys, "message": format!("updated Mog preferences: {}", set_keys.join(", ")) }),
        false,
    )
}
