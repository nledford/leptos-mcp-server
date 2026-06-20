#![cfg(feature = "stdio")]

use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};

const INITIALIZE_REQUEST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-test","version":"0.0.0"}}}"#;
const INITIALIZED_NOTIFICATION: &str =
    r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#;
const MAX_DIAGNOSTIC_CODE_BYTES: usize = 256 * 1024;

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

fn assert_no_internal_or_sensitive_diagnostics(label: &str, text: &str, secrets: &[&str]) {
    for forbidden in [
        "panicked at",
        "stack backtrace",
        "RUST_BACKTRACE",
        "thread 'main' panicked",
        env!("CARGO_MANIFEST_DIR"),
        "/src/main.rs",
        "/src/sdk.rs",
        "/src/tools.rs",
    ] {
        assert!(
            !text.contains(forbidden),
            "{label} exposed implementation diagnostic content: {forbidden}"
        );
    }

    for secret in secrets {
        assert!(
            !text.contains(secret),
            "{label} echoed sensitive input: {secret}"
        );
    }
}

fn assert_failure_response_is_sanitized(response: &Value, secrets: &[&str]) {
    assert!(
        response.get("error").is_some() || response["result"]["isError"] == true,
        "response should be an observable failure: {response:#?}"
    );
    assert_no_internal_or_sensitive_diagnostics(
        "failure response",
        &serde_json::to_string(response).expect("response should serialize"),
        secrets,
    );
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
fn stdio_tool_resource_and_prompt_failures_do_not_echo_sensitive_inputs() {
    const DOC_SECRET: &str = "SECRET_DOC_SECTION_ARGUMENT";
    const API_SECRET: &str = "SECRET_API_QUERY_ARGUMENT";
    const CRATE_SECRET: &str = "SECRET_API_CRATE_ARGUMENT";
    const RECIPE_SECRET: &str = "SECRET_RECIPE_ARGUMENT";
    const RESOURCE_SECRET: &str = "SECRET_RESOURCE_URI_PAYLOAD";
    const PROMPT_NAME_SECRET: &str = "SECRET_PROMPT_NAME_PAYLOAD";
    const PROMPT_BODY_SECRET: &str = "SECRET_PROMPT_BODY_PAYLOAD";
    const PROMPT_ARG_SECRET: &str = "SECRET_PROMPT_ARGUMENT_NAME";
    const DIAGNOSTIC_CODE_SECRET: &str = "SECRET_DIAGNOSTIC_CODE_PAYLOAD";

    let oversized_diagnostic_code = DIAGNOSTIC_CODE_SECRET
        .repeat((MAX_DIAGNOSTIC_CODE_BYTES / DIAGNOSTIC_CODE_SECRET.len()) + 1);
    let requests = [
        format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"get-documentation","arguments":{{"section":"{DOC_SECRET}"}}}}}}"#
        ),
        format!(
            r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"lookup-api","arguments":{{"query":{{"secret":"{API_SECRET}"}},"crate":"{CRATE_SECRET}"}}}}}}"#
        ),
        format!(
            r#"{{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{{"name":"leptos-axum-recipe","arguments":{{"recipe":"{RECIPE_SECRET}"}}}}}}"#
        ),
        format!(
            r#"{{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{{"uri":"leptos://docs/{RESOURCE_SECRET}"}}}}"#
        ),
        format!(
            r#"{{"jsonrpc":"2.0","id":6,"method":"prompts/get","params":{{"name":"{PROMPT_NAME_SECRET}","arguments":{{"symptom":"{PROMPT_BODY_SECRET}"}}}}}}"#
        ),
        format!(
            r#"{{"jsonrpc":"2.0","id":7,"method":"prompts/get","params":{{"name":"debug-hydration","arguments":{{"symptom":"{PROMPT_BODY_SECRET}","{PROMPT_ARG_SECRET}":"unused"}}}}}}"#
        ),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "leptos-diagnostics",
                "arguments": { "code": oversized_diagnostic_code }
            }
        })
        .to_string(),
    ];
    let request_refs = requests.iter().map(String::as_str).collect::<Vec<_>>();
    let output = run_server(&initialized_input(&request_refs));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 8);

    let secrets = [
        DOC_SECRET,
        API_SECRET,
        CRATE_SECRET,
        RECIPE_SECRET,
        RESOURCE_SECRET,
        PROMPT_NAME_SECRET,
        PROMPT_BODY_SECRET,
        PROMPT_ARG_SECRET,
        DIAGNOSTIC_CODE_SECRET,
    ];

    for id in 2..=8 {
        assert_failure_response_is_sanitized(response_by_id(&responses, id), &secrets);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_internal_or_sensitive_diagnostics("stdio failure logs", &stderr, &secrets);
    for request in &requests {
        assert!(
            !stderr.contains(request),
            "stdio failure logs echoed raw JSON request: {request}"
        );
    }
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
    assert!(
        initialize["result"]["capabilities"]
            .get("completions")
            .is_none()
    );
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
fn default_invocation_uses_stdio_not_network_transport() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ]));

    assert!(output.status.success());
    assert_eq!(stdout_json_lines(&output).len(), 2);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Starting Leptos MCP Server"));
    assert!(!stderr.contains("transport 'streamable-http'"));
    assert!(!stderr.contains("transport 'sse'"));
    assert!(!stderr.contains("no network listener was started"));
}

#[test]
fn stdio_logs_do_not_echo_request_bodies_or_tool_arguments() {
    const SECRET: &str = "SECRET_REQUEST_BODY_SHOULD_NOT_BE_LOGGED";
    let request = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"search-docs","arguments":{{"query":"{SECRET}"}}}}}}"#
    );
    let output = run_server(&initialized_input(&[&request]));

    assert!(output.status.success());
    assert_eq!(stdout_json_lines(&output).len(), 2);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Starting Leptos MCP Server"));
    assert!(!stderr.contains(SECRET));
    assert!(!stderr.contains(&request));
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
        [
            "list-sections",
            "get-documentation",
            "leptos-diagnostics",
            "search-docs",
            "lookup-api",
            "leptos-axum-recipe",
        ]
        .iter()
        .all(|name| response_by_id(&responses, 2)["result"]["tools"]
            .as_array()
            .expect("tools should be listed")
            .iter()
            .any(|tool| tool["name"] == *name))
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
fn stdio_process_calls_every_tool_reads_resources_gets_prompts_and_surfaces_tool_errors() {
    let output = run_server(&initialized_input(&[
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list-sections","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals"}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"leptos-diagnostics","arguments":{"code":"fn App() -> impl IntoView { let count = signal(0); view! { <p>{count.get()}</p> } }"}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"search-docs","arguments":{"query":"Axum state"}}}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"lookup-api","arguments":{"query":"file_and_error_handler","crate":"leptos_axum"}}}"#,
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"leptos-axum-recipe","arguments":{"recipe":"state"}}}"#,
        r#"{"jsonrpc":"2.0","id":8,"method":"resources/read","params":{"uri":"leptos://docs/signals"}}"#,
        r#"{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"debug-hydration","arguments":{"symptom":"WASM 404"}}}"#,
        r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"get-documentation","arguments":{}}}"#,
    ]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 10);

    let list_sections = response_by_id(&responses, 2);
    assert_eq!(list_sections["result"]["isError"], Value::Null);
    assert_eq!(
        list_sections["result"]["structuredContent"]["kind"],
        "list-sections"
    );
    assert!(
        list_sections["result"]["content"][0]["text"]
            .as_str()
            .expect("list-sections text should be a string")
            .contains("* id: signals")
    );

    let get_documentation = response_by_id(&responses, 3);
    assert_eq!(get_documentation["result"]["isError"], Value::Null);
    assert_eq!(
        get_documentation["result"]["structuredContent"]["kind"],
        "documentation"
    );
    assert!(
        get_documentation["result"]["content"][0]["text"]
            .as_str()
            .expect("tool text should be a string")
            .contains("Leptos Signals")
    );

    let diagnostics = response_by_id(&responses, 4);
    assert_eq!(diagnostics["result"]["isError"], Value::Null);
    assert_eq!(
        diagnostics["result"]["structuredContent"]["kind"],
        "diagnostics"
    );
    assert!(
        diagnostics["result"]["content"][0]["text"]
            .as_str()
            .expect("diagnostics text should be a string")
            .contains("leptos.signal-get-in-view")
    );

    let search_docs = response_by_id(&responses, 5);
    assert_eq!(search_docs["result"]["isError"], Value::Null);
    assert_eq!(
        search_docs["result"]["structuredContent"]["kind"],
        "search-docs"
    );
    assert!(
        search_docs["result"]["content"][0]["text"]
            .as_str()
            .expect("search text should be a string")
            .contains("* id: axum")
    );

    let lookup_api = response_by_id(&responses, 6);
    assert_eq!(lookup_api["result"]["isError"], Value::Null);
    assert_eq!(
        lookup_api["result"]["structuredContent"]["kind"],
        "api-lookup"
    );
    assert!(
        lookup_api["result"]["content"][0]["text"]
            .as_str()
            .expect("API lookup text should be a string")
            .contains("leptos_axum::file_and_error_handler")
    );

    let recipe = response_by_id(&responses, 7);
    assert_eq!(recipe["result"]["isError"], Value::Null);
    assert_eq!(recipe["result"]["structuredContent"]["kind"], "recipe");
    assert!(
        recipe["result"]["content"][0]["text"]
            .as_str()
            .expect("recipe text should be a string")
            .contains("Share Axum state with server functions")
    );

    let resource_read = response_by_id(&responses, 8);
    assert_eq!(
        resource_read["result"]["contents"][0]["uri"],
        "leptos://docs/signals"
    );
    assert_eq!(
        resource_read["result"]["contents"][0]["mimeType"],
        "text/markdown"
    );

    let prompt_get = response_by_id(&responses, 9);
    assert_eq!(prompt_get["result"]["messages"][0]["role"], "user");
    assert!(
        prompt_get["result"]["messages"][0]["content"]["text"]
            .as_str()
            .expect("prompt text should be a string")
            .contains("WASM 404")
    );

    let malformed_tool_call = response_by_id(&responses, 10);
    assert_eq!(malformed_tool_call["result"]["isError"], true);
    assert!(
        malformed_tool_call["result"]["content"][0]["text"]
            .as_str()
            .expect("tool error text should be a string")
            .contains("missing field `section`")
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Starting Leptos MCP Server"));
    assert!(
        !stderr.contains("Leptos Signals"),
        "stdio logs should not echo successful tool or resource contents"
    );
    assert!(
        !stderr.contains("WASM 404"),
        "stdio logs should not echo prompt arguments"
    );
}

#[test]
fn stdio_rejects_oversized_diagnostic_payload() {
    let oversized_code = "x".repeat(MAX_DIAGNOSTIC_CODE_BYTES + 1);
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "leptos-diagnostics",
            "arguments": { "code": oversized_code }
        }
    })
    .to_string();
    let output = run_server(&initialized_input(&[&request]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 2);

    let response = response_by_id(&responses, 2);
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool error text should be a string")
            .contains(&format!(
                "code must be at most {MAX_DIAGNOSTIC_CODE_BYTES} bytes"
            ))
    );
}

#[test]
fn stdio_rejects_malformed_tool_arguments_without_logging_body() {
    const SECRET: &str = "SECRET_MALFORMED_ARGUMENT_SHOULD_NOT_BE_LOGGED";
    let request = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"get-documentation","arguments":{{"section":{{"secret":"{SECRET}"}}}}}}}}"#
    );
    let output = run_server(&initialized_input(&[&request]));

    assert!(output.status.success());
    let responses = stdout_json_lines(&output);
    assert_eq!(responses.len(), 2);

    let response = response_by_id(&responses, 2);
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool error text should be a string")
            .contains("invalid type")
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains(SECRET));
    assert!(!stderr.contains(&request));
}
