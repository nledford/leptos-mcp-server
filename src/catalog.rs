//! Pure MCP capability and catalog response builders.

use crate::docs;
use crate::prompts;
use crate::tools::{
    GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
    LOOKUP_API_TOOL, MAX_DIAGNOSTIC_CODE_BYTES, SEARCH_DOCS_TOOL,
};
use serde_json::{Value, json};

pub fn initialize_result(protocol_version: &str, server_version: &str) -> Value {
    json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "leptos-mcp-server",
            "version": server_version
        }
    })
}

pub fn tools_list_result() -> Value {
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
                            "maxLength": MAX_DIAGNOSTIC_CODE_BYTES
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

pub fn resources_list_result() -> Value {
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

pub fn prompts_list_result() -> Value {
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
