//! Pure MCP capability and catalog response builders.

use crate::app::LeptosApp;
use crate::tools::MAX_DIAGNOSTIC_CODE_BYTES;
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

pub fn tools_list_result(app: &LeptosApp) -> Value {
    let tools = app.list_tools();

    json!({
        "tools": [
            {
                "name": tools.tools[0].name,
                "description": tools.tools[0].description,
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            },
            {
                "name": tools.tools[1].name,
                "description": tools.tools[1].description,
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
                "name": tools.tools[2].name,
                "description": tools.tools[2].description,
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
                "name": tools.tools[3].name,
                "description": tools.tools[3].description,
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
                "name": tools.tools[4].name,
                "description": tools.tools[4].description,
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
                "name": tools.tools[5].name,
                "description": tools.tools[5].description,
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

pub fn resources_list_result(app: &LeptosApp) -> Value {
    let resources = app
        .list_resources()
        .resources
        .into_iter()
        .map(|resource| {
            json!({
                "uri": resource.uri,
                "name": resource.name,
                "description": resource.description,
                "mimeType": resource.mime_type
            })
        })
        .collect::<Vec<_>>();

    json!({ "resources": resources })
}

pub fn prompts_list_result(app: &LeptosApp) -> Value {
    let prompts = app
        .list_prompts()
        .prompts
        .into_iter()
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
