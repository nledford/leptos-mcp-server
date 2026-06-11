//! SDK-facing MCP server support.
//!
//! This module owns the `rust-mcp-sdk` server details, handler construction,
//! stdio startup path, and tool list/call adaptation. Domain behavior remains
//! in the existing app facade.

use crate::app::{AppError, LeptosApp, ToolCall};
use crate::tools::{ToolError, ToolOutput};
use rust_mcp_sdk::macros::{JsonSchema, mcp_resource_template, mcp_tool};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::tool_box;
use rust_mcp_sdk::{
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    error::SdkResult,
    mcp_server::{McpServerOptions, ServerHandler, ServerRuntime, server_runtime},
    schema::{
        CallToolRequestParams, CallToolResult, ContentBlock, GetPromptRequestParams,
        GetPromptResult, Implementation, InitializeResult, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        Prompt as SdkPrompt, PromptArgument as SdkPromptArgument,
        PromptMessage as SdkPromptMessage, ProtocolVersion, ReadResourceRequestParams,
        ReadResourceResult, Resource, Role, RpcError, ServerCapabilities,
        ServerCapabilitiesPrompts, ServerCapabilitiesResources, ServerCapabilitiesTools,
        TextContent, TextResourceContents,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{future::Future, pin::Pin, sync::Arc};

#[mcp_tool(
    name = "list-sections",
    description = "List all available Leptos documentation sections with canonical ids, aliases, and version metadata"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListSectionsTool {}

#[mcp_tool(
    name = "get-documentation",
    description = "Get Leptos documentation for a canonical section id or declared alias"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetDocumentationTool {
    /// Canonical section id or declared alias from list-sections.
    pub section: String,
}

#[mcp_tool(
    name = "leptos-diagnostics",
    description = "Analyze Leptos code and return structured diagnostics"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LeptosDiagnosticsTool {
    /// Leptos code to analyze.
    #[json_schema(max_length = 262144)]
    pub code: String,
}

#[mcp_tool(
    name = "search-docs",
    description = "Search Leptos, leptos_axum, and Axum documentation sections by task, API, or failure mode"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchDocsTool {
    /// Task, API, error, or workflow to search for.
    pub query: String,
}

#[mcp_tool(
    name = "lookup-api",
    description = "Look up a curated Leptos, leptos_axum, or Axum public API symbol"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LookupApiTool {
    /// Symbol name or declared alias.
    pub query: String,
    /// Optional crate filter: leptos, leptos_axum, or axum.
    #[serde(rename = "crate")]
    pub crate_name: Option<String>,
}

#[mcp_tool(
    name = "leptos-axum-recipe",
    description = "Return a task-oriented recipe for common Leptos + Axum workflows"
)]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LeptosAxumRecipeTool {
    /// Recipe id or alias such as ssr-app, server-functions, static-assets,
    /// custom-handler, state-context, database-query-patterns, or wasm-runtime.
    pub recipe: String,
}

tool_box!(
    LeptosSdkTools,
    [
        ListSectionsTool,
        GetDocumentationTool,
        LeptosDiagnosticsTool,
        SearchDocsTool,
        LookupApiTool,
        LeptosAxumRecipeTool,
    ]
);

#[mcp_resource_template(
    name = "leptos-doc-section",
    description = "Leptos documentation section by canonical section id",
    title = "Leptos Documentation Section",
    mime_type = "text/markdown",
    uri_template = "leptos://docs/{section}"
)]
pub struct LeptosDocsResourceTemplate {}

pub struct LeptosSdkHandler {
    app: LeptosApp,
}

impl LeptosSdkHandler {
    pub fn new(app: LeptosApp) -> Self {
        Self { app }
    }

    pub fn app(&self) -> &LeptosApp {
        &self.app
    }

    pub fn list_tools_result(&self) -> ListToolsResult {
        ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: LeptosSdkTools::tools(),
        }
    }

    pub fn call_tool_result(
        &self,
        params: CallToolRequestParams,
    ) -> Result<CallToolResult, CallToolError> {
        let tool_name = params.name.clone();
        let arguments = params.arguments.unwrap_or_default();

        let result = match tool_name.as_str() {
            "list-sections" => self.app.call_tool(ToolCall::ListSections),
            "get-documentation" => {
                let tool: GetDocumentationTool = match parse_arguments(&tool_name, arguments) {
                    Ok(tool) => tool,
                    Err(result) => return Ok(result),
                };
                self.app.call_tool(ToolCall::GetDocumentation {
                    section: &tool.section,
                })
            }
            "leptos-diagnostics" => {
                let tool: LeptosDiagnosticsTool = match parse_arguments(&tool_name, arguments) {
                    Ok(tool) => tool,
                    Err(result) => return Ok(result),
                };
                self.app
                    .call_tool(ToolCall::DiagnoseLeptosCode { code: &tool.code })
            }
            "search-docs" => {
                let tool: SearchDocsTool = match parse_arguments(&tool_name, arguments) {
                    Ok(tool) => tool,
                    Err(result) => return Ok(result),
                };
                self.app
                    .call_tool(ToolCall::SearchDocs { query: &tool.query })
            }
            "lookup-api" => {
                let tool: LookupApiTool = match parse_arguments(&tool_name, arguments) {
                    Ok(tool) => tool,
                    Err(result) => return Ok(result),
                };
                self.app.call_tool(ToolCall::LookupApi {
                    query: &tool.query,
                    crate_name: tool.crate_name.as_deref(),
                })
            }
            "leptos-axum-recipe" => {
                let tool: LeptosAxumRecipeTool = match parse_arguments(&tool_name, arguments) {
                    Ok(tool) => tool,
                    Err(result) => return Ok(result),
                };
                self.app.call_tool(ToolCall::LeptosAxumRecipe {
                    recipe: &tool.recipe,
                })
            }
            _ => return Ok(tool_error_result(CallToolError::unknown_tool(tool_name))),
        };

        Ok(match result {
            Ok(output) => tool_output_to_call_result(output)?,
            Err(error) => tool_domain_error_result(error),
        })
    }

    pub fn list_resources_result(&self) -> ListResourcesResult {
        ListResourcesResult {
            meta: None,
            next_cursor: None,
            resources: self
                .app
                .list_resources()
                .resources
                .into_iter()
                .map(resource_descriptor_to_sdk_resource)
                .collect(),
        }
    }

    pub fn list_resource_templates_result(&self) -> ListResourceTemplatesResult {
        ListResourceTemplatesResult {
            meta: None,
            next_cursor: None,
            resource_templates: vec![LeptosDocsResourceTemplate::resource_template()],
        }
    }

    pub fn read_resource_result(
        &self,
        params: ReadResourceRequestParams,
    ) -> Result<ReadResourceResult, RpcError> {
        let content = self
            .app
            .read_resource(&params.uri)
            .map_err(resource_read_error)?;

        Ok(ReadResourceResult {
            contents: vec![
                TextResourceContents::new(content.text, content.uri)
                    .with_mime_type(content.mime_type)
                    .into(),
            ],
            meta: None,
        })
    }

    pub fn list_prompts_result(&self) -> ListPromptsResult {
        ListPromptsResult {
            meta: None,
            next_cursor: None,
            prompts: self
                .app
                .list_prompts()
                .prompts
                .into_iter()
                .map(prompt_descriptor_to_sdk_prompt)
                .collect(),
        }
    }

    pub fn get_prompt_result(
        &self,
        params: GetPromptRequestParams,
    ) -> Result<GetPromptResult, RpcError> {
        let arguments = params.arguments.unwrap_or_default();
        let prompt = self
            .app
            .get_prompt(&params.name, &arguments)
            .map_err(prompt_app_error)?;

        Ok(GetPromptResult {
            description: Some(prompt.description.to_string()),
            messages: prompt
                .messages
                .into_iter()
                .map(prompt_message_to_sdk_message)
                .collect::<Result<Vec<_>, _>>()?,
            meta: None,
        })
    }
}

impl Default for LeptosSdkHandler {
    fn default() -> Self {
        Self::new(LeptosApp::new())
    }
}

impl ServerHandler for LeptosSdkHandler {
    fn handle_list_resources_request<'life0, 'async_trait>(
        &'life0 self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListResourcesResult, RpcError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { Ok(self.list_resources_result()) })
    }

    fn handle_list_resource_templates_request<'life0, 'async_trait>(
        &'life0 self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<ListResourceTemplatesResult, RpcError>>
                + Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { Ok(self.list_resource_templates_result()) })
    }

    fn handle_read_resource_request<'life0, 'async_trait>(
        &'life0 self,
        params: ReadResourceRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, RpcError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { self.read_resource_result(params) })
    }

    fn handle_list_tools_request<'life0, 'async_trait>(
        &'life0 self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListToolsResult, RpcError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { Ok(self.list_tools_result()) })
    }

    fn handle_call_tool_request<'life0, 'async_trait>(
        &'life0 self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, CallToolError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { self.call_tool_result(params) })
    }

    fn handle_list_prompts_request<'life0, 'async_trait>(
        &'life0 self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<ListPromptsResult, RpcError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { Ok(self.list_prompts_result()) })
    }

    fn handle_get_prompt_request<'life0, 'async_trait>(
        &'life0 self,
        params: GetPromptRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, RpcError>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { self.get_prompt_result(params) })
    }
}

fn resource_descriptor_to_sdk_resource(resource: crate::app::ResourceDescriptor) -> Resource {
    Resource {
        annotations: None,
        description: Some(resource.description.to_string()),
        icons: Vec::new(),
        meta: None,
        mime_type: Some(resource.mime_type.to_string()),
        name: resource.name.to_string(),
        size: None,
        title: Some(resource.name.to_string()),
        uri: resource.uri,
    }
}

fn resource_read_error(error: ToolError) -> RpcError {
    RpcError::invalid_params().with_message(error.message())
}

fn prompt_descriptor_to_sdk_prompt(prompt: crate::app::PromptDescriptor) -> SdkPrompt {
    SdkPrompt {
        arguments: prompt
            .arguments
            .iter()
            .map(|argument| SdkPromptArgument {
                description: Some(argument.description.to_string()),
                name: argument.name.to_string(),
                required: Some(argument.required),
                title: None,
            })
            .collect(),
        description: Some(prompt.description.to_string()),
        icons: Vec::new(),
        meta: None,
        name: prompt.name.to_string(),
        title: None,
    }
}

fn prompt_message_to_sdk_message(
    message: crate::app::PromptMessage,
) -> Result<SdkPromptMessage, RpcError> {
    let role = match message.role {
        "user" => Role::User,
        "assistant" => Role::Assistant,
        other => {
            return Err(RpcError::internal_error()
                .with_message(format!("Unsupported prompt message role: {other}")));
        }
    };

    let content = match message.content_type {
        "text" => ContentBlock::text_content(message.text),
        other => {
            return Err(RpcError::internal_error()
                .with_message(format!("Unsupported prompt message content type: {other}")));
        }
    };

    Ok(SdkPromptMessage { role, content })
}

fn prompt_lookup_error_message(error: crate::prompts::PromptLookupError) -> String {
    match error {
        crate::prompts::PromptLookupError::Empty => "prompt name must be non-empty".to_string(),
        crate::prompts::PromptLookupError::Unknown { name } => format!("Unknown prompt: {name}"),
    }
}

fn prompt_app_error(error: AppError) -> RpcError {
    let message = match error {
        AppError::Tool(error) => error.message(),
        AppError::PromptLookup(error) => prompt_lookup_error_message(error),
        AppError::PromptRender(error) => error.to_string(),
    };

    RpcError::invalid_params().with_message(message)
}

fn parse_arguments<T>(tool_name: &str, arguments: Map<String, Value>) -> Result<T, CallToolResult>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(Value::Object(arguments)).map_err(|error| {
        tool_error_result(CallToolError::invalid_arguments(
            tool_name,
            Some(error.to_string()),
        ))
    })
}

fn tool_domain_error_result(error: ToolError) -> CallToolResult {
    match error {
        ToolError::UnknownTool(name) => tool_error_result(CallToolError::unknown_tool(name)),
        other => tool_error_result(CallToolError::from_message(other.message())),
    }
}

fn tool_error_result(error: CallToolError) -> CallToolResult {
    CallToolResult::with_error(error)
}

fn tool_output_to_call_result(output: ToolOutput) -> Result<CallToolResult, CallToolError> {
    let structured = serde_json::to_value(&output.structured).map_err(|error| {
        CallToolError::from_message(format!("Failed to serialize output: {error}"))
    })?;
    let structured = match structured {
        Value::Object(map) => map,
        _ => {
            return Err(CallToolError::from_message(
                "Structured tool output did not serialize to an object",
            ));
        }
    };

    Ok(
        CallToolResult::from_content(vec![ContentBlock::TextContent(TextContent::new(
            output.text,
            None,
            None,
        ))])
        .with_structured_content(structured),
    )
}

pub fn server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "leptos-mcp-server".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("Leptos MCP Server".into()),
            description: Some("MCP server for Leptos framework documentation and tools".into()),
            icons: Vec::new(),
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            resources: Some(ServerCapabilitiesResources {
                list_changed: None,
                subscribe: None,
            }),
            prompts: Some(ServerCapabilitiesPrompts { list_changed: None }),
            ..Default::default()
        },
        instructions: Some(
            "Use this server for curated Leptos, leptos_axum, and Axum documentation, recipes, API lookups, and diagnostics."
                .into(),
        ),
        meta: None,
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    }
}

pub fn create_handler() -> LeptosSdkHandler {
    LeptosSdkHandler::default()
}

pub async fn start_stdio() -> SdkResult<()> {
    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = create_handler().to_mcp_server_handler();
    let server: Arc<ServerRuntime> = server_runtime::create_server(McpServerOptions {
        server_details: server_details(),
        transport,
        handler,
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });

    server.start().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{
        GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL,
        LIST_SECTIONS_TOOL, LOOKUP_API_TOOL, MAX_DIAGNOSTIC_CODE_BYTES, SEARCH_DOCS_TOOL,
    };
    use crate::{docs, prompts};
    use rust_mcp_sdk::schema::ReadResourceContent;
    use serde_json::json;

    #[test]
    fn sdk_initialize_details_expose_server_metadata_and_capabilities() {
        let details = server_details();

        assert_eq!(details.server_info.name, "leptos-mcp-server");
        assert_eq!(details.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            details.server_info.title.as_deref(),
            Some("Leptos MCP Server")
        );
        assert_eq!(
            details.protocol_version,
            Into::<String>::into(ProtocolVersion::V2025_11_25)
        );
        assert!(details.instructions.is_some());
        assert!(details.capabilities.tools.is_some());
        assert!(details.capabilities.resources.is_some());
        assert!(details.capabilities.prompts.is_some());
    }

    #[test]
    fn sdk_tools_list_exposes_current_names_and_input_schemas() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let tools = handler.list_tools_result().tools;
        let names = tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                LIST_SECTIONS_TOOL,
                GET_DOCUMENTATION_TOOL,
                LEPTOS_DIAGNOSTICS_TOOL,
                SEARCH_DOCS_TOOL,
                LOOKUP_API_TOOL,
                LEPTOS_AXUM_RECIPE_TOOL,
            ]
        );

        let schema_for = |name: &str| {
            &tools
                .iter()
                .find(|tool| tool.name == name)
                .expect("tool should be listed")
                .input_schema
        };

        assert_eq!(
            schema_for(LIST_SECTIONS_TOOL).required,
            Vec::<String>::new()
        );
        assert_eq!(
            schema_for(GET_DOCUMENTATION_TOOL).required,
            vec!["section".to_string()]
        );
        assert_eq!(
            schema_for(LEPTOS_DIAGNOSTICS_TOOL).required,
            vec!["code".to_string()]
        );
        assert_eq!(
            schema_for(SEARCH_DOCS_TOOL).required,
            vec!["query".to_string()]
        );
        assert_eq!(
            schema_for(LOOKUP_API_TOOL).required,
            vec!["query".to_string()]
        );
        assert_eq!(
            schema_for(LEPTOS_AXUM_RECIPE_TOOL).required,
            vec!["recipe".to_string()]
        );

        assert!(
            schema_for(GET_DOCUMENTATION_TOOL)
                .properties
                .as_ref()
                .expect("properties")
                .contains_key("section")
        );
        assert!(
            schema_for(LOOKUP_API_TOOL)
                .properties
                .as_ref()
                .expect("properties")
                .contains_key("crate")
        );
    }

    #[test]
    fn sdk_tool_call_preserves_successful_text_and_structured_output() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let expected = handler
            .app()
            .call_tool(ToolCall::GetDocumentation { section: "signals" })
            .expect("facade call should succeed");

        let result = handler
            .call_tool_result(call_params(
                GET_DOCUMENTATION_TOOL,
                json!({ "section": "signals" }),
            ))
            .expect("SDK call should succeed");

        assert_eq!(text_content(&result), expected.text);
        assert_eq!(
            result.structured_content,
            Some(
                serde_json::to_value(expected.structured)
                    .expect("structured output serializes")
                    .as_object()
                    .expect("structured output is an object")
                    .clone()
            )
        );
        assert_eq!(result.is_error, None);
    }

    #[test]
    fn sdk_tool_call_accepts_missing_arguments_for_list_sections() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let expected = handler
            .app()
            .call_tool(ToolCall::ListSections)
            .expect("facade call should succeed");

        let result = handler
            .call_tool_result(CallToolRequestParams::new(LIST_SECTIONS_TOOL))
            .expect("SDK call should succeed");

        assert_eq!(text_content(&result), expected.text);
        assert!(result.structured_content.is_some());
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_unknown_tool() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .call_tool_result(CallToolRequestParams::new("missing-tool"))
            .expect("unknown tool should be represented as a tool error payload");

        assert_tool_error(&result, "Unknown tool: missing-tool");
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_invalid_arguments() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .call_tool_result(call_params(GET_DOCUMENTATION_TOOL, json!({})))
            .expect("invalid arguments should be represented as a tool error payload");

        assert_tool_error(
            &result,
            "Invalid arguments for tool 'get-documentation': missing field `section`",
        );
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_missing_documentation_section() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .call_tool_result(call_params(
                GET_DOCUMENTATION_TOOL,
                json!({ "section": "not-a-section" }),
            ))
            .expect("missing documentation should be represented as a tool error payload");

        assert_tool_error(&result, "Unknown documentation section: not-a-section");
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_missing_recipe() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .call_tool_result(call_params(
                LEPTOS_AXUM_RECIPE_TOOL,
                json!({ "recipe": "not-a-recipe" }),
            ))
            .expect("missing recipe should be represented as a tool error payload");

        assert_tool_error(&result, "Unknown Leptos Axum recipe: not-a-recipe");
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_missing_api_symbol() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .call_tool_result(call_params(
                LOOKUP_API_TOOL,
                json!({ "query": "not_an_api_symbol" }),
            ))
            .expect("missing API symbol should be represented as a tool error payload");

        assert_tool_error(&result, "Unknown API symbol: not_an_api_symbol");
    }

    #[test]
    fn sdk_tool_call_returns_error_payload_for_oversized_diagnostics() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let oversized_code = "x".repeat(MAX_DIAGNOSTIC_CODE_BYTES + 1);

        let result = handler
            .call_tool_result(call_params(
                LEPTOS_DIAGNOSTICS_TOOL,
                json!({ "code": oversized_code }),
            ))
            .expect("oversized diagnostics should be represented as a tool error payload");

        assert_tool_error(
            &result,
            &format!("code must be at most {MAX_DIAGNOSTIC_CODE_BYTES} bytes"),
        );
    }

    #[test]
    fn sdk_resources_list_exposes_all_concrete_docs_resources() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let result = handler.list_resources_result();
        let sections = docs::list_sections().iter().collect::<Vec<_>>();

        assert_eq!(result.meta, None);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.resources.len(), sections.len());
        for (resource, section) in result.resources.iter().zip(sections) {
            assert_eq!(resource.uri, docs::resource_uri(section));
            assert_eq!(resource.name, section.title);
            assert_eq!(resource.title.as_deref(), Some(section.title));
            assert_eq!(resource.description.as_deref(), Some(section.use_cases));
            assert_eq!(
                resource.mime_type.as_deref(),
                Some(crate::app::MARKDOWN_MIME_TYPE)
            );
            assert!(resource.annotations.is_none());
            assert!(resource.icons.is_empty());
            assert_eq!(resource.meta, None);
            assert_eq!(resource.size, None);
        }
    }

    #[test]
    fn sdk_resource_read_returns_markdown_for_canonical_section_uri() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let result = handler
            .read_resource_result(read_resource_params("leptos://docs/signals"))
            .expect("canonical section resource should read");
        let signals = docs::get_section_by_resource_uri("leptos://docs/signals")
            .expect("signals section should exist");

        assert_eq!(result.meta, None);
        match result.contents.as_slice() {
            [ReadResourceContent::TextResourceContents(text)] => {
                assert_eq!(text.uri, "leptos://docs/signals");
                assert_eq!(
                    text.mime_type.as_deref(),
                    Some(crate::app::MARKDOWN_MIME_TYPE)
                );
                assert_eq!(
                    text.text,
                    format!("# {}\n\n{}", signals.title, signals.content)
                );
            }
            other => panic!("expected one text resource content block, got {other:?}"),
        }
    }

    #[test]
    fn sdk_resource_read_errors_for_unknown_section_uri() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let error = handler
            .read_resource_result(read_resource_params("leptos://docs/not-a-section"))
            .expect_err("unknown section should fail resources/read");

        assert!(
            format!("{error:?}").contains("Unknown documentation section: not-a-section"),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn sdk_resource_template_lists_docs_section_template() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let result = handler.list_resource_templates_result();

        assert_eq!(result.meta, None);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.resource_templates.len(), 1);
        let template = &result.resource_templates[0];
        assert_eq!(template.name, "leptos-doc-section");
        assert_eq!(template.uri_template, "leptos://docs/{section}");
        assert_eq!(
            template.mime_type.as_deref(),
            Some(crate::app::MARKDOWN_MIME_TYPE)
        );
        assert_eq!(
            template.description.as_deref(),
            Some("Leptos documentation section by canonical section id")
        );
    }

    #[test]
    fn sdk_prompts_list_exposes_all_static_prompt_metadata() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let result = handler.list_prompts_result();
        let expected = prompts::all_prompts();

        assert_eq!(result.meta, None);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.prompts.len(), expected.len());
        for (prompt, expected) in result.prompts.iter().zip(expected) {
            assert_eq!(prompt.name, expected.name);
            assert_eq!(prompt.title, None);
            assert_eq!(prompt.description.as_deref(), Some(expected.description));
            assert!(prompt.icons.is_empty());
            assert_eq!(prompt.meta, None);
            assert_eq!(prompt.arguments.len(), expected.arguments.len());
            for (argument, expected) in prompt.arguments.iter().zip(expected.arguments) {
                assert_eq!(argument.name, expected.name);
                assert_eq!(argument.title, None);
                assert_eq!(argument.description.as_deref(), Some(expected.description));
                assert_eq!(argument.required, Some(expected.required));
            }
        }
    }

    #[test]
    fn sdk_prompt_get_renders_user_text_message_and_normalizes_name() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());
        let params = get_prompt_params(
            "review_sql_access",
            json!({ "backend": "SQLite", "code": "SELECT 1" }),
        );

        let result = handler
            .get_prompt_result(params)
            .expect("prompt should render through SDK handler");

        assert_eq!(
            result.description.as_deref(),
            Some("Review sqlx or SeaQuery usage in a Leptos/Axum application.")
        );
        assert_eq!(result.meta, None);
        match result.messages.as_slice() {
            [
                SdkPromptMessage {
                    role: Role::User,
                    content: ContentBlock::TextContent(text),
                },
            ] => {
                assert!(text.text.contains("SQLite"));
                assert!(text.text.contains("SELECT 1"));
                assert!(text.text.contains("bind parameters"));
            }
            other => panic!("expected one user text prompt message, got {other:?}"),
        }
    }

    #[test]
    fn sdk_prompt_get_allows_missing_optional_arguments() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let result = handler
            .get_prompt_result(get_prompt_params(
                "debug-hydration",
                json!({ "symptom": "WASM 404" }),
            ))
            .expect("missing optional prompt argument should render");

        match result.messages.as_slice() {
            [
                SdkPromptMessage {
                    content: ContentBlock::TextContent(text),
                    ..
                },
            ] => {
                assert!(text.text.contains("WASM 404"));
                assert!(text.text.contains("Environment: ."));
            }
            other => panic!("expected one text prompt message, got {other:?}"),
        }
    }

    #[test]
    fn sdk_prompt_get_errors_for_missing_required_unknown_argument_and_unknown_prompt() {
        let handler = LeptosSdkHandler::new(LeptosApp::new());

        let missing_required = handler
            .get_prompt_result(get_prompt_params(
                "review-sql-access",
                json!({ "backend": "SQLite" }),
            ))
            .expect_err("missing required prompt argument should fail");
        assert!(
            format!("{missing_required:?}")
                .contains("prompt `review-sql-access` is missing required argument(s): code"),
            "unexpected error: {missing_required:?}"
        );

        let unknown_argument = handler
            .get_prompt_result(get_prompt_params(
                "debug-hydration",
                json!({ "symptom": "WASM 404", "extra": "nope" }),
            ))
            .expect_err("unknown prompt argument should fail");
        assert!(
            format!("{unknown_argument:?}")
                .contains("prompt `debug-hydration` has unknown argument: extra"),
            "unexpected error: {unknown_argument:?}"
        );

        let unknown_prompt = handler
            .get_prompt_result(get_prompt_params("not-a-prompt", json!({})))
            .expect_err("unknown prompt should fail");
        assert!(
            format!("{unknown_prompt:?}").contains("Unknown prompt: not-a-prompt"),
            "unexpected error: {unknown_prompt:?}"
        );
    }

    fn call_params(name: &str, arguments: Value) -> CallToolRequestParams {
        let arguments = arguments
            .as_object()
            .expect("test arguments must be object")
            .clone();
        CallToolRequestParams::new(name).with_arguments(arguments)
    }

    fn read_resource_params(uri: &str) -> ReadResourceRequestParams {
        ReadResourceRequestParams {
            meta: None,
            uri: uri.to_string(),
        }
    }

    fn get_prompt_params(name: &str, arguments: Value) -> GetPromptRequestParams {
        let arguments = arguments
            .as_object()
            .expect("test arguments must be object")
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value
                        .as_str()
                        .expect("test prompt arguments must be strings")
                        .to_string(),
                )
            })
            .collect();

        GetPromptRequestParams {
            arguments: Some(arguments),
            meta: None,
            name: name.to_string(),
        }
    }

    fn text_content(result: &CallToolResult) -> String {
        match result.content.as_slice() {
            [ContentBlock::TextContent(text)] => text.text.clone(),
            other => panic!("expected one text content block, got {other:?}"),
        }
    }

    fn assert_tool_error(result: &CallToolResult, expected_message: &str) {
        assert_eq!(result.is_error, Some(true));
        assert_eq!(text_content(result), expected_message);
        assert_eq!(result.structured_content, None);
    }
}
