//! `mog setup`: make a fresh install usable in one command.
//!
//! Setup does two independent jobs and prints one caveat:
//!
//! 1. **Store bootstrap** (the core): ensure the store root exists, then populate
//!    the managed mog dir `<root>/mogs/market` from the mog set embedded in
//!    the binary at build time. The managed set is always overwritten (one
//!    directory per mog). The user's own mog dir `<root>/mogs/user` is
//!    ensured to exist but NEVER written into or cleared. Nothing is preserved in
//!    market; stale layouts are deleted so their contents are not surfaced as
//!    mogs: the old `user/` / `community/` / `factory/` source subfolders, and
//!    any mog dir the prior flat refactor left directly under `<root>`.
//! 2. **MCP registration** (a helper): register the in-engine MCP server with
//!    Claude Code via `claude mcp add -s user mog -- <this-exe> mcp` (the server
//!    is a subcommand of this binary now, so there is no Node entry point or
//!    `MOG_BIN`). When `claude` is on PATH it is run; otherwise the command is
//!    printed for the user to run. MCP registration never fails setup.
//!
//! Because MCP servers load at client start, the tools appear only in a NEW
//! session; setup prints that caveat at the end.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use include_dir::{include_dir, Dir};
use serde_json::{json, Value};

/// The mog set embedded at build time (scripts + their fixture siblings),
/// assembled under `mog-rs/factory/` in the repo tree. Written out to
/// `<root>/mogs/market/` on setup (the repo dir name is kept to minimize churn).
static FACTORY_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/factory");

/// Options for [`run`], mapped from the CLI flags. `--mog-dir` and `--json` are
/// global CLI flags, threaded in here as `mog_dir` / `json`.
#[derive(Debug, Default)]
pub struct SetupOptions {
    /// Library root override (`--mog-dir`); falls back to `MOG_HOME` / the
    /// per-user default when `None`.
    pub mog_dir: Option<PathBuf>,
    /// Dry run: report every action, write and run nothing.
    pub print: bool,
    /// Skip the store bootstrap (part A).
    pub no_store: bool,
    /// Skip the MCP registration (part B).
    pub no_mcp: bool,
    /// Write `reports = false` into `config.toml` at install (opt out of the
    /// agent-surface dashboards up front).
    pub no_reports: bool,
    /// Emit a structured JSON summary instead of human-readable lines.
    pub json: bool,
}

/// Run `mog setup`. Returns the process exit code (0 on success). The store
/// bootstrap is the part that can fail the command; MCP registration is
/// best-effort and only reported.
pub fn run(opts: &SetupOptions) -> Result<i32> {
    let mut summary = json!({ "print": opts.print });

    let store = if opts.no_store {
        json!({ "skipped": true })
    } else {
        bootstrap_store(opts)?
    };
    summary["store"] = store;

    // Opt-out persistence: `--no-reports` writes `reports = false` up front. This
    // is the only setup path that materializes config.toml; otherwise the file is
    // left absent (the default the MCP layer treats as "reports on").
    summary["config"] = if opts.no_reports {
        write_no_reports(opts)?
    } else {
        json!({ "skipped": true })
    };

    let mcp = if opts.no_mcp {
        json!({ "skipped": true })
    } else {
        register_mcp(opts)
    };
    summary["mcp"] = mcp;

    let caveat =
        "MCP tools load when the client starts, so they appear only after you start a NEW session.";
    summary["caveat"] = json!(caveat);

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_human(&summary, caveat, opts);
    }
    Ok(0)
}

/// Part A: ensure the store root and the two `mogs/` subdirs exist, then (re)write
/// the embedded mog set into `<root>/mogs/market`, overwriting. The user dir
/// `<root>/mogs/user` is ensured but never written into.
fn bootstrap_store(opts: &SetupOptions) -> Result<Value> {
    let root = resolve_root(opts)?;
    let market = crate::library::market_dir(&root);
    let user = crate::library::user_dir(&root);

    let mut created_dirs: Vec<String> = Vec::new();
    for dir in [&market, &user] {
        if !dir.is_dir() {
            created_dirs.push(dir.display().to_string());
            if !opts.print {
                std::fs::create_dir_all(dir)
                    .with_context(|| format!("failed to create '{}'", dir.display()))?;
            }
        }
    }

    let mut factory_files: Vec<String> = Vec::new();
    write_embedded(&FACTORY_DIR, &market, opts.print, &mut factory_files)?;

    // Write the embedded factory dashboard templates at the outer root, so users
    // can see/copy them (the renderer also has them embedded as a fallback).
    let templates_root = root.join("templates").join("factory");
    let mut template_files: Vec<String> = Vec::new();
    write_embedded(
        &crate::report::TEMPLATE_DIR,
        &templates_root,
        opts.print,
        &mut template_files,
    )?;

    Ok(json!({
        "root": root.display().to_string(),
        "market_dir": market.display().to_string(),
        "user_dir": user.display().to_string(),
        "created_dirs": created_dirs,
        "factory_files": factory_files,
        "template_files": template_files,
        "wrote": !opts.print,
    }))
}

/// Write `reports = false` into `config.toml` (the `--no-reports` opt-out). A dry
/// run reports the path without writing.
fn write_no_reports(opts: &SetupOptions) -> Result<Value> {
    let root = resolve_root(opts)?;
    let path = crate::config::config_path(&root);
    if !opts.print {
        let mut cfg = crate::config::load(&root)?;
        cfg.reports = Some(false);
        crate::config::save(&root, &cfg)?;
    }
    Ok(json!({
        "path": path.display().to_string(),
        "reports": false,
        "wrote": !opts.print,
    }))
}

/// Recursively write every file in an embedded [`Dir`] under `dest_root`,
/// overwriting existing files. Collects the destination paths (for the report).
/// Under `print`, records paths but writes nothing.
fn write_embedded(
    dir: &Dir<'_>,
    dest_root: &Path,
    print: bool,
    written: &mut Vec<String>,
) -> Result<()> {
    for file in dir.files() {
        // `file.path()` is relative to the embedded root (e.g. `tidy-list.mog`).
        let dest = dest_root.join(file.path());
        written.push(dest.display().to_string());
        if !print {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            std::fs::write(&dest, file.contents())
                .with_context(|| format!("failed to write '{}'", dest.display()))?;
        }
    }
    for sub in dir.dirs() {
        write_embedded(sub, dest_root, print, written)?;
    }
    Ok(())
}

/// Resolve the concrete library root to write into: `--mog-dir` > `MOG_HOME` >
/// the per-user default. Unlike the run path, setup needs a real destination, so
/// a missing default (no `HOME`/`APPDATA`) is an error rather than "no library".
fn resolve_root(opts: &SetupOptions) -> Result<PathBuf> {
    opts.mog_dir
        .clone()
        .or_else(crate::library::default_root)
        .ok_or_else(|| {
            anyhow!(
                "could not determine a library root: pass --mog-dir <DIR> or set MOG_HOME \
                 (no per-user default is available in this environment)"
            )
        })
}

/// Part B: register the in-engine MCP server (`mog mcp`) with Claude Code.
///
/// The MCP server is now a subcommand of this binary (no Node, no separate entry
/// point, no `MOG_BIN`), so registration is just `claude mcp add -s user mog --
/// <this-exe> mcp`, with `MOG_HOME` set to the resolved library root so the
/// server finds the same library setup just populated. Never returns an error:
/// a failure to register is reported, not fatal.
fn register_mcp(opts: &SetupOptions) -> Value {
    // The running binary IS the server; it registers itself as the command.
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "action": "print",
                "registered": false,
                "note": format!("could not resolve this executable's path: {e}"),
            });
        }
    };

    // The library root the server should read (its MOG_HOME): the same resolved
    // root as the store bootstrap. Best-effort: if it can't be resolved, register
    // without the env and let the server fall back to its own default.
    let home = resolve_root(opts).ok();
    let env_part = home
        .as_ref()
        .map(|h| format!(" --env MOG_HOME={}", h.display()))
        .unwrap_or_default();
    let command = format!(
        "claude mcp add -s user mog{} -- {} mcp",
        env_part,
        exe.display()
    );

    // Run it when `claude` is on PATH and this is not a dry run; otherwise print
    // the command for the user to run.
    let have_claude = find_on_path("claude").is_some();
    if have_claude && !opts.print {
        let mut cmd = std::process::Command::new("claude");
        cmd.args(["mcp", "add", "-s", "user", "mog"]);
        if let Some(h) = &home {
            cmd.arg("--env").arg(format!("MOG_HOME={}", h.display()));
        }
        cmd.arg("--").arg(&exe).arg("mcp");
        match cmd.status() {
            Ok(status) if status.success() => json!({
                "action": "run",
                "registered": true,
                "command": command,
            }),
            Ok(status) => json!({
                "action": "run",
                "registered": false,
                "command": command,
                "note": format!("`claude mcp add` exited with status {status}; run it manually"),
            }),
            Err(e) => json!({
                "action": "run",
                "registered": false,
                "command": command,
                "note": format!("could not launch `claude`: {e}; run the command manually"),
            }),
        }
    } else {
        let note = if !have_claude {
            "`claude` was not found on PATH; run the command above after installing Claude Code \
             (or use `mog mcp install` for other clients)"
        } else {
            "dry run (--print); run the command above to register"
        };
        json!({
            "action": "print",
            "registered": false,
            "command": command,
            "note": note,
        })
    }
}

/// Find an executable named `name` on `PATH` (honoring `PATHEXT` on Windows).
/// Returns the first match, or `None` when not found.
fn find_on_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string())
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };
    for dir in std::env::split_paths(&paths) {
        let bare = dir.join(name);
        if bare.is_file() {
            return Some(bare);
        }
        for ext in &exts {
            let cand = dir.join(format!("{name}{ext}"));
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// Print the human-readable setup report from the JSON summary.
fn print_human(summary: &Value, caveat: &str, opts: &SetupOptions) {
    if opts.print {
        println!("mog setup (dry run: nothing written or run)");
    } else {
        println!("mog setup");
    }

    // Store.
    let store = &summary["store"];
    if store["skipped"].as_bool() == Some(true) {
        println!("  store: skipped (--no-store)");
    } else {
        let root = store["root"].as_str().unwrap_or("");
        let verb = if opts.print { "would use" } else { "using" };
        println!("  store: {verb} root {root}");
        if let Some(m) = store["market_dir"].as_str() {
            println!("    market: {m}");
        }
        if let Some(u) = store["user_dir"].as_str() {
            println!("    user:   {u} (never overwritten)");
        }
        if let Some(dirs) = store["created_dirs"].as_array() {
            let verb = if opts.print {
                "would create"
            } else {
                "created"
            };
            if dirs.is_empty() {
                println!("    root already present");
            } else {
                for d in dirs {
                    println!("    {verb}: {}", d.as_str().unwrap_or(""));
                }
            }
        }
        if let Some(files) = store["factory_files"].as_array() {
            let verb = if opts.print { "would write" } else { "wrote" };
            println!("    mogs: {verb} {} file(s)", files.len());
        }
        if let Some(files) = store["template_files"].as_array() {
            let verb = if opts.print { "would write" } else { "wrote" };
            println!("    templates: {verb} {} file(s)", files.len());
        }
    }

    // Config (only present when --no-reports was given).
    let config = &summary["config"];
    if config["skipped"].as_bool() != Some(true) {
        let verb = if opts.print { "would write" } else { "wrote" };
        println!(
            "  config: {verb} reports = false ({})",
            config["path"].as_str().unwrap_or("")
        );
    }

    // MCP.
    let mcp = &summary["mcp"];
    if mcp["skipped"].as_bool() == Some(true) {
        println!("  mcp: skipped (--no-mcp)");
    } else if mcp["registered"].as_bool() == Some(true) {
        println!("  mcp: registered with Claude Code");
        if let Some(cmd) = mcp["command"].as_str() {
            println!("    ran: {cmd}");
        }
    } else {
        println!("  mcp: not registered");
        if let Some(cmd) = mcp["command"].as_str() {
            println!("    run this to register:");
            println!("      {cmd}");
        }
        if let Some(note) = mcp["note"].as_str() {
            println!("    note: {note}");
        }
    }

    println!();
    println!("Next: {caveat}");
}
