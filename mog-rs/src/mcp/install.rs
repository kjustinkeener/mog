//! `mog mcp install` / `mog mcp uninstall`: register (or unregister) this binary
//! as the `mog` MCP server with the user's MCP client apps, by editing each
//! client's own config. This is the "what do I do after downloading" step.
//!
//! Safety: never silently write to every client. A bare `mog mcp install` with
//! more than one client detected lists them and writes nothing (pass `--client`
//! or `--all`); with exactly one detected it registers that one. Every run also
//! PRINTS the ready-to-paste block as a manual fallback for unknown clients.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

/// A client app we know how to register with by editing its config JSON.
struct Client {
    /// Stable id used on the CLI (`--client <id>`).
    id: &'static str,
    /// Human label for messages.
    label: &'static str,
}

const CLIENTS: &[Client] = &[
    Client {
        id: "claude-code",
        label: "Claude Code",
    },
    Client {
        id: "claude-desktop",
        label: "Claude Desktop",
    },
    Client {
        id: "cursor",
        label: "Cursor",
    },
    Client {
        id: "windsurf",
        label: "Windsurf",
    },
];

fn find_client(id: &str) -> Option<&'static Client> {
    CLIENTS.iter().find(|c| c.id == id)
}

/// The absolute path to the running `mog` binary (the command a client invokes).
fn mog_exe() -> Result<PathBuf> {
    std::env::current_exe().map_err(|e| anyhow!("could not locate the running mog binary: {e}"))
}

/// The library dir to pass as `MOG_HOME`: `MOG_HOME` env > the per-user default.
fn mog_home() -> Option<PathBuf> {
    std::env::var_os("MOG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(crate::library::default_root)
}

/// The `mcpServers.mog` registration payload every client shares.
fn registration_block(exe: &Path, home: &Option<PathBuf>) -> Value {
    let mut block = json!({
        "command": exe.to_string_lossy(),
        "args": ["mcp"],
    });
    if let Some(home) = home {
        block["env"] = json!({ "MOG_HOME": home.to_string_lossy() });
    }
    block
}

/// The config file a given client keeps its `mcpServers` map in, if we know it
/// on this platform. `None` means "no known path here; print the manual block".
fn client_config_path(id: &str) -> Option<PathBuf> {
    let home = dirs_home()?;
    match id {
        // Claude Code keeps servers in ~/.claude.json (we prefer its CLI, but this
        // is the direct-edit fallback).
        "claude-code" => Some(home.join(".claude.json")),
        "claude-desktop" => {
            if cfg!(windows) {
                std::env::var_os("APPDATA").map(|a| {
                    PathBuf::from(a)
                        .join("Claude")
                        .join("claude_desktop_config.json")
                })
            } else if cfg!(target_os = "macos") {
                Some(
                    home.join("Library")
                        .join("Application Support")
                        .join("Claude")
                        .join("claude_desktop_config.json"),
                )
            } else {
                Some(
                    home.join(".config")
                        .join("Claude")
                        .join("claude_desktop_config.json"),
                )
            }
        }
        "cursor" => Some(home.join(".cursor").join("mcp.json")),
        "windsurf" => Some(
            home.join(".codeium")
                .join("windsurf")
                .join("mcp_config.json"),
        ),
        _ => None,
    }
}

/// The user's home directory (cross-platform), without pulling in the `dirs` crate.
fn dirs_home() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
    }
}

/// Is this client plausibly installed? A best-effort check so bare-install
/// detection doesn't offer apps that aren't here: the config file OR its parent
/// dir exists, or (claude-code) the `claude` CLI is on PATH.
fn client_detected(id: &str) -> bool {
    if id == "claude-code" && find_on_path("claude").is_some() {
        return true;
    }
    match client_config_path(id) {
        Some(p) => p.exists() || p.parent().map(|d| d.exists()).unwrap_or(false),
        None => false,
    }
}

/// Find an executable on PATH (honoring PATHEXT on Windows).
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

/// Read a client config JSON (or `{}` if absent), ensuring it is an object.
fn read_config(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("could not read {}: {e}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| anyhow!("{} is not valid JSON: {e}", path.display()))?;
    if !v.is_object() {
        bail!("{} is not a JSON object", path.display());
    }
    Ok(v)
}

fn write_config(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow!("could not create {}: {e}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)?;
    std::fs::write(path, text).map_err(|e| anyhow!("could not write {}: {e}", path.display()))?;
    Ok(())
}

/// Outcome of touching one client, for the JSON/text summary.
struct Outcome {
    client: &'static str,
    action: String,
    path: Option<String>,
    note: Option<String>,
}

/// Register with one client. Prefers the `claude` CLI for claude-code; otherwise
/// edits the client's config JSON in place (idempotent).
fn install_one(client: &Client, exe: &Path, home: &Option<PathBuf>) -> Result<Outcome> {
    // Claude Code: prefer its CLI so scope/format stay canonical.
    if client.id == "claude-code" {
        if let Some(claude) = find_on_path("claude") {
            let mut cmd = std::process::Command::new(claude);
            cmd.args(["mcp", "add", "-s", "user", "mog"]);
            if let Some(home) = home {
                cmd.arg("--env")
                    .arg(format!("MOG_HOME={}", home.to_string_lossy()));
            }
            cmd.arg("--").arg(exe).arg("mcp");
            match cmd.status() {
                Ok(s) if s.success() => {
                    return Ok(Outcome {
                        client: client.id,
                        action: "claude-cli".into(),
                        path: None,
                        note: Some("registered via `claude mcp add -s user`".into()),
                    })
                }
                Ok(s) => {
                    // Fall through to direct edit, noting the CLI attempt.
                    eprintln!("mog mcp: `claude mcp add` exited {s}; falling back to editing ~/.claude.json");
                }
                Err(e) => {
                    eprintln!("mog mcp: could not launch `claude` ({e}); editing ~/.claude.json");
                }
            }
        }
    }

    let path = client_config_path(client.id)
        .ok_or_else(|| anyhow!("no known config path for {} on this platform", client.label))?;
    let mut cfg = read_config(&path)?;
    let block = registration_block(exe, home);
    // Ensure mcpServers is an object, then set/replace `mog` (idempotent).
    let servers = cfg
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        bail!(
            "{}: mcpServers is present but not an object",
            path.display()
        );
    }
    let existed = servers.get("mog").is_some();
    servers.as_object_mut().unwrap().insert("mog".into(), block);
    write_config(&path, &cfg)?;
    Ok(Outcome {
        client: client.id,
        action: if existed {
            "updated".into()
        } else {
            "added".into()
        },
        path: Some(path.display().to_string()),
        note: None,
    })
}

/// Unregister from one client (removes the `mog` server key). Idempotent.
fn uninstall_one(client: &Client) -> Result<Outcome> {
    if client.id == "claude-code" {
        if let Some(claude) = find_on_path("claude") {
            match std::process::Command::new(claude)
                .args(["mcp", "remove", "-s", "user", "mog"])
                .status()
            {
                Ok(s) if s.success() => {
                    return Ok(Outcome {
                        client: client.id,
                        action: "claude-cli".into(),
                        path: None,
                        note: Some("removed via `claude mcp remove -s user`".into()),
                    })
                }
                _ => { /* fall through to direct edit */ }
            }
        }
    }
    let path = client_config_path(client.id)
        .ok_or_else(|| anyhow!("no known config path for {} on this platform", client.label))?;
    if !path.exists() {
        return Ok(Outcome {
            client: client.id,
            action: "skipped".into(),
            path: Some(path.display().to_string()),
            note: Some("no config file".into()),
        });
    }
    let mut cfg = read_config(&path)?;
    let mut removed = false;
    if let Some(servers) = cfg.get_mut("mcpServers").and_then(Value::as_object_mut) {
        removed = servers.remove("mog").is_some();
    }
    if removed {
        write_config(&path, &cfg)?;
    }
    Ok(Outcome {
        client: client.id,
        action: if removed {
            "removed".into()
        } else {
            "skipped".into()
        },
        path: Some(path.display().to_string()),
        note: if removed {
            None
        } else {
            Some("no `mog` registration found".into())
        },
    })
}

/// The manual paste block, always printed as a fallback.
fn manual_block_text(exe: &Path, home: &Option<PathBuf>) -> String {
    let block = json!({ "mog": registration_block(exe, home) });
    serde_json::to_string_pretty(&block).unwrap_or_default()
}

fn outcomes_json(outcomes: &[Outcome]) -> Value {
    Value::Array(
        outcomes
            .iter()
            .map(|o| {
                json!({
                    "client": o.client,
                    "action": o.action,
                    "path": o.path,
                    "note": o.note,
                })
            })
            .collect(),
    )
}

/// Entry point for `mog mcp install`.
pub fn install(client: Option<&str>, all: bool, json_out: bool) -> Result<i32> {
    let exe = mog_exe()?;
    let home = mog_home();

    let targets: Vec<&'static Client> = if all {
        CLIENTS.iter().collect()
    } else if let Some(id) = client {
        match find_client(id) {
            Some(c) => vec![c],
            None => bail!(
                "unknown client '{id}'. Known: {}",
                CLIENTS.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")
            ),
        }
    } else {
        // Bare: detect. Register exactly one; never quietly edit several.
        let detected: Vec<&'static Client> =
            CLIENTS.iter().filter(|c| client_detected(c.id)).collect();
        match detected.len() {
            0 => {
                let manual = manual_block_text(&exe, &home);
                if json_out {
                    print_json(&json!({
                        "installed": [],
                        "detected": [],
                        "manual_block": manual,
                        "note": "no known MCP client detected; paste manual_block into your client's mcpServers, or pass --client <id>",
                    }));
                } else {
                    println!("No known MCP client detected.");
                    println!("Paste this into your client's mcpServers config, or pass --client <id> / --all:\n");
                    println!("{manual}");
                }
                return Ok(0);
            }
            1 => detected,
            _ => {
                let ids: Vec<&str> = detected.iter().map(|c| c.id).collect();
                if json_out {
                    print_json(&json!({
                        "installed": [],
                        "detected": ids,
                        "note": "multiple clients detected; pass --client <id> or --all (nothing written)",
                    }));
                } else {
                    println!("Multiple MCP clients detected: {}", ids.join(", "));
                    println!("Nothing was written. Re-run with --client <id> for one, or --all for every detected client.");
                }
                return Ok(0);
            }
        }
    };

    let mut outcomes = Vec::new();
    let mut had_error = false;
    for c in targets {
        match install_one(c, &exe, &home) {
            Ok(o) => outcomes.push(o),
            Err(e) => {
                had_error = true;
                outcomes.push(Outcome {
                    client: c.id,
                    action: "error".into(),
                    path: None,
                    note: Some(e.to_string()),
                });
            }
        }
    }

    let manual = manual_block_text(&exe, &home);
    if json_out {
        print_json(&json!({ "installed": outcomes_json(&outcomes), "manual_block": manual }));
    } else {
        for o in &outcomes {
            match o.path.as_deref() {
                Some(p) => println!("{}: {} ({})", o.client, o.action, p),
                None => println!("{}: {}", o.client, o.action),
            }
            if let Some(n) = &o.note {
                println!("    {n}");
            }
        }
        println!(
            "\nRestart the client app to pick up the server. Manual block (any other client):\n"
        );
        println!("{manual}");
    }
    Ok(if had_error { 1 } else { 0 })
}

/// Entry point for `mog mcp uninstall`.
pub fn uninstall(client: Option<&str>, all: bool, json_out: bool) -> Result<i32> {
    let targets: Vec<&'static Client> = if all {
        CLIENTS.iter().collect()
    } else if let Some(id) = client {
        match find_client(id) {
            Some(c) => vec![c],
            None => bail!(
                "unknown client '{id}'. Known: {}",
                CLIENTS.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")
            ),
        }
    } else {
        // Bare uninstall: only touch clients that actually have a registration.
        CLIENTS.iter().filter(|c| client_detected(c.id)).collect()
    };

    let mut outcomes = Vec::new();
    let mut had_error = false;
    for c in targets {
        match uninstall_one(c) {
            Ok(o) => outcomes.push(o),
            Err(e) => {
                had_error = true;
                outcomes.push(Outcome {
                    client: c.id,
                    action: "error".into(),
                    path: None,
                    note: Some(e.to_string()),
                });
            }
        }
    }

    if json_out {
        print_json(&json!({ "uninstalled": outcomes_json(&outcomes) }));
    } else {
        for o in &outcomes {
            match o.path.as_deref() {
                Some(p) => println!("{}: {} ({})", o.client, o.action, p),
                None => println!("{}: {}", o.client, o.action),
            }
            if let Some(n) = &o.note {
                println!("    {n}");
            }
        }
    }
    Ok(if had_error { 1 } else { 0 })
}

fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}
