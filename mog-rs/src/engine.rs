//! The mog run loop and action registry. Mirrors .NET `MogEngine.cs` +
//! `ActionRegistry.cs`, plus scoped steps and the `run_mog` compose action.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use fancy_regex::Regex;

use crate::actions::transpile;
use crate::actions::{
    affix, assert, block, case, compare, config, data, encode, eol, escape, filter, flag, json,
    line, numbers, replace, text, whitespace, xml,
};
use crate::line_text::{join, split};
use crate::model::{BlockScope, CharRange, FieldScope, LineRange, Mog, Scope, Step};

/// An action: transforms input text into output text for a given step.
pub type ActionFn = fn(&str, &Step) -> Result<String>;

/// A step-boundary observer: called just BEFORE each enabled step runs, so a slow
/// or hanging step is visible to a caller. The engine stays silent unless a caller
/// installs one (used by the CLI's opt-in `--progress`). Send + Sync so it can be
/// moved onto the worker thread of a timeout-guarded run.
pub type StepObserver = Arc<dyn Fn(StepProgress) + Send + Sync>;

/// What the engine reports to a [`StepObserver`] at each step boundary.
pub struct StepProgress<'a> {
    /// 1-based position among ENABLED steps (disabled steps are skipped, not counted).
    pub index: usize,
    /// Count of enabled steps in the pipeline.
    pub total: usize,
    /// The step's action name.
    pub action: &'a str,
    /// Byte length of the working text entering this step.
    pub input_bytes: usize,
}

/// Maximum `run_mog` nesting depth before we bail with a clear error.
const MAX_RUN_MOG_DEPTH: usize = 25;

/// Resolve an action name/alias to its function. Case-sensitive (Ordinal),
/// matching the .NET registry. Returns None for unknown names.
///
/// Note: `run_mog` is NOT resolved here; it is handled directly by the engine
/// because it needs the base directory and recursion depth as context.
pub fn resolve(name: &str) -> Option<ActionFn> {
    let f: ActionFn = match name {
        // Replace
        "replace" | "r" => replace::replace,
        "replace_extended" => replace::replace_extended,
        "replace_regex" | "rr" => replace::replace_regex,
        "replace_regex_multiline" | "rrm" => replace::replace_regex_multiline,
        "replace_map" => replace::replace_map,
        "lookup_replace" => replace::lookup_replace,
        "smart_case_replace" => replace::smart_case_replace,
        "replace_nth" => replace::replace_nth,
        "replace_first" => replace::replace_first,
        "replace_last" => replace::replace_last,
        // Line
        "remove_empty_lines" | "rel" => line::remove_empty_lines,
        "remove_duplicate_lines" | "dedupe" => line::remove_duplicate_lines,
        "remove_consecutive_duplicate_lines" | "uniq" => line::remove_consecutive_duplicate_lines,
        "dedupe_by" => line::dedupe_by,
        "reverse_lines" | "reverse" => line::reverse_lines,
        "shuffle_lines" | "shuffle" => line::shuffle_lines,
        "join_lines" | "join" => line::join_lines,
        "sort_lines" | "sort" => line::sort_lines,
        "sort_ip" => line::sort_ip,
        "sort_versions" => line::sort_versions,
        "sort_imports" => line::sort_imports,
        "number_lines" => line::number_lines,
        "squeeze_blank_lines" => line::squeeze_blank_lines,
        "split_lines" => line::split_lines,
        "keep_duplicate_lines" => line::keep_duplicate_lines,
        "unique_with_count" => line::unique_with_count,
        "stamp_sequence" => line::stamp_sequence,
        // Line filtering
        "keep_lines_matching" => filter::keep_lines_matching,
        "remove_lines_matching" => filter::remove_lines_matching,
        "insert_before_matching" => filter::insert_before_matching,
        "insert_after_matching" => filter::insert_after_matching,
        "flag_matching" => flag::flag_matching,
        // Detect (scan and flag)
        "detect_secrets" => flag::detect_secrets,
        "detect_pii" => flag::detect_pii,
        // Assert (post-condition gate)
        "assert" => assert::assert,
        // Region-aware
        "hoist_from_block" => block::hoist_from_block,
        // Block (multi-line record) ops
        "sort_blocks" => block::sort_blocks,
        "dedupe_blocks" => block::dedupe_blocks,
        // JSON (reading: JSON in -> text/lines/CSV out)
        "json_extract" => json::json_extract,
        "json_filter" => json::json_filter,
        "json_keys" => json::json_keys,
        "json_minify" => json::json_minify,
        "json_pretty" => json::json_pretty,
        "json_unescape" => json::json_unescape,
        "json_to_csv" => json::json_to_csv,
        "csv_to_json" => json::csv_to_json,
        "logfmt_to_json" => json::logfmt_to_json,
        "json_to_logfmt" => json::json_to_logfmt,
        "yaml_to_json" => json::yaml_to_json,
        "json_to_yaml" => json::json_to_yaml,
        "json_to_jsonl" => json::json_to_jsonl,
        "jsonl_to_json" => json::jsonl_to_json,
        "toml_to_json" => config::toml_to_json,
        "json_to_toml" => config::json_to_toml,
        "xml_to_json" => xml::xml_to_json,
        "json_to_xml" => xml::json_to_xml,
        "env_to_json" => json::env_to_json,
        "json_to_env" => json::json_to_env,
        "querystring_to_json" => json::querystring_to_json,
        "json_to_querystring" => json::json_to_querystring,
        "properties_to_json" => json::properties_to_json,
        "json_merge" => json::json_merge,
        "json_wrap" => json::json_wrap,
        "extract_json" => json::extract_json,
        "json_set" => json::json_set,
        "json_delete" => json::json_delete,
        "json_rename" => json::json_rename,
        "ipynb_to_python" => json::ipynb_to_python,
        // Data (tabular)
        "records_to_columns" => data::records_to_columns,
        "transpose" => data::transpose,
        "unpivot" => data::unpivot,
        "pivot" => data::pivot,
        "change_delimiter" => data::change_delimiter,
        "fill_down" => data::fill_down,
        "mask_field" => data::mask_field,
        "hash_field" => data::hash_field,
        "epoch_to_iso" => data::epoch_to_iso,
        "iso_to_epoch" => data::iso_to_epoch,
        "shift_dates" => data::shift_dates,
        "row_to_template" => data::row_to_template,
        "cut_fields" => data::cut_fields,
        "explode_field" => data::explode_field,
        "csv_to_markdown" => data::csv_to_markdown,
        "markdown_table_to_csv" => data::markdown_table_to_csv,
        "csv_to_sql" => data::csv_to_sql,
        "csv_to_html" => data::csv_to_html,
        "html_table_to_csv" => data::html_table_to_csv,
        "fixed_width_to_csv" => data::fixed_width_to_csv,
        "access_log_to_csv" => data::access_log_to_csv,
        "toml_to_ini" => config::toml_to_ini,
        "ini_to_toml" => config::ini_to_toml,
        "decimal_separator_normalize" => data::decimal_separator_normalize,
        // Whitespace
        "trim_whitespace_right" | "twr" => whitespace::trim_right,
        "trim_whitespace_left" | "twl" => whitespace::trim_left,
        "trim_whitespace" | "tw" => whitespace::trim_both,
        "tabs_to_spaces" | "t2s" => whitespace::tabs_to_spaces,
        "spaces_to_tabs" | "s2t" => whitespace::spaces_to_tabs,
        "collapse_whitespace" => whitespace::collapse_whitespace,
        "squeeze_spaces" => whitespace::squeeze_spaces,
        "eol_to_space" => whitespace::eol_to_space,
        "trim_and_eol_to_space" => whitespace::trim_and_eol_to_space,
        "dedent" => whitespace::dedent,
        "align_columns" => whitespace::align_columns,
        "wrap_text" | "reflow" | "fill_paragraph" => whitespace::wrap_text,
        "unwrap_text" | "unwrap" | "unfill" => whitespace::unwrap_text,
        // EOL
        "eol_crlf" | "eol_windows" => eol::to_crlf,
        "eol_lf" | "eol_unix" => eol::to_lf,
        "eol_cr" | "eol_mac" => eol::to_cr,
        // Text (character-level cleanup)
        "keep_chars" => text::keep_chars,
        "remove_chars" => text::remove_chars,
        "strip_control_chars" => text::strip_control_chars,
        "normalize" => text::normalize,
        "fix_mojibake" => text::fix_mojibake,
        "slugify" => text::slugify,
        "chunk_text" => text::chunk_text,
        "strip_html_tags" => text::strip_html_tags,
        "strip_markdown" => text::strip_markdown,
        "convert_comment_style" => text::convert_comment_style,
        // Numbers / math
        "arithmetic" => numbers::arithmetic,
        "increment_numbers" => numbers::increment_numbers,
        "round_numbers" => numbers::round_numbers,
        "pad_numbers" => numbers::pad_numbers,
        // Case
        "to_upper" | "upper" => case::to_upper,
        "to_lower" | "lower" => case::to_lower,
        "to_proper" | "proper" => case::to_proper,
        "to_sentence" | "sentence" => case::to_sentence,
        "invert_case" => case::invert_case,
        "to_camel" | "camel" => case::to_camel,
        "to_pascal" | "pascal" => case::to_pascal,
        "random_case" => case::random_case,
        // Affix (whole-document)
        "prepend" => affix::prepend,
        "append" => affix::append,
        "insert_if_absent" => affix::insert_if_absent,
        // Affix (per-line)
        "prefix_lines" => affix::prefix_lines,
        "suffix_lines" => affix::suffix_lines,
        "wrap_lines" => affix::wrap_lines,
        "indent" => affix::indent,
        "outdent" => affix::outdent,
        "format_list" => affix::format_list,
        // Encoding helpers (whole-document)
        "url_encode" => encode::url_encode,
        "url_decode" => encode::url_decode,
        "html_encode" => encode::html_encode,
        "html_decode" | "html_unescape" => encode::html_decode,
        "base64_encode" => encode::base64_encode,
        "base64_decode" => encode::base64_decode,
        "rot13" => encode::rot13,
        "hex_encode" => encode::hex_encode,
        "hex_decode" => encode::hex_decode,
        "normalize_url" => encode::normalize_url,
        "escape_regex" => escape::escape_regex,
        "escape_shell" => escape::escape_shell,
        "escape_sql" => escape::escape_sql,
        "escape_csv" => escape::escape_csv,
        "escape_json" => escape::escape_json,
        "escape_xml" => escape::escape_xml,
        "escape_c" => escape::escape_c,
        // Full semantic transpile via polyglot-sql (correctness tier).
        "sql_transpile" => transpile::sql_transpile,
        // Parse-validate SQL against a dialect and annotate problems in place.
        "sql_lint" => transpile::sql_lint,
        // Canonically re-format SQL within one dialect.
        "sql_format" => transpile::sql_format,
        // List the physical source tables a query reads.
        "sql_tables" => transpile::sql_tables,
        // Normalize identifier casing/quoting to a dialect's canonical form.
        "sql_canonicalize_identifiers" => transpile::sql_canonicalize_identifiers,
        // Convert a data type (per line) from one dialect's spelling to another's.
        "sql_datatype_convert" => transpile::sql_datatype_convert,
        // List per-output-column lineage (output col -> source cols).
        "sql_lineage" => transpile::sql_lineage,
        _ => return None,
    };
    Some(f)
}

/// Whether an action name is known to the engine (registry or built-in).
pub(crate) fn action_exists(name: &str) -> bool {
    name == "run_mog"
        || name == "for_each_block"
        || name == "fill_from_list"
        || name == "paste_column"
        || matches!(
            name,
            "intersect" | "subtract" | "diff" | "reconcile" | "lookup" | "sql_qualify"
        )
        || resolve(name).is_some()
}

/// Execution context threaded through the run loop: the base directory used to
/// resolve `run_mog` paths, the library root for name lookups, and the current
/// recursion depth.
#[derive(Clone)]
struct ExecCtx {
    base_dir: Option<PathBuf>,
    lib_root: Option<PathBuf>,
    depth: usize,
    /// External data sources (name -> lines), loaded by the CLI. Shared cheaply
    /// via Arc; propagated to `run_mog` / `for_each_block` children.
    sources: Arc<BTreeMap<String, Vec<String>>>,
    /// Optional step-boundary observer (the CLI's `--progress`). `None` by default
    /// so the library is silent. Deliberately NOT propagated to `run_mog` /
    /// `for_each_block` children, so nested sub-steps do not confuse the i/N count.
    on_step: Option<StepObserver>,
    /// The run's pinned RNG seed (`--pin-seed`, or `mog --test`'s fixed seed),
    /// threaded to the per-occurrence `$uuid` replacement token so its per-match
    /// stream is reproducible under a pin. `None` = unpinned (random each run). IS
    /// propagated to `run_mog` / `for_each_block` children so nested `$uuid` is
    /// pinned too.
    seed: Option<u64>,
}

/// Execute a whole pipeline over `input`. Equivalent to `execute_at(mog, input,
/// None)`; kept for existing callers/tests. `run_mog` steps require a base
/// directory, so they only work via `execute_at`.
pub fn execute(mog: &Mog, input: &str) -> Result<String> {
    execute_at(mog, input, None)
}

/// Execute a pipeline, resolving any `run_mog` steps relative to `base_dir`
/// (typically the directory containing the .mog script). No library lookup:
/// child scripts must resolve by absolute or base-relative path.
pub fn execute_at(mog: &Mog, input: &str, base_dir: Option<&Path>) -> Result<String> {
    let ctx = ExecCtx {
        base_dir: base_dir.map(|p| p.to_path_buf()),
        lib_root: None,
        depth: 0,
        sources: Arc::new(BTreeMap::new()),
        on_step: None,
        seed: None,
    };
    execute_inner(mog, input, &ctx)
}

/// Execute a pipeline with library resolution enabled: `run_mog` children are
/// resolved absolute -> base-relative -> library (see [`crate::library`]).
pub fn execute_with_library(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
) -> Result<String> {
    execute_with_library_sources(mog, input, base_dir, lib_root, Arc::new(BTreeMap::new()))
}

/// Like [`execute_with_library`], but with external data `sources` (name ->
/// lines) available to source-aware actions (e.g. `fill_from_list`).
pub fn execute_with_library_sources(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    sources: Arc<BTreeMap<String, Vec<String>>>,
) -> Result<String> {
    execute_with_library_sources_observed(mog, input, base_dir, lib_root, sources, None, None)
}

/// Like [`execute_with_library_sources`], but with an optional [`StepObserver`]
/// fired at each step boundary (the CLI's `--progress`) and an optional pinned RNG
/// `seed` (the run's `--pin-seed` / `mog --test`) that makes the per-occurrence
/// `$uuid` replacement token reproducible. `None`/`None` behaves exactly like the
/// plain variant.
pub fn execute_with_library_sources_observed(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    sources: Arc<BTreeMap<String, Vec<String>>>,
    on_step: Option<StepObserver>,
    seed: Option<u64>,
) -> Result<String> {
    let ctx = ExecCtx {
        base_dir: base_dir.map(|p| p.to_path_buf()),
        lib_root: lib_root.map(|p| p.to_path_buf()),
        depth: 0,
        sources,
        on_step,
        seed,
    };
    execute_inner(mog, input, &ctx)
}

/// Optional resource bounds for one apply. All off by default, so a trusted
/// large-file run is never interrupted; enable them for untrusted input (e.g. a
/// sandboxed store apply, or reviewing a submitted recipe). Engine hardening
/// H2/H3 (store-marketplace-spec.md section 2.2).
#[derive(Debug, Clone, Copy, Default)]
pub struct RunLimits {
    /// Reject input larger than this many bytes (H3). None = unlimited.
    pub max_input_bytes: Option<usize>,
    /// Reject a pipeline with more than this many (enabled) steps (H3).
    pub max_steps: Option<usize>,
    /// Abort the apply if it runs longer than this wall-clock duration (H2). This
    /// is the catch-all for a catastrophic-backtracking regex, which cooperative
    /// between-step checks cannot interrupt. None = no timeout.
    pub timeout: Option<std::time::Duration>,
}

impl RunLimits {
    pub fn is_unbounded(&self) -> bool {
        self.max_input_bytes.is_none() && self.max_steps.is_none() && self.timeout.is_none()
    }
}

/// Execute a pipeline (with library resolution) under optional [`RunLimits`].
/// Size/step caps are pre-checked; with a timeout set, the work runs on a
/// separate thread and the call returns a timeout error rather than blocking past
/// the deadline. An overrun thread is abandoned and reclaimed at process exit,
/// which is acceptable for a one-shot CLI apply.
pub fn execute_guarded(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    limits: &RunLimits,
) -> Result<String> {
    execute_guarded_with_sources(
        mog,
        input,
        base_dir,
        lib_root,
        limits,
        Arc::new(BTreeMap::new()),
        None,
    )
}

/// Like [`execute_guarded`], but with external data `sources` threaded through to
/// source-aware actions (e.g. `fill_from_list`) and an optional pinned RNG `seed`
/// (see [`execute_with_library_sources_observed`]).
pub fn execute_guarded_with_sources(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    limits: &RunLimits,
    sources: Arc<BTreeMap<String, Vec<String>>>,
    seed: Option<u64>,
) -> Result<String> {
    execute_guarded_with_sources_observed(
        mog, input, base_dir, lib_root, limits, sources, None, seed,
    )
}

/// Like [`execute_guarded_with_sources`], but with an optional [`StepObserver`]
/// fired at each step boundary (the CLI's `--progress`). The observer is Send +
/// Sync, so it survives the worker-thread hop taken when a timeout is set. `None`
/// behaves exactly like the plain variant. `seed` is the run's pinned RNG seed
/// (`--pin-seed` / `mog --test`) for the per-occurrence `$uuid` token.
// The `execute_guarded_*` family threads run context positionally; collapsing it
// into a context struct is tracked as a separate refactor.
#[allow(clippy::too_many_arguments)]
pub fn execute_guarded_with_sources_observed(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    limits: &RunLimits,
    sources: Arc<BTreeMap<String, Vec<String>>>,
    on_step: Option<StepObserver>,
    seed: Option<u64>,
) -> Result<String> {
    if let Some(max) = limits.max_input_bytes {
        if input.len() > max {
            bail!("input is {} bytes, over the {max} byte limit", input.len());
        }
    }
    if let Some(max) = limits.max_steps {
        let n = mog.steps.iter().filter(|s| !s.disabled).count();
        if n > max {
            bail!("pipeline has {n} steps, over the {max} step limit");
        }
    }
    match limits.timeout {
        None => execute_with_library_sources_observed(
            mog, input, base_dir, lib_root, sources, on_step, seed,
        ),
        Some(dur) => {
            let (tx, rx) = std::sync::mpsc::channel();
            let mog = mog.clone();
            let input = input.to_string();
            let base = base_dir.map(|p| p.to_path_buf());
            let root = lib_root.map(|p| p.to_path_buf());
            std::thread::spawn(move || {
                let out = execute_with_library_sources_observed(
                    &mog,
                    &input,
                    base.as_deref(),
                    root.as_deref(),
                    sources,
                    on_step,
                    seed,
                );
                let _ = tx.send(out);
            });
            match rx.recv_timeout(dur) {
                Ok(res) => res,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => bail!(
                    "apply exceeded the {dur:?} timeout (possible catastrophic regex); aborted"
                ),
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    bail!("apply worker terminated unexpectedly")
                }
            }
        }
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;
    use crate::parse_mog;
    use std::time::Duration;

    fn mog(json: &str) -> Mog {
        parse_mog(json).unwrap()
    }

    #[test]
    fn unbounded_matches_plain_execute() {
        let m = mog(r#"{"steps":[{"action":"to_upper"}]}"#);
        let out = execute_guarded(&m, "abc", None, None, &RunLimits::default()).unwrap();
        assert_eq!(out, "ABC");
    }

    #[test]
    fn caps_reject_oversized_input_and_too_many_steps() {
        let m = mog(r#"{"steps":[{"action":"to_upper"}]}"#);
        let over_size = RunLimits {
            max_input_bytes: Some(3),
            ..Default::default()
        };
        assert!(execute_guarded(&m, "abcdef", None, None, &over_size).is_err());
        // Exactly at the limit is fine.
        assert!(execute_guarded(&m, "abc", None, None, &over_size).is_ok());

        let m2 = mog(r#"{"steps":[{"action":"to_upper"},{"action":"to_lower"}]}"#);
        let over_steps = RunLimits {
            max_steps: Some(1),
            ..Default::default()
        };
        assert!(execute_guarded(&m2, "abc", None, None, &over_steps).is_err());
    }

    #[test]
    fn observer_reports_each_enabled_step_in_order() {
        use std::sync::Mutex;
        // Two enabled steps around one disabled step: the observer sees 1/2 and 2/2
        // only, and the disabled step neither fires nor advances the count.
        let m = mog(r#"{"steps":[
                {"action":"to_upper"},
                {"action":"to_lower","disabled":true},
                {"action":"trim_whitespace"}
            ]}"#);
        let seen: Arc<Mutex<Vec<(usize, usize, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        let obs: StepObserver = Arc::new(move |p: StepProgress| {
            sink.lock()
                .unwrap()
                .push((p.index, p.total, p.action.to_string()));
        });
        let out = execute_guarded_with_sources_observed(
            &m,
            " Ab ",
            None,
            None,
            &RunLimits::default(),
            Arc::new(BTreeMap::new()),
            Some(obs),
            None,
        )
        .unwrap();
        assert_eq!(out, "AB"); // to_upper then trim; to_lower skipped
        let seen = seen.lock().unwrap();
        assert_eq!(
            *seen,
            vec![
                (1, 2, "to_upper".to_string()),
                (2, 2, "trim_whitespace".to_string()),
            ]
        );
    }

    #[test]
    fn observer_none_matches_plain_output() {
        let m = mog(r#"{"steps":[{"action":"to_upper"},{"action":"trim_whitespace"}]}"#);
        let plain = execute_guarded(&m, " ab ", None, None, &RunLimits::default()).unwrap();
        let observed = execute_guarded_with_sources_observed(
            &m,
            " ab ",
            None,
            None,
            &RunLimits::default(),
            Arc::new(BTreeMap::new()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(plain, observed);
    }

    #[test]
    fn timeout_aborts_a_long_run() {
        // A cache-busting input (40 MB, well past any CPU cache) through many
        // full-buffer steps is > 1 GB of work, guaranteed to run past a 20ms
        // deadline on any machine, so the wall-clock timeout aborts it. Bulk work,
        // not a crafted regex, keeps this deterministic regardless of the regex
        // engine's own backtrack limit.
        let input = "a".repeat(40_000_000);
        let steps = vec![r#"{"action":"to_upper"}"#; 30].join(",");
        let m = mog(&format!(r#"{{"steps":[{steps}]}}"#));
        let limits = RunLimits {
            timeout: Some(Duration::from_millis(20)),
            ..Default::default()
        };
        let err = execute_guarded(&m, &input, None, None, &limits).unwrap_err();
        assert!(err.to_string().contains("timeout"), "got: {err}");
    }
}

/// The internal run loop. Disabled steps are skipped (but counted, matching .NET).
fn execute_inner(mog: &Mog, input: &str, ctx: &ExecCtx) -> Result<String> {
    let mut working = input.to_string();
    let mut step_number = 0usize;
    let total = mog.steps.iter().filter(|s| !s.disabled).count();
    for step in &mog.steps {
        if step.disabled {
            continue;
        }
        step_number += 1;
        let action_name = step.action.as_deref().unwrap_or("");
        if !action_exists(action_name) {
            return Err(anyhow!(
                "Unknown action '{action_name}' in step {step_number}."
            ));
        }
        // Fire the step-boundary observer BEFORE running the step, so a hang on
        // step N shows as "step N started", not silence after N-1.
        if let Some(obs) = &ctx.on_step {
            obs(StepProgress {
                index: step_number,
                total,
                action: action_name,
                input_bytes: working.len(),
            });
        }
        working = apply_step(step, &working, ctx)
            .map_err(|e| anyhow!("Step {step_number} ('{action_name}') failed: {e}"))?;
    }
    Ok(working)
}

/// Apply one step, honoring the optional line scope (`only_lines_matching` /
/// `except_lines_matching`). When a scope is present the action is applied to
/// each matching line individually (as a one-line document); non-matching lines
/// pass through unchanged, preserving order and EOL style.
fn apply_step(step: &Step, working: &str, ctx: &ExecCtx) -> Result<String> {
    let action_name = step.action.as_deref().unwrap_or("");
    if let Some(scope) = &step.scope {
        // A structured scope and a line filter (`only_lines_matching` /
        // `except_lines_matching`) COMPOSE: the filter narrows which lines the
        // scope's transform reaches. `apply_with_scope` applies that intersection.
        return apply_with_scope(action_name, step, working, ctx, scope);
    }
    match scope_of(step)? {
        Some((re, keep_when_match)) => {
            let s = split(working);
            let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
            for line_text in &s.lines {
                let matched = re.is_match(line_text).map_err(|e| anyhow!(e))?;
                if matched == keep_when_match {
                    out.push(run_action_on(action_name, step, line_text, ctx)?);
                } else {
                    out.push(line_text.clone());
                }
            }
            Ok(join(&out, s.eol, s.trailing_eol))
        }
        None => run_action_on(action_name, step, working, ctx),
    }
}

/// Run a single action (registry action or the built-in `run_mog`) over `text`.
fn run_action_on(name: &str, step: &Step, text: &str, ctx: &ExecCtx) -> Result<String> {
    if name == "run_mog" {
        return run_mog(text, step, ctx);
    }
    if name == "for_each_block" {
        return for_each_block(text, step, ctx);
    }
    if name == "fill_from_list" {
        return fill_from_list(text, step, ctx);
    }
    if name == "paste_column" {
        return data::paste_column(text, step, &ctx.sources);
    }
    // Two-input actions: compare the primary text against a named reference source.
    match name {
        "intersect" => return compare::intersect(text, step, &ctx.sources),
        "subtract" => return compare::subtract(text, step, &ctx.sources),
        "diff" => return compare::diff(text, step, &ctx.sources),
        "reconcile" => return compare::reconcile(text, step, &ctx.sources),
        "lookup" => return compare::lookup(text, step, &ctx.sources),
        // Schema-aware: qualify columns against a `ref` schema source.
        "sql_qualify" => return transpile::sql_qualify(text, step, &ctx.sources),
        _ => {}
    }
    // Regex replace carries the run's pinned RNG seed to the per-occurrence `$uuid`
    // token (the rest of the pure-fn action set does not need it).
    match name {
        "replace_regex" | "rr" => return replace::replace_regex_seeded(text, step, ctx.seed),
        "replace_regex_multiline" | "rrm" => {
            return replace::replace_regex_multiline_seeded(text, step, ctx.seed)
        }
        _ => {}
    }
    let action = resolve(name).ok_or_else(|| anyhow!("Unknown action '{name}'."))?;
    action(text, step)
}

/// Build the scope matcher for a step, if any. Returns `(regex, keep_when_match)`
/// where `keep_when_match` is true for `only_lines_matching` and false for
/// `except_lines_matching`. `only_lines_matching` wins if both are present.
fn scope_of(step: &Step) -> Result<Option<(Regex, bool)>> {
    let ic = step.match_ignore_case;
    if let Some(pat) = &step.only_lines_matching {
        Ok(Some((build_scope_regex(pat, ic)?, true)))
    } else if let Some(pat) = &step.except_lines_matching {
        Ok(Some((build_scope_regex(pat, ic)?, false)))
    } else {
        Ok(None)
    }
}

fn build_scope_regex(pattern: &str, ignore_case: bool) -> Result<Regex> {
    let full = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.to_string()
    };
    Regex::new(&full).map_err(|e| anyhow!("invalid scope pattern '{pattern}': {e}"))
}

/// Apply a step under a structured [`Scope`]: compute the selected line spans,
/// run the action once per span (over the span's text), and splice results back;
/// unselected lines pass through. This is how any action becomes block- or
/// range-scoped (e.g. sort within each block, uppercase a line range).
fn apply_with_scope(
    action_name: &str,
    step: &Step,
    working: &str,
    ctx: &ExecCtx,
    scope: &Scope,
) -> Result<String> {
    // At most one selector kind may be set.
    let selectors = scope.in_block.is_some() as u8
        + scope.line_range.is_some() as u8
        + scope.field.is_some() as u8
        + scope.char_range.is_some() as u8;
    if selectors > 1 {
        bail!("scope: set only one of 'in_block' / 'line_range' / 'field' / 'char_range'");
    }
    // Optional per-line filter that COMPOSES with the scope: the effective
    // transformed set is (scope-selected) INTERSECT (filter-selected). When no
    // filter is present this is `None` and behavior is byte-identical to before.
    let filter = scope_of(step)?;

    // Within-line selectors transform a substring of each line, not line spans.
    if scope.field.is_some() || scope.char_range.is_some() {
        if scope.invert {
            bail!("scope: 'invert' is not supported with 'field' / 'char_range'");
        }
        return apply_within_line(action_name, step, working, ctx, scope, filter.as_ref());
    }

    let s = split(working);
    let n = s.lines.len();
    if n == 0 {
        return Ok(working.to_string());
    }
    let mut spans = scope_spans(scope, &s.lines)?;
    if scope.invert {
        spans = complement_spans(&spans, n);
    }
    // Intersect the scope-selected line spans with the line filter: split each
    // span into maximal runs of filter-passing lines; filter-failing lines drop
    // out of the transformed set and pass through unchanged.
    if let Some((re, keep_when_match)) = &filter {
        spans = intersect_spans_with_filter(&spans, &s.lines, re, *keep_when_match)?;
    }
    if spans.is_empty() {
        return Ok(join(&s.lines, s.eol, s.trailing_eol));
    }
    let start_to_end: BTreeMap<usize, usize> = spans.iter().map(|&(a, b)| (a, b)).collect();
    let mut out: Vec<String> = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if let Some(&end) = start_to_end.get(&i) {
            let span_text = join(&s.lines[i..=end], s.eol, true);
            let transformed = run_action_on(action_name, step, &span_text, ctx)?;
            out.extend(split(&transformed).lines);
            i = end + 1;
        } else {
            out.push(s.lines[i].clone());
            i += 1;
        }
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Resolve a scope to sorted, non-overlapping, inclusive 0-based line spans.
fn scope_spans(scope: &Scope, lines: &[String]) -> Result<Vec<(usize, usize)>> {
    match (&scope.in_block, &scope.line_range) {
        (Some(_), Some(_)) => bail!("scope: set only one of 'in_block' / 'line_range'"),
        (Some(b), None) => block_spans(b, lines),
        (None, Some(r)) => Ok(line_range_span(r, lines.len())),
        (None, None) => Ok(vec![(0, lines.len() - 1)]),
    }
}

/// Spans for each `start`..`end` region (end searched on a later line, matching
/// `for_each_block`). An unterminated final block yields no span.
fn block_spans(b: &BlockScope, lines: &[String]) -> Result<Vec<(usize, usize)>> {
    let start_re = build_scope_regex(&b.start, b.ignore_case)?;
    let end_re = build_scope_regex(&b.end, b.ignore_case)?;
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if start_re.is_match(&lines[i]).map_err(|e| anyhow!(e))? {
            let mut k = i + 1;
            let mut end = None;
            while k < lines.len() {
                if end_re.is_match(&lines[k]).map_err(|e| anyhow!(e))? {
                    end = Some(k);
                    break;
                }
                k += 1;
            }
            match end {
                Some(endk) => {
                    spans.push((i, endk));
                    i = endk + 1;
                }
                None => i += 1,
            }
        } else {
            i += 1;
        }
    }
    Ok(spans)
}

/// A single span from a 1-based inclusive line range (negatives count from the
/// end). Returns empty if the range is empty after clamping.
fn line_range_span(r: &LineRange, n: usize) -> Vec<(usize, usize)> {
    if n == 0 {
        return Vec::new();
    }
    let ni = n as i64;
    let resolve = |v: i64| -> i64 {
        if v < 0 {
            ni + v + 1
        } else {
            v
        }
    };
    let from = r.from.map(resolve).unwrap_or(1).max(1);
    let to = r.to.map(resolve).unwrap_or(ni).min(ni);
    if from > to {
        return Vec::new();
    }
    vec![((from - 1) as usize, (to - 1) as usize)]
}

/// The complement of sorted, non-overlapping inclusive spans over `[0, n)`.
fn complement_spans(spans: &[(usize, usize)], n: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    for &(a, b) in spans {
        if a > cursor {
            out.push((cursor, a - 1));
        }
        cursor = b + 1;
    }
    if n > 0 && cursor < n {
        out.push((cursor, n - 1));
    }
    out
}

/// Intersect sorted, non-overlapping inclusive spans with a per-line filter.
/// Each input span is split into the maximal runs of consecutive lines that pass
/// the filter (`is_match == keep_when_match`); filter-failing lines are excluded,
/// so they fall out of the transformed set and pass through unchanged. The result
/// stays sorted and non-overlapping (spans are only ever subdivided).
fn intersect_spans_with_filter(
    spans: &[(usize, usize)],
    lines: &[String],
    re: &Regex,
    keep_when_match: bool,
) -> Result<Vec<(usize, usize)>> {
    let mut out = Vec::new();
    for &(a, b) in spans {
        let mut run_start: Option<usize> = None;
        for (offset, line) in lines[a..=b].iter().enumerate() {
            let i = a + offset;
            let passes = re.is_match(line).map_err(|e| anyhow!(e))? == keep_when_match;
            if passes {
                if run_start.is_none() {
                    run_start = Some(i);
                }
            } else if let Some(start) = run_start.take() {
                out.push((start, i - 1));
            }
        }
        if let Some(start) = run_start.take() {
            out.push((start, b));
        }
    }
    Ok(out)
}

/// Apply the action to a substring of EACH line (a `field` or `char_range`
/// selector), splicing the transformed piece back into the line. The rest of each
/// line is preserved exactly.
fn apply_within_line(
    action_name: &str,
    step: &Step,
    working: &str,
    ctx: &ExecCtx,
    scope: &Scope,
    filter: Option<&(Regex, bool)>,
) -> Result<String> {
    let s = split(working);
    let mut out: Vec<String> = Vec::with_capacity(s.lines.len());
    for line in &s.lines {
        // Compose with the line filter: a line that fails the filter passes
        // through untouched (its field / char_range is not transformed).
        let passes = match filter {
            Some((re, keep_when_match)) => {
                re.is_match(line).map_err(|e| anyhow!(e))? == *keep_when_match
            }
            None => true,
        };
        let replaced = if !passes {
            line.clone()
        } else if let Some(f) = &scope.field {
            apply_to_field(action_name, step, line, ctx, f)?
        } else if let Some(c) = &scope.char_range {
            apply_to_char_range(action_name, step, line, ctx, c)?
        } else {
            line.clone()
        };
        out.push(replaced);
    }
    Ok(join(&out, s.eol, s.trailing_eol))
}

/// Run the action on a within-line fragment (a one-line document) and drop a
/// single trailing EOL the action may have appended, so it splices cleanly.
fn transform_fragment(
    action_name: &str,
    step: &Step,
    fragment: &str,
    ctx: &ExecCtx,
) -> Result<String> {
    let out = run_action_on(action_name, step, fragment, ctx)?;
    let out = out.strip_suffix('\n').unwrap_or(&out);
    let out = out.strip_suffix('\r').unwrap_or(out);
    Ok(out.to_string())
}

/// Resolve a 1-based (negative-from-end) index against a count, to a 0-based index
/// in range, or None when out of range.
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

/// Transform only field `index` of a line, split on the delimiter (default `,`).
/// Everything else, including the delimiters, is preserved. A line with too few
/// fields (or index 0) passes through unchanged.
fn apply_to_field(
    action_name: &str,
    step: &Step,
    line: &str,
    ctx: &ExecCtx,
    f: &FieldScope,
) -> Result<String> {
    let delim = f.delimiter.clone().unwrap_or_else(|| ",".to_string());
    if delim.is_empty() {
        bail!("scope field: 'delimiter' must not be empty");
    }
    let parts: Vec<&str> = line.split(delim.as_str()).collect();
    let Some(i) = resolve_1based(f.index, parts.len()) else {
        return Ok(line.to_string());
    };
    let transformed = transform_fragment(action_name, step, parts[i], ctx)?;
    let mut new_parts: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
    new_parts[i] = transformed;
    Ok(new_parts.join(delim.as_str()))
}

/// Transform only a 1-based inclusive character range of a line (negatives count
/// from the end). An empty or reversed range leaves the line unchanged.
fn apply_to_char_range(
    action_name: &str,
    step: &Step,
    line: &str,
    ctx: &ExecCtx,
    c: &CharRange,
) -> Result<String> {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    if n == 0 {
        return Ok(line.to_string());
    }
    let ni = n as i64;
    let resolve = |v: i64| -> i64 {
        if v < 0 {
            ni + v + 1
        } else {
            v
        }
    };
    let from = c.from.map(resolve).unwrap_or(1).max(1);
    let to = c.to.map(resolve).unwrap_or(ni).min(ni);
    if from > to {
        return Ok(line.to_string());
    }
    let (a, b) = ((from - 1) as usize, (to - 1) as usize);
    let prefix: String = chars[..a].iter().collect();
    let target: String = chars[a..=b].iter().collect();
    let suffix: String = chars[b + 1..].iter().collect();
    let transformed = transform_fragment(action_name, step, &target, ctx)?;
    Ok(format!("{prefix}{transformed}{suffix}"))
}

/// `run_mog`: load another .mog file (resolved by [`crate::library::resolve_script`]
/// absolute -> base-relative -> library) and run its steps inline over `input`.
fn run_mog(input: &str, step: &Step, ctx: &ExecCtx) -> Result<String> {
    if ctx.depth >= MAX_RUN_MOG_DEPTH {
        bail!("run_mog recursion limit ({MAX_RUN_MOG_DEPTH}) exceeded; check for a cycle");
    }
    let file = step
        .get_string("file")
        .ok_or_else(|| anyhow!("run_mog requires a 'file' option"))?;
    let file_path = Path::new(&file);
    crate::library::ensure_confined(file_path)?;
    let path = crate::library::resolve_script(
        file_path,
        ctx.base_dir.as_deref(),
        ctx.lib_root.as_deref(),
    )?;
    // `with: { name: value }` passes constants into the child: they overlay (and
    // override) the child's own `constants`, so a shared fragment can be
    // parameterized by its caller. Values must be scalars (string/number/bool).
    let defines = with_defines(step)?;
    let child = crate::load_mog_file_with_defines(&path, &defines)?;
    let child_ctx = ExecCtx {
        base_dir: path.parent().map(|p| p.to_path_buf()),
        lib_root: ctx.lib_root.clone(),
        depth: ctx.depth + 1,
        sources: ctx.sources.clone(),
        on_step: None, // nested sub-steps stay silent (would confuse the i/N count)
        seed: ctx.seed,
    };
    execute_inner(&child, input, &child_ctx)
}

/// Parse a `run_mog` step's optional `with` map into constant overrides for the
/// child recipe. Absent -> empty. A non-object `with`, or a non-scalar value,
/// is a hard error.
fn with_defines(step: &Step) -> Result<std::collections::BTreeMap<String, String>> {
    let mut out = std::collections::BTreeMap::new();
    match step.options.get("with") {
        None => Ok(out),
        Some(serde_json::Value::Object(map)) => {
            for (k, v) in map {
                out.insert(k.clone(), crate::interpolate::scalar_to_string(k, v)?);
            }
            Ok(out)
        }
        Some(_) => bail!("run_mog 'with' must be an object of name -> scalar value"),
    }
}

/// `fill_from_list`: replace each occurrence of a marker (`find`, default "?")
/// with the next value from a named external source (a sequential cursor). This
/// is the mail-merge / placeholder-fill primitive. `source` names a loaded source
/// (see [`crate::model::Mog::sources`] / `--source`); `on_exhausted` = leave
/// (default, keep the marker) / blank / wrap (cycle) / error.
fn fill_from_list(input: &str, step: &Step, ctx: &ExecCtx) -> Result<String> {
    let source = step
        .get_string("source")
        .ok_or_else(|| anyhow!("fill_from_list requires a 'source' option"))?;
    let find = step.get_string_or("find", "?");
    if find.is_empty() {
        return Ok(input.to_string());
    }
    let on_exhausted = step.get_enum(
        "on_exhausted",
        "fill_from_list",
        "leave",
        &["leave", "blank", "wrap", "error"],
    )?;
    let values = ctx.sources.get(&source).ok_or_else(|| {
        anyhow!(
            "fill_from_list: unknown source '{source}' \
             (bind it with --source {source}=<path> or a 'sources' entry)"
        )
    })?;

    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    let mut i = 0usize;
    while let Some(idx) = rest.find(find.as_str()) {
        out.push_str(&rest[..idx]);
        let val = if i < values.len() {
            Some(values[i].as_str())
        } else if on_exhausted == "wrap" && !values.is_empty() {
            Some(values[i % values.len()].as_str())
        } else {
            None
        };
        match val {
            Some(v) => out.push_str(v),
            None => match on_exhausted.as_str() {
                "blank" => {}
                "error" => bail!(
                    "fill_from_list: source '{source}' exhausted after {} value(s)",
                    values.len()
                ),
                _ => out.push_str(&find), // "leave": keep the marker as-is
            },
        }
        rest = &rest[idx + find.len()..];
        i += 1;
    }
    out.push_str(rest);
    Ok(out)
}

/// `for_each_block`: for each `block_start`..`block_end` region, bind the header
/// line's captures as constants (`bind`) and run a sub-.mog (`run`) over the
/// block, replacing it with the result. This is how a stateless pipeline carries
/// header context (e.g. a table name) into per-block edits.
fn for_each_block(input: &str, step: &Step, ctx: &ExecCtx) -> Result<String> {
    if ctx.depth >= MAX_RUN_MOG_DEPTH {
        bail!("for_each_block recursion limit ({MAX_RUN_MOG_DEPTH}) exceeded; check for a cycle");
    }
    let ic = step.get_bool("ignore_case", false)?;
    let start_re = compile_block_regex(step, "block_start", ic)?;
    let end_re = compile_block_regex(step, "block_end", ic)?;
    let file = step
        .get_string("run")
        .ok_or_else(|| anyhow!("for_each_block requires a 'run' option"))?;
    let file_path = Path::new(&file);
    crate::library::ensure_confined(file_path)?;
    let path = crate::library::resolve_script(
        file_path,
        ctx.base_dir.as_deref(),
        ctx.lib_root.as_deref(),
    )?;
    let child_text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read for_each_block script '{}'", path.display()))?;
    let child_ctx = ExecCtx {
        base_dir: path.parent().map(|p| p.to_path_buf()),
        lib_root: ctx.lib_root.clone(),
        depth: ctx.depth + 1,
        sources: ctx.sources.clone(),
        on_step: None, // nested sub-steps stay silent (would confuse the i/N count)
        seed: ctx.seed,
    };
    let bind = step.options.get("bind").and_then(|v| v.as_object());

    let s = split(input);
    let lines = &s.lines;
    let mut out: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let start_caps = start_re.captures(&lines[i]).map_err(|e| anyhow!(e))?;
        let Some(caps) = start_caps else {
            out.push(lines[i].clone());
            i += 1;
            continue;
        };
        // Find the block end at a later line.
        let mut end = None;
        let mut k = i + 1;
        while k < lines.len() {
            if end_re.is_match(&lines[k]).map_err(|e| anyhow!(e))? {
                end = Some(k);
                break;
            }
            k += 1;
        }
        let Some(endk) = end else {
            out.push(lines[i].clone());
            i += 1;
            continue;
        };

        // Bind header captures as constants for the sub-script.
        let header = crate::actions::block::group_values(&caps);
        let mut defines: BTreeMap<String, String> = BTreeMap::new();
        if let Some(map) = bind {
            for (name, tmpl) in map {
                if let Some(t) = tmpl.as_str() {
                    defines.insert(name.clone(), crate::actions::block::expand_caps(t, &header));
                }
            }
        }
        let child = crate::parse_mog_with_defines(&child_text, &defines)
            .map_err(|e| anyhow!("in '{}': {e}", path.display()))?;

        // Run the sub-.mog over the (inclusive) block, splice the result back.
        let block_input = join(&lines[i..=endk], s.eol, true);
        let transformed = execute_inner(&child, &block_input, &child_ctx)?;
        out.extend(split(&transformed).lines);
        i = endk + 1;
    }

    Ok(join(&out, s.eol, s.trailing_eol))
}

fn compile_block_regex(step: &Step, key: &str, ignore_case: bool) -> Result<Regex> {
    let pat = step
        .get_string(key)
        .ok_or_else(|| anyhow!("for_each_block requires a '{key}' option"))?;
    let full = if ignore_case {
        format!("(?i){pat}")
    } else {
        pat.clone()
    };
    Regex::new(&full).map_err(|e| anyhow!("invalid '{key}' pattern '{pat}': {e}"))
}

#[cfg(test)]
mod scope_tests {
    use crate::parse_mog;

    fn run(json: &str, input: &str) -> String {
        crate::execute(&parse_mog(json).unwrap(), input).unwrap()
    }

    #[test]
    fn in_block_uppercases_only_inside_blocks() {
        // Uppercase lines only within each BEGIN..END block; outside passes through.
        // The block span includes its BEGIN/END boundary lines (already caps here).
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"in_block":{"start":"^BEGIN","end":"^END"}}}]}"#,
            "keep\nBEGIN\nfoo\nEND\nkeep2\nBEGIN\nbar\nEND",
        );
        assert_eq!(out, "keep\nBEGIN\nFOO\nEND\nkeep2\nBEGIN\nBAR\nEND");
    }

    #[test]
    fn in_block_dedupe_is_block_local() {
        // remove_duplicate_lines runs once per block (unique BEGIN/END boundaries
        // are unaffected).
        let out = run(
            r#"{"steps":[{"action":"remove_duplicate_lines","scope":{"in_block":{"start":"^BEGIN","end":"^END"}}}]}"#,
            "BEGIN\na\na\nb\nEND",
        );
        assert_eq!(out, "BEGIN\na\nb\nEND");
    }

    #[test]
    fn line_range_sort_is_range_local() {
        // Sort only lines 2..4; the surrounding lines keep their place.
        let out = run(
            r#"{"steps":[{"action":"sort_lines","scope":{"line_range":{"from":2,"to":4}}}]}"#,
            "z\nc\na\nb\nz",
        );
        assert_eq!(out, "z\na\nb\nc\nz");
    }

    #[test]
    fn line_range_scopes_to_rows() {
        // Uppercase only lines 2..3 (1-based inclusive).
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"line_range":{"from":2,"to":3}}}]}"#,
            "a\nb\nc\nd",
        );
        assert_eq!(out, "a\nB\nC\nd");
    }

    #[test]
    fn line_range_negative_from_end() {
        // from -1 => last line only.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"line_range":{"from":-1}}}]}"#,
            "a\nb\nc",
        );
        assert_eq!(out, "a\nb\nC");
    }

    #[test]
    fn invert_applies_to_complement() {
        // Invert a line_range: uppercase everything EXCEPT line 1.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"line_range":{"from":1,"to":1},"invert":true}}]}"#,
            "a\nb\nc",
        );
        assert_eq!(out, "a\nB\nC");
    }

    #[test]
    fn scope_conflict_errors() {
        let m = parse_mog(
            r#"{"steps":[{"action":"to_upper","scope":{"in_block":{"start":"a","end":"b"},"line_range":{"from":1}}}]}"#,
        )
        .unwrap();
        assert!(crate::execute(&m, "x").is_err());
    }

    #[test]
    fn field_scope_transforms_one_column() {
        // Uppercase only field 2 (comma-delimited); the rest is preserved exactly.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":2}}}]}"#,
            "a,b,c\nx,y,z",
        );
        assert_eq!(out, "a,B,c\nx,Y,z");
    }

    #[test]
    fn field_scope_custom_delimiter_and_negative_index() {
        // Last field, pipe-delimited.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":-1,"delimiter":"|"}}}]}"#,
            "a|b|c\nx|y|z",
        );
        assert_eq!(out, "a|b|C\nx|y|Z");
    }

    #[test]
    fn field_scope_out_of_range_leaves_line() {
        // A line with fewer fields than `index` passes through unchanged.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":3}}}]}"#,
            "a,b,c\nonly-one",
        );
        assert_eq!(out, "a,b,C\nonly-one");
    }

    #[test]
    fn char_range_transforms_slice_of_each_line() {
        // Uppercase characters 1..3 (1-based inclusive) of each line.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"char_range":{"from":1,"to":3}}}]}"#,
            "hello\nabcdef",
        );
        assert_eq!(out, "HELlo\nABCdef");
    }

    #[test]
    fn char_range_negative_and_unicode_safe() {
        // to -1 with default from => whole line; negative indices count from end.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"char_range":{"from":-2}}}]}"#,
            "café\nsnow",
        );
        // Last two chars of each line uppercased (é is one char).
        assert_eq!(out, "caFÉ\nsnOW");
    }

    #[test]
    fn field_and_char_range_conflict_errors() {
        let m = parse_mog(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":1},"char_range":{"from":1}}}]}"#,
        )
        .unwrap();
        assert!(crate::execute(&m, "x").is_err());
    }

    #[test]
    fn invert_with_within_line_scope_errors() {
        let m = parse_mog(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":1},"invert":true}}]}"#,
        )
        .unwrap();
        assert!(crate::execute(&m, "a,b").is_err());
    }

    // --- Scope + line-filter composition (intersection) ---------------------

    #[test]
    fn field_scope_composes_with_only_lines_matching() {
        // Field 1 is uppercased ONLY on lines that pass the filter (a KEY=VALUE
        // line starting with an alnum char). The comment and blank lines fail the
        // filter, so their field is NOT transformed; they pass through unchanged.
        let out = run(
            r#"{"steps":[{"action":"to_upper","only_lines_matching":"^[A-Za-z0-9].*=","scope":{"field":{"index":1,"delimiter":"="}}}]}"#,
            "# a comment\n\nkey=value\nother=x",
        );
        assert_eq!(out, "# a comment\n\nKEY=value\nOTHER=x");
    }

    #[test]
    fn field_scope_composes_with_except_lines_matching() {
        // except_lines_matching: transform field 1 on every line EXCEPT those the
        // pattern matches (here, comment lines starting with '#').
        let out = run(
            r#"{"steps":[{"action":"to_upper","except_lines_matching":"^#","scope":{"field":{"index":1,"delimiter":"="}}}]}"#,
            "#skip=me\nkeep=this",
        );
        assert_eq!(out, "#skip=me\nKEEP=this");
    }

    #[test]
    fn line_range_composes_with_filter_intersection() {
        // Uppercase whole lines only where the line_range (2..5) AND the filter
        // (line contains 'x') both hold. Line 1 (out of range) and lines without
        // 'x' inside the range are left untouched.
        let out = run(
            r#"{"steps":[{"action":"to_upper","only_lines_matching":"x","scope":{"line_range":{"from":2,"to":5}}}]}"#,
            "x1\nx2\ny3\nx4\ny5\nx6",
        );
        // line 1 (x1): out of range -> unchanged. line 2 (x2): in range + match ->
        // X2. line 3 (y3): no match -> unchanged. line 4 (x4): in range + match ->
        // X4. line 5 (y5): no match -> unchanged. line 6 (x6): out of range.
        assert_eq!(out, "x1\nX2\ny3\nX4\ny5\nx6");
    }

    #[test]
    fn in_block_composes_with_filter_intersection() {
        // Uppercase only lines that are BOTH inside a BEGIN..END block AND pass the
        // filter (contain 'a'). 'foo' inside the block has no 'a' -> untouched;
        // 'bar' does -> uppercased. Outside-block lines never qualify.
        let out = run(
            r#"{"steps":[{"action":"to_upper","only_lines_matching":"a","scope":{"in_block":{"start":"^BEGIN","end":"^END"}}}]}"#,
            "cat\nBEGIN\nfoo\nbar\nEND\ncap",
        );
        assert_eq!(out, "cat\nBEGIN\nfoo\nBAR\nEND\ncap");
    }

    #[test]
    fn in_block_invert_composes_with_filter_intersection() {
        // invert selects the complement of the block set; the filter then narrows
        // it. Uppercase lines OUTSIDE the block that contain 'a'. Inside-block
        // lines are excluded by invert even if they contain 'a'.
        let out = run(
            r#"{"steps":[{"action":"to_upper","only_lines_matching":"a","scope":{"in_block":{"start":"^BEGIN","end":"^END"},"invert":true}}]}"#,
            "cat\nBEGIN\nbar\nEND\ncap\ndog",
        );
        // cat -> CAT (outside + 'a'); BEGIN/bar/END inside block -> untouched;
        // cap -> CAP (outside + 'a'); dog -> unchanged (no 'a').
        assert_eq!(out, "CAT\nBEGIN\nbar\nEND\nCAP\ndog");
    }

    #[test]
    fn scope_without_filter_is_unchanged() {
        // A structured scope with NO line filter must behave exactly as before:
        // every field-1 value is uppercased, including the comment line's.
        let out = run(
            r#"{"steps":[{"action":"to_upper","scope":{"field":{"index":1,"delimiter":"="}}}]}"#,
            "#comment=x\nkey=value",
        );
        assert_eq!(out, "#COMMENT=x\nKEY=value");
    }
}

#[cfg(test)]
mod uuid_seed_tests {
    use super::*;
    use crate::parse_mog;

    fn run_seeded(json: &str, input: &str, seed: Option<u64>) -> String {
        let m = parse_mog(json).unwrap();
        execute_with_library_sources_observed(
            &m,
            input,
            None,
            None,
            Arc::new(BTreeMap::new()),
            None,
            seed,
        )
        .unwrap()
    }

    #[test]
    fn pinned_seed_threads_from_exec_context_to_per_match_uuid() {
        // The seed the CLI pins (--pin-seed / mog --test) reaches the $uuid token
        // through the exec context, so the same input+recipe is byte-identical.
        let json = r#"{"steps":[{"action":"replace_regex","options":{"find":"x","replace_with":"$uuid"}}]}"#;
        let a = run_seeded(json, "x x x", Some(0));
        let b = run_seeded(json, "x x x", Some(0));
        assert_eq!(a, b, "seed pinned via exec context must be reproducible");
        let ids: std::collections::HashSet<&str> = a.split(' ').collect();
        assert_eq!(ids.len(), 3, "each match gets a fresh uuid: {a}");
        // First per-match uuid equals {{@uuid}} under the same seed.
        assert!(a.starts_with(&crate::builtins::make_uuid(Some(0))), "{a}");
    }

    #[test]
    fn unseeded_exec_is_random_but_valid_shape() {
        let json = r#"{"steps":[{"action":"replace_regex_multiline","options":{"find":"x","replace_with":"$uuid"}}]}"#;
        let a = run_seeded(json, "x", None);
        assert_eq!(a.len(), 36, "{a}");
        assert_eq!(a.matches('-').count(), 4, "{a}");
    }

    #[test]
    fn seed_propagates_into_run_mog_children() {
        // A nested recipe's $uuid is pinned by the parent run's seed too.
        let json = r#"{"steps":[{"action":"replace_regex","options":{"find":"x","replace_with":"$uuid"}}]}"#;
        let a = run_seeded(json, "x", Some(7));
        let b = run_seeded(json, "x", Some(7));
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod sources_tests {
    use super::*;
    use crate::parse_mog;

    fn src(pairs: &[(&str, &[&str])]) -> Arc<BTreeMap<String, Vec<String>>> {
        let mut m = BTreeMap::new();
        for (name, vals) in pairs {
            m.insert(
                name.to_string(),
                vals.iter().map(|s| s.to_string()).collect(),
            );
        }
        Arc::new(m)
    }

    fn run(json: &str, input: &str, sources: Arc<BTreeMap<String, Vec<String>>>) -> Result<String> {
        let m = parse_mog(json).unwrap();
        execute_with_library_sources(&m, input, None, None, sources)
    }

    #[test]
    fn fill_from_list_sequential() {
        let out = run(
            r#"{"steps":[{"action":"fill_from_list","options":{"source":"names","find":"<N>"}}]}"#,
            "Dear <N>,\nDear <N>,",
            src(&[("names", &["Ada", "Bob"])]),
        )
        .unwrap();
        assert_eq!(out, "Dear Ada,\nDear Bob,");
    }

    #[test]
    fn fill_from_list_leave_when_exhausted() {
        let out = run(
            r#"{"steps":[{"action":"fill_from_list","options":{"source":"n","find":"?"}}]}"#,
            "? ? ?",
            src(&[("n", &["a", "b"])]),
        )
        .unwrap();
        assert_eq!(out, "a b ?");
    }

    #[test]
    fn fill_from_list_wrap() {
        let out = run(
            r#"{"steps":[{"action":"fill_from_list","options":{"source":"n","find":"?","on_exhausted":"wrap"}}]}"#,
            "? ? ?",
            src(&[("n", &["a", "b"])]),
        )
        .unwrap();
        assert_eq!(out, "a b a");
    }

    #[test]
    fn fill_from_list_unknown_source_errors() {
        assert!(run(
            r#"{"steps":[{"action":"fill_from_list","options":{"source":"missing","find":"?"}}]}"#,
            "?",
            src(&[]),
        )
        .is_err());
    }
}
