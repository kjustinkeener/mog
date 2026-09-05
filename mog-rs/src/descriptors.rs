//! Action descriptor registry: machine-readable metadata for every action, so a
//! GUI (Mog Studio) can build its palette and a schema-driven option form
//! without hard-coding per-action knowledge. Emitted as JSON by `mog
//! --list-actions --json`. This is the single source of truth the studio reads;
//! keep it in sync with `engine::resolve` and the `actions` modules.
//!
//! To support "progressive disclosure" (an AI agent or the studio learning the
//! action set without loading every descriptor at once) each descriptor carries a
//! [`Tier`] (Core vs Full) and an optional one-line `example`. Three views are
//! offered: the full table ([`descriptors_json`]), a compact index with no params
//! ([`compact_descriptors_json`]), and a single action by name or alias
//! ([`find_descriptor`]).

use serde::Serialize;
use serde_json::{json, Value};

/// How commonly an action is used. Core actions are the everyday set a caller can
/// learn first; Full is everything else. Serialized as "core" / "full".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,
    Full,
}

/// An action's runtime complexity class in the input size. Drives the Studio
/// "complexity" pill and the Phase 4 streaming classification. Serialized as
/// "linear" / "whole_file" / "regex_backtracking".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Complexity {
    /// O(n), single streaming pass.
    Linear,
    /// Needs the whole input buffered at once (sorting, global dedupe, columnar
    /// reshaping, block/compose builtins); memory-bound, cannot stream.
    WholeFile,
    /// Regex action: linear for ordinary patterns, but O(n^2) when the pattern
    /// uses backreferences or lookaround (backtracking-engine fallback).
    RegexBacktracking,
}

/// Static complexity class for an action by canonical name. Centralized (not a
/// per-`d()` argument) so the whole classification is reviewable in one place and
/// stays the single source of truth for both the studio pill and streaming.
pub fn complexity_of(name: &str) -> Complexity {
    // Blocking: buffer the whole input. Mirrors the Phase 4 streaming plan's
    // whole-file/blocking set. Compose/scope builtins buffer too.
    const WHOLE_FILE: &[&str] = &[
        "sort_lines",
        "sort_ip",
        "sort_versions",
        "sort_imports",
        "paste_column",
        "remove_duplicate_lines",
        "reverse_lines",
        "shuffle_lines",
        "align_columns",
        "wrap_text",
        "unwrap_text",
        "records_to_columns",
        "transpose",
        "unpivot",
        "pivot",
        "fill_down",
        "unique_with_count",
        "keep_duplicate_lines",
        "dedent",
        "format_list",
        "join_lines",
        "fill_from_list",
        "stamp_sequence",
        "for_each_block",
        "run_mog",
        "hoist_from_block",
        "sort_blocks",
        "dedupe_blocks",
        "dedupe_by",
        "replace_nth",
        "replace_first",
        "replace_last",
        "json_minify",
        "json_pretty",
        "json_to_csv",
        "yaml_to_json",
        "json_to_yaml",
        "json_to_jsonl",
        "jsonl_to_json",
        "toml_to_json",
        "json_to_toml",
        "xml_to_json",
        "json_to_xml",
        "env_to_json",
        "json_to_env",
        "querystring_to_json",
        "json_to_querystring",
        "csv_to_sql",
        "csv_to_html",
        "html_table_to_csv",
        "properties_to_json",
        "json_wrap",
        "json_merge",
        "json_set",
        "json_delete",
        "json_rename",
        "extract_json",
        "chunk_text",
        "ipynb_to_python",
        "intersect",
        "subtract",
        "diff",
        "reconcile",
        "lookup",
        "sql_transpile",
        "sql_lint",
        "sql_format",
        "sql_tables",
        "sql_canonicalize_identifiers",
        "sql_lineage",
        "sql_qualify",
    ];
    // Whole-file regex replace: the only actions that can go O(n^2) (fancy-regex
    // fallback). Per-line regex actions (filters, detectors) stay linear in file
    // size because they match one bounded line at a time.
    const REGEX_BACKTRACKING: &[&str] = &["replace_regex", "replace_regex_multiline"];
    if WHOLE_FILE.contains(&name) {
        Complexity::WholeFile
    } else if REGEX_BACKTRACKING.contains(&name) {
        Complexity::RegexBacktracking
    } else {
        Complexity::Linear
    }
}

/// Whether an action is reversible: information-preserving with a clear inverse, so
/// the original can be recovered from the output. Centralized (not a per-`d()`
/// argument) so the whole classification is reviewable in one place. Deliberately
/// CONSERVATIVE -- only encode/decode pairs, EOL conversions, and self-inverse
/// reshapes qualify; anything that drops, folds, sorts, trims, or rewrites content
/// is `false`. This is a trust hint (does re-applying an inverse restore the input?),
/// never a runtime behavior.
pub fn reversibility_of(name: &str) -> bool {
    const REVERSIBLE: &[&str] = &[
        // Encode/decode pairs (exact round-trip).
        "url_encode",
        "url_decode",
        "html_encode",
        "html_decode",
        "base64_encode",
        "base64_decode",
        "hex_encode",
        "hex_decode",
        // ROT13 is its own inverse.
        "rot13",
        // EOL conversions: swap the terminator, lose nothing.
        "eol_crlf",
        "eol_lf",
        "eol_cr",
        // Self-inverse reshapes: applying twice returns the original.
        "invert_case",
        "reverse_lines",
    ];
    REVERSIBLE.contains(&name)
}

/// One selectable option on an action, describing how the studio should render
/// its form field and what the engine defaults to.
#[derive(Debug, Clone, Serialize)]
pub struct ParamSpec {
    pub key: &'static str,
    pub label: &'static str,
    /// One of: "text", "multiline_text", "bool", "int", "enum".
    pub kind: &'static str,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Accepted values when `kind == "enum"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<&'static [&'static str]>,
    pub help: &'static str,
}

/// A single action's full descriptor.
#[derive(Debug, Clone, Serialize)]
pub struct ActionDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    /// One of: replace, line, text, data, detect, whitespace, eol, case, affix,
    /// encode, compose, assert.
    pub category: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    /// How commonly the action is used (see [`Tier`]).
    pub tier: Tier,
    /// Runtime complexity class in the input size (see [`Complexity`]).
    pub complexity: Complexity,
    /// Whether this action is line-local / chunk-concat-safe, so a pipeline of only
    /// such actions can run under `--stream` (bounded memory) and `--parallel`
    /// (intra-file, multi-core). The "parallel-capable" indicator.
    pub streamable: bool,
    /// Whether the transform is information-preserving with a clear inverse, so the
    /// original can be recovered from the output (e.g. an encode/decode pair, an EOL
    /// conversion, a self-inverse). Conservative: actions that drop, fold, sort, or
    /// otherwise lose information are `false`. A trust/safety hint for callers, not a
    /// runtime behavior. See [`reversibility_of`].
    pub reversible: bool,
    /// A tiny usage example: a representative `.mog` step as a JSON snippet, or a
    /// one-line description. `None` when no example is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    pub params: Vec<ParamSpec>,
}

impl ActionDescriptor {
    /// Mark this descriptor as Core (default is Full). Chainable builder helper.
    fn core(mut self) -> Self {
        self.tier = Tier::Core;
        self
    }
    /// Attach a usage example. Chainable builder helper.
    fn example(mut self, ex: &'static str) -> Self {
        self.example = Some(ex.to_string());
        self
    }
}

/// A compact descriptor for the index view: identity and summary only, no params.
/// Emitted by `mog --list-actions --compact --json`.
#[derive(Debug, Clone, Serialize)]
pub struct CompactDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub tier: Tier,
    pub complexity: Complexity,
    pub streamable: bool,
    pub category: &'static str,
    pub summary: &'static str,
}

// -- param constructors -------------------------------------------------------

fn text(
    key: &'static str,
    label: &'static str,
    default: &'static str,
    help: &'static str,
) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "text",
        required: false,
        default: Some(json!(default)),
        values: None,
        help,
    }
}
fn text_req(key: &'static str, label: &'static str, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "text",
        required: true,
        default: None,
        values: None,
        help,
    }
}
fn multiline(
    key: &'static str,
    label: &'static str,
    default: &'static str,
    help: &'static str,
) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "multiline_text",
        required: false,
        default: Some(json!(default)),
        values: None,
        help,
    }
}
fn boolean(key: &'static str, label: &'static str, default: bool, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "bool",
        required: false,
        default: Some(json!(default)),
        values: None,
        help,
    }
}
fn int(key: &'static str, label: &'static str, default: i64, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "int",
        required: false,
        default: Some(json!(default)),
        values: None,
        help,
    }
}
/// An optional integer with no default (its *presence* is meaningful, e.g. a seed).
fn int_opt(key: &'static str, label: &'static str, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "int",
        required: false,
        default: None,
        values: None,
        help,
    }
}
/// An optional text field with no default (absence is meaningful, e.g. an explicit
/// character set that, when omitted, falls back to a class).
fn text_opt(key: &'static str, label: &'static str, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "text",
        required: false,
        default: None,
        values: None,
        help,
    }
}
/// A required JSON-object option (a `{find: replace, ...}` map). Kind "map"; the
/// studio renders a key/value editor.
fn map_req(key: &'static str, label: &'static str, help: &'static str) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "map",
        required: true,
        default: None,
        values: None,
        help,
    }
}
fn enum_(
    key: &'static str,
    label: &'static str,
    default: &'static str,
    values: &'static [&'static str],
    help: &'static str,
) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "enum",
        required: false,
        default: Some(json!(default)),
        values: Some(values),
        help,
    }
}
/// A required combobox: pick from `values` or type your own (no default).
fn combobox(
    key: &'static str,
    label: &'static str,
    values: &'static [&'static str],
    help: &'static str,
) -> ParamSpec {
    ParamSpec {
        key,
        label,
        kind: "combobox",
        required: true,
        default: None,
        values: Some(values),
        help,
    }
}

/// Build a descriptor. Defaults to [`Tier::Full`] with no example; use the
/// `.core()` / `.example(..)` builder helpers to override.
fn d(
    name: &'static str,
    aliases: &'static [&'static str],
    category: &'static str,
    label: &'static str,
    summary: &'static str,
    params: Vec<ParamSpec>,
) -> ActionDescriptor {
    ActionDescriptor {
        name,
        aliases,
        category,
        label,
        summary,
        tier: Tier::Full,
        complexity: complexity_of(name),
        streamable: crate::stream::is_streamable_canonical(name),
        reversible: reversibility_of(name),
        example: None,
        params,
    }
}

/// The complete descriptor table, in palette-display order.
pub fn descriptors() -> Vec<ActionDescriptor> {
    let items = vec![
        // -- Replace ----------------------------------------------------------
        d("replace", &["r"], "replace", "Replace (literal)",
          "Literal find/replace (no regex, no $-expansion).",
          vec![
            text_req("find", "Find", "Literal text to find (required, non-empty)."),
            multiline("replace_with", "Replace with", "", "Literal replacement text."),
            boolean("ignore_case", "Ignore case", false, "Case-insensitive matching."),
            boolean("whole_word", "Whole word", false, "Match only as a standalone token (bounded by \\b), e.g. 'datetime' but not inside 'sysdatetime'."),
          ])
          .core()
          .example(r#"{"action":"replace","options":{"find":"foo","replace_with":"bar"}}"#),
        d("replace_extended", &[], "replace", "Replace (extended)",
          "Literal replace that first decodes Notepad++ escapes in both fields.",
          vec![
            text_req("find", "Find", "Decodes \\n \\r \\t \\0 \\\\ \\xHH \\uHHHH, then matches literally."),
            multiline("replace_with", "Replace with", "", "Same escape decoding as Find, then inserted literally."),
            boolean("ignore_case", "Ignore case", false, "Case-insensitive matching."),
          ])
          .core()
          .example(r#"{"action":"replace_extended","options":{"find":"\\t","replace_with":" "}}"#),
        d("replace_regex", &["rr"], "replace", "Replace (regex)",
          "Regex find/replace (fancy-regex). Backrefs use $1 / ${name} / $$. Per-occurrence tokens: $# = 1-based match number, $lineno = 1-based line of the match, $uuid = a fresh uuid v4 per match (deterministic under --pin-seed / mog --test, random otherwise).",
          vec![
            text_req("find", "Find", "fancy-regex pattern (supports backreferences and lookaround)."),
            multiline("replace_with", "Replace with", "", "Replacement; $1 / ${name} substitution, $$ for a literal $, plus $# (match number), $lineno (match line), and $uuid (a fresh uuid v4 per match). NOT \\1."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
            boolean("decode_replacement", "Decode replacement escapes", false, "Decode \\n \\t \\xHH etc. in the replacement before $-expansion."),
          ])
          .core()
          .example(r#"{"action":"replace_regex","options":{"find":"(\\w+)=(\\d+)","replace_with":"$2:$1"}}"#)
          .example(r#"{"action":"replace_regex","options":{"find":"^- ","replace_with":"- [$uuid] "}}"#),
        d("replace_regex_multiline", &["rrm"], "replace", "Replace (regex, multiline)",
          "Like Replace (regex) with (?m): ^ and $ match at every line. Per-occurrence tokens $# (match number), $lineno (match line), and $uuid (a fresh uuid v4 per match; deterministic under --pin-seed / mog --test) are available.",
          vec![
            text_req("find", "Find", "fancy-regex pattern; (?m) is prepended."),
            multiline("replace_with", "Replace with", "", "Replacement; $1 / ${name} substitution, $$ for a literal $, plus $# (match number), $lineno (match line), and $uuid (a fresh uuid v4 per match)."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
            boolean("decode_replacement", "Decode replacement escapes", false, "Decode \\n \\t \\xHH etc. in the replacement before $-expansion."),
          ])
          .example(r#"{"action":"replace_regex_multiline","options":{"find":"^","replace_with":"> "}}"#)
          .example(r#"{"action":"replace_regex_multiline","options":{"find":"^","replace_with":"$uuid\t"}}"#),
        d("replace_map", &[], "replace", "Replace (map)",
          "Apply many literal find->replace pairs in ONE pass (longest find wins), so replacements never chain into each other.",
          vec![
            map_req("map", "Map", "Object of find -> replace pairs, e.g. {\"CA\":\"California\"}."),
            boolean("ignore_case", "Ignore case", false, "Match keys case-insensitively."),
            boolean("whole_word", "Whole word", false, "Match keys only as standalone tokens (bounded by \\b)."),
          ])
          .example(r#"{"action":"replace_map","options":{"map":{"CA":"California","NY":"New York"},"whole_word":true}}"#),
        d("lookup_replace", &[], "replace", "Lookup replace",
          "For each match of a regex, look a capture up in a table and replace the whole match with the mapped value.",
          vec![
            text_req("pattern", "Pattern", "Regex; each match is replaced from the map."),
            map_req("map", "Map", "Object of key -> value; the captured group is the key."),
            int("group", "Group", 1, "Capture group used as the lookup key (0 = whole match)."),
            enum_("on_miss", "On miss", "leave", &["leave", "blank", "key"], "When the key is not in the map: leave the match, blank it, or emit the key."),
            boolean("ignore_case", "Ignore case", false, "Match and look up case-insensitively."),
          ])
          .example(r#"{"action":"lookup_replace","options":{"pattern":"\\b(\\d{3})\\b","map":{"404":"Not Found"}}}"#),
        d("smart_case_replace", &[], "replace", "Replace (preserve case)",
          "Literal find/replace that casts the replacement to each match's case pattern (UPPER, lower, Title).",
          vec![
            text_req("find", "Find", "Literal text to find (matched case-insensitively)."),
            multiline("replace_with", "Replace with", "", "Replacement; re-cased to match each occurrence."),
          ])
          .example(r#"{"action":"smart_case_replace","options":{"find":"color","replace_with":"colour"}}"#),
        d("replace_nth", &[], "replace", "Replace (Nth match)",
          "Replace ONLY the Nth occurrence of a regex find. n is 1-based; a negative n counts from the end (n=-1 = last). Occurrences are counted across the whole input (or within a scope span). An out-of-range n is a no-op.",
          vec![
            text_req("find", "Find", "fancy-regex pattern (supports backreferences and lookaround)."),
            multiline("replace_with", "Replace with", "", "Replacement; $1 / ${name} substitution, $$ for a literal $. NOT \\1."),
            int("n", "N", 1, "Which occurrence to replace (1-based; negative counts from the end, so -1 is the last). Out of range = no change."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .example(r#"{"action":"replace_nth","options":{"find":"foo","replace_with":"bar","n":2}}"#),
        d("replace_first", &[], "replace", "Replace (first match)",
          "Replace only the FIRST occurrence of a regex find (replace_nth with n=1).",
          vec![
            text_req("find", "Find", "fancy-regex pattern (supports backreferences and lookaround)."),
            multiline("replace_with", "Replace with", "", "Replacement; $1 / ${name} substitution, $$ for a literal $. NOT \\1."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .example(r#"{"action":"replace_first","options":{"find":"foo","replace_with":"bar"}}"#),
        d("replace_last", &[], "replace", "Replace (last match)",
          "Replace only the LAST occurrence of a regex find (replace_nth with n=-1).",
          vec![
            text_req("find", "Find", "fancy-regex pattern (supports backreferences and lookaround)."),
            multiline("replace_with", "Replace with", "", "Replacement; $1 / ${name} substitution, $$ for a literal $. NOT \\1."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .example(r#"{"action":"replace_last","options":{"find":"foo","replace_with":"bar"}}"#),
        // -- Line -------------------------------------------------------------
        d("remove_empty_lines", &["rel"], "line", "Remove empty lines",
          "Drop empty lines.",
          vec![boolean("include_whitespace", "Include whitespace-only", false, "Also drop lines that contain only whitespace.")])
          .core()
          .example(r#"{"action":"remove_empty_lines","options":{"include_whitespace":true}}"#),
        d("remove_duplicate_lines", &["dedupe"], "line", "Remove duplicate lines",
          "Keep the first occurrence of each line; drop later duplicates (whole file).",
          vec![boolean("ignore_case", "Ignore case", false, "Compare lines case-insensitively.")])
          .core()
          .example(r#"{"action":"remove_duplicate_lines"}"#),
        d("remove_consecutive_duplicate_lines", &["uniq"], "line", "Remove consecutive duplicates",
          "Collapse runs of identical adjacent lines to one.",
          vec![boolean("ignore_case", "Ignore case", false, "Compare lines case-insensitively.")])
          .example(r#"{"action":"remove_consecutive_duplicate_lines"}"#),
        d("dedupe_by", &[], "line", "Dedupe by key",
          "Drop duplicate lines by an EXTRACTED key (keep first per key), across non-adjacent lines. Pick the key with key_regex (match or capture group 1) OR key_field (Nth delimited field). A line with no key match uses the whole line as its key.",
          vec![
            text_opt("key_regex", "Key regex", "Key is this regex's match (or capture group 1 when it has one). Mutually exclusive with key_field."),
            int_opt("key_field", "Key field", "Key is the Nth key_delimiter-split field (1-based; negative counts from the end). Mutually exclusive with key_regex."),
            text("key_delimiter", "Key delimiter", "\t", "Field delimiter for key_field (default a TAB)."),
            boolean("ignore_case", "Ignore case", false, "Compare keys case-insensitively."),
          ])
          .example(r#"{"action":"dedupe_by","options":{"key_field":1,"key_delimiter":","}}"#),
        d("reverse_lines", &["reverse"], "line", "Reverse lines",
          "Reverse the order of all lines.", vec![])
          .example(r#"{"action":"reverse_lines"}"#),
        d("shuffle_lines", &["shuffle"], "line", "Shuffle lines",
          "Randomly reorder lines. Provide a seed for a reproducible order.",
          vec![int_opt("seed", "Seed", "Optional. If set, the shuffle is deterministic; if omitted, it is random each run.")])
          .example(r#"{"action":"shuffle_lines","options":{"seed":42}}"#),
        d("join_lines", &["join"], "line", "Join lines",
          "Join all lines into one, separated by a string.",
          vec![text("separator", "Separator", "", "Text placed between joined lines.")])
          .example(r#"{"action":"join_lines","options":{"separator":", "}}"#),
        d("sort_lines", &["sort"], "line", "Sort lines",
          "Sort lines (stable). Sort by the whole line, a delimited field (key_field), a regex match (key_regex), or by how often each key occurs (by_frequency).",
          vec![
            enum_("order", "Order", "asc", &["asc", "desc"], "Ascending or descending."),
            boolean("ignore_case", "Ignore case", false, "Case-insensitive comparison."),
            boolean("numeric", "Numeric", false, "Compare as numbers when possible (highest precedence)."),
            boolean("natural", "Natural", false, "Version/number-aware order so file2 sorts before file10."),
            boolean("by_length", "By length", false, "Sort by line length, then lexicographically."),
            int_opt("key_field", "Key field", "Sort by the Nth key_delimiter-split field (1-based; negative counts from the end) instead of the whole line. Mutually exclusive with key_regex."),
            text("key_delimiter", "Key delimiter", "\t", "Field delimiter for key_field (default a TAB)."),
            text_opt("key_regex", "Key regex", "Sort by this regex's match (or capture group 1 when the pattern has one). Mutually exclusive with key_field."),
            boolean("by_frequency", "By frequency", false, "Order lines by descending count of their (identical) key; ties keep first-seen order. order:\"asc\" flips to ascending count."),
          ])
          .core()
          .example(r#"{"action":"sort_lines","options":{"order":"asc"}}"#),
        d("sort_ip", &[], "line", "Sort by IP address",
          "Sort lines by IP address value (numeric, not lexical; IPv4 before IPv6). The IP is the first whitespace-separated token of each line, so bare IP lists and 'IP rest-of-line' logs both sort correctly. Lines whose first token is not a valid IP keep their order at the end.",
          vec![
            enum_("order", "Order", "asc", &["asc", "desc"], "Ascending or descending."),
            boolean("unique", "Unique", false, "Drop adjacent duplicate lines after sorting."),
          ])
          .example(r#"{"action":"sort_ip"}"#),
        d("sort_versions", &[], "line", "Sort by version",
          "Sort lines by version number (like sort -V) with semver pre-release rules, so 1.9.0 sorts before 1.10.0 and 1.0.0-rc.1 before 1.0.0. The version is the first token of each line (a leading 'v' is ignored). Lines whose first token is not version-like keep their order at the end.",
          vec![
            enum_("order", "Order", "asc", &["asc", "desc"], "Ascending or descending."),
            boolean("unique", "Unique", false, "Drop adjacent duplicate lines after sorting."),
          ])
          .example(r#"{"action":"sort_versions"}"#),
        d("sort_imports", &[], "line", "Sort import lines",
          "Sort each run of consecutive import lines in place, leaving blank lines and code as anchors (so grouped/sectioned import blocks stay separate). `pattern` decides what counts as an import (default matches import / from / #include / using / use / require / @import).",
          vec![
            text("pattern", "Import pattern", r"^\s*(?:import|from|#include|#import|using|use|require|@import)\b", "Regex marking an import line."),
            boolean("ignore_case", "Ignore case", false, "Fold case when sorting each run."),
            boolean("dedupe", "Dedupe", false, "Drop exact duplicate imports within a run."),
          ])
          .example(r#"{"action":"sort_imports"}"#),
        d("number_lines", &[], "line", "Number lines",
          "Prefix each line with a right-aligned line number.",
          vec![
            int("start", "Start at", 1, "First line number."),
            text("separator", "Separator", ". ", "Text between the number and the line (default \". \")."),
          ])
          .example(r#"{"action":"number_lines","options":{"start":1,"separator": ". "}}"#),
        d("squeeze_blank_lines", &[], "line", "Squeeze blank lines",
          "Collapse runs of blank lines to a single blank line.",
          vec![boolean("include_whitespace", "Include whitespace-only", false, "Treat whitespace-only lines as blank.")])
          .example(r#"{"action":"squeeze_blank_lines"}"#),
        d("split_lines", &[], "line", "Split lines",
          "Split each line on a literal separator, emitting each piece as its own line (inverse of join lines).",
          vec![
            text("separator", "Separator", ",", "Literal delimiter to split each line on."),
            boolean("trim", "Trim", false, "Trim whitespace from each piece."),
          ])
          .example(r#"{"action":"split_lines","options":{"separator":",","trim":true}}"#),
        d("keep_duplicate_lines", &[], "line", "Keep duplicate lines",
          "Keep only lines that occur more than once (the inverse of dedupe).",
          vec![
            boolean("ignore_case", "Ignore case", false, "Compare lines case-insensitively."),
            boolean("all", "All occurrences", false, "Keep every occurrence instead of one per duplicated line."),
          ])
          .example(r#"{"action":"keep_duplicate_lines"}"#),
        d("unique_with_count", &[], "line", "Unique with count",
          "Collapse to distinct lines, each prefixed with its occurrence count (like sort | uniq -c).",
          vec![boolean("ignore_case", "Ignore case", false, "Compare lines case-insensitively.")])
          .example(r#"{"action":"unique_with_count"}"#),
        d("stamp_sequence", &[], "line", "Stamp sequence",
          "Replace each occurrence of a marker with an incrementing number (sequential IDs / Bates numbering).",
          vec![
            text("find", "Marker", "#", "Literal marker replaced by the next number."),
            int("start", "Start at", 1, "First number."),
            int("step", "Step", 1, "Increment between numbers."),
            int("width", "Zero-pad width", 0, "Zero-pad numbers to this width (0 = no padding)."),
          ])
          .example(r#"{"action":"stamp_sequence","options":{"find":"<n>","start":1,"width":3}}"#),
        d("keep_lines_matching", &[], "line", "Keep lines matching",
          "Keep only lines matching a regex; drop the rest. context (or before/after) also keeps that many surrounding lines, the way grep -C does, without duplicating a line two matches share.",
          vec![
            text_req("pattern", "Pattern", "Regex; a line is kept if it matches."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
            int("context", "Context lines", 0, "Also keep this many lines on each side of a match (grep -C). before/after override it per side."),
            int("before", "Lines before", 0, "Lines to keep before each match (grep -B); defaults to context."),
            int("after", "Lines after", 0, "Lines to keep after each match (grep -A); defaults to context."),
          ])
          .core()
          .example(r#"{"action":"keep_lines_matching","options":{"pattern":"ERROR"}}"#),
        d("remove_lines_matching", &[], "line", "Remove lines matching",
          "Drop lines matching a regex; keep the rest.",
          vec![
            text_req("pattern", "Pattern", "Regex; a line is removed if it matches."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .core()
          .example(r#"{"action":"remove_lines_matching","options":{"pattern":"^#"}}"#),
        d("insert_before_matching", &[], "line", "Insert before matching",
          "Insert a new line of text before every line matching a regex.",
          vec![
            text_req("pattern", "Pattern", "Regex; text is inserted before a line that matches."),
            multiline("text", "Text", "", "The line to insert."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .example(r#"{"action":"insert_before_matching","options":{"pattern":"^BEGIN","text":"-- start"}}"#),
        d("insert_after_matching", &[], "line", "Insert after matching",
          "Insert a new line of text after every line matching a regex.",
          vec![
            text_req("pattern", "Pattern", "Regex; text is inserted after a line that matches."),
            multiline("text", "Text", "", "The line to insert."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .core()
          .example(r#"{"action":"insert_after_matching","options":{"pattern":"^BEGIN","text":"-- inserted"}}"#),
        d("flag_matching", &[], "line", "Flag matching",
          "Annotate lines matching a regex with a canonical, detectable marker: <comment> <TAG>(mog): <message>. The CLI and studio report these for review.",
          vec![
            text_req("pattern", "Pattern", "Regex; matching lines get the marker."),
            text_req("message", "Message", "The human-readable note."),
            combobox("comment", "Comment prefix", &["//", "#", "--", ";"], "Line-comment prefix for the target language (required)."),
            enum_("tag", "Tag", "FIXME", crate::actions::flag::TAGS, "Marker tag (all are (mog)-scoped so all are detected)."),
            enum_("position", "Position", "inline", &["inline", "before", "after"], "Place the marker on the line (inline) or on its own line before/after."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to the pattern."),
          ])
          .example(r#"{"action":"flag_matching","options":{"pattern":"TODO","message":"review","comment":"--"}}"#),
        d("hoist_from_block", &[], "line", "Hoist from block",
          "Lift lines out of a start..end block and re-emit them as statements before/after it, carrying the block header's captures. Templates use ${bN} for header captures and ${N} for extracted-line captures.",
          vec![
            text_req("block_start", "Block start", "Regex matching the block's opening line; its captures are ${bN}."),
            text_req("block_end", "Block end", "Regex matching the block's closing line."),
            text_req("extract", "Extract", "Regex matching body lines to lift out; its captures are ${N}."),
            text("emit_before", "Emit before", "", "Template emitted before the block per extracted line."),
            text("emit_after", "Emit after", "", "Template emitted after the block per extracted line."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to all three patterns."),
          ]),
        d("sort_blocks", &[], "line", "Sort blocks (records)",
          "Sort whole multi-line records as units (stable). Records are paragraphs separated by a blank line. Sort by the whole block, its first line, or a regex key. Key semantics (numeric/natural/ignore_case) mirror sort_lines.",
          vec![
            enum_("by", "By", "whole_block", &["whole_block", "first_line", "key_regex"], "Record key: the whole block, its first line, or a regex (with key_regex)."),
            text_opt("key_regex", "Key regex", "When by=key_regex: the regex match (or capture group 1 when it has one) is the sort key."),
            enum_("order", "Order", "asc", &["asc", "desc"], "Ascending or descending."),
            boolean("ignore_case", "Ignore case", false, "Case-insensitive comparison."),
            boolean("numeric", "Numeric", false, "Compare keys as numbers when possible (highest precedence)."),
            boolean("natural", "Natural", false, "Version/number-aware order so item2 sorts before item10."),
          ])
          .example(r#"{"action":"sort_blocks","options":{"by":"first_line"}}"#),
        d("dedupe_blocks", &[], "line", "Dedupe blocks (records)",
          "Remove duplicate multi-line records, keeping the first (works on non-adjacent duplicates). Records are paragraphs separated by a blank line. Compare by the whole block, its first line, or a regex key.",
          vec![
            enum_("by", "By", "whole_block", &["whole_block", "first_line", "key_regex"], "Record key: the whole block, its first line, or a regex (with key_regex)."),
            text_opt("key_regex", "Key regex", "When by=key_regex: the regex match (or capture group 1 when it has one) is the dedupe key."),
            boolean("ignore_case", "Ignore case", false, "Compare keys case-insensitively."),
          ])
          .example(r#"{"action":"dedupe_blocks","options":{"by":"first_line"}}"#),
        // -- Data -------------------------------------------------------------
        d("records_to_columns", &[], "data", "Records to columns",
          "Parse repeated key/value stanzas (separated by a blank line) into a CSV table: header is the union of keys in first-seen order, one row per record.",
          vec![
            text("separator", "Key/value separator", ":", "Splits each line into a key and a value."),
            text("delimiter", "Output delimiter", ",", "CSV delimiter for the output table."),
            boolean("header", "Header row", true, "Emit a header row of column names."),
          ])
          .example(r#"{"action":"records_to_columns"}"#),
        d("transpose", &[], "data", "Transpose table",
          "Swap the rows and columns of a delimited table (quote-aware). Jagged rows are padded to the widest row; fully-blank rows are dropped. Whole-file.",
          vec![
            text("delimiter", "Delimiter", ",", "Field delimiter of the table."),
          ])
          .example(r#"{"action":"transpose"}"#),
        d("unpivot", &[], "data", "Unpivot (wide to long)",
          "Reshape a wide table to long (id, key, value) rows: keep the id_columns and turn every other column into one row per value. With has_header the former column name is the key.",
          vec![
            text("id_columns", "ID columns", "1", "Comma-separated 1-based columns to keep on every row."),
            text("key_name", "Key column name", "key", "Header for the former column name."),
            text("value_name", "Value column name", "value", "Header for the value."),
            boolean("has_header", "Has header", true, "First row names the columns (used as keys)."),
            text("delimiter", "Delimiter", ",", "Field delimiter of the table."),
          ])
          .example(r#"{"action":"unpivot","options":{"id_columns":"1"}}"#),
        d("pivot", &[], "data", "Pivot (long to wide)",
          "Reshape a long table to wide: the distinct values of key_column become columns, value_column fills the cells, and rows group by id_columns (default: all other columns). aggregate = first/last/error when a group+key repeats. Guardrail max_columns (default 1000) refuses a runaway grid; set it to 0 to lift the cap. Cost is O(groups x keys), which can exceed the input for sparse data.",
          vec![
            text_req("key_column", "Key column", "1-based column whose distinct values become the new columns."),
            text_req("value_column", "Value column", "1-based column that fills the cells."),
            text_opt("id_columns", "ID columns", "Comma-separated 1-based grouping columns. Default: every column except the key and value."),
            enum_("aggregate", "Aggregate", "first", &["first", "last", "error"], "Value to keep when a group and key repeat."),
            int("max_columns", "Max columns", 1000, "Guardrail: refuse more than this many distinct keys. 0 lifts the cap."),
            boolean("has_header", "Has header", true, "First row names the columns."),
            text("delimiter", "Delimiter", ",", "Field delimiter of the table."),
          ])
          .example(r#"{"action":"pivot","options":{"key_column":"2","value_column":"3"}}"#),
        d("change_delimiter", &[], "data", "Change delimiter",
          "Re-delimit each line, quote-aware: split on one delimiter and re-join with another, re-quoting fields that need it (CSV <-> TSV <-> pipe).",
          vec![
            text("from", "From", ",", "Delimiter to split on (first character used)."),
            text("to", "To", "\t", "Delimiter to join with (default a tab)."),
          ])
          .example(r#"{"action":"change_delimiter","options":{"from":",","to":"\t"}}"#),
        d("fill_down", &[], "data", "Fill down",
          "Fill each blank cell with the last non-blank value seen in that column (the merged-cell export fix). Blank lines pass through.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter (first character used).")])
          .example(r#"{"action":"fill_down","options":{"delimiter":","}}"#),
        d("mask_field", &[], "data", "Mask field",
          "Mask the middle of one delimited field, keeping some leading and trailing characters (card / account masking).",
          vec![
            int("field", "Field", 1, "1-based column to mask."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
            int("keep_start", "Keep start", 0, "Leading characters left visible."),
            int("keep_end", "Keep end", 4, "Trailing characters left visible."),
            text("mask_char", "Mask char", "*", "Character used for the masked middle."),
          ])
          .example(r#"{"action":"mask_field","options":{"field":2,"keep_end":4}}"#),
        d("hash_field", &[], "data", "Hash field",
          "Replace a field (or the whole line) with a stable hash, so the data stays joinable across runs but the value is pseudonymized. Deterministic: the same input always maps to the same token, so it pairs with detect_pii (detect, then hash). field is 1-based (0 = whole line); empty cells and blank lines are left untouched. Pick the algorithm, encoding, an optional length to truncate the token, an explicit salt, and a literal prefix.",
          vec![
            int("field", "Field", 1, "1-based column to hash; 0 hashes the whole line."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
            enum_("algorithm", "Algorithm", "sha256", &["sha256", "sha512"], "Hash algorithm."),
            enum_("encoding", "Encoding", "hex", &["hex", "base64"], "Token encoding: lowercase hex, or URL-safe base64 (no padding)."),
            int("length", "Length", 0, "Truncate the token to this many characters (0 = full)."),
            text("salt", "Salt", "", "Explicit (non-random) salt mixed in before hashing; keep tokens stable and joinable."),
            text("prefix", "Prefix", "", "Literal prefix on the token, e.g. \"user_\"."),
          ])
          .example(r#"{"action":"hash_field","options":{"field":1,"prefix":"user_","length":16}}"#),
        d("cut_fields", &[], "data", "Cut fields",
          "Select and/or reorder columns by 1-based index (quote-aware). Out-of-range indices yield an empty cell.",
          vec![
            text_req("fields", "Fields", "Comma-separated 1-based indices, e.g. \"1,3,2\"."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
          ])
          .example(r#"{"action":"cut_fields","options":{"fields":"1,3"}}"#),
        d("epoch_to_iso", &[], "data", "Epoch to ISO date",
          "Convert a Unix-epoch number to an ISO date/time. field is 1-based (0 = the whole line); format is datetime (default, YYYY-MM-DDTHH:MM:SSZ) or date (YYYY-MM-DD); unit is seconds (default) or millis. Non-integer values and empty cells pass through.",
          vec![
            int("field", "Field", 1, "1-based column to convert; 0 converts the whole line."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
            enum_("format", "Format", "datetime", &["datetime", "date"], "ISO datetime (with time) or just the date."),
            enum_("unit", "Unit", "seconds", &["seconds", "millis"], "Whether the epoch value is in seconds or milliseconds."),
          ])
          .example(r#"{"action":"epoch_to_iso","options":{"field":2}}"#),
        d("iso_to_epoch", &[], "data", "ISO date to epoch",
          "Convert an ISO date/datetime (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS[Z], UTC) to a Unix-epoch number. field is 1-based (0 = the whole line); unit is seconds (default) or millis. Values that do not parse, and empty cells, pass through.",
          vec![
            int("field", "Field", 1, "1-based column to convert; 0 converts the whole line."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
            enum_("unit", "Unit", "seconds", &["seconds", "millis"], "Emit epoch seconds or milliseconds."),
          ])
          .example(r#"{"action":"iso_to_epoch","options":{"field":1}}"#),
        d("shift_dates", &[], "data", "Shift dates",
          "Offset every ISO date (YYYY-MM-DD) in the text by a fixed number of days (may be negative), preserving the interval between dates. Useful for anonymizing test data or moving a schedule while keeping relative timing. Only whole YYYY-MM-DD tokens are touched; other text is left alone.",
          vec![ int("days", "Days", 0, "Number of days to add (negative to subtract).") ])
          .example(r#"{"action":"shift_dates","options":{"days":-365}}"#),
        d("row_to_template", &[], "data", "Row to template",
          "Render each delimited row through a template string, replacing ${N} (1-based column) and, with has_header, ${name} placeholders with that row's fields ($$ is a literal $). Generates SQL, config, HTML, YAML, or form text from a table. Blank rows are skipped; with has_header the first row supplies names and is not emitted.",
          vec![
            text_req("template", "Template", "Template with ${N} / ${name} placeholders; $$ is a literal $."),
            text("delimiter", "Delimiter", ",", "Field delimiter (first character used)."),
            boolean("has_header", "Has header", false, "Treat the first row as a header providing ${name} placeholders (not emitted)."),
          ])
          .example(r#"{"action":"row_to_template","options":{"template":"INSERT INTO t VALUES (${1}, '${2}');"}}"#),
        d("explode_field", &[], "data", "Explode field",
          "Split a field's inner list into multiple rows, one per value (the rest of the row repeated).",
          vec![
            int("field", "Field", 1, "1-based column holding the list."),
            text("delimiter", "Delimiter", ",", "Row field delimiter (first character used)."),
            text("separator", "List separator", ";", "Separator inside the field's list."),
          ])
          .example(r#"{"action":"explode_field","options":{"field":2,"separator":";"}}"#),
        d("csv_to_markdown", &[], "data", "CSV to Markdown table",
          "Render CSV (first line = header) as a GitHub-flavored Markdown table. Quote-aware; a | inside a cell is escaped; ragged rows are padded.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter of the input CSV.")])
          .example(r#"{"action":"csv_to_markdown"}"#),
        d("markdown_table_to_csv", &[], "data", "Markdown table to CSV",
          "Parse a Markdown table into CSV, dropping the --- separator row. Cells are trimmed and CSV-quoted; non-table lines are ignored.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter of the output CSV.")])
          .example(r#"{"action":"markdown_table_to_csv"}"#),
        d("csv_to_sql", &[], "data", "CSV to SQL INSERT",
          "Convert CSV (first line = header) into INSERT INTO <table> (...) VALUES (...); statements. Numbers are bare, empty cells become NULL, everything else is a single-quoted escaped string.",
          vec![
            text_req("table", "Table", "Target table name for the INSERT statements."),
            text("delimiter", "Delimiter", ",", "Field delimiter of the input CSV."),
          ])
          .example(r#"{"action":"csv_to_sql","options":{"table":"users"}}"#),
        d("csv_to_html", &[], "data", "CSV to HTML table",
          "Render CSV (first line = header) as an HTML <table> (thead + tbody). Cell text is HTML-escaped.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter of the input CSV.")])
          .example(r#"{"action":"csv_to_html"}"#),
        d("html_table_to_csv", &[], "data", "HTML table to CSV",
          "Extract an HTML <table> into CSV (regex-based, for simple tables): cell tags are stripped and a few entities decoded. Non-table markup is ignored.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter of the output CSV.")])
          .example(r#"{"action":"html_table_to_csv"}"#),
        d("fixed_width_to_csv", &[], "data", "Fixed-width to CSV",
          "Split each line into fields by fixed column widths (comma-separated), trimming each field. Characters past the last width are dropped.",
          vec![
            text_req("widths", "Widths", "Comma-separated column widths, e.g. 10,8,20."),
            text("delimiter", "Delimiter", ",", "Field delimiter of the output CSV."),
          ])
          .example(r#"{"action":"fixed_width_to_csv","options":{"widths":"10,8,20"}}"#),
        d("access_log_to_csv", &[], "data", "Access log to CSV",
          "Parse Common/Combined access-log lines into CSV: ip,timestamp,method,path,protocol,status,size,referer,user_agent. Lines that do not match are skipped.",
          vec![text("delimiter", "Delimiter", ",", "Field delimiter of the output CSV.")])
          .example(r#"{"action":"access_log_to_csv"}"#),
        d("toml_to_ini", &[], "data", "TOML to INI",
          "Convert TOML to INI: top-level scalars first, then each table as a [section] (nested tables become dotted sections), arrays comma-joined. Best-effort for the flat/one-level shape.",
          vec![])
          .example(r#"{"action":"toml_to_ini"}"#),
        d("ini_to_toml", &[], "data", "INI to TOML",
          "Convert INI to TOML: [section] headers pass through, key=value lines get a type-inferred TOML value (bool/int/float else quoted string), and ;/# comments become # comments.",
          vec![])
          .example(r#"{"action":"ini_to_toml"}"#),
        d("decimal_separator_normalize", &[], "data", "Normalize decimal separators",
          "Rewrite numbers from one decimal/thousands convention to another (e.g. European 1.234,56 -> US 1234.56). Only tokens that actually carry a separator are touched, so bare integers (like a year) are never regrouped. from/to name the conventions.",
          vec![
            enum_("from", "From", "eu", &["us", "eu", "swiss", "space_comma", "space_dot"], "Input convention (thousands/decimal)."),
            enum_("to", "To", "us", &["us", "eu", "swiss", "space_comma", "space_dot"], "Output convention."),
            boolean("grouping", "Group thousands", true, "Emit thousands separators in the output."),
          ])
          .example(r#"{"action":"decimal_separator_normalize","options":{"from":"eu","to":"us"}}"#),
        d("fill_from_list", &[], "data", "Fill from list",
          "Replace each occurrence of a marker with the next value from a named external source (mail-merge / placeholder fill). Bind the source via `sources` in the .mog or --source name=path.",
          vec![
            text_req("source", "Source", "Name of a loaded source; its lines are the values."),
            text("find", "Marker", "?", "Literal marker replaced by each next value."),
            enum_("on_exhausted", "On exhausted", "leave", &["leave", "blank", "wrap", "error"], "When values run out: keep the marker, blank it, cycle, or error."),
          ])
          .example(r#"{"action":"fill_from_list","options":{"source":"names","find":"<NAME>"}}"#),
        d("paste_column", &[], "data", "Paste column",
          "Append (or prepend) a column from a named source, pairing input line i with source line i (Unix paste). Bind the source via `sources` or --source name=path. delimiter (default a tab) joins the two; side = right (append) or left (prepend); on_exhausted = blank (empty cell), skip (leave the line), or error when the source is shorter than the input.",
          vec![
            text_req("source", "Source", "Name of a loaded source; line i is pasted onto input line i."),
            text("delimiter", "Delimiter", "\t", "Separator between the input line and the pasted value."),
            enum_("side", "Side", "right", &["right", "left"], "Append the value on the right or prepend it on the left."),
            enum_("on_exhausted", "On exhausted", "blank", &["blank", "skip", "error"], "When the source has fewer lines: empty cell, leave the line, or error."),
          ])
          .example(r#"{"action":"paste_column","options":{"source":"ids","delimiter":","}}"#),
        // -- compare (two-input: primary text vs a named reference source) -----
        d("intersect", &[], "compare", "Intersect with list",
          "Keep only the input lines whose text also appears in a reference source (set intersection). Order- and duplicate-preserving. Bind the reference via `sources` in the .mog or --source name=path.",
          vec![
            text_req("ref", "Reference", "Name of a loaded source whose lines are the reference set."),
            boolean("trim", "Trim", false, "Trim each line before comparing."),
            boolean("ignore_case", "Ignore case", false, "Compare case-insensitively."),
          ])
          .example(r#"{"action":"intersect","options":{"ref":"allowlist"}}"#),
        d("subtract", &[], "compare", "Subtract list",
          "Keep only the input lines whose text does NOT appear in a reference source (set difference, input minus reference). Order- and duplicate-preserving. Bind the reference via `sources` or --source name=path.",
          vec![
            text_req("ref", "Reference", "Name of a loaded source whose lines are removed from the input."),
            boolean("trim", "Trim", false, "Trim each line before comparing."),
            boolean("ignore_case", "Ignore case", false, "Compare case-insensitively."),
          ])
          .example(r#"{"action":"subtract","options":{"ref":"seen"}}"#),
        d("diff", &[], "compare", "Diff against list",
          "Emit a line diff of a reference source (old side) against the input (new side). format: marker (git-style +/-/space, the default), unified, only_added (input lines not in the reference), only_removed (reference lines not in the input).",
          vec![
            text_req("ref", "Reference", "Name of a loaded source: the old/baseline side of the diff."),
            enum_("format", "Format", "marker", &["marker", "unified", "only_added", "only_removed"], "Diff shape: git-style markers, a unified diff, or just the added/removed lines."),
          ])
          .example(r#"{"action":"diff","options":{"ref":"baseline","format":"only_added"}}"#),
        d("reconcile", &[], "compare", "Reconcile records",
          "Match input rows against a reference source by a key column, then report the differences. report: annotated (default; each input row prefixed with ADDED/CHANGED/SAME, then reference-only rows as REMOVED), added, removed, changed, or common. key is a 1-based column index, or a header name when has_header is set.",
          vec![
            text_req("ref", "Reference", "Name of a loaded source: the other record set to reconcile against."),
            text_req("key", "Key column", "1-based column index, or a header name when has_header is set."),
            text("delimiter", "Delimiter", ",", "Field delimiter of both inputs (quote-aware)."),
            boolean("has_header", "Has header", false, "Treat the first line of each input as a header row (enables name keys)."),
            enum_("report", "Report", "annotated", &["annotated", "added", "removed", "changed", "common"], "What to emit: annotated status column, or just the added/removed/changed/common rows."),
          ])
          .example(r#"{"action":"reconcile","options":{"ref":"yesterday","key":"id","has_header":true}}"#),
        d("lookup", &[], "compare", "Lookup / join",
          "Enrich each input row with value column(s) pulled from a reference source, matched by a key column (a keyed join / VLOOKUP). ref names the reference; key is the input's key column (1-based index, or a header name when has_header is set); ref_key is the reference's key column (defaults to key); value picks which reference column(s) to append (comma-separated indices/names; default = every reference column except the key). on_miss = blank (empty cells, default), leave (row unchanged), drop (omit the row), or error. The first matching reference row wins on a duplicate key.",
          vec![
            text_req("ref", "Reference", "Name of a loaded source: the lookup table to join against."),
            text_req("key", "Key column", "Input key column: a 1-based index, or a header name when has_header is set."),
            text_opt("ref_key", "Reference key", "Reference key column (1-based index or header name). Defaults to the input key."),
            text_opt("value", "Value columns", "Reference column(s) to append: comma-separated 1-based indices or header names. Default = every reference column except the key."),
            enum_("on_miss", "On miss", "blank", &["blank", "leave", "drop", "error"], "When a key has no reference match: append blanks, leave the row, drop it, or error."),
            text("delimiter", "Delimiter", ",", "Field delimiter of both inputs (quote-aware)."),
            boolean("has_header", "Has header", false, "Treat the first line of each input as a header row (enables name keys and an appended header)."),
          ])
          .example(r#"{"action":"lookup","options":{"ref":"directory","key":"id","value":"name","has_header":true}}"#),
        // -- JSON (reading: JSON in -> text/lines/CSV out) ---------------------
        d("json_extract", &[], "json", "JSON extract",
          "Read scalar values out of each JSONL line. path is a dotted path with [n] index and [*].field array-collect, and a||b tries fallbacks; with a template, fill ${path} placeholders instead. Lines that yield nothing are dropped. Reading only, not a query language: for predicates/transforms use jq.",
          vec![
            text_opt("path", "Path", "Dotted path: keys, [n] index, [*].field array-collect. Use a||b to try fallbacks. Yields scalar leaves (string/number/bool)."),
            text_opt("template", "Template", "Output template with ${path} placeholders, to format several fields per line. $$ is a literal $."),
            text("separator", "Separator", "\n", "Joins multiple values collected by [*]."),
            boolean("raw", "Raw JSON", false, "Emit objects/arrays as compact JSON instead of skipping non-scalars."),
            enum_("on_invalid", "On invalid line", "skip", &["skip", "keep", "error"], "For a line that is not valid JSON: skip it, keep it verbatim, or fail."),
          ])
          .example(r#"{"action":"json_extract","options":{"path":"message.content[*].text||message.content"}}"#),
        d("json_filter", &[], "json", "JSON filter",
          "Keep the JSONL lines where path matches: equals for an exact value, matches for a regex, or neither to keep lines where the path resolves to any scalar (existence). invert drops matches instead.",
          vec![
            text_req("path", "Path", "Path to test (same syntax as json_extract)."),
            text_opt("equals", "Equals", "Keep lines where the extracted value equals this exactly."),
            text_opt("matches", "Matches", "Keep lines where the extracted value matches this regex."),
            boolean("invert", "Invert", false, "Drop matching lines instead of keeping them."),
            boolean("raw", "Raw JSON", false, "Count objects/arrays as present when testing existence."),
            text("separator", "Separator", ",", "Joins multiple collected values before comparison."),
            enum_("on_invalid", "On invalid line", "skip", &["skip", "keep"], "For a non-JSON line: skip it or keep it verbatim."),
          ])
          .example(r#"{"action":"json_filter","options":{"path":"type","matches":"^(user|assistant)$"}}"#),
        d("json_keys", &[], "json", "JSON keys",
          "Emit each JSONL object's keys, joined by the separator.",
          vec![
            text("separator", "Separator", ", ", "Joins the key names."),
            enum_("on_invalid", "On invalid line", "skip", &["skip", "keep"], "For a non-JSON line: skip it or keep it verbatim."),
          ])
          .example(r#"{"action":"json_keys"}"#),
        d("json_minify", &[], "json", "JSON minify",
          "Parse the whole input as one JSON value and re-serialize it compactly (valid by construction).",
          vec![])
          .example(r#"{"action":"json_minify"}"#),
        d("json_pretty", &[], "json", "JSON pretty",
          "Parse the whole input as one JSON value and re-serialize it with 2-space indentation.",
          vec![])
          .example(r#"{"action":"json_pretty"}"#),
        d("json_unescape", &[], "json", "JSON unescape",
          "Decode JSON string escapes (\\n \\t \\r \\uXXXX etc.) in the input text. The inverse of json-escape.",
          vec![])
          .example(r#"{"action":"json_unescape"}"#),
        d("json_to_csv", &[], "json", "JSON to CSV",
          "Flatten a JSON array (or JSONL) of flat objects into CSV: header is the union of keys, nested values become compact JSON, missing keys are empty cells.",
          vec![])
          .example(r#"{"action":"json_to_csv"}"#),
        d("csv_to_json", &[], "json", "CSV to JSON",
          "Read CSV (first line = header) into JSON objects keyed by the header. Emits JSONL (one object per line) by default, or one JSON array with `array`. Values are strings unless `infer` types them. Quote-aware, single-line.",
          vec![
            boolean("array", "As array", false, "Emit one JSON array instead of newline-delimited objects."),
            boolean("infer", "Infer types", false, "Parse true/false and numbers (empty -> null) instead of keeping every value a string."),
            text("delimiter", "Delimiter", ",", "Field delimiter of the input CSV."),
          ])
          .example(r#"{"action":"csv_to_json","options":{"infer":true}}"#),
        d("logfmt_to_json", &[], "json", "logfmt to JSON",
          "Convert each logfmt line (k=v k2=\"v 2\" flag) to a flat JSON object (JSONL). A bare word becomes key=true; values are strings.",
          vec![])
          .example(r#"{"action":"logfmt_to_json"}"#),
        d("json_to_logfmt", &[], "json", "JSON to logfmt",
          "Convert each JSONL object to a logfmt line (k=v ...), quoting values that hold a space, = or quote. Null values are skipped.",
          vec![])
          .example(r#"{"action":"json_to_logfmt"}"#),
        d("yaml_to_json", &[], "json", "YAML to JSON",
          "Parse one YAML document and emit it as JSON (pretty by default; `compact` for one line). Key order is preserved.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"yaml_to_json"}"#),
        d("json_to_yaml", &[], "json", "JSON to YAML",
          "Parse the whole input as one JSON value and emit it as YAML.",
          vec![])
          .example(r#"{"action":"json_to_yaml"}"#),
        d("json_to_jsonl", &[], "json", "JSON to JSONL",
          "Expand a JSON array into JSONL (one compact value per line). A non-array document becomes a single line.",
          vec![])
          .example(r#"{"action":"json_to_jsonl"}"#),
        d("jsonl_to_json", &[], "json", "JSONL to JSON",
          "Collect JSONL (one value per line) into a single JSON array (pretty by default; `compact` for one line).",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"jsonl_to_json"}"#),
        d("toml_to_json", &[], "json", "TOML to JSON",
          "Parse TOML and emit it as JSON (pretty by default; `compact` for one line). Key order preserved.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"toml_to_json"}"#),
        d("json_to_toml", &[], "json", "JSON to TOML",
          "Parse a JSON object and emit it as TOML. The top level must be an object and JSON null has no TOML equivalent (both are errors).",
          vec![])
          .example(r#"{"action":"json_to_toml"}"#),
        d("xml_to_json", &[], "json", "XML to JSON",
          "Parse XML into JSON (reading only). Attributes become @name keys, child elements become keys by tag (repeats become arrays), and text becomes #text (or the element's value if it has no attributes or children). Pretty by default; `compact` for one line.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"xml_to_json"}"#),
        d("json_to_xml", &[], "json", "JSON to XML",
          "Emit JSON as XML, inverting xml_to_json: @keys become attributes, #text becomes text, other keys become child elements (arrays repeat). The top level must be an object; a single key is the root element, else it is wrapped in <root>.",
          vec![])
          .example(r#"{"action":"json_to_xml"}"#),
        d("env_to_json", &[], "json", ".env to JSON",
          "Parse a .env file (KEY=VALUE, # comments, optional export, optional quotes) into a flat JSON object of string values.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"env_to_json"}"#),
        d("json_to_env", &[], "json", "JSON to .env",
          "Convert a flat JSON object to KEY=VALUE .env lines. Values with spaces/special characters are double-quoted; null is skipped.",
          vec![])
          .example(r#"{"action":"json_to_env"}"#),
        d("querystring_to_json", &[], "json", "Query string to JSON",
          "Parse a URL query string (a=1&b=2) into a JSON object, url-decoded; a repeated key becomes an array.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"querystring_to_json"}"#),
        d("json_to_querystring", &[], "json", "JSON to query string",
          "Convert a flat JSON object to a URL query string (a=1&b=2), url-encoded; an array value repeats its key. Null values are skipped.",
          vec![])
          .example(r#"{"action":"json_to_querystring"}"#),
        d("properties_to_json", &[], "json", ".properties to JSON",
          "Parse a Java .properties file (key=value or key:value, #/! comments) into a flat JSON object of string values. Dotted keys are kept flat.",
          vec![boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty.")])
          .example(r#"{"action":"properties_to_json"}"#),
        d("json_wrap", &[], "json", "JSON wrap",
          "Nest the whole JSON value under a dotted path, creating the intermediate objects (path services.app.environment wraps the input under those keys). A shaping primitive for composition.",
          vec![
            text_req("path", "Path", "Dotted path to nest the value under, e.g. services.app.environment."),
            boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty."),
          ])
          .example(r#"{"action":"json_wrap","options":{"path":"data"}}"#),
        d("json_merge", &[], "json", "JSON merge",
          "Shallow-merge a JSON object literal in front of the input object: the object's keys come first, the input's keys are added after (and win on conflict). A shaping primitive, e.g. to add apiVersion/kind/metadata around wrapped data.",
          vec![
            text_req("object", "Object", "A JSON object to merge in (its keys come first)."),
            boolean("compact", "Compact", false, "Emit compact single-line JSON instead of pretty."),
          ])
          .example(r#"{"action":"json_merge","options":{"object":"{\"kind\":\"ConfigMap\"}"}}"#),
        d("extract_json", &[], "json", "Extract JSON",
          "Pull a JSON value out of surrounding prose (e.g. an LLM reply): prefers a fenced ```json block, else takes the first balanced { } object or [ ] array (quote-aware). With pretty, the result is validated and re-indented; otherwise returned as found. Useful for deterministically post-processing model output before parsing.",
          vec![ boolean("pretty", "Pretty", false, "Validate and re-indent the extracted JSON.") ])
          .example(r#"{"action":"extract_json","options":{"pretty":true}}"#),
        d("json_set", &[], "json", "JSON set",
          "Set the value at a dotted path in a JSON document (a bounded write, not a query language): keys navigate objects, [n] indexes an array, and [*] sets every array element. Missing object keys along the path are created. value is parsed as JSON when it can be (so [] , null , 42 , true work), otherwise treated as a string. With jsonl, applies to each JSON object line.",
          vec![
            text_req("path", "Path", "Dotted path, e.g. cells[*].outputs or server.port."),
            text("value", "Value", "", "New value: parsed as JSON if possible, else a string."),
            boolean("jsonl", "JSONL", false, "Treat each line as its own JSON object."),
          ])
          .example(r#"{"action":"json_set","options":{"path":"cells[*].outputs","value":"[]"}}"#),
        d("json_delete", &[], "json", "JSON delete",
          "Remove the key or element at a dotted path in a JSON document (bounded write): keys navigate objects, [n] removes an array element, [*] clears an array. A missing path is a no-op. With jsonl, applies to each JSON object line. Useful for stripping fields (e.g. secrets, notebook execution counts).",
          vec![
            text_req("path", "Path", "Dotted path to remove, e.g. metadata.secret or cells[*].execution_count."),
            boolean("jsonl", "JSONL", false, "Treat each line as its own JSON object."),
          ])
          .example(r#"{"action":"json_delete","options":{"path":"metadata.token"}}"#),
        d("json_rename", &[], "json", "JSON rename key",
          "Rename the key at a dotted path to a new name, keeping its value (a bounded write). The path must end in a key; an earlier [*] renames the key inside every array element (e.g. messages[*].from -> role). A missing key is a no-op. With jsonl, applies to each JSON object line. Handy for schema/field migrations and dataset format conversions.",
          vec![
            text_req("path", "Path", "Dotted path ending in the key to rename, e.g. messages[*].from."),
            text_req("to", "To", "The new key name."),
            boolean("jsonl", "JSONL", false, "Treat each line as its own JSON object."),
          ])
          .example(r#"{"action":"json_rename","options":{"path":"conversations","to":"messages"}}"#),
        d("ipynb_to_python", &[], "json", "Notebook to Python",
          "Convert a Jupyter notebook (.ipynb JSON) to a Python script. Each code cell follows a `# %%` cell marker (the Jupytext / VS Code percent format); a markdown cell becomes a `# %% [markdown]` block with each line commented, unless markdown is set to skip. A cell's source may be a JSON string or an array of line strings.",
          vec![
            text("cell_marker", "Cell marker", "# %%", "The comment that separates cells (Jupytext percent format)."),
            enum_("markdown", "Markdown cells", "comment", &["comment", "skip"], "Emit markdown cells as commented blocks, or skip them."),
          ])
          .example(r#"{"action":"ipynb_to_python"}"#),
        // -- Whitespace -------------------------------------------------------
        d("trim_whitespace_right", &["twr"], "whitespace", "Trim right",
          "Remove trailing whitespace from each line.", vec![])
          .core()
          .example(r#"{"action":"trim_whitespace_right"}"#),
        d("trim_whitespace_left", &["twl"], "whitespace", "Trim left",
          "Remove leading whitespace from each line.", vec![])
          .example(r#"{"action":"trim_whitespace_left"}"#),
        d("trim_whitespace", &["tw"], "whitespace", "Trim both",
          "Remove leading and trailing whitespace from each line.", vec![])
          .core()
          .example(r#"{"action":"trim_whitespace"}"#),
        d("tabs_to_spaces", &["t2s"], "whitespace", "Tabs to spaces",
          "Replace tabs with spaces.",
          vec![int("width", "Tab width", 4, "Spaces per tab (values below 1 fall back to 4).")])
          .example(r#"{"action":"tabs_to_spaces","options":{"width":4}}"#),
        d("spaces_to_tabs", &["s2t"], "whitespace", "Spaces to tabs",
          "Replace runs of spaces with tabs.",
          vec![
            int("width", "Tab width", 4, "Spaces per tab (values below 1 fall back to 4)."),
            boolean("leading_only", "Leading only", false, "Convert only the leading run of spaces on each line."),
          ])
          .example(r#"{"action":"spaces_to_tabs","options":{"width":4,"leading_only":true}}"#),
        d("collapse_whitespace", &[], "whitespace", "Collapse whitespace",
          "Collapse runs of spaces/tabs to a single space (newlines untouched).", vec![])
          .example(r#"{"action":"collapse_whitespace"}"#),
        d("squeeze_spaces", &[], "whitespace", "Squeeze spaces",
          "Collapse runs of 2+ spaces to one, preserving leading indentation and tabs.", vec![])
          .example(r#"{"action":"squeeze_spaces"}"#),
        d("eol_to_space", &[], "whitespace", "EOL to space",
          "Replace line endings with single spaces.", vec![])
          .example(r#"{"action":"eol_to_space"}"#),
        d("trim_and_eol_to_space", &[], "whitespace", "Trim + EOL to space",
          "Trim each line, then join them with single spaces (drops the trailing newline).", vec![])
          .example(r#"{"action":"trim_and_eol_to_space"}"#),
        d("dedent", &[], "whitespace", "Dedent",
          "Remove the longest common leading-whitespace prefix shared by all non-blank lines.", vec![])
          .example(r#"{"action":"dedent"}"#),
        d("align_columns", &[], "whitespace", "Align columns",
          "Align cells on a separator across the block: trim, pad each column to its max width, rejoin. Lines without the separator pass through.",
          vec![text("separator", "Separator", "|", "Delimiter to split and align on.")])
          .example(r#"{"action":"align_columns","options":{"separator":"|"}}"#),
        d("wrap_text", &["reflow", "fill_paragraph"], "whitespace", "Wrap text",
          "Reflow text to a target column width (greedy first-fit, like Emacs fill-paragraph / Vim gq). Deterministic, no cursor state.",
          vec![
            int("width", "Width", 80, "Target column width."),
            boolean("preserve_paragraphs", "Preserve paragraphs", true, "Blank lines are hard breaks; lines within a paragraph reflow together. When false, the whole input is one paragraph."),
            boolean("prefix_aware", "Prefix aware", false, "Carry a leading quote/comment prefix (> , # , // , -- ) from a paragraph's first line onto every wrapped line."),
            boolean("break_long_words", "Break long words", false, "Hard-split a single token longer than width at the width boundary (default leaves it intact)."),
          ])
          .example(r#"{"action":"wrap_text","options":{"width":72}}"#),
        d("unwrap_text", &["unwrap", "unfill"], "whitespace", "Unwrap text",
          "Collapse the soft-wrapped lines of each paragraph back to one line, joined by a separator. Blank lines are preserved as breaks. The inverse of wrap_text.",
          vec![
            text("separator", "Separator", " ", "String that joins the lines of a paragraph."),
            boolean("prefix_aware", "Prefix aware", false, "Strip a common > / # prefix before joining and re-emit it once on the joined line."),
          ])
          .example(r#"{"action":"unwrap_text","options":{"separator":" "}}"#),
        // -- EOL --------------------------------------------------------------
        d("eol_crlf", &["eol_windows"], "eol", "EOL to CRLF",
          "Convert all line endings to CRLF (Windows).", vec![])
          .core()
          .example(r#"{"action":"eol_crlf"}"#),
        d("eol_lf", &["eol_unix"], "eol", "EOL to LF",
          "Convert all line endings to LF (Unix).", vec![])
          .core()
          .example(r#"{"action":"eol_lf"}"#),
        d("eol_cr", &["eol_mac"], "eol", "EOL to CR",
          "Convert all line endings to CR (classic Mac).", vec![])
          .example(r#"{"action":"eol_cr"}"#),
        // -- Text (character-level cleanup) -----------------------------------
        d("keep_chars", &[], "text", "Keep characters",
          "Keep only characters in a class (or an explicit set); drop the rest. Newlines are kept by default.",
          vec![
            enum_("class", "Class", "alnum", &["alnum", "letters", "digits", "ascii", "printable", "whitespace", "punctuation"], "Character class to keep (ignored if Set is given)."),
            text_opt("set", "Set", "Explicit characters to keep; overrides Class when present."),
            boolean("keep_newlines", "Keep newlines", true, "Always keep line breaks so line structure survives."),
          ])
          .example(r#"{"action":"keep_chars","options":{"class":"digits"}}"#),
        d("remove_chars", &[], "text", "Remove characters",
          "Drop characters in a class (or an explicit set); keep the rest. Newlines are kept by default.",
          vec![
            enum_("class", "Class", "punctuation", &["alnum", "letters", "digits", "ascii", "printable", "whitespace", "punctuation"], "Character class to remove (ignored if Set is given)."),
            text_opt("set", "Set", "Explicit characters to remove; overrides Class when present."),
            boolean("keep_newlines", "Keep newlines", true, "Never remove line breaks, even if the class would."),
          ])
          .example(r#"{"action":"remove_chars","options":{"class":"punctuation"}}"#),
        d("strip_control_chars", &[], "text", "Strip control characters",
          "Remove non-printable control characters, always keeping tab, newline, and carriage return.",
          vec![
            enum_("mode", "Mode", "delete", &["delete", "replace"], "Delete the control chars or replace each with Replacement."),
            text("replacement", "Replacement", " ", "Text substituted for each control char when Mode is replace."),
          ])
          .example(r#"{"action":"strip_control_chars"}"#),
        d("normalize", &[], "text", "Normalize characters",
          "Clean up look-alike and invisible characters an agent cannot see: curly quotes, fancy dashes, ellipsis, odd/zero-width whitespace, and (opt-in) accents.",
          vec![
            boolean("quotes", "Quotes", true, "Curly single/double quotes -> straight ' and \"."),
            boolean("dashes", "Dashes", true, "En/em/figure dashes and minus sign -> hyphen."),
            boolean("ellipsis", "Ellipsis", true, "The ellipsis character -> three dots."),
            boolean("spaces", "Spaces", true, "Non-breaking and other Unicode spaces -> a normal space; zero-width chars removed."),
            boolean("accents", "De-accent", false, "Map common accented Latin letters to their ASCII base (off by default)."),
          ])
          .example(r#"{"action":"normalize","options":{"accents":true}}"#),
        d("fix_mojibake", &[], "text", "Fix mojibake",
          "Repair text that was UTF-8 decoded as Windows-1252 (e.g. \"Ã©\" back to \"e-acute\", \"a-EUR(tm)\" back to a curly quote).",
          vec![])
          .example(r#"{"action":"fix_mojibake"}"#),
        d("slugify", &[], "text", "Slugify",
          "Turn each line into a URL/filename slug: de-accent to ASCII, keep alphanumerics, collapse everything else to a single separator.",
          vec![
            text("separator", "Separator", "-", "Character between slug words."),
            boolean("lowercase", "Lowercase", true, "Lowercase the result."),
          ])
          .example(r#"{"action":"slugify"}"#),
        d("chunk_text", &[], "text", "Chunk text",
          "Split text into fixed-size chunks for RAG ingestion: each chunk is about size characters, snapped back to a whitespace boundary so words are not cut, with overlap characters repeated at the start of the next chunk to preserve context. Chunks are joined by separator (default a \\n---\\n rule). Deterministic.",
          vec![
            int("size", "Size", 1000, "Approximate characters per chunk."),
            int("overlap", "Overlap", 0, "Characters repeated from the previous chunk."),
            text("separator", "Separator", "\n---\n", "Text placed between chunks."),
          ])
          .example(r#"{"action":"chunk_text","options":{"size":500,"overlap":50}}"#),
        d("strip_html_tags", &[], "text", "Strip HTML tags",
          "Remove HTML/XML tags (<...>), leaving the text content. Chain html_decode to decode entities.",
          vec![])
          .example(r#"{"action":"strip_html_tags"}"#),
        d("strip_markdown", &[], "text", "Strip Markdown",
          "Reduce common Markdown to plain text: drop code-fence lines, unwrap links/images to their text, and remove header/blockquote/list markers, horizontal rules, and */**/`/~~ emphasis. Underscore emphasis is left as-is (to protect snake_case). Best-effort strip, not a Markdown parser.",
          vec![])
          .example(r#"{"action":"strip_markdown"}"#),
        d("convert_comment_style", &[], "text", "Convert comment style",
          "Swap the line-comment marker at the start of each comment line from one style to another: hash (#), slashes (//), dash (--), or semicolon (;). Only a line whose first non-whitespace is the from marker is changed; indentation and the comment text are preserved. Block comments are not handled.",
          vec![
            enum_("from", "From", "slashes", &["hash", "slashes", "dash", "semicolon"], "The current comment marker style."),
            enum_("to", "To", "hash", &["hash", "slashes", "dash", "semicolon"], "The comment marker style to convert to."),
          ])
          .example(r#"{"action":"convert_comment_style","options":{"from":"slashes","to":"hash"}}"#),
        // -- Numbers / math ---------------------------------------------------
        d("arithmetic", &[], "text", "Arithmetic on numbers",
          "Apply an operation to each numeric token in scope (add/subtract/multiply/divide by a constant). Handles negatives and decimals. If 'places' is set the result is formatted to that many decimals; otherwise integral results drop the decimal point and fractional results use their minimal decimal form. Under a field scope it does column math.",
          vec![
            enum_("op", "Operation", "add", &["add", "subtract", "multiply", "divide"], "The operation to apply to each number (required)."),
            text_req("by", "By", "The constant operand (required). May be negative or decimal."),
            text_opt("find", "Find pattern", "Regex restricting which number tokens are matched (default: every standalone integer or decimal, -?\\d+(\\.\\d+)?)."),
            int_opt("places", "Decimal places", "Round/format the result to N decimals. When omitted, integral results have no decimal point."),
          ])
          .example(r#"{"action":"arithmetic","options":{"op":"multiply","by":1.1,"places":2}}"#),
        d("increment_numbers", &[], "text", "Increment numbers",
          "Add 'by' (default 1) to each matched number. Same matching, formatting, and scope rules as arithmetic.",
          vec![
            text("by", "By", "1", "Amount to add to each number (default 1; may be negative or decimal)."),
            text_opt("find", "Find pattern", "Regex restricting which number tokens are matched (default -?\\d+(\\.\\d+)?)."),
            int_opt("places", "Decimal places", "Round/format the result to N decimals."),
          ])
          .example(r#"{"action":"increment_numbers","options":{"by":1}}"#),
        d("round_numbers", &[], "text", "Round numbers",
          "Round each matched number to 'places' decimals (default 0). Same matching and scope rules as arithmetic.",
          vec![
            int("places", "Decimal places", 0, "Number of decimals to round to (default 0 = whole number)."),
            text_opt("find", "Find pattern", "Regex restricting which number tokens are matched (default -?\\d+(\\.\\d+)?)."),
          ])
          .example(r#"{"action":"round_numbers","options":{"places":2}}"#),
        d("pad_numbers", &[], "text", "Pad numbers",
          "Left-pad each integer run (\\d+) to 'width' using 'pad' (default \"0\"). A run already at least 'width' characters is left unchanged; only integer runs are targeted.",
          vec![
            int_opt("width", "Width", "Target width for each integer run (required)."),
            text("pad", "Pad string", "0", "String used to pad on the left (default \"0\")."),
          ])
          .example(r#"{"action":"pad_numbers","options":{"width":4}}"#),
        // -- Case -------------------------------------------------------------
        d("to_upper", &["upper"], "case", "Uppercase",
          "Convert text to UPPERCASE.", vec![])
          .core()
          .example(r#"{"action":"to_upper"}"#),
        d("to_lower", &["lower"], "case", "Lowercase",
          "Convert text to lowercase.", vec![])
          .core()
          .example(r#"{"action":"to_lower"}"#),
        d("to_proper", &["proper"], "case", "Proper case",
          "Capitalize the first letter of each word.",
          vec![boolean("blend", "Blend", false, "Keep the rest of each word as-is instead of lowercasing it.")])
          .example(r#"{"action":"to_proper"}"#),
        d("to_sentence", &["sentence"], "case", "Sentence case",
          "Capitalize the first letter of each sentence.",
          vec![boolean("blend", "Blend", false, "Keep the rest as-is instead of lowercasing first.")])
          .example(r#"{"action":"to_sentence"}"#),
        d("invert_case", &[], "case", "Invert case",
          "Swap the case of every letter.", vec![])
          .example(r#"{"action":"invert_case"}"#),
        d("to_camel", &["camel"], "case", "camelCase identifiers",
          "Recase each identifier to camelCase: `user_id` and `user-id` become `userId`. Splits identifiers on `_`/`-` delimiters and on existing case boundaries (`getUserId`, `HTTPServer`), lowercases the first word, and title-cases the rest. Surrounding separators (leading underscores, dunders) are left in place, so `__init__` becomes `init`. The one recasing the replacement layer cannot express (no case-changing backreference).",
          vec![])
          .example(r#"{"action":"to_camel"}"#),
        d("to_pascal", &["pascal"], "case", "PascalCase identifiers",
          "Recase each identifier to PascalCase (UpperCamelCase): `user_id` and `user-id` become `UserId`. Same identifier splitting as to_camel, but every word is title-cased.",
          vec![])
          .example(r#"{"action":"to_pascal"}"#),
        d("random_case", &[], "case", "Random case",
          "Randomly upper/lower each letter. Provide a seed for a reproducible result.",
          vec![int_opt("seed", "Seed", "Optional. If set, the result is deterministic; if omitted, it is random each run.")])
          .example(r#"{"action":"random_case","options":{"seed":42}}"#),
        // -- Affix ------------------------------------------------------------
        d("prepend", &[], "affix", "Prepend",
          "Insert text at the start of the whole document.",
          vec![multiline("text", "Text", "", "Text inserted before everything.")])
          .example(r#"{"action":"prepend","options":{"text":"-- header\n"}}"#),
        d("append", &[], "affix", "Append",
          "Insert text at the end of the whole document.",
          vec![multiline("text", "Text", "", "Text inserted after everything.")])
          .example(r#"{"action":"append","options":{"text":"\n-- footer"}}"#),
        d("insert_if_absent", &[], "affix", "Insert if absent",
          "Prepend (or append) text only when a marker is not already present, so re-running is idempotent -- no double insertion. marker defaults to the text itself; set regex to treat it as a pattern. position is prepend (default) or append. Ideal for stamping a license or banner header exactly once.",
          vec![
            multiline("text", "Text", "", "Text to insert if the marker is absent."),
            text_opt("marker", "Marker", "Substring (or pattern, with regex) that means the text is already present. Defaults to the text."),
            boolean("regex", "Marker is regex", false, "Treat the marker as a regular expression."),
            enum_("position", "Position", "prepend", &["prepend", "append"], "Where to insert when absent."),
          ])
          .example(r##"{"action":"insert_if_absent","options":{"text":"# Copyright 2026 Acme\n","marker":"Copyright"}}"##),
        d("prefix_lines", &[], "affix", "Prefix lines",
          "Insert text at the start of every line.",
          vec![text("text", "Text", "", "Text placed before each line.")])
          .core()
          .example(r#"{"action":"prefix_lines","options":{"text":"> "}}"#),
        d("suffix_lines", &[], "affix", "Suffix lines",
          "Append text to the end of every line.",
          vec![text("text", "Text", "", "Text placed after each line.")])
          .core()
          .example(r#"{"action":"suffix_lines","options":{"text":";"}}"#),
        d("wrap_lines", &[], "affix", "Wrap lines",
          "Wrap every line with a prefix and a suffix.",
          vec![
            text("prefix", "Prefix", "", "Text before each line."),
            text("suffix", "Suffix", "", "Text after each line."),
          ])
          .example(r#"{"action":"wrap_lines","options":{"prefix":"'","suffix":"',"}}"#),
        d("indent", &[], "affix", "Indent",
          "Add leading indentation to every line.",
          vec![
            int("spaces", "Spaces", 4, "Number of leading spaces (ignored if Text is set)."),
            text("text", "Text (overrides spaces)", "", "If set, this exact string is used as the indent instead of Spaces."),
          ])
          .example(r#"{"action":"indent","options":{"spaces":4}}"#),
        d("outdent", &[], "affix", "Outdent",
          "Remove leading indentation from every line.",
          vec![int("spaces", "Spaces", 4, "Max leading spaces to remove; a single leading tab counts as one level.")])
          .example(r#"{"action":"outdent","options":{"spaces":4}}"#),
        d("format_list", &[], "affix", "Format list",
          "Turn the lines into one structured snippet: wrap each line in an item template, join with a separator, and enclose in a header/footer (SQL IN lists, JSON arrays, etc.).",
          vec![
            text("header", "Header", "", "Text before all items."),
            text("item", "Item template", "$0", "Per-line template; $0 is the line, $$ is a literal $."),
            text("separator", "Separator", "", "Text between items (not after the last)."),
            text("footer", "Footer", "", "Text after all items."),
          ])
          .example(r#"{"action":"format_list","options":{"header":"IN (","item":"'$0'","separator":", ","footer":")"}}"#),
        // -- Encode -----------------------------------------------------------
        d("url_encode", &[], "encode", "URL encode",
          "Percent-encode the text for use in a URL.", vec![])
          .example(r#"{"action":"url_encode"}"#),
        d("url_decode", &[], "encode", "URL decode",
          "Decode percent-encoded (URL) text.", vec![])
          .example(r#"{"action":"url_decode"}"#),
        d("html_encode", &[], "encode", "HTML encode",
          "Escape HTML special characters.", vec![])
          .example(r#"{"action":"html_encode"}"#),
        d("html_decode", &["html_unescape"], "encode", "HTML decode",
          "Unescape HTML entities.", vec![])
          .example(r#"{"action":"html_decode"}"#),
        d("base64_encode", &[], "encode", "Base64 encode",
          "Encode the text as Base64.", vec![])
          .example(r#"{"action":"base64_encode"}"#),
        d("base64_decode", &[], "encode", "Base64 decode",
          "Decode Base64 text.", vec![])
          .example(r#"{"action":"base64_decode"}"#),
        d("rot13", &[], "encode", "ROT13",
          "Rotate ASCII letters by 13 (A-Z, a-z); other characters are unchanged. Self-inverse: apply twice to restore.", vec![])
          .example(r#"{"action":"rot13"}"#),
        d("hex_encode", &[], "encode", "Hex encode",
          "Hexlify: each byte becomes two lowercase hex digits, no separators.", vec![])
          .example(r#"{"action":"hex_encode"}"#),
        d("hex_decode", &[], "encode", "Hex decode",
          "Unhexlify a hex string back to text. Errors on odd length or a non-hex character.", vec![])
          .example(r#"{"action":"hex_decode"}"#),
        d("normalize_url", &[], "encode", "Normalize URL",
          "Canonicalize each http(s)-style URL line, leaving non-URL lines untouched: lowercase the scheme and host, remove a default port (80/443), drop a trailing dot on the host, and drop an empty ? query. Best-effort for scheme:// URLs; it does not re-encode paths.",
          vec![
            boolean("lowercase_host", "Lowercase host", true, "Lowercase the host (and scheme)."),
            boolean("remove_default_port", "Remove default port", true, "Drop :80 for http and :443 for https."),
            boolean("sort_query", "Sort query", false, "Sort the &-separated query parameters."),
            boolean("strip_fragment", "Strip fragment", false, "Drop the #fragment."),
          ])
          .example(r#"{"action":"normalize_url","options":{"sort_query":true}}"#),
        d("escape_regex", &[], "encode", "Escape regex",
          "Escape regex metacharacters in each line so the text matches literally when dropped into a pattern. Uses the same escaping as mog's own regex engine.",
          vec![])
          .example(r#"{"action":"escape_regex"}"#),
        d("escape_shell", &[], "encode", "Escape for shell",
          "Single-quote each line for safe use as one POSIX-shell word, writing any embedded single quote as the '\\'' idiom. A blank line becomes ''.",
          vec![])
          .example(r#"{"action":"escape_shell"}"#),
        d("escape_sql", &[], "encode", "Escape SQL literal",
          "SQL string-literal escape each line (embedded ' doubled to ''). With quote (default true) the value is wrapped in single quotes; set it false to escape without wrapping.",
          vec![
            boolean("quote", "Wrap in quotes", true, "Wrap each value in single quotes."),
          ])
          .example(r#"{"action":"escape_sql"}"#),
        d("escape_csv", &[], "encode", "Escape CSV field",
          "Quote each line as one RFC-4180 CSV field, doubling embedded quotes. A field is quoted only when it contains the delimiter, a quote, or a CR/LF; set always to quote every field.",
          vec![
            text("delimiter", "Delimiter", ",", "The delimiter to guard against."),
            boolean("always", "Always quote", false, "Quote every field, not just those that need it."),
          ])
          .example(r#"{"action":"escape_csv"}"#),
        d("escape_json", &[], "encode", "Escape JSON string",
          "Escape each line as a JSON string value: control characters become their short escapes (\\n \\t \\r \\b \\f) or \\u00XX, and \" and \\ are backslash-escaped. With quote (default true) the value is wrapped in double quotes; set it false to escape without wrapping. Non-ASCII passes through (JSON is UTF-8).",
          vec![boolean("quote", "Wrap in quotes", true, "Wrap the escaped value in double quotes.")])
          .example(r#"{"action":"escape_json"}"#),
        d("escape_xml", &[], "encode", "Escape XML/HTML",
          "XML/HTML-escape each line, replacing & < > \" ' with their entity references (&amp; &lt; &gt; &quot; &apos;). Safe for both element text and attribute values.",
          vec![])
          .example(r#"{"action":"escape_xml"}"#),
        d("escape_c", &[], "encode", "Escape C string",
          "Escape each line as a C/C++/Java string literal: \\ and \" are backslash-escaped, common control chars use their short escapes (\\t \\r \\n), and other control chars use a 3-digit octal escape. With quote (default true) the value is wrapped in double quotes.",
          vec![boolean("quote", "Wrap in quotes", true, "Wrap the escaped value in double quotes.")])
          .example(r#"{"action":"escape_c"}"#),
        // -- Compose ----------------------------------------------------------
        d("run_mog", &[], "compose", "Run mog (compose)",
          "Run another .mog pipeline inline at this point. Pass constants into it with an optional `with` map (name -> scalar): these overlay and override the child's own `constants`, so a shared fragment can be parameterized by its caller.",
          vec![text_req("file", "Script", "A library script name or a path (relative to this .mog) to run.")])
          .example(r#"{"action":"run_mog","options":{"file":"std/sql/quote-identifiers.mog","with":{"quote":"\""}}}"#),
        d("for_each_block", &[], "compose", "For each block (compose)",
          "For each block_start..block_end region, bind the header line's captures as constants (bind) and run a sub-.mog (run) over the block, replacing it with the result.",
          vec![
            text_req("block_start", "Block start", "Regex matching the block's opening line; its captures feed 'bind' via ${N}."),
            text_req("block_end", "Block end", "Regex matching the block's closing line."),
            text_req("run", "Script", "A library script name or path to run on each block."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to block_start / block_end."),
          ]),
        // -- Detect -----------------------------------------------------------
        d("detect_secrets", &[], "detect", "Detect secrets",
          "Flag lines that look like they contain a secret (AWS/Google/Stripe/GitHub/Slack keys, JWTs, private keys, api_key=... assignments), for review by the CLI report / --check.",
          vec![
            text("comment", "Comment prefix", "#", "Line-comment prefix for the marker."),
            enum_("tag", "Tag", "WARN", crate::actions::flag::TAGS, "Marker tag (all (mog)-scoped)."),
            enum_("position", "Position", "inline", &["inline", "before", "after"], "Where to place the marker."),
          ])
          .example(r#"{"action":"detect_secrets"}"#),
        d("detect_pii", &[], "detect", "Detect PII",
          "Flag lines that look like they contain shaped PII (email, SSN, credit card, phone, IP). Names need NER and are out of scope (flag, don't fake).",
          vec![
            text("comment", "Comment prefix", "#", "Line-comment prefix for the marker."),
            enum_("tag", "Tag", "WARN", crate::actions::flag::TAGS, "Marker tag (all (mog)-scoped)."),
            enum_("position", "Position", "inline", &["inline", "before", "after"], "Where to place the marker."),
          ])
          .example(r#"{"action":"detect_pii"}"#),
        // -- Assert -----------------------------------------------------------
        d("assert", &[], "assert", "Assert (post-condition)",
          "Fail the run unless every condition holds; otherwise pass the text through unchanged. Turns a scrub into an enforced guarantee and gives CI a content gate.",
          vec![
            text_opt("not_matches", "Not matches", "Fail if this regex is found in the output (e.g. a leftover secret)."),
            text_opt("matches", "Matches", "Fail if this regex is NOT found in the output (required content)."),
            int_opt("line_count", "Line count", "Fail unless the output has exactly this many lines."),
            int_opt("min_lines", "Min lines", "Fail if the output has fewer than this many lines."),
            int_opt("max_lines", "Max lines", "Fail if the output has more than this many lines."),
            boolean("ignore_case", "Ignore case", false, "Prepends (?i) to matches / not_matches."),
            text_opt("message", "Message", "Custom failure message on assertion failure."),
          ])
          .example(r#"{"action":"assert","options":{"not_matches":"sk_live_"}}"#),
        // -- SQL transpile (correctness tier) ---------------------------------
        d("sql_transpile", &[], "data", "SQL transpile (semantic)",
          "Transpile SQL between dialects with a real parser (polyglot-sql, a pure-Rust sqlglot port): handles function/type/quoting semantics the regex converters cannot. It REFORMATS the SQL (not a minimal diff) -- pick this when you want it to RUN in the target warehouse, and the regex converters when you want a reviewable minimal diff. Dialect names are lowercase (snowflake, bigquery, redshift, postgres, databricks, spark, ...).",
          vec![
            text_req("from", "From dialect", "Source SQL dialect, e.g. snowflake."),
            text_req("to", "To dialect", "Target SQL dialect, e.g. bigquery."),
          ])
          .example(r#"{"action":"sql_transpile","options":{"from":"snowflake","to":"bigquery"}}"#),
        d("sql_lint", &[], "data", "SQL lint (validate)",
          "Parse-validate SQL against a target dialect (via polyglot-sql) and annotate problems in place as `-- LINT ...` comment lines above the offending statement: syntax errors with line/column, plus (when semantic is on) query-quality warnings like SELECT * or LIMIT without ORDER BY. Clean SQL is returned unchanged, so `mog --check` exits non-zero exactly when there are findings. Ideal as the tail of a transpile recipe -- lint the OUTPUT against the target dialect to surface whatever the transpiler could not carry. Dialect names are lowercase (snowflake, bigquery, redshift, postgres, ...).",
          vec![
            text_req("dialect", "Dialect", "SQL dialect to validate against, e.g. snowflake."),
            boolean("semantic", "Quality warnings", true, "Also report query-quality warnings (SELECT *, aggregate without GROUP BY, DISTINCT+ORDER BY, LIMIT without ORDER BY)."),
            boolean("strict", "Strict syntax", false, "Reject non-canonical syntax the parser would otherwise tolerate, e.g. a trailing comma before FROM."),
            text_opt("note_prefix", "Note prefix", "Comment marker for inserted notes (default `-- LINT`)."),
          ])
          .example(r#"{"action":"sql_lint","options":{"dialect":"snowflake"}}"#),
        d("sql_format", &[], "data", "SQL format (pretty-print)",
          "Canonically re-format SQL within a single dialect (via polyglot-sql): reflows whitespace, casing, and layout to the dialect's standard style. Unlike `sql_transpile` it does NOT translate between dialects -- same dialect in and out -- so use it to normalize hand-written SQL before diffing or committing. Reformats (not a minimal diff). Dialect names are lowercase (snowflake, bigquery, postgres, duckdb, ...).",
          vec![
            text_req("dialect", "Dialect", "SQL dialect to format as, e.g. postgres."),
          ])
          .example(r#"{"action":"sql_format","options":{"dialect":"postgres"}}"#),
        d("sql_tables", &[], "data", "SQL source tables",
          "Parse SQL and REPLACE it with the list of physical source tables the query reads (one per line), resolved through CTEs, subqueries, and set operations -- CTE names and derived tables are excluded. Answers 'what does this query depend on?' with no warehouse connection. Names are fully qualified (schema.table as written) unless `qualified` is false. The list is deduped and sorted. `dialect` is optional (defaults to a dialect-agnostic grammar).",
          vec![
            text_opt("dialect", "Dialect", "SQL dialect to parse as (optional; default dialect-agnostic), e.g. snowflake."),
            boolean("qualified", "Qualified names", true, "Emit fully-qualified names (schema.table); set false for the bare table name."),
          ])
          .example(r#"{"action":"sql_tables","options":{"dialect":"snowflake"}}"#),
        d("sql_canonicalize_identifiers", &[], "data", "SQL canonicalize identifiers",
          "Parse SQL and regenerate it with every identifier normalized to the target dialect's canonical case (via polyglot-sql): Snowflake upper-cases unquoted identifiers, Postgres lower-cases them, etc. Set `quote` to wrap every identifier in the dialect's quote so the exact spelling survives a round-trip. A same-dialect rewrite that touches only identifier spelling (not layout, unlike `sql_format`) -- use it to make hand-written SQL consistent before diffing or transpiling. Dialect names are lowercase (snowflake, bigquery, postgres, ...).",
          vec![
            text_req("dialect", "Dialect", "SQL dialect whose identifier-casing rules to apply, e.g. snowflake."),
            boolean("quote", "Quote identifiers", false, "Also quote every identifier so its exact spelling is preserved across dialects."),
          ])
          .example(r#"{"action":"sql_canonicalize_identifiers","options":{"dialect":"snowflake"}}"#),
        d("sql_datatype_convert", &[], "data", "SQL data type convert",
          "Treat each non-blank line as a single SQL data type in the `from` dialect and rewrite it as the equivalent `to`-dialect type (via polyglot-sql): Oracle `VARCHAR2(50)` -> Postgres `VARCHAR(50)`, Snowflake `NUMBER(38,0)` -> BigQuery `NUMERIC(38, 0)`. Blank lines pass through. Operates per line (no surrounding statement needed), so it is the tool for converting a column-type list or building a type-mapping table -- for types embedded in full SQL use `sql_transpile`. Both dialects are required; names are lowercase.",
          vec![
            text_req("from", "From dialect", "Dialect the input types are written in, e.g. oracle."),
            text_req("to", "To dialect", "Dialect to convert the types to, e.g. postgres."),
          ])
          .example(r#"{"action":"sql_datatype_convert","options":{"from":"oracle","to":"postgres"}}"#),
        d("sql_lineage", &[], "data", "SQL column lineage",
          "Parse SQL and REPLACE it with a per-output-column lineage listing (`out_col <- table.src_col, ...`, one line each), resolving each SELECT projection back to the source column(s) it derives from through CTEs and subqueries. A literal/constant column, or a `*` that would need a schema to expand, shows `<- (none)`. The column-level companion to `sql_tables`: 'where does each output column come from?' with no warehouse connection. `dialect` is optional (defaults to a dialect-agnostic grammar).",
          vec![
            text_opt("dialect", "Dialect", "SQL dialect to parse as (optional; default dialect-agnostic), e.g. snowflake."),
          ])
          .example(r#"{"action":"sql_lineage","options":{"dialect":"snowflake"}}"#),
        d("sql_qualify", &[], "compare", "SQL qualify columns",
          "Parse SQL and regenerate it with every column reference qualified by its table (`col` -> `table.col`), resolved against a schema you supply as a named `ref` source of `table,column,type` rows (an optional header row is skipped). With `expand_stars` on, `SELECT *` becomes the explicit column list. The one schema-aware SQL action: it disambiguates columns a schema-free parse cannot. Reformats. `dialect` is optional. Bind the schema with `--source name=<file.csv>` or a `sources` entry.",
          vec![
            text_req("ref", "Schema source", "Name of a loaded source: CSV rows `table,column,type` describing the tables."),
            text_opt("dialect", "Dialect", "SQL dialect to parse/generate as (optional; default dialect-agnostic), e.g. snowflake."),
            boolean("expand_stars", "Expand stars", false, "Expand `SELECT *` into the explicit column list from the schema."),
            boolean("expand_alias_refs", "Expand alias refs", false, "Replace references to a SELECT alias with the underlying expression."),
          ])
          .example(r#"{"action":"sql_qualify","options":{"ref":"schema","dialect":"postgres","expand_stars":true}}"#),
    ];
    items
}

/// Serialize the descriptor table to pretty JSON (for `--list-actions --json`).
pub fn descriptors_json() -> String {
    serde_json::to_string_pretty(&descriptors()).expect("descriptors serialize")
}

/// One category in the action catalog: its canonical `name` (the value each
/// [`ActionDescriptor`] carries in `category`), a human `label` for display, and
/// its `rank` (0-based position in the intended display order). This registry is
/// the ONE source of truth for category identity, labels, and ordering, so the
/// studio and any other consumer read it instead of hardcoding the list and
/// drifting.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CategoryInfo {
    pub name: &'static str,
    pub label: &'static str,
    pub rank: usize,
}

/// The category registry as `(canonical name, display label)` pairs, in the
/// intended display order (index = `rank`). Every [`ActionDescriptor`]'s
/// `category` must be one of these names; the `categories_are_known` test derives
/// its check from this list so there is only ever one list to maintain.
const CATEGORY_TABLE: &[(&str, &str)] = &[
    ("replace", "Replace"),
    ("line", "Lines"),
    ("text", "Text"),
    ("whitespace", "Whitespace"),
    ("eol", "Line Endings"),
    ("case", "Case"),
    ("affix", "Affix"),
    ("encode", "Encode & Escape"),
    ("data", "Data & Tables"),
    ("json", "JSON & Formats"),
    ("compare", "Compare"),
    ("compose", "Compose"),
    ("detect", "Detect"),
    ("assert", "Assert"),
];

/// The ordered, labeled category registry. Consumers (the studio, tooling) use it
/// for palette grouping and ordering instead of hardcoding categories.
pub fn categories() -> Vec<CategoryInfo> {
    CATEGORY_TABLE
        .iter()
        .enumerate()
        .map(|(rank, &(name, label))| CategoryInfo { name, label, rank })
        .collect()
}

/// The category registry as JSON: `[{ "name", "label", "rank" }, ...]`. Emitted by
/// `mog --list-categories --json`.
pub fn categories_json() -> String {
    serde_json::to_string_pretty(&categories()).expect("categories serialize")
}

/// A JSON Schema (draft-07) for the `.mog` file structure: the top-level shape
/// plus the step shape, with the action name constrained to the known set. This
/// is the *execution-free* shape check, for editors and the store intake worker,
/// which cannot run `mog`. Per-action option validation stays the engine's job
/// (`validate_mog`), so `options` is an open object here.
pub fn schema_json() -> String {
    let mut names: Vec<String> = Vec::new();
    for d in descriptors() {
        names.push(d.name.to_string());
        for a in d.aliases {
            names.push((*a).to_string());
        }
    }
    names.sort();
    names.dedup();
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "mog script",
        "type": "object",
        "additionalProperties": false,
        "required": ["steps"],
        "properties": {
            "name": {"type": "string"},
            "description": {"type": "string"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "tier": {"enum": ["core", "full"]},
            "constants": {"type": "object"},
            "sources": {"type": "object", "additionalProperties": {"type": "string"}},
            "output_encoding": {"type": "string"},
            "steps": {"type": "array", "items": {"$ref": "#/definitions/step"}}
        },
        "definitions": {
            "step": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "action": {"enum": names},
                    "description": {"type": "string"},
                    "section": {"type": "string"},
                    "disabled": {"type": "boolean"},
                    "only_lines_matching": {"type": "string"},
                    "except_lines_matching": {"type": "string"},
                    "match_ignore_case": {"type": "boolean"},
                    "scope": {"$ref": "#/definitions/scope"},
                    "options": {"type": "object"}
                }
            },
            "scope": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "in_block": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["start", "end"],
                        "properties": {
                            "start": {"type": "string"},
                            "end": {"type": "string"},
                            "ignore_case": {"type": "boolean"}
                        }
                    },
                    "line_range": {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "from": {"type": "integer"},
                            "to": {"type": "integer"}
                        }
                    },
                    "field": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["index"],
                        "properties": {
                            "index": {"type": "integer"},
                            "delimiter": {"type": "string"}
                        }
                    },
                    "char_range": {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "from": {"type": "integer"},
                            "to": {"type": "integer"}
                        }
                    },
                    "invert": {"type": "boolean"}
                }
            }
        }
    });
    serde_json::to_string_pretty(&schema).expect("schema serialize")
}

/// The compact index view: identity, tier, category, and summary for every
/// action, with no params.
pub fn compact_descriptors() -> Vec<CompactDescriptor> {
    descriptors()
        .into_iter()
        .map(|d| CompactDescriptor {
            name: d.name,
            aliases: d.aliases,
            tier: d.tier,
            complexity: d.complexity,
            streamable: d.streamable,
            category: d.category,
            summary: d.summary,
        })
        .collect()
}

/// Serialize the compact index to pretty JSON (for `--list-actions --compact
/// --json`).
pub fn compact_descriptors_json() -> String {
    serde_json::to_string_pretty(&compact_descriptors()).expect("compact descriptors serialize")
}

/// Find a single descriptor by canonical name or by any of its aliases. Used by
/// `mog --describe <ACTION>`; the lookup resolves aliases to the canonical action.
pub fn find_descriptor(name: &str) -> Option<ActionDescriptor> {
    descriptors()
        .into_iter()
        .find(|d| d.name == name || d.aliases.contains(&name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::resolve;
    use std::collections::HashSet;

    #[test]
    fn every_descriptor_name_and_alias_resolves() {
        for desc in descriptors() {
            if matches!(
                desc.name,
                "run_mog"
                    | "for_each_block"
                    | "fill_from_list"
                    | "paste_column"
                    | "intersect"
                    | "subtract"
                    | "diff"
                    | "reconcile"
                    | "lookup"
                    | "sql_qualify"
            ) {
                continue; // built-ins, handled by the engine, not in resolve()
            }
            assert!(
                resolve(desc.name).is_some(),
                "descriptor '{}' is not a real action",
                desc.name
            );
            for alias in desc.aliases {
                assert!(
                    resolve(alias).is_some(),
                    "alias '{}' of '{}' is not a real action",
                    alias,
                    desc.name
                );
            }
        }
    }

    #[test]
    fn reversibility_flag_is_set_conservatively() {
        let by_name: std::collections::HashMap<&str, bool> = descriptors()
            .iter()
            .map(|d| (d.name, d.reversible))
            .collect();
        // Lossless, invertible transforms are reversible.
        for n in [
            "base64_encode",
            "url_decode",
            "eol_lf",
            "invert_case",
            "reverse_lines",
        ] {
            assert_eq!(by_name.get(n), Some(&true), "{n} should be reversible");
        }
        // Information-losing transforms are not.
        for n in [
            "to_upper",
            "sort_lines",
            "trim_whitespace",
            "remove_duplicate_lines",
        ] {
            assert_eq!(by_name.get(n), Some(&false), "{n} should not be reversible");
        }
    }

    #[test]
    fn descriptor_names_are_unique() {
        let mut seen = HashSet::new();
        for desc in descriptors() {
            assert!(
                seen.insert(desc.name),
                "duplicate descriptor '{}'",
                desc.name
            );
        }
    }

    #[test]
    fn every_descriptor_and_alias_dispatches() {
        // Guards the metadata/dispatch split: an action documented here must be one
        // the engine recognizes (`action_exists` is the same check the run loop uses
        // to reject unknown actions), or it would appear in --list-actions yet fail
        // to run. Aliases must dispatch too. A typo in either place is caught loudly.
        for desc in descriptors() {
            assert!(
                crate::engine::action_exists(desc.name),
                "descriptor '{}' is not a dispatchable action",
                desc.name
            );
            for alias in desc.aliases {
                assert!(
                    crate::engine::action_exists(alias),
                    "alias '{alias}' of '{}' does not dispatch",
                    desc.name
                );
            }
        }
    }

    #[test]
    fn categories_are_known() {
        // Derived from the one registry, so there is a single list to maintain.
        let known: HashSet<&str> = CATEGORY_TABLE.iter().map(|&(name, _)| name).collect();
        for desc in descriptors() {
            assert!(
                known.contains(desc.category),
                "bad category '{}'",
                desc.category
            );
        }
    }

    #[test]
    fn every_registered_category_is_used() {
        // The registry carries no dead entries: every category has >= 1 action.
        let used: HashSet<&str> = descriptors().iter().map(|d| d.category).collect();
        for (name, _) in CATEGORY_TABLE {
            assert!(
                used.contains(name),
                "registered category '{name}' has no actions"
            );
        }
    }

    #[test]
    fn categories_json_shape_is_stable() {
        let v: Value = serde_json::from_str(&categories_json()).unwrap();
        let arr = v.as_array().unwrap();
        assert_eq!(arr.len(), CATEGORY_TABLE.len());
        // Ranks are 0-based and match order; each has name + label.
        for (i, c) in arr.iter().enumerate() {
            assert_eq!(c["rank"].as_u64().unwrap() as usize, i);
            assert!(c["name"].as_str().is_some_and(|s| !s.is_empty()));
            assert!(c["label"].as_str().is_some_and(|s| !s.is_empty()));
        }
    }

    #[test]
    fn json_is_valid_and_substantial() {
        let value: Value = serde_json::from_str(&descriptors_json()).unwrap();
        assert!(value.as_array().unwrap().len() >= 40);
    }

    #[test]
    fn every_descriptor_has_a_tier_and_core_set_is_present() {
        // Every entry must serialize a "core"/"full" tier, and the curated Core
        // set must all be present and marked Core.
        let value: Value = serde_json::from_str(&descriptors_json()).unwrap();
        for entry in value.as_array().unwrap() {
            let tier = entry["tier"].as_str().expect("tier field present");
            assert!(
                tier == "core" || tier == "full",
                "bad tier '{tier}' on {}",
                entry["name"]
            );
        }
        let expected_core: HashSet<&str> = [
            "replace",
            "replace_regex",
            "replace_extended",
            "remove_lines_matching",
            "keep_lines_matching",
            "remove_empty_lines",
            "remove_duplicate_lines",
            "sort_lines",
            "trim_whitespace",
            "trim_whitespace_right",
            "to_upper",
            "to_lower",
            "prefix_lines",
            "suffix_lines",
            "insert_after_matching",
            "eol_lf",
            "eol_crlf",
        ]
        .into_iter()
        .collect();
        let core: HashSet<&str> = descriptors()
            .into_iter()
            .filter(|d| d.tier == Tier::Core)
            .map(|d| d.name)
            .collect();
        assert_eq!(core, expected_core, "Core tier set drifted");
    }

    #[test]
    fn every_core_action_has_an_example() {
        for desc in descriptors() {
            if desc.tier == Tier::Core {
                assert!(
                    desc.example.is_some(),
                    "core action '{}' is missing an example",
                    desc.name
                );
            }
        }
    }

    #[test]
    fn compact_matches_full_and_carries_a_tier() {
        let full = descriptors();
        let compact = compact_descriptors();
        assert_eq!(full.len(), compact.len(), "compact drops entries");
        for (f, c) in full.iter().zip(compact.iter()) {
            assert_eq!(f.name, c.name);
            assert_eq!(f.tier, c.tier);
            assert_eq!(f.category, c.category);
        }
        // The compact JSON must include a tier on every entry and omit params.
        let value: Value = serde_json::from_str(&compact_descriptors_json()).unwrap();
        for entry in value.as_array().unwrap() {
            assert!(entry.get("tier").is_some(), "compact entry lacks a tier");
            assert!(
                entry.get("params").is_none(),
                "compact entry should not carry params"
            );
        }
    }

    #[test]
    fn find_descriptor_resolves_name_and_alias() {
        // Canonical name.
        let by_name = find_descriptor("replace").expect("replace exists");
        assert_eq!(by_name.name, "replace");
        // Alias resolves to the same canonical action.
        let by_alias = find_descriptor("rr").expect("alias rr exists");
        assert_eq!(by_alias.name, "replace_regex");
        // Unknown name.
        assert!(find_descriptor("no_such_action").is_none());
    }

    #[test]
    fn schema_is_valid_json_and_constrains_action() {
        let v: Value = serde_json::from_str(&schema_json()).unwrap();
        assert_eq!(v["required"][0], "steps");
        let enum_ = v["definitions"]["step"]["properties"]["action"]["enum"]
            .as_array()
            .expect("action enum is an array");
        assert!(enum_.iter().any(|x| x == "replace"));
        assert!(enum_.iter().any(|x| x == "fill_from_list"));
        // additionalProperties must be locked so unknown top-level keys are caught.
        assert_eq!(v["additionalProperties"], serde_json::json!(false));
    }
}
