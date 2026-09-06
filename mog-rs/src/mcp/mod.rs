//! `mog mcp`: the Mog MCP server, folded into the engine binary.
//!
//! This supersedes the standalone Node/TypeScript server (retired `mcp/`). It
//! speaks the MCP stdio protocol - newline-delimited JSON-RPC 2.0 on stdout,
//! logs on stderr - and exposes the SAME nine tools with byte-identical names,
//! schemas, and result shapes. The JSON-RPC framing is hand-rolled (no MCP SDK
//! dependency): the surface is small and fixed, and hand-rolling gives exact
//! control over the `resource_link` content the run dashboard needs.
//!
//! The tool handlers live in [`tools`]; client registration in [`install`].

pub mod install;
mod tools;

use std::io::{BufRead, Write};

use anyhow::Result;
use serde_json::{json, Value};

/// The protocol version we speak. If a client requests a different one during
/// `initialize`, we echo the client's version back (best-effort negotiation);
/// this is the fallback when it sends none.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Server-level instructions: the client injects these into the model's context,
/// so this is where we teach an agent WHEN to reach for Mog un-prompted. Ported
/// verbatim from the Node server's `INSTRUCTIONS` (must-preserve behavior).
const INSTRUCTIONS: &str = "\
Mog is a deterministic text-transform engine: it applies the SAME mechanical edit to
every matching line or file EXACTLY, with a preview and a backup, from reusable recipes.
It is the write-side complement to grep: grep FINDS text, Mog CHANGES it -- do NOT use
Mog for pure searching/filtering.

Before hand-editing text in a repetitive, rule-based, or error-prone way, search mog_market
(query) for a recipe; if none fits, author a small .mog from mog_actions instead of doing
the edits by hand. This covers everyday tasks, not just code:
  - Clean up a list: de-duplicate, sort, trim, drop blanks (emails, contacts, requirements,
    tags, todo). Hand-editing a long list can silently drop or alter entries; Mog is exact.
  - Whitespace / line endings / tabs / final newline; strip ANSI color codes from output.
  - Redact secrets or PII before sharing a log, config, or message thread.
  - Encode / escape: base64, URL, HTML-escape (hand-escaping is easy to get subtly wrong).
  - Format many lines the same way: comment out a block, markdown bullets/checklist,
    number a list, prefix/indent, join/split on a delimiter.
  - Any find/replace, especially regex, that spans MANY files or many occurrences.

Prefer Mog specifically when: the change repeats across MANY files or lines (not just one
or two); the transform is regex/byte-fiddly enough that a hand edit risks silent errors;
or determinism and a reviewable diff matter. For a single small one-off edit, just edit it.

Workflow: mog_market (find a recipe) or mog_actions (discover actions to author one) ->
mog_preview (impact) -> mog_apply (reversible modes only). mog_validate / mog_test check it.";

/// The nine tool definitions (name, description, JSON-Schema input). Ported from
/// the Node server's `registerTool` calls; names and schemas are kept identical.
fn tool_definitions() -> Value {
    json!([
        {
            "name": "mog_actions",
            "description": "Discover Mog's action vocabulary to AUTHOR a text-transform recipe when mog_market has no ready one for a repetitive, bulk, or fiddly text edit. query to find actions by task: a ranked keyword search (tokenized OR-match, synonym-aware, over name/summary/aliases/category, across all tiers) -- multi-word queries like 'whitespace cleanup' work, best matches first. name for one action's full params + example. No query/name: compact core-tier list (name/tier/category/summary); tier:'full' for the whole catalog. category to filter. Author a .mog (JSON) from these, then preview/apply it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Return this single action's full descriptor." },
                    "query": { "type": "string", "description": "Ranked keyword search (tokenized, synonym-aware) over name/summary/aliases/category, all tiers." },
                    "tier": { "type": "string", "enum": ["core", "full"], "description": "core (default) = common actions; full = whole catalog. Ignored when query is set." },
                    "category": { "type": "string", "description": "Filter by category: replace|line|whitespace|eol|case|affix|encode|compose." }
                }
            }
        },
        {
            "name": "mog_validate",
            "description": "Parse/validate a .mog JSON string. Returns {ok:true} or a structured error {message,step,action}. No files touched.",
            "inputSchema": {
                "type": "object",
                "properties": { "mog": { "type": "string", "description": "The .mog script as JSON text." } },
                "required": ["mog"]
            }
        },
        {
            "name": "mog_preview",
            "description": "Safe read-only impact report: per-file changed/added/removed lines and flags. Name an installed recipe with recipe (cheapest, from mog_market) or pass a script inline with mog. Give inputs (files/globs) or inline text. Writes nothing. Emits an HTML run dashboard by default (a resource_link to what WOULD change); set report:false, or persist mog_config {reports:false}, to disable it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mogname": { "type": "string", "description": "An installed recipe name (from mog_market), resolved from the library. Use this instead of pasting a script you already have installed; give exactly one of mogname or mog." },
                    "mog": { "type": "string", "description": "The .mog script as JSON text, for a recipe you are authoring. Omit when passing mogname." },
                    "inputs": { "type": "array", "items": { "type": "string" }, "description": "Files or glob patterns to preview against." },
                    "text": { "type": "string", "description": "Inline text to run through instead of files (stdin filter)." },
                    "report": { "type": "boolean", "description": "Emit an HTML dashboard (default on). false to skip. Overrides persisted config/MOG_REPORT." }
                },
                "required": []
            }
        },
        {
            "name": "mog_apply",
            "description": "Transform files with an installed recipe (recipe) or an inline script (mog). WRITES TO DISK. output is {mode:'out_dir',dir,overwrite?} (new files) or {mode:'in_place_backup',backup_suffix} (edits in place, keeps a backup). Bare in-place with no backup is refused. Preview first. Emits an HTML run dashboard by default (a resource_link to the result); set report:false, or persist mog_config {reports:false}, to disable it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mogname": { "type": "string", "description": "An installed recipe name (from mog_market), resolved from the library. Use this instead of pasting a script you already have installed; give exactly one of mogname or mog." },
                    "mog": { "type": "string", "description": "The .mog script as JSON text, for a recipe you are authoring. Omit when passing mogname." },
                    "inputs": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "Files or glob patterns to transform." },
                    "report": { "type": "boolean", "description": "Emit an HTML dashboard (default on). false to skip. Overrides persisted config/MOG_REPORT." },
                    "output": {
                        "description": "Where output goes. Reversible modes only.",
                        "oneOf": [
                            {
                                "type": "object",
                                "properties": {
                                    "mode": { "const": "out_dir" },
                                    "dir": { "type": "string", "description": "Directory to write transformed copies into." },
                                    "overwrite": { "type": "boolean", "description": "Allow overwriting existing files in dir." }
                                },
                                "required": ["mode", "dir"]
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "mode": { "const": "in_place_backup" },
                                    "backup_suffix": { "type": "string", "description": "Backup suffix, e.g. '.bak'. Required; bare in-place is refused." }
                                },
                                "required": ["mode", "backup_suffix"]
                            }
                        ]
                    }
                },
                "required": ["inputs", "output"]
            }
        },
        {
            "name": "mog_test",
            "description": "Run a .mog's sibling torture fixtures (TestInput -> TestExpectedOutput) and return pass/fail + diffs.",
            "inputSchema": {
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Path to a .mog file (or a directory to search)." } },
                "required": ["path"]
            }
        },
        {
            "name": "mog_check",
            "description": "Validate a candidate .mog against inline {input, expected} examples: run the recipe on each input and report per-example pass/fail plus the actual output for mismatches. Use this to author a recipe BY EXAMPLE -- propose a .mog (from mog_actions), check it against a few before/after pairs, and iterate on the failures until all_pass. Deterministic; writes nothing.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mog": { "type": "string", "description": "The candidate .mog script as JSON text." },
                    "examples": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "input": { "type": "string", "description": "Example input text." },
                                "expected": { "type": "string", "description": "The exact output the recipe should produce for this input." }
                            },
                            "required": ["input", "expected"]
                        },
                        "description": "Before/after pairs the recipe must reproduce."
                    }
                },
                "required": ["mog", "examples"]
            }
        },
        {
            "name": "mog_market",
            "description": "Find or install a ready-made text-transform recipe before hand-editing. SEARCH HERE FIRST whenever you are about to clean up or reshape text in a repetitive or error-prone way: de-duplicate/sort/normalize a list (emails, contacts, tags), strip ANSI color codes from output, fix whitespace / line endings / tabs / final newline, redact secrets or PII, encode or escape (base64/URL/HTML), comment-out or bulletize/number lines, or a bulk find/replace across many files. A long list hand-edited by an LLM can silently drop or alter entries; a recipe here is exact, previewable, and backed up. Prefer query: a ranked, synonym-aware search over name/description/tags (multi-word phrasings like 'strip color codes from output' or 'dedupe a list of emails' work, best first) across your installed recipes AND the catalog. name shows one recipe's detail. op:install downloads and verifies (signature + hash) a catalog recipe; op:list browses. To refresh/upgrade installed recipes use the separate mog_update tool. If nothing fits, author a .mog with mog_actions. Run a recipe with -m <name>.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "op": { "type": "string", "enum": ["search", "list", "show", "install"], "description": "Explicit op: search (needs query), list, show (needs name), install (needs name; downloads + verifies a catalog recipe). Omit to infer: name -> show, query -> search, else list." },
                    "name": { "type": "string", "description": "A recipe name; the target of show / install." },
                    "query": { "type": "string", "description": "Task/keyword search over name/description/tags (installed recipes + catalog)." }
                }
            }
        },
        {
            "name": "mog_update",
            "description": "Bring Mog current with the registry: pull new/changed marketplace recipes (diffed by content hash, so only what actually changed is fetched) into your library, and self-replace the mog engine binary if a newer signed build exists. Call with check:true first to see what would change without writing. marketOnly:true skips the binary swap. prune:true removes recipes that are gone from the catalog. Recipe updates are picked up immediately on the next search; an engine swap takes effect when the server next restarts (engine.applied in the result says whether a swap happened).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "check": { "type": "boolean", "description": "Report what would change without writing anything." },
                    "marketOnly": { "type": "boolean", "description": "Only sync recipes; skip the engine binary swap." },
                    "prune": { "type": "boolean", "description": "Remove recipes on disk that are gone from the catalog." }
                }
            }
        },
        {
            "name": "mog_config",
            "description": "Persist Mog preferences, e.g. turn run dashboards on/off (reports). Call with reports:false to stop mog_apply/mog_preview emitting dashboards; reports:true to re-enable. template sets the default dashboard template. No args returns the current config.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "reports": { "type": "boolean", "description": "Turn run dashboards on (true) or off (false) for future runs." },
                    "template": { "type": "string", "description": "Default dashboard template name (user > community > factory)." }
                }
            }
        }
    ])
}

/// Route a `tools/call` to its handler by name.
fn dispatch_tool(name: &str, args: &Value) -> Option<Value> {
    Some(match name {
        "mog_actions" => tools::mog_actions(args),
        "mog_validate" => tools::mog_validate(args),
        "mog_preview" => tools::mog_preview(args),
        "mog_apply" => tools::mog_apply(args),
        "mog_test" => tools::mog_test(args),
        "mog_check" => tools::mog_check(args),
        "mog_market" => tools::mog_market(args),
        "mog_update" => tools::mog_update(args),
        "mog_config" => tools::mog_config(args),
        _ => return None,
    })
}

/// JSON-RPC error codes we use.
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// Build a JSON-RPC success response.
fn ok_response(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

/// Build a JSON-RPC error response.
fn err_response(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Handle one parsed JSON-RPC request/notification. Returns `Some(response)` for
/// a request (has an `id`), `None` for a notification.
fn handle_message(msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(Value::as_str)?;
    let id = msg.get("id").cloned();
    let params = msg.get("params").cloned().unwrap_or(json!({}));

    // Notifications carry no id and expect no response.
    let is_notification = id.is_none();

    match method {
        "initialize" => {
            let client_version = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL_VERSION);
            let result = json!({
                "protocolVersion": client_version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "mog-mcp", "version": env!("CARGO_PKG_VERSION") },
                "instructions": INSTRUCTIONS,
            });
            id.as_ref().map(|id| ok_response(id, result))
        }
        "notifications/initialized" | "initialized" => None,
        "ping" => id.as_ref().map(|id| ok_response(id, json!({}))),
        "tools/list" => {
            let result = json!({ "tools": tool_definitions() });
            id.as_ref().map(|id| ok_response(id, result))
        }
        "tools/call" => {
            let id = id?; // a call without an id is malformed; ignore.
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match dispatch_tool(name, &arguments) {
                Some(result) => Some(ok_response(&id, result)),
                None => Some(err_response(
                    &id,
                    INVALID_PARAMS,
                    &format!("unknown tool: {name}"),
                )),
            }
        }
        other => {
            if is_notification {
                None
            } else {
                id.as_ref().map(|id| {
                    err_response(id, METHOD_NOT_FOUND, &format!("unknown method: {other}"))
                })
            }
        }
    }
}

/// Run the stdio MCP server loop until stdin closes. Reads newline-delimited
/// JSON-RPC messages on stdin, writes newline-delimited responses on stdout, and
/// keeps stdout exclusively for the protocol (all logging goes to stderr).
pub fn run_server() -> Result<i32> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    eprintln!("mog mcp: connected (stdio)");

    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("mog mcp: stdin read error: {e}");
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                // Parse error with no recoverable id -> emit a generic error.
                eprintln!("mog mcp: parse error: {e}");
                let resp = json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "parse error" } });
                writeln!(out, "{resp}")?;
                out.flush()?;
                continue;
            }
        };
        if let Some(resp) = handle_message(&msg) {
            writeln!(out, "{}", serde_json::to_string(&resp)?)?;
            out.flush()?;
        }
    }
    eprintln!("mog mcp: stdin closed, exiting");
    Ok(0)
}
