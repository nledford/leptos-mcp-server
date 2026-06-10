//! MCP protocol implementation.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio.

use crate::tools::{
    LeptosTools, ToolError, GET_DOCUMENTATION_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};

const JSON_RPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

pub struct McpServer {
    tools: LeptosTools,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug)]
enum ProtocolError {
    ParseError(String),
    InvalidRequest(String),
    MethodNotFound(String),
    InvalidParams(String),
    InternalError(String),
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
struct DocumentationArgs {
    section: String,
}

#[derive(Debug, Deserialize)]
struct DiagnosticsArgs {
    code: String,
}

#[derive(Debug, Serialize)]
struct ToolContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    text: String,
}

#[derive(Debug, Serialize)]
struct ToolResult {
    content: Vec<ToolContent>,
    #[serde(rename = "structuredContent")]
    structured_content: Value,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: LeptosTools::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    tracing::error!(%error, "failed to read request line");
                    break;
                }
            };

            if let Some(response) = self.handle_line(&line).await {
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{response_json}")?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    pub async fn handle_line(&self, line: &str) -> Option<JsonRpcResponse> {
        if line.trim().is_empty() {
            return None;
        }

        let value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(%error, "failed to parse JSON-RPC request");
                return Some(JsonRpcResponse::error(
                    Value::Null,
                    ProtocolError::ParseError("Parse error".to_string()),
                ));
            }
        };

        let request_id = value.get("id").cloned().unwrap_or(Value::Null);
        let request: JsonRpcRequest = match serde_json::from_value(value) {
            Ok(request) => request,
            Err(error) => {
                return Some(JsonRpcResponse::error(
                    request_id,
                    ProtocolError::InvalidRequest(error.to_string()),
                ));
            }
        };

        let is_notification = request.id.is_none();
        match self.handle_request(request).await {
            Ok(_) if is_notification => {
                tracing::debug!("handled JSON-RPC notification");
                None
            }
            Ok(result) => Some(JsonRpcResponse::success(request_id, result)),
            Err(error) if is_notification && !matches!(error, ProtocolError::InvalidRequest(_)) => {
                tracing::debug!("ignored failed JSON-RPC notification");
                None
            }
            Err(error) => Some(JsonRpcResponse::error(request_id, error)),
        }
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> Result<Value, ProtocolError> {
        validate_jsonrpc_version(&request)?;
        let method = request
            .method
            .as_deref()
            .ok_or_else(|| ProtocolError::InvalidRequest("missing method".to_string()))?;

        tracing::debug!(method, "handling request");

        match method {
            "initialize" => Ok(self.handle_initialize()),
            "tools/list" => Ok(self.handle_list_tools()),
            "tools/call" => self.handle_call_tool(request.params),
            method => Err(ProtocolError::MethodNotFound(method.to_string())),
        }
    }

    fn handle_initialize(&self) -> Value {
        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "leptos-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        })
    }

    fn handle_list_tools(&self) -> Value {
        json!({
            "tools": [
                {
                    "name": LIST_SECTIONS_TOOL,
                    "description": "List all available Leptos documentation sections with canonical ids, aliases, and version metadata",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }
                },
                {
                    "name": GET_DOCUMENTATION_TOOL,
                    "description": "Get Leptos documentation for a canonical section id or declared alias",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "section": {
                                "type": "string",
                                "description": "Canonical section id or declared alias from list-sections"
                            }
                        },
                        "required": ["section"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": LEPTOS_DIAGNOSTICS_TOOL,
                    "description": "Analyze Leptos code and return structured diagnostics",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "code": {
                                "type": "string",
                                "description": "Leptos code to analyze"
                            }
                        },
                        "required": ["code"],
                        "additionalProperties": false
                    }
                }
            ]
        })
    }

    fn handle_call_tool(&self, params: Option<Value>) -> Result<Value, ProtocolError> {
        let params = params.ok_or_else(|| {
            ProtocolError::InvalidParams("tools/call params are required".to_string())
        })?;
        let call: ToolCallParams = serde_json::from_value(params)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;

        let output = match call.name.as_str() {
            LIST_SECTIONS_TOOL => self.tools.list_sections(),
            GET_DOCUMENTATION_TOOL => {
                let args: DocumentationArgs = parse_arguments(call.arguments)?;
                self.tools.get_documentation(&args.section)?
            }
            LEPTOS_DIAGNOSTICS_TOOL => {
                let args: DiagnosticsArgs = parse_arguments(call.arguments)?;
                self.tools.diagnose_leptos_code(&args.code)?
            }
            _ => return Err(ToolError::UnknownTool(call.name).into()),
        };

        let result = ToolResult {
            content: vec![ToolContent {
                content_type: "text",
                text: output.text,
            }],
            structured_content: serde_json::to_value(output.structured)
                .map_err(|error| ProtocolError::InternalError(error.to_string()))?,
        };

        serde_json::to_value(result)
            .map_err(|error| ProtocolError::InternalError(error.to_string()))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION,
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, error: ProtocolError) -> Self {
        let error = JsonRpcError {
            code: error.code(),
            message: error.message(),
        };

        Self {
            jsonrpc: JSON_RPC_VERSION,
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl ProtocolError {
    fn code(&self) -> i32 {
        match self {
            ProtocolError::ParseError(_) => -32700,
            ProtocolError::InvalidRequest(_) => -32600,
            ProtocolError::MethodNotFound(_) => -32601,
            ProtocolError::InvalidParams(_) => -32602,
            ProtocolError::InternalError(_) => -32603,
        }
    }

    fn message(self) -> String {
        match self {
            ProtocolError::ParseError(message)
            | ProtocolError::InvalidRequest(message)
            | ProtocolError::InvalidParams(message)
            | ProtocolError::InternalError(message) => message,
            ProtocolError::MethodNotFound(method) => format!("Method not found: {method}"),
        }
    }
}

impl From<ToolError> for ProtocolError {
    fn from(error: ToolError) -> Self {
        ProtocolError::InvalidParams(error.message())
    }
}

fn validate_jsonrpc_version(request: &JsonRpcRequest) -> Result<(), ProtocolError> {
    match request.jsonrpc.as_deref() {
        Some(JSON_RPC_VERSION) => Ok(()),
        Some(version) => Err(ProtocolError::InvalidRequest(format!(
            "unsupported JSON-RPC version: {version}"
        ))),
        None => Err(ProtocolError::InvalidRequest(
            "missing JSON-RPC version".to_string(),
        )),
    }
}

fn parse_arguments<T>(arguments: Value) -> Result<T, ProtocolError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(arguments)
        .map_err(|error| ProtocolError::InvalidParams(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_code(response: &JsonRpcResponse) -> i32 {
        response.error.as_ref().expect("expected error").code
    }

    fn result(response: &JsonRpcResponse) -> &Value {
        response.result.as_ref().expect("expected result")
    }

    #[tokio::test]
    async fn malformed_json_returns_parse_error_response() {
        let server = McpServer::new();
        let response = server
            .handle_line("{bad json}")
            .await
            .expect("parse errors should receive a response");

        assert_eq!(error_code(&response), -32700);
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"unknown"}"#)
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32601);
    }

    #[tokio::test]
    async fn invalid_jsonrpc_version_is_rejected() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"1.0","id":1,"method":"tools/list"}"#)
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32600);
    }

    #[tokio::test]
    async fn notifications_do_not_receive_responses() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
            .await;

        assert!(response.is_none());
    }

    #[tokio::test]
    async fn missing_documentation_section_is_invalid_params() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32602);
    }

    #[tokio::test]
    async fn diagnostics_tool_returns_structured_content() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"leptos-diagnostics","arguments":{"code":"fn App() -> impl IntoView { view! { <p>{count.get()}</p> } }"}}}"#,
            )
            .await
            .expect("request should receive a response");

        let structured = &result(&response)["structuredContent"];
        assert_eq!(structured["kind"], "diagnostics");
        assert!(structured["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| { diagnostic["rule_id"] == "leptos.missing-component-attribute" }));
    }
}
