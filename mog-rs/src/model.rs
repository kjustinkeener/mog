//! Data model for a .mog script plus the tolerant option accessors that mirror
//! the .NET `Step.cs` behavior (GetString / GetBool / GetInt).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The store tier of a script. `core` is
/// the small curated set shown by `mog ls` by default; `full` is everything and
/// is opt-in. Serializes as the lowercase strings `"core"` / `"full"`; defaults
/// to `Full` so an omitted field parses as `full`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,
    #[default]
    Full,
}

impl Tier {
    /// Used by `skip_serializing_if` so the common `full` default round-trips
    /// without writing the field back out.
    pub fn is_full(&self) -> bool {
        matches!(self, Tier::Full)
    }

    /// The lowercase wire label (`"core"` / `"full"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Core => "core",
            Tier::Full => "full",
        }
    }
}

/// Top-level .mog document: `{ name?, description?, tags?, tier?, constants?, steps: [...] }`.
/// `deny_unknown_fields` makes a mistyped top-level key a hard parse error rather
/// than a silent no-op -- e.g. `{"actions":[...]}` (should be `steps`) used to
/// deserialize to an empty pipeline that ran successfully and changed nothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mog {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Short human-facing browse blurb: one line, shown in the terse human
    /// `mog market list`/`search` row (the Studio-style browse view). Distinct from
    /// the long, verbose, agent-oriented `description`: `summary` is what a person
    /// skims a listing by, `description` is what an agent (`--json`/MCP) reads in
    /// full. When absent, the human browse row falls back to a truncated first
    /// sentence of `description`; the `--json`/MCP channel always carries the full
    /// `description` regardless.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Natural-language task phrases that describe, in a user's own words, the job
    /// this mog does ("strip ANSI colour codes from a log", "convert Oracle SQL
    /// to Postgres"). Purely a discovery-index field: `mog market search` weights
    /// these highly so a mog surfaces for the phrasing a person (or an agent
    /// relaying one) would actually type, without polluting the terse `summary` row
    /// or the agent-facing `description`. Optional and additive; an omitted or empty
    /// list is not written back out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub task_phrases: Vec<String>,
    /// Store keywords for filtering (`sql`, `redaction`, `codemod`). Optional and
    /// additive: an omitted field is an empty list, and an empty list is not
    /// written back out. Used by `mog ls --tag` and the Studio filters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Other store mog names this mog composes (via `run_mog` /
    /// `for_each_block`). `mog store install` resolves and installs these
    /// transitively. Names only, never paths (engine hardening H1 confines
    /// composition), so a marketplace mog stays portable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Store tier (`core` / `full`, default `full`). Author intent that travels
    /// with the script through copy/`add`/marketplace. Drives the compact default
    /// listing (`mog ls`).
    #[serde(default, skip_serializing_if = "Tier::is_full")]
    pub tier: Tier,
    /// Named string constants. Any `{{name}}` placeholder in a step's option
    /// strings (or line-scope patterns) is expanded from this map at load time,
    /// before any action runs. Values may be strings, numbers, or bools (all
    /// stringified). Overridable per-run via the CLI `--define name=value`.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub constants: Map<String, Value>,
    /// Named external data sources: `name -> path`. The CLI loads these (a
    /// mog-declared path is confined to the .mog's directory; `--source
    /// name=path` binds any path and overrides a same-named declaration). Consumed
    /// by source-aware actions such as `fill_from_list`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sources: BTreeMap<String, String>,
    /// Output encoding for this pipeline: "preserve" (match the input), "utf-8",
    /// "utf-8-bom", "utf-16le", "utf-16be", "ansi"/"windows-1252", or another
    /// label. A mog that must emit a specific encoding (e.g. a Windows tool that
    /// requires UTF-8 BOM) declares it here so callers need not remember the flag.
    /// The CLI `--output-encoding`, when given explicitly, overrides this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_encoding: Option<String>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

/// A single pipeline step. `deny_unknown_fields` catches mistyped step keys
/// (action-specific options live inside the `options` map, which is validated
/// separately against the action's descriptor at load time).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional section header: a label that begins a named section grouping this
    /// step and the following ones (until the next `section`) in the Studio outline.
    /// Pure metadata; ignored by execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
    /// Scope: apply this step's action ONLY to lines matching this regex.
    /// Each matching line is transformed on its own (as a one-line document);
    /// non-matching lines pass through unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_lines_matching: Option<String>,
    /// Scope: apply this step's action only to lines NOT matching this regex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub except_lines_matching: Option<String>,
    /// Case-insensitive matching for `only_lines_matching` / `except_lines_matching`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub match_ignore_case: bool,
    /// Structured scope: run this action only on selected spans (a block region or
    /// a line range), rather than the whole document. Takes precedence over
    /// `only_lines_matching` / `except_lines_matching` when present. The action is
    /// run once per selected span (so a block-scoped sort/dedupe works within each
    /// block); unselected lines pass through unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// Option keys are read verbatim (NOT re-cased), matching the .NET loader.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub options: Map<String, Value>,
}

/// A structured scope on a step: which part of the document the action applies
/// to. At most one selector is expected. The line-span selectors (`in_block`,
/// `line_range`) run the action once per selected multi-line region; the
/// within-line selectors (`field`, `char_range`) transform a substring of EACH
/// line and splice it back. With no selector, the whole document is one span.
/// `invert` selects the complement (line-span selectors only).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Select each `start`..`end` block region (inclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_block: Option<BlockScope>,
    /// Select a contiguous 1-based inclusive line range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_range: Option<LineRange>,
    /// Transform only one delimited field of each line (within-line scope).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<FieldScope>,
    /// Transform only a character range of each line (within-line scope).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub char_range: Option<CharRange>,
    /// Apply to the complement of the selected line spans instead. Not supported
    /// with the within-line selectors (`field` / `char_range`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub invert: bool,
}

/// A within-line field selector: the action transforms only field `index` of each
/// line (1-based; negative counts from the end), split on `delimiter` (default
/// `,`). The rest of the line, including the delimiters, is preserved exactly. A
/// line with fewer fields than `index` passes through unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldScope {
    pub index: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<String>,
}

/// A within-line character range: 1-based inclusive character positions (negative
/// counts from the end of the line). `from` defaults to the first character, `to`
/// to the last. The action transforms only that slice of each line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
}

/// A block selector: lines from one matching `start` through the next matching
/// `end` (inclusive), repeated down the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockScope {
    pub start: String,
    pub end: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ignore_case: bool,
}

/// A 1-based inclusive line range. `from` defaults to the first line, `to` to the
/// last; negative values count from the end (-1 = last line).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
}

impl Step {
    /// Returns the value only if it is a JSON string; otherwise `None`.
    /// Mirrors .NET `GetString` (a JSON number `4` is NOT coerced to "4").
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.options.get(key) {
            Some(Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// String option with a fallback when absent or non-string.
    pub fn get_string_or(&self, key: &str, fallback: &str) -> String {
        self.get_string(key).unwrap_or_else(|| fallback.to_string())
    }

    /// A string option constrained to a fixed set of values. Absent yields
    /// `default`; a value in `allowed` is returned as-is; a present value NOT in
    /// `allowed` is an error naming the accepted set, so a typo fails loudly instead
    /// of silently taking the lenient default branch.
    pub fn get_enum(
        &self,
        key: &str,
        action: &str,
        default: &str,
        allowed: &[&str],
    ) -> anyhow::Result<String> {
        match self.options.get(key) {
            None => Ok(default.to_string()),
            Some(Value::String(s)) if allowed.contains(&s.as_str()) => Ok(s.clone()),
            Some(v) => anyhow::bail!(
                "{action}: invalid '{key}' value {v} (expected one of: {})",
                allowed.join(", ")
            ),
        }
    }

    /// Strict integer read for a key that must hold an integer when present: a JSON
    /// integer or an integer string. A present-but-non-integer value is an error,
    /// not a silent fallback. Used where a wrong value must fail loudly rather than
    /// disarm a check (e.g. an `assert` line-count bound).
    pub fn get_i64_strict(&self, key: &str, action: &str) -> anyhow::Result<i64> {
        match self.options.get(key) {
            Some(Value::Number(n)) => n
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("{action}: '{key}' must be an integer, got {n}")),
            Some(Value::String(s)) => s
                .trim()
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("{action}: '{key}' must be an integer, got \"{s}\"")),
            Some(v) => anyhow::bail!("{action}: '{key}' must be an integer, got {v}"),
            None => anyhow::bail!("{action}: '{key}' is required"),
        }
    }

    /// JSON true/false, or the strings "true"/"false" (case-insensitive,
    /// surrounding whitespace trimmed). Absent yields `fallback`; a present value
    /// that is neither a bool nor "true"/"false" is an error, not a silent fallback,
    /// so a typo (`"yes"`) fails loudly. A non-string, non-bool JSON value errors too.
    pub fn get_bool(&self, key: &str, fallback: bool) -> anyhow::Result<bool> {
        match self.options.get(key) {
            None => Ok(fallback),
            Some(Value::Bool(b)) => Ok(*b),
            Some(Value::String(s)) => match s.trim().to_ascii_lowercase().as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => anyhow::bail!("option '{key}': expected a boolean (true/false), got \"{s}\""),
            },
            Some(v) => anyhow::bail!("option '{key}': expected a boolean, got {v}"),
        }
    }

    /// JSON integer, or a numeric string parsed as i64. Absent yields `fallback`; a
    /// present value that is not an integer (a float, a non-numeric string, or a
    /// non-number JSON value) is an error rather than a silent fallback.
    pub fn get_i64(&self, key: &str, fallback: i64) -> anyhow::Result<i64> {
        match self.options.get(key) {
            None => Ok(fallback),
            Some(Value::Number(n)) => n
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("option '{key}': expected an integer, got {n}")),
            Some(Value::String(s)) => s
                .trim()
                .parse::<i64>()
                .map_err(|_| anyhow::anyhow!("option '{key}': expected an integer, got \"{s}\"")),
            Some(v) => anyhow::bail!("option '{key}': expected an integer, got {v}"),
        }
    }

    /// JSON number (integer or float), or a numeric string parsed as f64. Absent
    /// yields `fallback`; a present non-numeric value is an error, not a silent
    /// fallback. Used by the number/math actions where decimals are meaningful.
    pub fn get_f64(&self, key: &str, fallback: f64) -> anyhow::Result<f64> {
        match self.options.get(key) {
            None => Ok(fallback),
            Some(Value::Number(n)) => n
                .as_f64()
                .ok_or_else(|| anyhow::anyhow!("option '{key}': expected a number, got {n}")),
            Some(Value::String(s)) => s
                .trim()
                .parse::<f64>()
                .map_err(|_| anyhow::anyhow!("option '{key}': expected a number, got \"{s}\"")),
            Some(v) => anyhow::bail!("option '{key}': expected a number, got {v}"),
        }
    }

    /// Non-negative usize view of [`get_i64`]. Absent yields `fallback`; a present
    /// value that is not a non-negative integer is an error, not a silent fallback.
    pub fn get_usize(&self, key: &str, fallback: usize) -> anyhow::Result<usize> {
        if self.options.get(key).is_none() {
            return Ok(fallback);
        }
        let v = self.get_i64(key, fallback as i64)?;
        if v < 0 {
            anyhow::bail!("option '{key}': expected a non-negative integer, got {v}");
        }
        Ok(v as usize)
    }

    /// Whether an option key is present at all (any value).
    /// Matches .NET `Options.ContainsKey`, used for seed detection.
    pub fn has_key(&self, key: &str) -> bool {
        self.options.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_mog_without_tags_or_tier_still_parses() {
        // A pre-store .mog has neither field: tags default empty, tier default full.
        let m: Mog =
            serde_json::from_str(r#"{ "name": "x", "steps": [ { "action": "to_upper" } ] }"#)
                .unwrap();
        assert!(m.tags.is_empty());
        assert_eq!(m.tier, Tier::Full);
        assert_eq!(m.steps.len(), 1);
    }

    #[test]
    fn tags_and_tier_parse_from_json() {
        let m: Mog = serde_json::from_str(
            r#"{ "name": "x", "tags": ["sql","codemod"], "tier": "core", "steps": [] }"#,
        )
        .unwrap();
        assert_eq!(m.tags, vec!["sql", "codemod"]);
        assert_eq!(m.tier, Tier::Core);
    }

    #[test]
    fn round_trip_is_strict_json_and_omits_defaults() {
        // A core-tier, tagged script round-trips; the empty/default fields are
        // dropped so the serialized form stays lean and re-parses identically.
        let src = r#"{ "name": "x", "tags": ["sql"], "tier": "core",
            "steps": [ { "action": "to_upper" } ] }"#;
        let m: Mog = serde_json::from_str(src).unwrap();
        let out = serde_json::to_string(&m).unwrap();
        // No empty constants / no absent optionals leak into the output.
        assert!(!out.contains("constants"));
        assert!(!out.contains("description"));
        assert!(out.contains("\"tier\":\"core\""));
        // Re-parsing the serialized form yields the same values.
        let m2: Mog = serde_json::from_str(&out).unwrap();
        assert_eq!(m2.tags, vec!["sql"]);
        assert_eq!(m2.tier, Tier::Core);
    }

    #[test]
    fn full_tier_is_not_serialized() {
        // The common default (`full`) round-trips without writing the field.
        let m: Mog = serde_json::from_str(r#"{ "steps": [] }"#).unwrap();
        let out = serde_json::to_string(&m).unwrap();
        assert!(
            !out.contains("tier"),
            "default full tier should be omitted: {out}"
        );
    }
}
