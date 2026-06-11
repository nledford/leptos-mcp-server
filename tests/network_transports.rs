use std::process::{Command, Output};

const SECRET_BODY: &str = r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"leptos-diagnostics","arguments":{"code":"SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT"}}}"#;

fn run_server(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_leptos-mcp-server"))
        .args(args)
        .output()
        .expect("server binary should run")
}

fn run_server_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_leptos-mcp-server"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("server binary should run")
}

fn assert_no_public_network_or_permissive_security_claims(text: &str) {
    for forbidden in [
        "0.0.0.0",
        "[::]",
        "Access-Control-Allow-Origin: *",
        "Access-Control-Allow-Origin=*",
        "allow any origin",
        "allow all origins",
        "permissive CORS",
        "CORS enabled",
        "authentication disabled",
        "no authentication required",
        "Authorization header optional",
    ] {
        assert!(
            !text.contains(forbidden),
            "unexpected public/permissive claim: {forbidden}"
        );
    }
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

fn assert_network_transport_deferred(transport: &str) {
    let output = run_server(&[
        "--transport",
        transport,
        "--host",
        "SECRET_REQUEST_HOST",
        "--port",
        SECRET_BODY,
    ]);

    assert!(
        !output.status.success(),
        "network transport should fail closed"
    );
    assert!(
        output.stdout.is_empty(),
        "network deferral should not emit protocol output on stdout"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!("transport '{transport}' is not supported")));
    assert!(stderr.contains("deferred/disabled"));
    assert!(stderr.contains("no network listener was started"));
    assert!(stderr.contains("request body/message limits"));
    assert!(stderr.contains("read/request/handler timeouts"));
    assert!(stderr.contains("host 127.0.0.1"));
    assert!(!stderr.contains("Starting Leptos MCP Server"));
    assert!(!stderr.contains("SECRET_REQUEST_HOST"));
    assert!(!stderr.contains("SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT"));
    assert!(!stderr.contains(SECRET_BODY));
    assert_no_internal_or_sensitive_diagnostics(
        "network deferral stderr",
        &stderr,
        &[
            "SECRET_REQUEST_HOST",
            "SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT",
            SECRET_BODY,
        ],
    );
    assert_no_public_network_or_permissive_security_claims(&stderr);
}

#[test]
fn streamable_http_is_deferred_without_starting_listener_or_echoing_inputs() {
    assert_network_transport_deferred("streamable-http");
}

#[test]
fn sse_is_deferred_without_starting_listener_or_echoing_inputs() {
    assert_network_transport_deferred("sse");
}

#[test]
fn http_alias_is_deferred_without_starting_listener_or_echoing_inputs() {
    let output = run_server(&[
        "--transport=http",
        "--host=SECRET_REQUEST_HOST",
        &format!("--port={SECRET_BODY}"),
    ]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("transport 'streamable-http' is not supported"));
    assert!(stderr.contains("deferred/disabled"));
    assert!(stderr.contains("no network listener was started"));
    assert!(stderr.contains("host 127.0.0.1"));
    assert!(!stderr.contains("Starting Leptos MCP Server"));
    assert!(!stderr.contains("SECRET_REQUEST_HOST"));
    assert!(!stderr.contains("SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT"));
    assert!(!stderr.contains(SECRET_BODY));
    assert_no_internal_or_sensitive_diagnostics(
        "http alias deferral stderr",
        &stderr,
        &[
            "SECRET_REQUEST_HOST",
            "SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT",
            SECRET_BODY,
        ],
    );
    assert_no_public_network_or_permissive_security_claims(&stderr);
}

#[test]
fn env_selected_network_transport_fails_closed_before_startup() {
    let output = run_server_with_env(
        &[],
        &[
            ("LEPTOS_MCP_TRANSPORT", "streamable-http"),
            ("LEPTOS_MCP_HOST", "SECRET_REQUEST_HOST"),
            ("LEPTOS_MCP_PORT", SECRET_BODY),
        ],
    );

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("transport 'streamable-http' is not supported"));
    assert!(stderr.contains("no network listener was started"));
    assert!(!stderr.contains("Starting Leptos MCP Server"));
    assert!(!stderr.contains("SECRET_REQUEST_HOST"));
    assert!(!stderr.contains("SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT"));
    assert!(!stderr.contains(SECRET_BODY));
    assert_no_internal_or_sensitive_diagnostics(
        "env network deferral stderr",
        &stderr,
        &[
            "SECRET_REQUEST_HOST",
            "SECRET_TOOL_ARGS_PROMPT_RESOURCE_CONTENT",
            SECRET_BODY,
        ],
    );
    assert_no_public_network_or_permissive_security_claims(&stderr);
}

#[test]
fn help_documents_network_limit_timeout_deferral() {
    let output = run_server(&["--help"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Network transports are currently deferred/unsupported"));
    assert!(stdout.contains("default/env: stdio"));
    assert!(stdout.contains("default: 127.0.0.1"));
    assert!(stdout.contains("request body/message limits"));
    assert!(stdout.contains("read/request/handler timeouts"));
    assert!(stdout.contains("sanitized malformed-input errors"));
    assert!(!stdout.contains("default: 0.0.0.0"));
    assert_no_public_network_or_permissive_security_claims(&stdout);
}

#[test]
fn readme_does_not_document_public_bind_cors_or_auth_shortcuts() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README should be readable");

    assert!(readme.contains("Only stdio is implemented"));
    assert!(readme.contains("fail closed before tracing or server startup"));
    assert!(readme.contains("network listener is started"));
    assert!(readme.contains("request body/message limits"));
    assert!(readme.contains("read/request/handler timeouts"));
    assert_no_public_network_or_permissive_security_claims(&readme);
}

#[test]
fn cargo_features_do_not_enable_network_sdk_transports() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml should be readable");

    assert!(manifest.contains("default = [\"stdio\"]"));
    assert!(manifest.contains("stdio = [\"rust-mcp-sdk/stdio\"]"));
    assert!(!manifest.contains("rust-mcp-sdk/streamable-http"));
    assert!(!manifest.contains("rust-mcp-sdk/sse"));
    assert!(!manifest.contains("rust-mcp-sdk/http"));
    assert!(!manifest.contains("tower-http"));
    assert!(!manifest.contains("axum"));
}
