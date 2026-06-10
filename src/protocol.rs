//! MCP protocol implementation.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio.

use crate::docs;
use crate::prompts;
use crate::tools::{
    GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
    LOOKUP_API_TOOL, LeptosTools, SEARCH_DOCS_TOOL, ToolError,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Write};

const JSON_RPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const MAX_JSON_RPC_LINE_BYTES: usize = 1024 * 1024;

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
#[serde(deny_unknown_fields)]
struct ToolCallParams {
    name: String,
    #[serde(default = "empty_arguments")]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListSectionsArgs {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentationArgs {
    section: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticsArgs {
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchDocsArgs {
    query: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApiLookupArgs {
    query: String,
    #[serde(default, rename = "crate")]
    crate_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeArgs {
    recipe: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceReadParams {
    uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromptGetParams {
    name: String,
    #[serde(default)]
    arguments: BTreeMap<String, String>,
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

enum LineRead {
    Line(String),
    Oversized,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tools: LeptosTools::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        loop {
            let line = match read_limited_line(&mut reader, MAX_JSON_RPC_LINE_BYTES) {
                Ok(Some(LineRead::Line(line))) => line,
                Ok(Some(LineRead::Oversized)) => {
                    let response = JsonRpcResponse::error(
                        Value::Null,
                        ProtocolError::InvalidRequest(format!(
                            "JSON-RPC request line must be at most {MAX_JSON_RPC_LINE_BYTES} bytes"
                        )),
                    );
                    let response_json = serde_json::to_string(&response)?;
                    writeln!(stdout, "{response_json}")?;
                    stdout.flush()?;
                    continue;
                }
                Ok(None) => break,
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
        if line.len() > MAX_JSON_RPC_LINE_BYTES {
            return Some(JsonRpcResponse::error(
                Value::Null,
                ProtocolError::InvalidRequest(format!(
                    "JSON-RPC request line must be at most {MAX_JSON_RPC_LINE_BYTES} bytes"
                )),
            ));
        }

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
            "resources/list" => Ok(self.handle_list_resources()),
            "resources/read" => self.handle_read_resource(request.params),
            "prompts/list" => Ok(self.handle_list_prompts()),
            "prompts/get" => self.handle_get_prompt(request.params),
            method => Err(ProtocolError::MethodNotFound(method.to_string())),
        }
    }

    fn handle_initialize(&self) -> Value {
        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
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
                                "description": "Leptos code to analyze",
                                "maxLength": crate::tools::MAX_DIAGNOSTIC_CODE_BYTES
                            }
                        },
                        "required": ["code"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": SEARCH_DOCS_TOOL,
                    "description": "Search Leptos, leptos_axum, and Axum documentation sections by task, API, or failure mode",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Task, API, error, or workflow to search for"
                            }
                        },
                        "required": ["query"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": LOOKUP_API_TOOL,
                    "description": "Look up a curated Leptos, leptos_axum, or Axum public API symbol",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Symbol name or declared alias"
                            },
                            "crate": {
                                "type": "string",
                                "description": "Optional crate filter: leptos, leptos_axum, or axum",
                                "enum": ["leptos", "leptos_axum", "axum"]
                            }
                        },
                        "required": ["query"],
                        "additionalProperties": false
                    }
                },
                {
                    "name": LEPTOS_AXUM_RECIPE_TOOL,
                    "description": "Return a task-oriented recipe for common Leptos + Axum workflows",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "recipe": {
                                "type": "string",
                                "description": "Recipe id or alias such as ssr-app, server-functions, static-assets, custom-handler, state-context, database-query-patterns, or wasm-runtime"
                            }
                        },
                        "required": ["recipe"],
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
            LIST_SECTIONS_TOOL => {
                let _: ListSectionsArgs = parse_arguments(call.arguments)?;
                self.tools.list_sections()
            }
            GET_DOCUMENTATION_TOOL => {
                let args: DocumentationArgs = parse_arguments(call.arguments)?;
                self.tools.get_documentation(&args.section)?
            }
            LEPTOS_DIAGNOSTICS_TOOL => {
                let args: DiagnosticsArgs = parse_arguments(call.arguments)?;
                self.tools.diagnose_leptos_code(&args.code)?
            }
            SEARCH_DOCS_TOOL => {
                let args: SearchDocsArgs = parse_arguments(call.arguments)?;
                self.tools.search_docs(&args.query)?
            }
            LOOKUP_API_TOOL => {
                let args: ApiLookupArgs = parse_arguments(call.arguments)?;
                self.tools
                    .lookup_api(&args.query, args.crate_name.as_deref())?
            }
            LEPTOS_AXUM_RECIPE_TOOL => {
                let args: RecipeArgs = parse_arguments(call.arguments)?;
                self.tools.leptos_axum_recipe(&args.recipe)?
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

    fn handle_list_resources(&self) -> Value {
        let resources = docs::list_sections()
            .iter()
            .map(|section| {
                json!({
                    "uri": docs::resource_uri(section),
                    "name": section.title,
                    "description": section.use_cases,
                    "mimeType": "text/markdown"
                })
            })
            .collect::<Vec<_>>();

        json!({ "resources": resources })
    }

    fn handle_read_resource(&self, params: Option<Value>) -> Result<Value, ProtocolError> {
        let params = params.ok_or_else(|| {
            ProtocolError::InvalidParams("resources/read params are required".to_string())
        })?;
        let params: ResourceReadParams = serde_json::from_value(params)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;
        let section = docs::get_section_by_resource_uri(&params.uri).map_err(|error| {
            ProtocolError::InvalidParams(ToolError::DocumentationLookup(error).message())
        })?;

        Ok(json!({
            "contents": [
                {
                    "uri": params.uri,
                    "mimeType": "text/markdown",
                    "text": format!("# {}\n\n{}", section.title, section.content)
                }
            ]
        }))
    }

    fn handle_list_prompts(&self) -> Value {
        let prompts = prompts::all_prompts()
            .iter()
            .map(|prompt| {
                json!({
                    "name": prompt.name,
                    "description": prompt.description,
                    "arguments": prompt.arguments
                })
            })
            .collect::<Vec<_>>();

        json!({ "prompts": prompts })
    }

    fn handle_get_prompt(&self, params: Option<Value>) -> Result<Value, ProtocolError> {
        let params = params.ok_or_else(|| {
            ProtocolError::InvalidParams("prompts/get params are required".to_string())
        })?;
        let params: PromptGetParams = serde_json::from_value(params)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;
        let prompt = prompts::get_prompt(&params.name)
            .map_err(|error| ProtocolError::InvalidParams(prompt_error_message(error)))?;
        let text = prompts::render_prompt(prompt, &params.arguments);

        Ok(json!({
            "description": prompt.description,
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": text
                    }
                }
            ]
        }))
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

fn prompt_error_message(error: prompts::PromptLookupError) -> String {
    match error {
        prompts::PromptLookupError::Empty => "prompt name must be non-empty".to_string(),
        prompts::PromptLookupError::Unknown { name } => format!("Unknown prompt: {name}"),
    }
}

fn parse_arguments<T>(arguments: Value) -> Result<T, ProtocolError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(arguments)
        .map_err(|error| ProtocolError::InvalidParams(error.to_string()))
}

fn empty_arguments() -> Value {
    json!({})
}

fn read_limited_line<R: BufRead>(reader: &mut R, max_bytes: usize) -> io::Result<Option<LineRead>> {
    let mut bytes = Vec::new();

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Ok(Some(LineRead::Line(decode_line(bytes))))
            };
        }

        if let Some(newline_index) = available.iter().position(|byte| *byte == b'\n') {
            let take = newline_index + 1;
            if bytes.len() + newline_index > max_bytes {
                reader.consume(take);
                return Ok(Some(LineRead::Oversized));
            }
            bytes.extend_from_slice(&available[..take]);
            reader.consume(take);
            return Ok(Some(LineRead::Line(decode_line(bytes))));
        }

        let take = available.len();
        if bytes.len() + take > max_bytes {
            reader.consume(take);
            discard_until_newline(reader)?;
            return Ok(Some(LineRead::Oversized));
        }

        bytes.extend_from_slice(available);
        reader.consume(take);
    }
}

fn discard_until_newline<R: BufRead>(reader: &mut R) -> io::Result<()> {
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(());
        }

        if let Some(newline_index) = available.iter().position(|byte| *byte == b'\n') {
            reader.consume(newline_index + 1);
            return Ok(());
        }

        let take = available.len();
        reader.consume(take);
    }
}

fn decode_line(mut bytes: Vec<u8>) -> String {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::MAX_DIAGNOSTIC_CODE_BYTES;

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
    async fn extra_tool_call_params_are_rejected() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections","arguments":{},"extra":true}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32602);
    }

    #[tokio::test]
    async fn extra_tool_arguments_are_rejected() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals","extra":true}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32602);
    }

    #[tokio::test]
    async fn list_sections_rejects_arguments() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections","arguments":{"extra":true}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&response), -32602);
    }

    #[tokio::test]
    async fn list_sections_accepts_missing_arguments() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections"}}"#,
            )
            .await
            .expect("request should receive a response");

        assert!(result(&response)["structuredContent"]["sections"].is_array());
    }

    #[tokio::test]
    async fn initialize_advertises_resources_and_prompts() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)
            .await
            .expect("initialize should receive a response");

        let capabilities = &result(&response)["capabilities"];
        assert!(capabilities["tools"].is_object());
        assert!(capabilities["resources"].is_object());
        assert!(capabilities["prompts"].is_object());
    }

    #[tokio::test]
    async fn tools_list_includes_search_api_lookup_and_recipes() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#)
            .await
            .expect("tools/list should receive a response");

        let names: Vec<&str> = result(&response)["tools"]
            .as_array()
            .expect("tools should be an array")
            .iter()
            .map(|tool| tool["name"].as_str().expect("tool should have name"))
            .collect();

        assert!(names.contains(&SEARCH_DOCS_TOOL));
        assert!(names.contains(&LOOKUP_API_TOOL));
        assert!(names.contains(&LEPTOS_AXUM_RECIPE_TOOL));
    }

    #[tokio::test]
    async fn api_lookup_tool_returns_symbol_metadata() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"lookup-api","arguments":{"query":"ResponseOptions","crate":"leptos_axum"}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(
            result(&response)["structuredContent"]["symbol"]["name"],
            "leptos_axum::ResponseOptions"
        );
    }

    #[tokio::test]
    async fn recipe_tool_returns_workflow_files() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"leptos-axum-recipe","arguments":{"recipe":"database-query-patterns"}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert!(result(&response)["structuredContent"]["recipe"]["files"].is_array());
        assert_eq!(
            result(&response)["structuredContent"]["recipe"]["id"],
            "database-query-patterns"
        );
    }

    #[tokio::test]
    async fn resources_list_and_read_expose_documentation_sections() {
        let server = McpServer::new();
        let list = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}"#)
            .await
            .expect("resources/list should receive a response");

        assert!(
            result(&list)["resources"]
                .as_array()
                .expect("resources should be an array")
                .iter()
                .any(|resource| resource["uri"] == "leptos://docs/axum")
        );

        let read = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"leptos://docs/axum"}}"#,
            )
            .await
            .expect("resources/read should receive a response");

        assert!(
            result(&read)["contents"][0]["text"]
                .as_str()
                .expect("text content should exist")
                .contains("Axum 0.8.9")
        );
    }

    #[tokio::test]
    async fn prompts_list_and_get_render_workflow_prompt() {
        let server = McpServer::new();
        let list = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{}}"#)
            .await
            .expect("prompts/list should receive a response");

        assert!(
            result(&list)["prompts"]
                .as_array()
                .expect("prompts should be an array")
                .iter()
                .any(|prompt| prompt["name"] == "review-sql-access")
        );

        let prompt = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"review-sql-access","arguments":{"backend":"SQLite","code":"sqlx::query!(\"SELECT 1\")"}}}"#,
            )
            .await
            .expect("prompts/get should receive a response");

        assert!(
            result(&prompt)["messages"][0]["content"]["text"]
                .as_str()
                .expect("prompt text should exist")
                .contains("bind parameters")
        );
    }

    #[tokio::test]
    async fn oversized_json_rpc_line_is_rejected() {
        let server = McpServer::new();
        let oversized = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/list","padding":"{}"}}"#,
            "x".repeat(MAX_JSON_RPC_LINE_BYTES)
        );
        let response = server
            .handle_line(&oversized)
            .await
            .expect("oversized requests should receive a response");

        assert_eq!(error_code(&response), -32600);
    }

    #[test]
    fn max_sized_stdin_line_with_newline_is_accepted() {
        let input = format!("{}\n", "x".repeat(MAX_JSON_RPC_LINE_BYTES));
        let mut reader = BufReader::new(input.as_bytes());
        let line = read_limited_line(&mut reader, MAX_JSON_RPC_LINE_BYTES)
            .expect("line reader should succeed")
            .expect("line should be present");

        match line {
            LineRead::Line(line) => assert_eq!(line.len(), MAX_JSON_RPC_LINE_BYTES),
            LineRead::Oversized => panic!("line at the exact limit should be accepted"),
        }
    }

    #[tokio::test]
    async fn oversized_diagnostics_code_is_rejected() {
        let server = McpServer::new();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "leptos-diagnostics",
                "arguments": {
                    "code": "x".repeat(MAX_DIAGNOSTIC_CODE_BYTES + 1)
                }
            }
        });
        let response = server
            .handle_line(&request.to_string())
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
        assert!(
            structured["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| {
                    diagnostic["rule_id"] == "leptos.missing-component-attribute"
                })
        );
    }
}
