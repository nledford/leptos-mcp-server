use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

const INITIALIZE_REQUEST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-test","version":"0.0.0"}}}"#;
const INITIALIZED_NOTIFICATION: &str =
    r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#;

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

fn response_by_id(responses: &[Value], id: i64) -> &Value {
    responses
        .iter()
        .find(|response| response["id"] == id)
        .unwrap_or_else(|| panic!("response id {id} should be present in {responses:#?}"))
}

fn initialized_input(requests: &[&str]) -> String {
    let mut input = format!("{INITIALIZE_REQUEST}\n{INITIALIZED_NOTIFICATION}\n");
    for request in requests {
        input.push_str(request);
        input.push('\n');
    }
    input
}

#[test]
fn stdio_process_initializes_and_returns_json_responses() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 2);
    let initialize = response_by_id(&responses, 1);
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        "leptos-mcp-server"
    );
    assert_eq!(initialize["result"]["protocolVersion"], "2025-11-25");
    assert!(initialize["result"]["capabilities"]["tools"].is_object());
    assert!(initialize["result"]["capabilities"]["resources"].is_object());
    assert!(initialize["result"]["capabilities"]["prompts"].is_object());
    assert!(response_by_id(&responses, 2)["result"]["tools"].is_array());
}

#[test]
fn stdio_process_keeps_logs_on_stderr() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]));

    assert!(output.status.success());
    assert_eq!(stdout_json_lines(&output).len(), 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Starting Leptos MCP Server"));
}

#[test]
fn stdio_process_does_not_write_responses_for_notifications() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 2);
    assert!(response_by_id(&responses, 1)["result"]["serverInfo"].is_object());
    assert!(response_by_id(&responses, 2)["result"]["tools"].is_array());
}

#[test]
fn stdio_process_exposes_tools_resources_templates_and_prompts() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"resources/templates/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"prompts/list","params":{}}"#,
    ]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 5);
    assert!(
        response_by_id(&responses, 2)["result"]["tools"]
            .as_array()
            .expect("tools should be listed")
            .iter()
            .any(|tool| tool["name"] == "lookup-api")
    );
    assert!(
        response_by_id(&responses, 3)["result"]["resources"]
            .as_array()
            .expect("resources should be listed")
            .iter()
            .any(|resource| resource["uri"] == "leptos://docs/leptos-axum")
    );
    assert!(
        response_by_id(&responses, 4)["result"]["resourceTemplates"]
            .as_array()
            .expect("resource templates should be listed")
            .iter()
            .any(|template| template["uriTemplate"] == "leptos://docs/{section}")
    );
    assert!(
        response_by_id(&responses, 5)["result"]["prompts"]
            .as_array()
            .expect("prompts should be listed")
            .iter()
            .any(|prompt| prompt["name"] == "wire-leptos-axum-ssr")
    );
}

#[test]
fn stdio_process_calls_tools_reads_resources_gets_prompts_and_surfaces_tool_errors() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals"}}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"leptos://docs/signals"}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"prompts/get","params":{"name":"debug-hydration","arguments":{"symptom":"WASM 404"}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get-documentation","arguments":{}}}"#,
    ]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 5);

    let tool_call = response_by_id(&responses, 2);
    assert_eq!(tool_call["result"]["isError"], Value::Null);
    assert_eq!(
        tool_call["result"]["structuredContent"]["kind"],
        "documentation"
    );
    assert!(
        tool_call["result"]["content"][0]["text"]
            .as_str()
            .expect("tool text should be a string")
            .contains("Leptos Signals")
    );

    let resource_read = response_by_id(&responses, 3);
    assert_eq!(
        resource_read["result"]["contents"][0]["uri"],
        "leptos://docs/signals"
    );
    assert_eq!(
        resource_read["result"]["contents"][0]["mimeType"],
        "text/markdown"
    );

    let prompt_get = response_by_id(&responses, 4);
    assert_eq!(prompt_get["result"]["messages"][0]["role"], "user");
    assert!(
        prompt_get["result"]["messages"][0]["content"]["text"]
            .as_str()
            .expect("prompt text should be a string")
            .contains("WASM 404")
    );

    let malformed_tool_call = response_by_id(&responses, 5);
    assert_eq!(malformed_tool_call["result"]["isError"], true);
    assert!(
        malformed_tool_call["result"]["content"][0]["text"]
            .as_str()
            .expect("tool error text should be a string")
            .contains("missing field `section`")
    );
}
