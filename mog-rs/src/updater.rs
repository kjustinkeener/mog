//! `mog update`: one command that brings a machine current with the registry --
//! mogs and the engine binary both. It is the single implementation the CLI,
//! the MCP server, and Studio all drive (the latter two shell out to it), so the
//! update logic lives in exactly one place.
//!
//! Mogs sync by content hash (see [`crate::market_client::sync`]); the engine
//! self-replaces from a signed manifest (see [`crate::selfupdate`]). Either track
//! can be skipped. `--check` reports what would change without touching anything.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Serialize;

/// What to do this run.
pub struct Options {
    /// Report only; write nothing (mogs and engine both).
    pub check: bool,
    /// Skip the mog sync.
    pub no_recipes: bool,
    /// Skip the engine binary swap.
    pub no_engine: bool,
    /// Remove orphaned installs (on disk, gone from the catalog). Off by default.
    pub prune: bool,
    /// Machine-readable output.
    pub json: bool,
}

#[derive(Serialize)]
struct EngineReport {
    update_available: bool,
    applied: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    version: String,
}

pub fn run(root: Option<&Path>, opts: &Options) -> Result<i32> {
    let root = root.ok_or_else(|| anyhow!("no library root (set MOG_HOME or --mog-dir)"))?;
    let base = crate::market_client::market_base_url()?;

    // --- mogs ---
    let plan = if opts.no_recipes {
        None
    } else {
        Some(crate::market_client::sync(
            root, &base, opts.check, opts.prune,
        )?)
    };

    // --- engine ---
    let engine = if opts.no_engine {
        None
    } else if opts.check {
        let st = crate::selfupdate::check(&base)?;
        Some(EngineReport {
            update_available: st.update_available,
            applied: false,
            version: st.version,
        })
    } else {
        let st = crate::selfupdate::check(&base)?;
        let applied = if st.update_available {
            crate::selfupdate::apply(&base)?
        } else {
            false
        };
        Some(EngineReport {
            update_available: st.update_available,
            applied,
            version: st.version,
        })
    };

    if opts.json {
        println!(
            "{}",
            serde_json::json!({
                "check": opts.check,
                "recipes": plan,
                "engine": engine,
            })
        );
        return Ok(0);
    }

    // Human render.
    if let Some(p) = &plan {
        let verb = if opts.check { "would " } else { "" };
        let quiet = p.is_empty();
        println!("mogs: {} in catalog", p.catalog);
        for n in &p.added {
            println!("  {verb}add {n}");
        }
        for n in &p.changed {
            println!("  {verb}update {n}");
        }
        for n in &p.revoked {
            println!("  {verb}remove {n} (revoked)");
        }
        for n in &p.orphaned {
            let tail = if opts.prune {
                "(orphaned, removed)"
            } else {
                "(orphaned; --prune to remove)"
            };
            println!("  {verb}skip {n} {tail}");
        }
        if quiet {
            println!("  everything up to date");
        }
    }
    if let Some(e) = &engine {
        if !e.update_available {
            println!("engine: up to date");
        } else if opts.check {
            println!("engine: update available ({})", e.version);
        } else if e.applied {
            println!(
                "engine: updated to {} (takes effect on the next mog run)",
                e.version
            );
        }
    }
    Ok(0)
}
