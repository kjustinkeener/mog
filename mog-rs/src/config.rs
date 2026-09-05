//! Persistent preferences in `MOG_HOME/config.toml` (the durable store the
//! chat-driven opt-out writes to). Keys: `reports` (bool), `template` (name),
//! and the `[report]` table of per-cap overrides (`report.max_diff_files`, ...)
//! that bound the run report.
//!
//! Note the layering: the CLI's `--report` stays opt-in and does NOT read
//! `config.reports`; this file is the preference store the MCP layer (Phase 3)
//! consults. `mog config` just reads and writes it.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

/// The on-disk `config.toml`. Unset keys are omitted (so an untouched key never
/// materializes a value), which keeps "unset" distinct from "set to the default".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// The `[report]` table: per-cap overrides for the bounded run report (see
    /// the spec, section 8). Omitted entirely until a `report.*` key is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<ReportConfig>,
}

/// The `[report]` table in `config.toml`: one entry per cap. Each value is
/// stored as a raw count where `0` means "unlimited" (the human opt-out); an
/// unset key falls through to the aggressive default. An empty table is never
/// written (all keys skip when `None`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_skip_over_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_diff_lines: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_diff_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_diff_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_diff_bytes: Option<u64>,
}

impl ReportConfig {
    /// Read one cap key's stored value (`0` = unlimited). An unknown key errors.
    fn field(&self, key: &str) -> Result<Option<u64>> {
        Ok(match key {
            "diff_skip_over_bytes" => self.diff_skip_over_bytes,
            "max_diff_lines" => self.max_diff_lines,
            "max_diff_bytes" => self.max_diff_bytes,
            "max_diff_files" => self.max_diff_files,
            "max_files" => self.max_files,
            "total_diff_bytes" => self.total_diff_bytes,
            other => bail!(
                "unknown report cap 'report.{other}' (expected one of: {})",
                crate::report::ReportCaps::KEYS.join(", ")
            ),
        })
    }

    /// Set one cap key. `cap` is `None` for "unlimited" (stored as `0`).
    fn set_field(&mut self, key: &str, cap: Option<usize>) -> Result<()> {
        let stored = cap.map(|n| n as u64).unwrap_or(0);
        match key {
            "diff_skip_over_bytes" => self.diff_skip_over_bytes = Some(stored),
            "max_diff_lines" => self.max_diff_lines = Some(stored),
            "max_diff_bytes" => self.max_diff_bytes = Some(stored),
            "max_diff_files" => self.max_diff_files = Some(stored),
            "max_files" => self.max_files = Some(stored),
            "total_diff_bytes" => self.total_diff_bytes = Some(stored),
            other => bail!(
                "unknown report cap 'report.{other}' (expected one of: {})",
                crate::report::ReportCaps::KEYS.join(", ")
            ),
        }
        Ok(())
    }

    /// Overlay every set cap onto `caps` (config precedence: over defaults, under
    /// CLI overrides). A stored `0` lifts that cap to unlimited.
    pub fn apply_to_caps(&self, caps: &mut crate::report::ReportCaps) {
        for key in crate::report::ReportCaps::KEYS {
            if let Ok(Some(v)) = self.field(key) {
                let cap = if v == 0 { None } else { Some(v as usize) };
                // KEYS is exactly the accepted set, so set_key never errors here.
                let _ = caps.set_key(key, cap);
            }
        }
    }
}

/// The config file path under a library root.
pub fn config_path(root: &Path) -> PathBuf {
    root.join("config.toml")
}

/// Load the config from `root/config.toml`, or the default (all unset) when the
/// file is absent. A malformed file is an error.
pub fn load(root: &Path) -> Result<Config> {
    let path = config_path(root);
    if !path.is_file() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read '{}'", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse '{}'", path.display()))
}

/// Write the config to `root/config.toml`, creating the root if needed.
pub fn save(root: &Path, cfg: &Config) -> Result<PathBuf> {
    std::fs::create_dir_all(root)
        .with_context(|| format!("failed to create '{}'", root.display()))?;
    let path = config_path(root);
    let text = toml::to_string(cfg).context("failed to serialize config")?;
    std::fs::write(&path, text).with_context(|| format!("failed to write '{}'", path.display()))?;
    Ok(path)
}

/// Parse the loose boolean spellings accepted for `reports`.
fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Ok(true),
        "false" | "off" | "no" | "0" => Ok(false),
        other => bail!("`reports` expects a boolean (on/off, true/false), got '{other}'"),
    }
}

/// The message listing the accepted config keys (top-level plus the `report.*`
/// cap keys), for "unknown key" errors.
fn known_keys_hint() -> String {
    let caps = crate::report::ReportCaps::KEYS
        .iter()
        .map(|k| format!("report.{k}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("expected 'reports', 'template', or one of: {caps}")
}

/// Apply `key=value` to a config in memory, validating the key and value. The
/// `report.*` keys route into the `[report]` table (a cap value of `0` /
/// `unlimited` lifts that cap).
pub fn apply(cfg: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "reports" => cfg.reports = Some(parse_bool(value)?),
        "template" => {
            if value.trim().is_empty() {
                bail!("`template` cannot be empty");
            }
            cfg.template = Some(value.to_string());
        }
        k if k.starts_with("report.") => {
            let sub = &k["report.".len()..];
            let cap = crate::report::parse_cap(value)?;
            cfg.report
                .get_or_insert_with(ReportConfig::default)
                .set_field(sub, cap)?;
        }
        other => bail!("unknown config key '{other}' ({})", known_keys_hint()),
    }
    Ok(())
}

/// Render one stored cap value for display (`"(unset)"`, `"unlimited"`, or `n`).
fn cap_display(v: Option<u64>) -> String {
    match v {
        None => "(unset)".to_string(),
        Some(0) => "unlimited".to_string(),
        Some(n) => n.to_string(),
    }
}

/// Read one key's value as a display string (`"(unset)"` when absent).
fn value_str(cfg: &Config, key: &str) -> Result<String> {
    match key {
        "reports" => Ok(cfg
            .reports
            .map(|b| b.to_string())
            .unwrap_or_else(|| "(unset)".to_string())),
        "template" => Ok(cfg
            .template
            .clone()
            .unwrap_or_else(|| "(unset)".to_string())),
        k if k.starts_with("report.") => {
            let sub = &k["report.".len()..];
            let stored = cfg
                .report
                .as_ref()
                .map(|r| r.field(sub))
                .transpose()?
                .flatten();
            // Validate the sub-key even when the whole table is unset.
            if cfg.report.is_none() {
                ReportConfig::default().field(sub)?;
            }
            Ok(cap_display(stored))
        }
        other => bail!("unknown config key '{other}' ({})", known_keys_hint()),
    }
}

/// `mog config get [KEY]`: print one key or the whole config.
pub fn get(lib_root: Option<&Path>, key: Option<&str>, json: bool) -> Result<i32> {
    let root =
        lib_root.ok_or_else(|| anyhow!("no library root: set MOG_HOME or pass --mog-dir"))?;
    let cfg = load(root)?;
    match key {
        Some(k) => {
            // Validate the key even when unset.
            let _ = value_str(&cfg, k)?;
            if json {
                let v = if let Some(sub) = k.strip_prefix("report.") {
                    match cfg
                        .report
                        .as_ref()
                        .and_then(|r| r.field(sub).ok())
                        .flatten()
                    {
                        Some(n) => serde_json::json!(n),
                        None => serde_json::Value::Null,
                    }
                } else {
                    match k {
                        "reports" => serde_json::json!(cfg.reports),
                        "template" => serde_json::json!(cfg.template),
                        _ => serde_json::Value::Null,
                    }
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ k: v }))?
                );
            } else {
                println!("{}", value_str(&cfg, k)?);
            }
        }
        None => {
            if json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                println!("reports = {}", value_str(&cfg, "reports")?);
                println!("template = {}", value_str(&cfg, "template")?);
                // The `[report]` caps, listed only when at least one is set.
                if cfg.report.is_some() {
                    for cap in crate::report::ReportCaps::KEYS {
                        let key = format!("report.{cap}");
                        let disp = value_str(&cfg, &key)?;
                        if disp != "(unset)" {
                            println!("{key} = {disp}");
                        }
                    }
                }
            }
        }
    }
    Ok(0)
}

/// `mog config set <KEY> <VALUE>`: validate, persist, and report.
pub fn set(lib_root: Option<&Path>, key: &str, value: &str, json: bool) -> Result<i32> {
    let root =
        lib_root.ok_or_else(|| anyhow!("no library root: set MOG_HOME or pass --mog-dir"))?;
    let mut cfg = load(root)?;
    apply(&mut cfg, key, value)?;
    let path = save(root, &cfg)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "path": path.display().to_string(),
                "config": cfg,
            }))?
        );
    } else {
        println!("set {key} = {} ({})", value_str(&cfg, key)?, path.display());
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn set_then_load_round_trips_reports_off() {
        let root = tempdir().unwrap();
        set(Some(root.path()), "reports", "off", false).unwrap();
        let cfg = load(root.path()).unwrap();
        assert_eq!(cfg.reports, Some(false));
        assert!(config_path(root.path()).is_file());
    }

    #[test]
    fn unknown_key_is_rejected() {
        let mut cfg = Config::default();
        assert!(apply(&mut cfg, "bogus", "x").is_err());
    }

    #[test]
    fn bad_bool_is_rejected() {
        let mut cfg = Config::default();
        assert!(apply(&mut cfg, "reports", "maybe").is_err());
    }

    #[test]
    fn report_cap_set_get_round_trips_and_unlimited_lifts() {
        let root = tempdir().unwrap();
        set(Some(root.path()), "report.max_diff_files", "5", false).unwrap();
        assert_eq!(
            value_str(&load(root.path()).unwrap(), "report.max_diff_files").unwrap(),
            "5"
        );

        // `unlimited` and `0` both lift the cap (stored as 0, shown as unlimited).
        set(
            Some(root.path()),
            "report.max_diff_files",
            "unlimited",
            false,
        )
        .unwrap();
        assert_eq!(
            value_str(&load(root.path()).unwrap(), "report.max_diff_files").unwrap(),
            "unlimited"
        );
        set(Some(root.path()), "report.max_files", "0", false).unwrap();
        assert_eq!(
            value_str(&load(root.path()).unwrap(), "report.max_files").unwrap(),
            "unlimited"
        );

        // Existing keys keep working alongside the table.
        set(Some(root.path()), "reports", "off", false).unwrap();
        let cfg = load(root.path()).unwrap();
        assert_eq!(cfg.reports, Some(false));
        assert_eq!(cfg.report.as_ref().unwrap().max_diff_files, Some(0));
    }

    #[test]
    fn report_cap_applies_to_caps_over_default() {
        let mut cfg = Config::default();
        apply(&mut cfg, "report.max_diff_files", "5").unwrap();
        apply(&mut cfg, "report.total_diff_bytes", "unlimited").unwrap();
        let mut caps = crate::report::ReportCaps::default();
        cfg.report.as_ref().unwrap().apply_to_caps(&mut caps);
        assert_eq!(caps.max_diff_files, Some(5));
        assert_eq!(caps.total_diff_bytes, None);
        // An untouched cap keeps its aggressive default.
        assert_eq!(caps.max_files, Some(500));
    }

    #[test]
    fn unknown_report_cap_is_rejected() {
        let mut cfg = Config::default();
        assert!(apply(&mut cfg, "report.bogus", "5").is_err());
    }
}
