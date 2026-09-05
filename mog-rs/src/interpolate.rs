//! Constant interpolation: expand `{{name}}` placeholders in a script's step
//! option strings (and line-scope patterns) from its `constants` block,
//! optionally overridden per-run (CLI `--define`), plus built-in values injected
//! under the reserved `@` namespace (`{{@today}}`, `{{@uuid}}`, ...).
//!
//! Interpolation runs ONCE at load time, before the engine sees any step, so
//! downstream actions receive fully-resolved option strings. In particular this
//! keeps placeholders clear of the regex replacement syntax: by the time
//! `replace_regex` expands `$1` / `${1}` backrefs, every `{{name}}` is already
//! gone.
//!
//! Two namespaces, two rules:
//!   - A user name (`{{name}}`, no `@`) expands only when the script actually
//!     opts into constants (a `constants` block or a non-`@` `--define`). With no
//!     user constants, a literal `{{name}}` is left verbatim so find/replacing
//!     mustache placeholders stays safe; once engaged, an undefined user name is a
//!     load error (typo guard).
//!   - A built-in (`{{@name}}`) expands whenever that value was injected (the CLI
//!     injects the `@` set); if it was not injected it is left verbatim, so raw
//!     `execute` (no CLI) never errors on a built-in.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use serde_json::{Map, Value};

use crate::model::Mog;

/// The per-file `@` placeholders, filled from the input file's path (see
/// [`expand_file_context`]). Kept separate from the load-time built-ins because
/// their value changes per input file.
const FILE_CONTEXT_KEYS: &[&str] = &["@filename", "@basename", "@ext", "@dirname"];

/// Build the file-context `@` defines from an input path: `@filename` (name with
/// extension), `@basename` (name without extension), `@ext` (extension, no dot),
/// `@dirname` (parent directory). A `None` path (stdin) yields empty strings.
pub fn file_context_defines(path: Option<&Path>) -> BTreeMap<String, String> {
    let str_of = |o: Option<std::ffi::OsString>| o.map(|s| s.to_string_lossy().into_owned());
    let (filename, basename, ext, dirname) = match path {
        Some(p) => (
            str_of(p.file_name().map(|s| s.to_os_string())).unwrap_or_default(),
            str_of(p.file_stem().map(|s| s.to_os_string())).unwrap_or_default(),
            str_of(p.extension().map(|s| s.to_os_string())).unwrap_or_default(),
            str_of(p.parent().map(|s| s.as_os_str().to_os_string())).unwrap_or_default(),
        ),
        None => Default::default(),
    };
    let mut m = BTreeMap::new();
    m.insert("@filename".to_string(), filename);
    m.insert("@basename".to_string(), basename);
    m.insert("@ext".to_string(), ext);
    m.insert("@dirname".to_string(), dirname);
    m
}

/// Whether the script references any file-context placeholder (`{{@filename}}`
/// etc.). A cheap over-approximating substring scan: a false positive only costs an
/// extra per-file clone, never a wrong result. Lets the CLI skip the per-file pass
/// entirely for the common case of a script that uses none.
pub fn uses_file_context(mog: &Mog) -> bool {
    fn scan(v: &Value, hit: &mut bool) {
        if *hit {
            return;
        }
        match v {
            Value::String(s) => {
                if FILE_CONTEXT_KEYS.iter().any(|k| s.contains(*k)) {
                    *hit = true;
                }
            }
            Value::Array(a) => a.iter().for_each(|x| scan(x, hit)),
            Value::Object(o) => o.values().for_each(|x| scan(x, hit)),
            _ => {}
        }
    }
    let mut hit = false;
    for step in &mog.steps {
        for p in [&step.only_lines_matching, &step.except_lines_matching]
            .into_iter()
            .flatten()
        {
            if FILE_CONTEXT_KEYS.iter().any(|k| p.contains(*k)) {
                return true;
            }
        }
        for v in step.options.values() {
            scan(v, &mut hit);
            if hit {
                return true;
            }
        }
    }
    false
}

/// Expand the per-file `@filename` / `@basename` / `@ext` / `@dirname` placeholders
/// in an already-loaded (load-time-interpolated) script for one input `path`. Run
/// per file, after the global load-time interpolation. Only these `@` placeholders
/// expand; user `{{name}}` placeholders are left as-is (already resolved at load).
pub fn expand_file_context(mog: &mut Mog, path: Option<&Path>) -> Result<()> {
    let defines = file_context_defines(path);
    for (i, step) in mog.steps.iter_mut().enumerate() {
        let n = i + 1;
        if let Some(p) = step.only_lines_matching.as_mut() {
            *p = expand(p, &defines, false)
                .map_err(|e| anyhow!("step {n}: only_lines_matching: {e}"))?;
        }
        if let Some(p) = step.except_lines_matching.as_mut() {
            *p = expand(p, &defines, false)
                .map_err(|e| anyhow!("step {n}: except_lines_matching: {e}"))?;
        }
        for (k, v) in step.options.iter_mut() {
            expand_value(v, &defines, false).map_err(|e| anyhow!("step {n}: option '{k}': {e}"))?;
        }
    }
    Ok(())
}

/// Expand every `{{name}}` placeholder in `mog`, drawing values from its own
/// `constants` overlaid by `overrides` (CLI `--define` plus any injected `@`
/// built-ins). Mutates the script in place. Errors on an undefined user
/// placeholder or a non-scalar constant value.
pub fn interpolate(mog: &mut Mog, overrides: &BTreeMap<String, String>) -> Result<()> {
    let consts = effective_constants(&mog.constants, overrides)?;
    // Nothing to do (and nothing that could error) when there are no constants
    // and no overrides: scripts predating this feature stay untouched.
    if consts.is_empty() {
        return Ok(());
    }
    // Non-`@` (user) constants are what gate typo-checking of user placeholders.
    // Built-ins alone (all keys `@`-prefixed) do NOT engage user expansion, so a
    // recipe with no constants still expands {{@today}} while leaving literal
    // {{mustache}} placeholders untouched.
    let user_engaged = mog.constants.keys().any(|k| !k.starts_with('@'))
        || overrides.keys().any(|k| !k.starts_with('@'));
    for (i, step) in mog.steps.iter_mut().enumerate() {
        let n = i + 1;
        if let Some(p) = step.only_lines_matching.as_mut() {
            *p = expand(p, &consts, user_engaged)
                .map_err(|e| anyhow!("step {n}: only_lines_matching: {e}"))?;
        }
        if let Some(p) = step.except_lines_matching.as_mut() {
            *p = expand(p, &consts, user_engaged)
                .map_err(|e| anyhow!("step {n}: except_lines_matching: {e}"))?;
        }
        for (k, v) in step.options.iter_mut() {
            expand_value(v, &consts, user_engaged)
                .map_err(|e| anyhow!("step {n}: option '{k}': {e}"))?;
        }
    }
    Ok(())
}

/// Merge script `constants` (stringified) with per-run `overrides` (which win).
fn effective_constants(
    defs: &Map<String, Value>,
    overrides: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for (k, v) in defs {
        out.insert(k.clone(), scalar_to_string(k, v)?);
    }
    for (k, v) in overrides {
        out.insert(k.clone(), v.clone());
    }
    Ok(out)
}

/// A constant value must be a scalar; stringify it the obvious way. Shared with
/// `run_mog`'s `with:` parameter passing so both accept the same value shapes.
pub(crate) fn scalar_to_string(key: &str, v: &Value) -> Result<String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        _ => bail!("constant '{key}' must be a string, number, or bool"),
    }
}

/// Expand placeholders inside any JSON string reachable from `v` (including
/// nested arrays/objects, so array-valued options are covered).
fn expand_value(
    v: &mut Value,
    consts: &BTreeMap<String, String>,
    user_engaged: bool,
) -> Result<()> {
    match v {
        Value::String(s) => *s = expand(s, consts, user_engaged)?,
        Value::Array(items) => {
            for it in items {
                expand_value(it, consts, user_engaged)?;
            }
        }
        Value::Object(map) => {
            for (_, vv) in map.iter_mut() {
                expand_value(vv, consts, user_engaged)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Expand `{{name}}` occurrences in `s`. All brace characters are ASCII, so the
/// byte scan stays on char boundaries; non-placeholder text is copied verbatim.
fn expand(s: &str, consts: &BTreeMap<String, String>, user_engaged: bool) -> Result<String> {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if bytes[i] == b'{' && i + 1 < s.len() && bytes[i + 1] == b'{' {
            if let Some(rel) = s[i + 2..].find("}}") {
                let end = i + 2 + rel; // index of the closing "}}"
                let name = s[i + 2..end].trim();
                let verbatim = &s[i..end + 2];
                if name.starts_with('@') {
                    // Built-in: expand if injected; else resolve an @env.NAME against
                    // the process environment on demand (unset -> empty); else leave
                    // verbatim.
                    match consts.get(name) {
                        Some(val) => out.push_str(val),
                        None => match name.strip_prefix("@env.") {
                            Some(var) => out.push_str(&std::env::var(var).unwrap_or_default()),
                            None => out.push_str(verbatim),
                        },
                    }
                } else if is_identifier(name) {
                    if !user_engaged {
                        // No user constants: leave literal {{x}} untouched.
                        out.push_str(verbatim);
                    } else {
                        match consts.get(name) {
                            Some(val) => out.push_str(val),
                            None => bail!(
                                "undefined constant '{{{{{name}}}}}'; \
                                 define it in \"constants\" or pass --define {name}=..."
                            ),
                        }
                    }
                } else {
                    // Not a placeholder (e.g. "{{ }}" or "{{1}}"): keep verbatim.
                    out.push_str(verbatim);
                }
                i = end + 2;
                continue;
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

/// A placeholder name: a user constant `[A-Za-z_][A-Za-z0-9_]*`, or a built-in in
/// the reserved `@` namespace (`@name` / `@group.name`, dots allowed).
fn is_identifier(s: &str) -> bool {
    if let Some(rest) = s.strip_prefix('@') {
        let mut chars = rest.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        return chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consts(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn expands_basic() {
        let c = consts(&[("schema", "public")]);
        assert_eq!(expand("{{schema}}.t", &c, true).unwrap(), "public.t");
    }

    #[test]
    fn allows_inner_whitespace() {
        let c = consts(&[("x", "Y")]);
        assert_eq!(expand("a{{ x }}b", &c, true).unwrap(), "aYb");
    }

    #[test]
    fn empty_value_strips() {
        let c = consts(&[("prefix", "")]);
        assert_eq!(expand("{{prefix}}Table", &c, true).unwrap(), "Table");
    }

    #[test]
    fn undefined_is_error_when_user_engaged() {
        let c = consts(&[("other", "x")]);
        assert!(expand("{{nope}}", &c, true).is_err());
    }

    #[test]
    fn undefined_is_verbatim_when_not_engaged() {
        // No user constants engaged: a literal {{x}} passes through untouched.
        let c = consts(&[("@today", "2020-01-01")]);
        assert_eq!(expand("{{x}}", &c, false).unwrap(), "{{x}}");
    }

    #[test]
    fn builtin_expands_and_mustache_passes_through() {
        // Only a built-in present (user not engaged): @today expands, {{x}} stays.
        let c = consts(&[("@today", "2020-01-01")]);
        assert_eq!(
            expand("d={{@today}} m={{x}}", &c, false).unwrap(),
            "d=2020-01-01 m={{x}}"
        );
    }

    #[test]
    fn unknown_builtin_is_verbatim() {
        let c = consts(&[("@today", "2020-01-01")]);
        assert_eq!(expand("{{@nope}}", &c, false).unwrap(), "{{@nope}}");
    }

    #[test]
    fn non_placeholder_braces_kept() {
        let c = consts(&[("a", "1")]);
        assert_eq!(expand("{{ }}", &c, true).unwrap(), "{{ }}");
        assert_eq!(expand("{{1x}}", &c, true).unwrap(), "{{1x}}");
        assert_eq!(
            expand("unclosed {{ still", &c, true).unwrap(),
            "unclosed {{ still"
        );
    }

    #[test]
    fn multiple_and_adjacent() {
        let c = consts(&[("a", "A"), ("b", "B")]);
        assert_eq!(expand("{{a}}{{b}}-{{a}}", &c, true).unwrap(), "AB-A");
    }

    #[test]
    fn overrides_win_over_defaults() {
        let mut defs = Map::new();
        defs.insert("s".into(), Value::String("public".into()));
        let over = consts(&[("s", "app")]);
        let eff = effective_constants(&defs, &over).unwrap();
        assert_eq!(eff.get("s").unwrap(), "app");
    }

    #[test]
    fn number_and_bool_stringify() {
        let mut defs = Map::new();
        defs.insert("n".into(), Value::Number(42.into()));
        defs.insert("b".into(), Value::Bool(true));
        let eff = effective_constants(&defs, &BTreeMap::new()).unwrap();
        assert_eq!(eff.get("n").unwrap(), "42");
        assert_eq!(eff.get("b").unwrap(), "true");
    }

    fn mog_from(json: &str) -> Mog {
        serde_json::from_str(json).expect("parse Mog")
    }

    #[test]
    fn no_constants_leaves_placeholders_untouched() {
        // A script with no constants and no overrides is passed through, so a
        // literal "{{x}}" in an option survives (e.g. mustache find/replace).
        let mut m = mog_from(
            r#"{ "steps": [ { "action": "replace",
                 "options": { "find": "{{x}}", "replace_with": "y" } } ] }"#,
        );
        interpolate(&mut m, &BTreeMap::new()).unwrap();
        assert_eq!(m.steps[0].get_string("find").unwrap(), "{{x}}");
    }

    #[test]
    fn undefined_errors_once_constants_engage() {
        // With constants present, an unknown placeholder is a load-time error.
        let mut m = mog_from(
            r#"{ "constants": { "a": "1" },
                 "steps": [ { "action": "replace",
                   "options": { "find": "{{typo}}", "replace_with": "y" } } ] }"#,
        );
        assert!(interpolate(&mut m, &BTreeMap::new()).is_err());
    }

    #[test]
    fn override_only_engages_interpolation() {
        // No constants block, but a --define makes the set non-empty, so a
        // matching placeholder expands.
        let mut m = mog_from(
            r#"{ "steps": [ { "action": "replace",
                 "options": { "find": "{{p}}x", "replace_with": "y" } } ] }"#,
        );
        let over: BTreeMap<String, String> = [("p".to_string(), "Q".to_string())].into();
        interpolate(&mut m, &over).unwrap();
        assert_eq!(m.steps[0].get_string("find").unwrap(), "Qx");
    }

    #[test]
    fn injected_builtin_expands_in_a_step() {
        // An injected @built-in expands even with no user constants; a bare
        // {{mustache}} in the same script would still pass through.
        let mut m = mog_from(
            r#"{ "steps": [ { "action": "prepend",
                 "options": { "text": "-- {{@today}}\n" } } ] }"#,
        );
        let over: BTreeMap<String, String> =
            [("@today".to_string(), "2026-08-23".to_string())].into();
        interpolate(&mut m, &over).unwrap();
        assert_eq!(m.steps[0].get_string("text").unwrap(), "-- 2026-08-23\n");
    }

    #[test]
    fn env_placeholder_resolves_on_demand() {
        std::env::set_var("MOG_TEST_ENV_ABC", "hi");
        let empty = consts(&[]);
        assert_eq!(
            expand("[{{@env.MOG_TEST_ENV_ABC}}]", &empty, false).unwrap(),
            "[hi]"
        );
        // An unset variable expands to empty.
        assert_eq!(
            expand("[{{@env.MOG_DEFINITELY_UNSET_ZZZ}}]", &empty, false).unwrap(),
            "[]"
        );
    }

    #[test]
    fn file_context_expands_filename_parts() {
        let mut m = mog_from(
            r#"{"steps":[{"action":"prepend",
                 "options":{"text":"f={{@filename}} b={{@basename}} e={{@ext}}\n"}}]}"#,
        );
        expand_file_context(&mut m, Some(Path::new("/tmp/data/report.csv"))).unwrap();
        assert_eq!(
            m.steps[0].get_string("text").unwrap(),
            "f=report.csv b=report e=csv\n"
        );
    }

    #[test]
    fn uses_file_context_detects_and_ignores() {
        let with =
            mog_from(r#"{"steps":[{"action":"prepend","options":{"text":"{{@filename}}"}}]}"#);
        assert!(uses_file_context(&with));
        let without = mog_from(r#"{"steps":[{"action":"to_upper"}]}"#);
        assert!(!uses_file_context(&without));
    }
}
