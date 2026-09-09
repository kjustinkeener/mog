//! mog: a CLI batch text-processing tool. Rust port of the .NET LibMog engine.

pub mod actions;
pub mod builtins;
pub mod config;
pub(crate) mod datetime;
pub mod descriptors;
pub mod docgen;
pub mod encoding;
pub mod engine;
pub(crate) mod factory_pack;
pub mod install;
pub mod interpolate;
pub mod library;
pub(crate) mod line_text;
pub mod market;
pub mod market_client;
pub mod market_index;
pub mod mcp;
pub mod model;
pub mod report;
pub mod selfupdate;
pub mod setup;
pub mod sources;
pub mod stream;
pub mod testkit;
pub mod updater;

use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};

pub use descriptors::{
    categories, categories_json, complexity_of, descriptors, descriptors_json, ActionDescriptor,
    CategoryInfo, Complexity,
};
pub use engine::{
    execute, execute_at, execute_guarded, execute_guarded_with_sources,
    execute_guarded_with_sources_observed, execute_with_library, execute_with_library_sources,
    execute_with_library_sources_observed, RunLimits, StepObserver, StepProgress,
};
pub use model::{Mog, Step, Tier};

/// Parse a .mog script from JSON text and expand its `{{name}}` constant
/// placeholders (from the script's own `constants` block).
///
/// `.mog` files are strict JSON: comments and trailing commas are a parse error.
/// Use the first-class `description` fields (on the pipeline and each step) for
/// human annotation.
pub fn parse_mog(json: &str) -> Result<Mog> {
    parse_mog_with_defines(json, &BTreeMap::new())
}

/// Like [`parse_mog`], but `defines` override (or add to) the script's own
/// `constants` when expanding `{{name}}` placeholders (CLI `--define`).
pub fn parse_mog_with_defines(json: &str, defines: &BTreeMap<String, String>) -> Result<Mog> {
    // Inline the serde message (not just a generic context) so a rejected unknown
    // key surfaces "unknown field `actions`, expected one of ... `steps`" -- the
    // detail that tells the author what to fix.
    let mut mog: Mog =
        serde_json::from_str(json).map_err(|e| anyhow!("failed to parse .mog JSON: {e}"))?;
    interpolate::interpolate(&mut mog, defines)?;
    validate_mog(&mog)?;
    Ok(mog)
}

/// Actions whose options are free-form / undeclared in descriptors (the engine
/// built-ins and the block hoister). Option-key validation is skipped for these;
/// every other action's option keys must belong to its descriptor.
const FREEFORM_OPTION_ACTIONS: &[&str] = &["run_mog", "for_each_block", "hoist_from_block"];

/// Structural validation beyond serde parsing: a pipeline must have at least one
/// step, and each step's option keys must actually belong to its action. This
/// turns two silent-failure traps into hard errors: an empty pipeline (e.g. from
/// a mistyped top-level key) that "succeeds" while changing nothing, and a
/// mistyped option key (e.g. `replace` instead of `replace_with`) that is ignored
/// and produces wrong output.
fn validate_mog(mog: &Mog) -> Result<()> {
    if mog.steps.is_empty() {
        bail!("a .mog needs at least one step, but 'steps' is missing or empty");
    }
    for (i, step) in mog.steps.iter().enumerate() {
        let action = match &step.action {
            Some(a) => a,
            None => continue, // an actionless step is reported at execute time
        };
        if FREEFORM_OPTION_ACTIONS.contains(&action.as_str()) {
            continue;
        }
        // Unknown action names (and aliases we don't index) get no descriptor;
        // leave those to execute-time resolution rather than false-flag them.
        if let Some(desc) = descriptors::find_descriptor(action) {
            let valid: HashSet<&str> = desc.params.iter().map(|p| p.key).collect();
            for key in step.options.keys() {
                if !valid.contains(key.as_str()) {
                    let mut known: Vec<&str> = valid.iter().copied().collect();
                    known.sort_unstable();
                    let known = if known.is_empty() {
                        "(this action takes no options)".to_string()
                    } else {
                        format!("valid options: {}", known.join(", "))
                    };
                    bail!(
                        "step {}: action '{}' has no option '{}' ({})",
                        i + 1,
                        action,
                        key,
                        known
                    );
                }
            }
        }
    }
    Ok(())
}

/// Load and parse a .mog script from a file path.
pub fn load_mog_file(path: &std::path::Path) -> Result<Mog> {
    load_mog_file_with_defines(path, &BTreeMap::new())
}

/// Like [`load_mog_file`], but with CLI `--define` constant overrides.
pub fn load_mog_file_with_defines(
    path: &std::path::Path,
    defines: &BTreeMap<String, String>,
) -> Result<Mog> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read script '{}'", path.display()))?;
    parse_mog_with_defines(&text, defines).map_err(|e| anyhow!("in '{}': {e}", path.display()))
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn rejects_unknown_top_level_key() {
        // `actions` instead of `steps` used to deserialize to an empty pipeline
        // that ran "successfully" and changed nothing.
        let err = parse_mog(r#"{"name":"x","actions":[{"action":"to_upper"}]}"#)
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown field"), "got: {err}");
        assert!(err.contains("steps"), "should point at the real key: {err}");
    }

    #[test]
    fn rejects_unknown_step_key() {
        // A mistyped step-level key is a hard error, not silently ignored.
        assert!(parse_mog(r#"{"steps":[{"actn":"to_upper"}]}"#).is_err());
    }

    #[test]
    fn rejects_empty_pipeline() {
        let err = parse_mog(r#"{"steps":[]}"#).unwrap_err().to_string();
        assert!(err.contains("at least one step"), "got: {err}");
        // A missing steps key (defaults to empty) is likewise rejected.
        assert!(parse_mog(r#"{"name":"x"}"#).is_err());
    }

    #[test]
    fn rejects_unknown_option_key() {
        let err =
            parse_mog(r#"{"steps":[{"action":"replace","options":{"find":"a","replace":"X"}}]}"#)
                .unwrap_err()
                .to_string();
        assert!(err.contains("has no option 'replace'"), "got: {err}");
        assert!(
            err.contains("replace_with"),
            "should list valid options: {err}"
        );
    }

    #[test]
    fn accepts_valid_pipeline() {
        let m = parse_mog(
            r#"{"steps":[{"action":"to_upper"},{"action":"replace","options":{"find":"a","replace_with":"b"}}]}"#,
        )
        .unwrap();
        assert_eq!(m.steps.len(), 2);
    }

    #[test]
    fn exempts_freeform_compose_actions_from_option_validation() {
        // run_mog / for_each_block / hoist_from_block options are undeclared in
        // descriptors; option-key validation must not reject them.
        parse_mog(r#"{"steps":[{"action":"run_mog","options":{"file":"tidy.mog"}}]}"#)
            .expect("run_mog options must not be strict-validated");
    }
}
