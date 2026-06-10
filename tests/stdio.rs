use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_server(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_leptos-mcp-server"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("server binary should start");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("request should write to stdin");

    child.wait_with_output().expect("server should exit on EOF")
}

fn stdout_json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("stdout line should be JSON"))
        .collect()
}

#[test]
fn stdio_process_returns_one_json_response_per_request() {
    let output = run_server(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
"#,
    );

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(responses[1]["id"], 2);
    assert!(responses[1]["result"]["tools"].is_array());
}

#[test]
fn stdio_process_keeps_logs_on_stderr() {
    let output = run_server(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#);

    assert!(output.status.success());
    assert_eq!(stdout_json_lines(&output).len(), 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Starting Leptos MCP Server"));
}

#[test]
fn stdio_process_returns_parse_errors_as_json() {
    let output = run_server("{bad json}\n");

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["error"]["code"], -32700);
}

#[test]
fn stdio_process_exposes_resources_prompts_and_new_tools() {
    let output = run_server(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}
"#,
    );

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 3);
    assert!(responses[0]["result"]["tools"]
        .as_array()
        .expect("tools should be listed")
        .iter()
        .any(|tool| tool["name"] == "lookup-api"));
    assert!(responses[1]["result"]["resources"]
        .as_array()
        .expect("resources should be listed")
        .iter()
        .any(|resource| resource["uri"] == "leptos://docs/leptos-axum"));
    assert!(responses[2]["result"]["prompts"]
        .as_array()
        .expect("prompts should be listed")
        .iter()
        .any(|prompt| prompt["name"] == "wire-leptos-axum-ssr"));
}
