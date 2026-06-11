//! MCP protocol implementation.
//!
//! This module owns JSON-RPC/MCP validation, notification semantics, and routing
//! requests to domain capability handlers. Byte framing belongs in `transport`,
//! while pure capability/catalog construction belongs in `catalog` and the
//! domain modules it aggregates.

use crate::catalog;
use crate::docs;
use crate::prompts;
use crate::tools::{
    GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
    LOOKUP_API_TOOL, LeptosTools, SEARCH_DOCS_TOOL, ToolError,
};
use crate::transport::{LineRead, read_limited_line};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{self, BufReader, Write};

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

#[derive(Debug)]
struct JsonRpcEnvelope {
    id: Option<Value>,
    call: JsonRpcCall,
}

#[derive(Debug)]
struct JsonRpcCall {
    method: String,
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
                    let response = oversized_line_error_response();
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
            return Some(oversized_line_error_response());
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

        let envelope = match validate_json_rpc_request(request) {
            Ok(envelope) => envelope,
            Err(error) => return Some(JsonRpcResponse::error(request_id, error)),
        };

        let request_id = envelope.id.clone().unwrap_or(Value::Null);
        let is_notification = envelope.id.is_none();
        match self.handle_request(envelope.call).await {
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

    async fn handle_request(&self, request: JsonRpcCall) -> Result<Value, ProtocolError> {
        let method = request.method.as_str();

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
        catalog::initialize_result(MCP_PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"))
    }

    fn handle_list_tools(&self) -> Value {
        catalog::tools_list_result()
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
        catalog::resources_list_result()
    }

    fn handle_read_resource(&self, params: Option<Value>) -> Result<Value, ProtocolError> {
        let params = params.ok_or_else(|| {
            ProtocolError::InvalidParams("resources/read params are required".to_string())
        })?;
        let params: ResourceReadParams = serde_json::from_value(params)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;
        let catalog_section =
            docs::get_catalog_section_by_resource_uri(&params.uri).map_err(|error| {
                ProtocolError::InvalidParams(ToolError::DocumentationLookup(error).message())
            })?;
        let section = catalog_section.section;

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
        catalog::prompts_list_result()
    }

    fn handle_get_prompt(&self, params: Option<Value>) -> Result<Value, ProtocolError> {
        let params = params.ok_or_else(|| {
            ProtocolError::InvalidParams("prompts/get params are required".to_string())
        })?;
        let params: PromptGetParams = serde_json::from_value(params)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;
        let prompt = prompts::get_prompt(&params.name)
            .map_err(|error| ProtocolError::InvalidParams(prompt_error_message(error)))?;
        let text = prompts::render_prompt_checked(prompt, &params.arguments)
            .map_err(|error| ProtocolError::InvalidParams(error.to_string()))?;

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

fn validate_json_rpc_request(request: JsonRpcRequest) -> Result<JsonRpcEnvelope, ProtocolError> {
    validate_jsonrpc_version(&request)?;
    let method = request
        .method
        .ok_or_else(|| ProtocolError::InvalidRequest("missing method".to_string()))?;

    Ok(JsonRpcEnvelope {
        id: request.id,
        call: JsonRpcCall {
            method,
            params: request.params,
        },
    })
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

fn oversized_line_error_response() -> JsonRpcResponse {
    // The read path rejects oversized input before a complete JSON-RPC frame is available.
    // Any apparent `id` inside that partial frame is untrusted and may not have been read, so
    // JSON-RPC requires the error response to use a null id.
    JsonRpcResponse::error(
        Value::Null,
        ProtocolError::InvalidRequest(format!(
            "JSON-RPC request line must be at most {MAX_JSON_RPC_LINE_BYTES} bytes"
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::MAX_DIAGNOSTIC_CODE_BYTES;
    use std::collections::BTreeSet;

    fn error_code(response: &JsonRpcResponse) -> i32 {
        response.error.as_ref().expect("expected error").code
    }

    fn error_message(response: &JsonRpcResponse) -> &str {
        response
            .error
            .as_ref()
            .expect("expected error")
            .message
            .as_str()
    }

    fn response_id(response: &JsonRpcResponse) -> &Value {
        &response.id
    }

    fn result(response: &JsonRpcResponse) -> &Value {
        response.result.as_ref().expect("expected result")
    }

    fn object_keys(value: &Value) -> BTreeSet<&str> {
        value
            .as_object()
            .expect("value should be an object")
            .keys()
            .map(String::as_str)
            .collect()
    }

    fn string_array(value: &Value) -> Vec<&str> {
        value
            .as_array()
            .expect("value should be an array")
            .iter()
            .map(|value| value.as_str().expect("array item should be a string"))
            .collect()
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
    async fn initialize_response_pins_wire_contract() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":"init-1","method":"initialize","params":{}}"#)
            .await
            .expect("initialize should receive a response");

        assert_eq!(response_id(&response), "init-1");
        let result = result(&response);
        assert_eq!(
            object_keys(result),
            BTreeSet::from(["capabilities", "protocolVersion", "serverInfo"])
        );
        assert_eq!(result["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert_eq!(
            object_keys(&result["capabilities"]),
            BTreeSet::from(["prompts", "resources", "tools"])
        );
        assert_eq!(
            object_keys(&result["capabilities"]["tools"]),
            BTreeSet::new()
        );
        assert_eq!(
            object_keys(&result["capabilities"]["resources"]),
            BTreeSet::new()
        );
        assert_eq!(
            object_keys(&result["capabilities"]["prompts"]),
            BTreeSet::new()
        );
        assert_eq!(result["serverInfo"]["name"], "leptos-mcp-server");
        assert!(result["serverInfo"]["version"].is_string());
    }

    #[tokio::test]
    async fn tools_list_pins_tool_names_and_input_schemas() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#)
            .await
            .expect("tools/list should receive a response");

        let tools = result(&response)["tools"]
            .as_array()
            .expect("tools should be an array");
        let by_name = tools
            .iter()
            .map(|tool| {
                (
                    tool["name"].as_str().expect("tool name should be a string"),
                    tool,
                )
            })
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            by_name.keys().copied().collect::<Vec<_>>(),
            vec![
                GET_DOCUMENTATION_TOOL,
                LEPTOS_AXUM_RECIPE_TOOL,
                LEPTOS_DIAGNOSTICS_TOOL,
                LIST_SECTIONS_TOOL,
                LOOKUP_API_TOOL,
                SEARCH_DOCS_TOOL,
            ]
        );

        let list_schema = &by_name[LIST_SECTIONS_TOOL]["inputSchema"];
        assert_eq!(list_schema["type"], "object");
        assert_eq!(list_schema["additionalProperties"], false);
        assert_eq!(object_keys(&list_schema["properties"]), BTreeSet::new());
        assert!(list_schema.get("required").is_none());

        let documentation_schema = &by_name[GET_DOCUMENTATION_TOOL]["inputSchema"];
        assert_eq!(
            object_keys(&documentation_schema["properties"]),
            BTreeSet::from(["section"])
        );
        assert_eq!(
            documentation_schema["properties"]["section"]["type"],
            "string"
        );
        assert_eq!(
            string_array(&documentation_schema["required"]),
            vec!["section"]
        );
        assert_eq!(documentation_schema["additionalProperties"], false);

        let diagnostics_schema = &by_name[LEPTOS_DIAGNOSTICS_TOOL]["inputSchema"];
        assert_eq!(
            object_keys(&diagnostics_schema["properties"]),
            BTreeSet::from(["code"])
        );
        assert_eq!(diagnostics_schema["properties"]["code"]["type"], "string");
        assert_eq!(
            diagnostics_schema["properties"]["code"]["maxLength"],
            MAX_DIAGNOSTIC_CODE_BYTES
        );
        assert_eq!(string_array(&diagnostics_schema["required"]), vec!["code"]);
        assert_eq!(diagnostics_schema["additionalProperties"], false);

        let search_schema = &by_name[SEARCH_DOCS_TOOL]["inputSchema"];
        assert_eq!(
            object_keys(&search_schema["properties"]),
            BTreeSet::from(["query"])
        );
        assert_eq!(search_schema["properties"]["query"]["type"], "string");
        assert_eq!(string_array(&search_schema["required"]), vec!["query"]);
        assert_eq!(search_schema["additionalProperties"], false);

        let lookup_schema = &by_name[LOOKUP_API_TOOL]["inputSchema"];
        assert_eq!(
            object_keys(&lookup_schema["properties"]),
            BTreeSet::from(["crate", "query"])
        );
        assert_eq!(lookup_schema["properties"]["query"]["type"], "string");
        assert_eq!(lookup_schema["properties"]["crate"]["type"], "string");
        assert_eq!(
            string_array(&lookup_schema["properties"]["crate"]["enum"]),
            vec!["leptos", "leptos_axum", "axum"]
        );
        assert_eq!(string_array(&lookup_schema["required"]), vec!["query"]);
        assert_eq!(lookup_schema["additionalProperties"], false);

        let recipe_schema = &by_name[LEPTOS_AXUM_RECIPE_TOOL]["inputSchema"];
        assert_eq!(
            object_keys(&recipe_schema["properties"]),
            BTreeSet::from(["recipe"])
        );
        assert_eq!(recipe_schema["properties"]["recipe"]["type"], "string");
        assert_eq!(string_array(&recipe_schema["required"]), vec!["recipe"]);
        assert_eq!(recipe_schema["additionalProperties"], false);
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
    async fn api_lookup_tool_maps_ambiguous_and_unknown_queries_to_invalid_params() {
        let server = McpServer::new();
        let ambiguous = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"lookup-api","arguments":{"query":"extractor"}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&ambiguous), -32602);
        assert!(ambiguous.result.is_none());
        assert!(error_message(&ambiguous).contains("Ambiguous API symbol 'extractor'"));

        let unknown = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"lookup-api","arguments":{"query":"not-a-real-symbol","crate":"leptos"}}}"#,
            )
            .await
            .expect("request should receive a response");

        assert_eq!(error_code(&unknown), -32602);
        assert!(unknown.result.is_none());
        assert_eq!(
            error_message(&unknown),
            "Unknown API symbol in crate leptos: not-a-real-symbol"
        );
    }

    #[tokio::test]
    async fn search_docs_tool_returns_ranked_structured_content() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search-docs","arguments":{"query":"server-fn"}}}"#,
            )
            .await
            .expect("request should receive a response");

        let structured = &result(&response)["structuredContent"];
        assert_eq!(structured["kind"], "search-docs");
        assert_eq!(structured["query"], "server-fn");

        let results = structured["results"]
            .as_array()
            .expect("search results should be an array");
        let first = results.first().expect("server-fn should match a section");
        assert_eq!(first["section"]["id"], "server-functions");
        assert_eq!(string_array(&first["matched_fields"]), vec!["aliases"]);
        assert!(first["score"].is_number());
        assert!(
            first["why"]
                .as_str()
                .expect("why should be a string")
                .contains("exact identity in aliases")
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
    async fn resources_list_pins_resource_shape() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}"#)
            .await
            .expect("resources/list should receive a response");

        let resources = result(&response)["resources"]
            .as_array()
            .expect("resources should be an array");
        assert_eq!(resources.len(), docs::list_sections().len());
        for resource in resources {
            assert_eq!(
                object_keys(resource),
                BTreeSet::from(["description", "mimeType", "name", "uri"])
            );
            assert!(
                resource["uri"]
                    .as_str()
                    .expect("uri should be a string")
                    .starts_with("leptos://docs/")
            );
            assert!(resource["name"].is_string());
            assert!(resource["description"].is_string());
            assert_eq!(resource["mimeType"], "text/markdown");
        }

        let read = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"leptos://docs/axum"}}"#,
            )
            .await
            .expect("resources/read should receive a response");
        let result = result(&read);
        assert_eq!(object_keys(result), BTreeSet::from(["contents"]));
        let contents = result["contents"]
            .as_array()
            .expect("resource contents should be an array");
        assert_eq!(contents.len(), 1);
        assert_eq!(
            object_keys(&contents[0]),
            BTreeSet::from(["mimeType", "text", "uri"])
        );
        assert_eq!(contents[0]["uri"], "leptos://docs/axum");
        assert_eq!(contents[0]["mimeType"], "text/markdown");
        assert!(contents[0]["text"].is_string());
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
    async fn prompts_list_and_get_pin_prompt_wire_shape() {
        let server = McpServer::new();
        let list = server
            .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{}}"#)
            .await
            .expect("prompts/list should receive a response");

        let prompts = result(&list)["prompts"]
            .as_array()
            .expect("prompts should be an array");
        let by_name = prompts
            .iter()
            .map(|prompt| {
                assert_eq!(
                    object_keys(prompt),
                    BTreeSet::from(["arguments", "description", "name"])
                );
                (
                    prompt["name"]
                        .as_str()
                        .expect("prompt name should be a string"),
                    prompt,
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            by_name.keys().copied().collect::<Vec<_>>(),
            vec![
                "add-server-function",
                "debug-hydration",
                "review-axum-integration",
                "review-sql-access",
                "wire-leptos-axum-ssr",
            ]
        );

        let sql_args = by_name["review-sql-access"]["arguments"]
            .as_array()
            .expect("prompt arguments should be an array");
        assert_eq!(sql_args.len(), 2);
        assert_eq!(sql_args[0]["name"], "code");
        assert_eq!(sql_args[0]["required"], true);
        assert_eq!(sql_args[1]["name"], "backend");
        assert_eq!(sql_args[1]["required"], false);

        let prompt = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"review-sql-access","arguments":{"backend":"SQLite","code":"SELECT 1"}}}"#,
            )
            .await
            .expect("prompts/get should receive a response");
        let result = result(&prompt);
        assert_eq!(
            object_keys(result),
            BTreeSet::from(["description", "messages"])
        );
        assert_eq!(
            result["description"],
            by_name["review-sql-access"]["description"]
        );
        let message = &result["messages"][0];
        assert_eq!(object_keys(message), BTreeSet::from(["content", "role"]));
        assert_eq!(message["role"], "user");
        assert_eq!(
            object_keys(&message["content"]),
            BTreeSet::from(["text", "type"])
        );
        assert_eq!(message["content"]["type"], "text");
        assert!(
            message["content"]["text"]
                .as_str()
                .expect("prompt text should be a string")
                .contains("SELECT 1")
        );
    }

    #[tokio::test]
    async fn notifications_ignore_success_and_non_invalid_request_errors() {
        let server = McpServer::new();

        assert!(
            server
                .handle_line(r#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#)
                .await
                .is_none()
        );
        assert!(
            server
                .handle_line(r#"{"jsonrpc":"2.0","method":"unknown","params":{}}"#)
                .await
                .is_none()
        );
        assert!(
            server
                .handle_line(
                    r#"{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get-documentation","arguments":{}}}"#,
                )
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn invalid_request_notifications_still_return_errors() {
        let server = McpServer::new();
        let response = server
            .handle_line(r#"{"jsonrpc":"1.0","method":"initialize","params":{}}"#)
            .await
            .expect("invalid request notifications should receive a response");

        assert_eq!(response_id(&response), &Value::Null);
        assert_eq!(error_code(&response), -32600);
    }

    #[tokio::test]
    async fn json_rpc_error_codes_are_pinned_by_failure_class() {
        let server = McpServer::new();
        let cases = [
            ("{bad json}", -32700),
            (r#"{"jsonrpc":"1.0","id":1,"method":"initialize"}"#, -32600),
            (
                r#"{"jsonrpc":"2.0","id":1,"method":"missing/method"}"#,
                -32601,
            ),
            (
                r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{}}}"#,
                -32602,
            ),
        ];

        for (line, expected_code) in cases {
            let response = server
                .handle_line(line)
                .await
                .expect("failure request should receive a response");
            assert_eq!(error_code(&response), expected_code, "line: {line}");
            assert!(response.result.is_none());
        }

        let internal = JsonRpcResponse::error(
            Value::Null,
            ProtocolError::InternalError("boom".to_string()),
        );
        assert_eq!(error_code(&internal), -32603);
    }

    #[tokio::test]
    async fn prompts_get_allows_missing_optional_arguments() {
        let server = McpServer::new();
        let prompt = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"prompts/get","params":{"name":"debug-hydration","arguments":{"symptom":"WASM 404"}}}"#,
            )
            .await
            .expect("prompts/get should receive a response");

        let text = result(&prompt)["messages"][0]["content"]["text"]
            .as_str()
            .expect("prompt text should exist");
        assert!(text.contains("WASM 404"));
        assert!(text.contains("Environment: ."));
        assert!(prompt.error.is_none());
    }

    #[tokio::test]
    async fn prompts_get_rejects_missing_required_arguments() {
        let server = McpServer::new();
        let response = server
            .handle_line(
                r#"{"jsonrpc":"2.0","id":1,"method":"prompts/get","params":{"name":"review-sql-access","arguments":{"backend":"SQLite"}}}"#,
            )
            .await
            .expect("prompts/get should receive a response");

        assert_eq!(error_code(&response), -32602);
        assert!(response.result.is_none());
        assert!(
            response
                .error
                .as_ref()
                .expect("expected error")
                .message
                .contains("code")
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
        assert_eq!(response_id(&response), &Value::Null);
    }

    #[test]
    fn oversized_line_error_response_is_invalid_request_with_null_id() {
        let response = oversized_line_error_response();
        assert_eq!(error_code(&response), -32600);
        assert_eq!(response_id(&response), &Value::Null);
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
        assert!(structured["summary"]["error_count"].is_number());
        assert!(structured["summary"]["warning_count"].is_number());
        assert!(structured["summary"]["info_count"].is_number());

        let diagnostics = structured["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array");
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic["rule_id"] == "leptos.missing-component-attribute")
            .expect("missing component attribute rule should be present");

        assert_eq!(diagnostic["severity"], "warning");
        assert_eq!(diagnostic["confidence"], "medium");
    }
}
