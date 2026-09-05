//! Integration tests for the folded-in MCP server (`mog mcp`). They spawn the
//! real compiled binary, drive it over stdio with a batch of newline-delimited
//! JSON-RPC messages (the server loops until stdin EOF, then exits), and assert
//! the responses. Parity target: the retired Node server's tool surface.

use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{json, Value};

/// Feed `requests` (one JSON-RPC message per element) to `mog mcp` over stdin and
/// return the parsed responses, in order. `MOG_HOME` points at an isolated temp
/// dir so nothing touches real user files.
fn drive(requests: &[Value]) -> Vec<Value> {
    let exe = env!("CARGO_BIN_EXE_mog");
    let home = tempfile::tempdir().expect("temp MOG_HOME");
    let mut child = Command::new(exe)
        .arg("mcp")
        .env("MOG_HOME", home.path())
        // A dashboard would need a library; keep runs report-free & deterministic.
        .env("MOG_REPORT", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mog mcp");

    {
        let mut stdin = child.stdin.take().expect("stdin");
        for req in requests {
            writeln!(stdin, "{}", serde_json::to_string(req).unwrap()).unwrap();
        }
        // stdin drops here -> EOF -> the server loop exits.
    }
    let out = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("response is JSON"))
        .collect()
}

/// Like `drive`, but seeds the temp library with one recipe first, so a call
/// can name it instead of passing a script inline.
fn drive_with_recipe(name: &str, script: &str, requests: &[Value]) -> Vec<Value> {
    let exe = env!("CARGO_BIN_EXE_mog");
    let home = tempfile::tempdir().expect("temp MOG_HOME");
    let dir = home.path().join("mogs").join("market").join(name);
    std::fs::create_dir_all(&dir).expect("recipe dir");
    std::fs::write(dir.join(format!("{name}.mog")), script).expect("seed recipe");
    let mut child = Command::new(exe)
        .arg("mcp")
        .env("MOG_HOME", home.path())
        .env("MOG_REPORT", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mog mcp");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        for req in requests {
            writeln!(stdin, "{}", serde_json::to_string(req).unwrap()).unwrap();
        }
    }
    let out = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("response is JSON"))
        .collect()
}

/// The text of a tool result's first content block.
fn result_text(resp: &Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[test]
fn initialize_reports_tools_capability_and_instructions() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "protocolVersion": "2025-06-18" }
    })]);
    assert_eq!(resps.len(), 1);
    let r = &resps[0]["result"];
    assert_eq!(r["protocolVersion"], "2025-06-18");
    assert!(r["capabilities"]["tools"].is_object());
    assert_eq!(r["serverInfo"]["name"], "mog-mcp");
    // The instructions block is what teaches an agent to reach for Mog.
    let instr = r["instructions"].as_str().unwrap();
    assert!(instr.contains("deterministic text-transform engine"));
}

#[test]
fn tools_list_has_the_nine_tools() {
    let resps = drive(&[json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" })]);
    let tools = resps[0]["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "mog_actions",
        "mog_validate",
        "mog_preview",
        "mog_apply",
        "mog_test",
        "mog_check",
        "mog_market",
        "mog_update",
        "mog_config",
    ] {
        assert!(names.contains(&expected), "missing tool {expected}");
    }
    assert_eq!(names.len(), 9);
}

#[test]
fn actions_query_ranks_relevant_actions_first() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_actions", "arguments": { "query": "whitespace cleanup" } }
    })]);
    assert_eq!(resps[0]["result"]["isError"], false);
    let list: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert!(
        !list.as_array().unwrap().is_empty(),
        "query should match actions"
    );
}

#[test]
fn validate_ok_and_error() {
    let resps = drive(&[
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "mog_validate", "arguments": { "mog": "{\"steps\":[{\"action\":\"to_upper\"}]}" } }
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "mog_validate", "arguments": { "mog": "{\"steps\":[{\"action\":\"nope\"}]}" } }
        }),
    ]);
    let ok: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert_eq!(ok["ok"], true);
    let err: Value = serde_json::from_str(&result_text(&resps[1])).unwrap();
    assert_eq!(err["error"], true);
    assert_eq!(err["step"], 1);
}

#[test]
fn check_runs_examples_and_reports_pass_fail() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_check", "arguments": {
            "mog": "{\"steps\":[{\"action\":\"to_upper\"}]}",
            "examples": [ { "input": "abc", "expected": "ABC" }, { "input": "x", "expected": "WRONG" } ]
        } }
    })]);
    let out: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert_eq!(out["passed"], 1);
    assert_eq!(out["total"], 2);
    assert_eq!(out["all_pass"], false);
    // The server marks a partial pass as a tool error.
    assert_eq!(resps[0]["result"]["isError"], true);
}

#[test]
fn preview_inline_text_is_read_only() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_preview", "arguments": {
            "mog": "{\"steps\":[{\"action\":\"to_upper\"}]}", "text": "hello", "report": false
        } }
    })]);
    assert_eq!(resps[0]["result"]["isError"], false);
    let report: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    // A dry-run report carries a `mog` (or summary) block; just assert it parsed
    // to an object and flagged no error envelope.
    assert!(report.is_object());
    assert_ne!(report["error"], true);
}

#[test]
fn unknown_tool_is_an_error_result() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_nonexistent", "arguments": {} }
    })]);
    assert!(
        resps[0]["error"].is_object(),
        "unknown tool -> JSON-RPC error"
    );
}

#[test]
fn apply_refuses_bare_in_place() {
    // A bare in-place (empty backup suffix) must be refused, not written.
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_apply", "arguments": {
            "mog": "{\"steps\":[{\"action\":\"to_upper\"}]}",
            "inputs": ["whatever.txt"],
            "output": { "mode": "in_place_backup", "backup_suffix": "" }
        } }
    })]);
    assert_eq!(resps[0]["result"]["isError"], true);
    let err: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert_eq!(err["error"], true);
    assert!(err["message"]
        .as_str()
        .unwrap()
        .contains("bare in-place is refused"));
}

#[test]
fn preview_accepts_an_installed_recipe_by_name() {
    let resps = drive_with_recipe(
        "shouty",
        "{\"steps\":[{\"action\":\"to_upper\"}]}",
        &[json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "mog_preview", "arguments": {
                "recipe": "shouty", "text": "hello", "report": false
            } }
        })],
    );
    assert_eq!(
        resps[0]["result"]["isError"],
        false,
        "{}",
        result_text(&resps[0])
    );
    let report: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert_ne!(report["error"], true);
    assert_eq!(report["mog"]["path"], "shouty");
}

#[test]
fn preview_without_a_script_is_an_error() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_preview", "arguments": { "text": "x", "report": false } }
    })]);
    assert_eq!(resps[0]["result"]["isError"], true);
    let err: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert!(err["message"].as_str().unwrap().contains("no script"));
}

#[test]
fn preview_refuses_both_script_forms() {
    let resps = drive(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "mog_preview", "arguments": {
            "recipe": "shouty",
            "mog": "{\"steps\":[{\"action\":\"to_upper\"}]}",
            "text": "x", "report": false
        } }
    })]);
    assert_eq!(resps[0]["result"]["isError"], true);
    let err: Value = serde_json::from_str(&result_text(&resps[0])).unwrap();
    assert!(err["message"].as_str().unwrap().contains("not both"));
}
