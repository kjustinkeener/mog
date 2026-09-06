//! The run report (the deterministic contract) plus the HTML dashboard renderer.
//!
//! Layering: the engine emits a
//! report JSON; rendering is a static-template-plus-data concern, so no HTML
//! lives in the deterministic core. A template is a single self-contained `.html`
//! with one injection token, `__MOG_REPORT_JSON__`; the renderer reads the
//! template and splices the serialized report in at emit time (the browser never
//! fetches it), producing a self-contained page that opens anywhere.
//!
//! `when` and the output filename use system time. That is the report layer
//! only; it never feeds a transform, so byte-deterministic output is preserved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use include_dir::{include_dir, Dir};
use serde::Serialize;

/// Report-template precedence subdirectories under `MOG_HOME/templates/`. This is
/// the template lookup order and is independent of the (now flat) mog store;
/// see `load_template`.
const TEMPLATE_SOURCES: [&str; 3] = ["user", "community", "factory"];

/// The injection token the renderer replaces with the serialized report JSON.
/// A template carries exactly one of these (see `templates/factory/default.html`).
pub const TOKEN: &str = "__MOG_REPORT_JSON__";

/// The factory dashboard templates embedded at build time, written to
/// `<root>/templates/factory/` on `mog setup` and used as the render fallback
/// before any setup has run.
pub static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/factory");

/// The caps that bound a run report by construction (see the spec, section 8):
/// its size and its compute cost are O(caps), not O(run size). `None` on any cap
/// means "unlimited" (the human opt-out); [`ReportCaps::default`] is the
/// aggressive default set the agent surface always gets.
///
/// Only the detailed diff sample is capped; the `summary` aggregates (files,
/// changed, bytes) stay exact and cheap regardless of these caps.
#[derive(Debug, Clone, Copy)]
pub struct ReportCaps {
    /// A file whose before OR after size exceeds this is never handed to the
    /// differ (stats only). This is where a huge file could OOM/hang, so raising
    /// it warns.
    pub diff_skip_over_bytes: Option<usize>,
    /// Per-file diff truncated past this many lines (a note is appended).
    pub max_diff_lines: Option<usize>,
    /// Per-file diff truncated past this many bytes (a note is appended).
    pub max_diff_bytes: Option<usize>,
    /// Only the first N changed files get diffs computed; the rest are stats only.
    pub max_diff_files: Option<usize>,
    /// Only the first N files are enumerated in `files[]`; the remainder are not
    /// listed individually (the summary still counts them exactly).
    pub max_files: Option<usize>,
    /// A running diff-byte budget; once spent, remaining files are stats only.
    pub total_diff_bytes: Option<usize>,
}

impl Default for ReportCaps {
    fn default() -> Self {
        ReportCaps {
            diff_skip_over_bytes: Some(1 << 20), // 1 MB
            max_diff_lines: Some(200),
            max_diff_bytes: Some(64 << 10), // 64 KB
            max_diff_files: Some(50),
            max_files: Some(500),
            total_diff_bytes: Some(1 << 20), // 1 MB
        }
    }
}

impl ReportCaps {
    /// The six cap keys, as used by `config.toml [report]` and `--report-limit`.
    pub const KEYS: [&'static str; 6] = [
        "diff_skip_over_bytes",
        "max_diff_lines",
        "max_diff_bytes",
        "max_diff_files",
        "max_files",
        "total_diff_bytes",
    ];

    /// Override one cap by key (`None` = unlimited). An unknown key is an error.
    pub fn set_key(&mut self, key: &str, cap: Option<usize>) -> Result<()> {
        match key {
            "diff_skip_over_bytes" => self.diff_skip_over_bytes = cap,
            "max_diff_lines" => self.max_diff_lines = cap,
            "max_diff_bytes" => self.max_diff_bytes = cap,
            "max_diff_files" => self.max_diff_files = cap,
            "max_files" => self.max_files = cap,
            "total_diff_bytes" => self.total_diff_bytes = cap,
            other => bail!(
                "unknown report cap '{other}' (expected one of: {})",
                Self::KEYS.join(", ")
            ),
        }
        Ok(())
    }

    /// Would a file with these before/after text sizes be skipped by the
    /// size gate (never handed to the differ)?
    pub fn skips_diff(&self, bytes_before: usize, bytes_after: usize) -> bool {
        self.diff_skip_over_bytes
            .is_some_and(|cap| bytes_before > cap || bytes_after > cap)
    }
}

/// Parse a cap value from `config.toml` or a `--report-limit` override. `"0"` or
/// `"unlimited"` (case-insensitive) lift the cap (return `None`); any other value
/// must parse as a nonnegative integer (returned as `Some(n)`).
pub fn parse_cap(value: &str) -> Result<Option<usize>> {
    let t = value.trim();
    if t.eq_ignore_ascii_case("unlimited") {
        return Ok(None);
    }
    let n: usize = t
        .parse()
        .with_context(|| format!("cap value must be a number, 0, or 'unlimited', got '{value}'"))?;
    Ok(if n == 0 { None } else { Some(n) })
}

/// Truncate a unified diff to the per-file caps, appending a note line when
/// anything is dropped (so the template needs no separate truncation field). A
/// `None` cap is unlimited; at least one line is always kept.
pub fn truncate_diff(diff: String, max_lines: Option<usize>, max_bytes: Option<usize>) -> String {
    if max_lines.is_none() && max_bytes.is_none() {
        return diff;
    }
    let mut out = String::new();
    let mut kept = 0usize;
    let mut total = 0usize;
    let mut truncated = false;
    for line in diff.split_inclusive('\n') {
        total += 1;
        if truncated {
            continue;
        }
        let over_lines = max_lines.is_some_and(|m| kept >= m);
        let over_bytes = max_bytes.is_some_and(|m| kept > 0 && out.len() + line.len() > m);
        if over_lines || over_bytes {
            truncated = true;
            continue;
        }
        out.push_str(line);
        kept += 1;
    }
    if truncated {
        let omitted = total - kept;
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("... (+{omitted} more lines, truncated)\n"));
    }
    out
}

/// Which `.mog` a report is about: its `name` field (or file stem) and the path
/// it was invoked by.
#[derive(Debug, Clone, Serialize)]
pub struct MogRef {
    pub name: String,
    pub path: String,
}

/// The run-level counts.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub files: usize,
    pub changed: usize,
    pub errors: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
}

/// One file's entry in a run report. `diff` is the unified diff for a changed
/// file (omitted for unchanged files and errors); `error` is always present
/// (null on success) to keep the existing `--json` shape.
#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub path: String,
    pub changed: bool,
    pub bytes_before: usize,
    pub bytes_after: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    pub error: Option<String>,
}

/// One mog flag marker, tagged with its file.
#[derive(Debug, Clone, Serialize)]
pub struct FlagRecord {
    pub tag: String,
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// The metadata that identifies a run (everything not derived from the files).
#[derive(Debug, Clone)]
pub struct ReportMeta {
    /// The `.mog`'s `name` field, or its file stem.
    pub name: String,
    /// The path the `.mog` was invoked by (the `-m` value).
    pub path: String,
    /// The run mode: `apply` | `preview` | `dry-run` | `test`.
    pub mode: String,
}

/// The full run report: the contract every template renders and what `--json`
/// returns. Field order is the serialized key order (see the spec's schema).
#[derive(Debug, Clone, Serialize)]
pub struct RunReport {
    pub mog: MogRef,
    pub mode: String,
    pub when: String,
    pub summary: Summary,
    pub files: Vec<FileReport>,
    pub flags: Vec<FlagRecord>,
    pub flag_counts: BTreeMap<String, usize>,
}

impl RunReport {
    /// Assemble a report from its parts, deriving `summary` and `flag_counts`
    /// from the files and flags. `when` is stamped from the system clock.
    pub fn new(meta: ReportMeta, files: Vec<FileReport>, flags: Vec<FlagRecord>) -> Self {
        let mut flag_counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in &flags {
            *flag_counts.entry(f.tag.clone()).or_default() += 1;
        }
        let summary = Summary {
            files: files.len(),
            changed: files.iter().filter(|f| f.changed).count(),
            errors: files.iter().filter(|f| f.error.is_some()).count(),
            lines_added: files.iter().map(|f| f.lines_added).sum(),
            lines_removed: files.iter().map(|f| f.lines_removed).sum(),
        };
        RunReport {
            mog: MogRef {
                name: meta.name,
                path: meta.path,
            },
            mode: meta.mode,
            when: now_rfc3339(),
            summary,
            files,
            flags,
            flag_counts,
        }
    }

    /// Assemble a report from an already-exact `summary` computed over ALL files
    /// in the run, with a possibly-capped `files` list. Use this (not [`new`])
    /// whenever `files` has been trimmed to `max_files`, so the summary counts
    /// stay exact while the enumeration is bounded.
    ///
    /// [`new`]: RunReport::new
    pub fn from_parts(
        meta: ReportMeta,
        summary: Summary,
        files: Vec<FileReport>,
        flags: Vec<FlagRecord>,
    ) -> Self {
        let mut flag_counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in &flags {
            *flag_counts.entry(f.tag.clone()).or_default() += 1;
        }
        RunReport {
            mog: MogRef {
                name: meta.name,
                path: meta.path,
            },
            mode: meta.mode,
            when: now_rfc3339(),
            summary,
            files,
            flags,
            flag_counts,
        }
    }

    /// The report as a JSON value with every per-file `diff` text removed: the
    /// model-facing "lean" serialization. File-list capping already happened at
    /// build time (`max_files`); this only drops the diff text, so the lean
    /// report never carries diffs while the dashboard serialization does.
    pub fn to_lean_value(&self) -> Result<serde_json::Value> {
        let mut v = serde_json::to_value(self).context("failed to serialize the run report")?;
        if let Some(files) = v.get_mut("files").and_then(|f| f.as_array_mut()) {
            for f in files.iter_mut() {
                if let Some(obj) = f.as_object_mut() {
                    obj.remove("diff");
                }
            }
        }
        Ok(v)
    }

    /// The offending paths (changed, or carrying a flag) for the `--check`
    /// report. Errored files are not "offending" (they raise the error count).
    pub fn offending(&self) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| f.changed || self.flags.iter().any(|fl| fl.file == f.path))
            .map(|f| f.path.clone())
            .collect()
    }

    /// The report serialized as compact JSON (what gets injected into a template).
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("failed to serialize the run report")
    }
}

/// Load a dashboard template by name. Resolution mirrors script resolution:
/// `MOG_HOME/templates/{user,community,factory}/<name>.html` in precedence
/// order, then the factory template embedded in the binary (so `--report` works
/// before any `mog setup`).
pub fn load_template(name: &str, lib_root: Option<&Path>) -> Result<String> {
    let file = format!("{name}.html");
    if let Some(root) = lib_root {
        for src in TEMPLATE_SOURCES {
            let p = root.join("templates").join(src).join(&file);
            if p.is_file() {
                return std::fs::read_to_string(&p)
                    .with_context(|| format!("failed to read template '{}'", p.display()));
            }
        }
    }
    if let Some(f) = TEMPLATE_DIR.get_file(&file) {
        return Ok(f.contents_utf8().unwrap_or_default().to_string());
    }
    bail!(
        "template '{name}' not found (looked in MOG_HOME/templates/{{user,community,factory}} \
         and the embedded factory set)"
    )
}

/// Render a report JSON into a self-contained HTML string by injecting it at the
/// template's single token. `</` in the JSON is escaped to `<\/` (a valid JSON
/// escape) so a `</script>` inside diff text cannot break out of the script tag.
pub fn render(report_json: &str, template: &str, lib_root: Option<&Path>) -> Result<String> {
    let tpl = load_template(template, lib_root)?;
    if !tpl.contains(TOKEN) {
        bail!("template '{template}' has no {TOKEN} injection token");
    }
    let safe = report_json.replace("</", "<\\/");
    Ok(tpl.replace(TOKEN, &safe))
}

/// Render a report to a self-contained HTML file. `explicit` picks the output
/// path; when `None`, writes `<reports_dir>/run-<timestamp>.html`, creating the
/// directory. Returns the path written.
pub fn write_dashboard(
    report: &RunReport,
    explicit: Option<&str>,
    template: &str,
    lib_root: Option<&Path>,
) -> Result<PathBuf> {
    let html = render(&report.to_json()?, template, lib_root)?;
    let out = match explicit {
        Some(p) => PathBuf::from(p),
        None => {
            let dir = reports_dir(lib_root);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("failed to create reports dir '{}'", dir.display()))?;
            dir.join(format!("run-{}.html", now_filestamp()))
        }
    };
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }
    }
    std::fs::write(&out, html)
        .with_context(|| format!("failed to write report '{}'", out.display()))?;
    Ok(out)
}

/// The default reports directory: `<lib_root>/reports`, or a temp dir when no
/// library root is available (the spec's answer to "`--report` with no MOG_HOME").
fn reports_dir(lib_root: Option<&Path>) -> PathBuf {
    match lib_root {
        Some(root) => root.join("reports"),
        None => std::env::temp_dir().join("mog").join("reports"),
    }
}

/// The current UTC time as `YYYY-MM-DDTHH:MM:SSZ` (RFC 3339). Report layer only.
pub fn now_rfc3339() -> String {
    let (y, mo, d, h, mi, s) = now_civil();
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// The current UTC time as a filename-safe stamp `YYYYMMDDTHHMMSSZ`.
pub fn now_filestamp() -> String {
    let (y, mo, d, h, mi, s) = now_civil();
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}Z")
}

/// Break the current instant into UTC civil parts, so a timestamp needs no
/// date/time crate. Uses Howard Hinnant's `civil_from_days` algorithm.
fn now_civil() -> (i64, i64, i64, i64, i64, i64) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, min, sec) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // civil_from_days: days since 1970-01-01 -> (year, month, day).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = y + i64::from(month <= 2);
    (year, month, day, hour, min, sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> ReportMeta {
        ReportMeta {
            name: "demo".to_string(),
            path: "user/demo.mog".to_string(),
            mode: "apply".to_string(),
        }
    }

    fn changed_file() -> FileReport {
        FileReport {
            path: "a.sql".to_string(),
            changed: true,
            bytes_before: 10,
            bytes_after: 12,
            lines_added: 2,
            lines_removed: 1,
            diff: Some("--- a/a.sql\n+++ b/a.sql\n".to_string()),
            error: None,
        }
    }

    #[test]
    fn summary_and_flag_counts_are_derived() {
        let flags = vec![FlagRecord {
            tag: "FIXME".to_string(),
            file: "a.sql".to_string(),
            line: 3,
            message: "check".to_string(),
        }];
        let r = RunReport::new(meta(), vec![changed_file()], flags);
        assert_eq!(r.summary.files, 1);
        assert_eq!(r.summary.changed, 1);
        assert_eq!(r.summary.lines_added, 2);
        assert_eq!(r.flag_counts.get("FIXME"), Some(&1));
    }

    #[test]
    fn render_injects_json_and_leaves_no_token() {
        let r = RunReport::new(meta(), vec![changed_file()], vec![]);
        let html = render(&r.to_json().unwrap(), "default", None).unwrap();
        assert!(!html.contains(TOKEN), "token must be replaced");
        assert!(html.contains("\"a.sql\""), "report JSON is injected");
    }

    #[test]
    fn render_escapes_closing_script_tags_in_diffs() {
        let mut f = changed_file();
        f.diff = Some("removed </script> here".to_string());
        let r = RunReport::new(meta(), vec![f], vec![]);
        let html = render(&r.to_json().unwrap(), "default", None).unwrap();
        assert!(!html.contains("</script> here"), "closing tag is escaped");
        assert!(html.contains("<\\/script> here"));
    }

    #[test]
    fn parse_cap_reads_unlimited_zero_and_numbers() {
        assert_eq!(parse_cap("unlimited").unwrap(), None);
        assert_eq!(parse_cap("UNLIMITED").unwrap(), None);
        assert_eq!(parse_cap("0").unwrap(), None);
        assert_eq!(parse_cap("50").unwrap(), Some(50));
        assert!(parse_cap("lots").is_err());
    }

    #[test]
    fn truncate_diff_appends_note_past_line_cap() {
        let diff = (0..10).map(|i| format!("+line {i}\n")).collect::<String>();
        let out = truncate_diff(diff, Some(3), None);
        assert_eq!(out.lines().filter(|l| l.starts_with('+')).count(), 3);
        assert!(
            out.contains("(+7 more lines, truncated)"),
            "note counts omitted lines: {out}"
        );
    }

    #[test]
    fn truncate_diff_is_a_noop_when_unlimited() {
        let diff = "a\nb\nc\n".to_string();
        assert_eq!(truncate_diff(diff.clone(), None, None), diff);
    }

    #[test]
    fn to_lean_value_drops_diff_text() {
        let r = RunReport::new(meta(), vec![changed_file()], vec![]);
        let v = r.to_lean_value().unwrap();
        assert!(
            v["files"][0].get("diff").is_none(),
            "lean serialization carries no diff text"
        );
        assert!(
            v["files"][0]["bytes_before"].is_number(),
            "per-file stats are kept"
        );
    }

    #[test]
    fn rfc3339_is_well_formed() {
        let s = now_rfc3339();
        assert_eq!(s.len(), 20, "YYYY-MM-DDTHH:MM:SSZ: {s}");
        assert!(s.ends_with('Z') && s.contains('T'), "{s}");
    }
}
