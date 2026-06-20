//! Application service facade for server-facing Leptos MCP capabilities.
//!
//! This module keeps protocol/SDK adapters thin by exposing typed application
//! operations that delegate to the domain modules without constructing JSON-RPC
//! response envelopes.

use crate::docs;
use crate::prompts::{self, PromptArgument, PromptLookupError, PromptRenderError};
use crate::tools::{
    GET_DOCUMENTATION_TOOL, LEPTOS_AXUM_RECIPE_TOOL, LEPTOS_DIAGNOSTICS_TOOL, LIST_SECTIONS_TOOL,
    LOOKUP_API_TOOL, LeptosTools, SEARCH_DOCS_TOOL, ToolError, ToolOutput,
};
use std::collections::BTreeMap;

pub const MARKDOWN_MIME_TYPE: &str = "text/markdown";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDescriptor {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolList {
    pub tools: Vec<ToolDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCall<'a> {
    ListSections,
    GetDocumentation {
        section: &'a str,
    },
    DiagnoseLeptosCode {
        code: &'a str,
    },
    SearchDocs {
        query: &'a str,
    },
    LookupApi {
        query: &'a str,
        crate_name: Option<&'a str>,
    },
    LeptosAxumRecipe {
        recipe: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub uri: String,
    pub name: &'static str,
    pub description: &'static str,
    pub mime_type: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceList {
    pub resources: Vec<ResourceDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: &'static str,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDescriptor {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: &'static [PromptArgument],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptList {
    pub prompts: Vec<PromptDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptMessage {
    pub role: &'static str,
    pub content_type: &'static str,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptOutput {
    pub description: &'static str,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    Tool(ToolError),
    PromptLookup(PromptLookupError),
    PromptRender(PromptRenderError),
}

pub struct LeptosApp {
    tools: LeptosTools,
}

impl LeptosApp {
    pub fn new() -> Self {
        Self {
            tools: LeptosTools::new(),
        }
    }

    pub fn list_tools(&self) -> ToolList {
        ToolList {
            tools: vec![
                ToolDescriptor {
                    name: LIST_SECTIONS_TOOL,
                    description: "List all available Leptos documentation sections with canonical ids, aliases, and version metadata",
                },
                ToolDescriptor {
                    name: GET_DOCUMENTATION_TOOL,
                    description: "Get Leptos documentation for a canonical section id or declared alias",
                },
                ToolDescriptor {
                    name: LEPTOS_DIAGNOSTICS_TOOL,
                    description: "Analyze Leptos code and return structured diagnostics",
                },
                ToolDescriptor {
                    name: SEARCH_DOCS_TOOL,
                    description: "Search Leptos, leptos_axum, and Axum documentation sections by task, API, or failure mode",
                },
                ToolDescriptor {
                    name: LOOKUP_API_TOOL,
                    description: "Look up curated Leptos, leptos_axum, or Axum API symbols, macros, aliases, and concepts",
                },
                ToolDescriptor {
                    name: LEPTOS_AXUM_RECIPE_TOOL,
                    description: "Return a task-oriented recipe for common Leptos + Axum workflows",
                },
            ],
        }
    }

    pub fn call_tool(&self, call: ToolCall<'_>) -> Result<ToolOutput, ToolError> {
        match call {
            ToolCall::ListSections => Ok(self.tools.list_sections()),
            ToolCall::GetDocumentation { section } => self.tools.get_documentation(section),
            ToolCall::DiagnoseLeptosCode { code } => self.tools.diagnose_leptos_code(code),
            ToolCall::SearchDocs { query } => self.tools.search_docs(query),
            ToolCall::LookupApi { query, crate_name } => self.tools.lookup_api(query, crate_name),
            ToolCall::LeptosAxumRecipe { recipe } => self.tools.leptos_axum_recipe(recipe),
        }
    }

    pub fn list_resources(&self) -> ResourceList {
        ResourceList {
            resources: docs::list_sections()
                .iter()
                .map(|section| ResourceDescriptor {
                    uri: docs::resource_uri(section),
                    name: section.title,
                    description: section.use_cases,
                    mime_type: MARKDOWN_MIME_TYPE,
                })
                .collect(),
        }
    }

    pub fn read_resource(&self, uri: &str) -> Result<ResourceContent, ToolError> {
        let catalog_section = docs::get_catalog_section_by_resource_uri(uri)
            .map_err(ToolError::DocumentationLookup)?;
        let section = catalog_section.section;

        Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: MARKDOWN_MIME_TYPE,
            text: format!("# {}\n\n{}", section.title, section.content),
        })
    }

    pub fn list_prompts(&self) -> PromptList {
        PromptList {
            prompts: prompts::all_prompts()
                .iter()
                .map(|prompt| PromptDescriptor {
                    name: prompt.name,
                    description: prompt.description,
                    arguments: prompt.arguments,
                })
                .collect(),
        }
    }

    pub fn get_prompt(
        &self,
        name: &str,
        arguments: &BTreeMap<String, String>,
    ) -> Result<PromptOutput, AppError> {
        let prompt = prompts::get_prompt(name).map_err(AppError::PromptLookup)?;
        let text =
            prompts::render_prompt_checked(prompt, arguments).map_err(AppError::PromptRender)?;

        Ok(PromptOutput {
            description: prompt.description,
            messages: vec![PromptMessage {
                role: "user",
                content_type: "text",
                text,
            }],
        })
    }
}

impl Default for LeptosApp {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ToolError> for AppError {
    fn from(error: ToolError) -> Self {
        AppError::Tool(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::StructuredToolOutput;

    #[test]
    fn facade_lists_current_tool_capabilities() {
        let app = LeptosApp::new();
        let names = app
            .list_tools()
            .tools
            .iter()
            .map(|tool| tool.name)
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
    }

    #[test]
    fn facade_tool_calls_match_leptos_tools_outputs() {
        let app = LeptosApp::new();
        let tools = LeptosTools::new();

        assert_eq!(
            app.call_tool(ToolCall::ListSections)
                .expect("list sections"),
            tools.list_sections()
        );
        assert_eq!(
            app.call_tool(ToolCall::GetDocumentation { section: "signal" })
                .expect("get documentation"),
            tools
                .get_documentation("signal")
                .expect("direct documentation")
        );
        assert_eq!(
            app.call_tool(ToolCall::SearchDocs {
                query: "Axum state"
            })
            .expect("search docs"),
            tools.search_docs("Axum state").expect("direct search")
        );
        assert_eq!(
            app.call_tool(ToolCall::LookupApi {
                query: "file_and_error_handler",
                crate_name: Some("leptos_axum"),
            })
            .expect("lookup api"),
            tools
                .lookup_api("file_and_error_handler", Some("leptos_axum"))
                .expect("direct lookup")
        );
        assert_eq!(
            app.call_tool(ToolCall::LeptosAxumRecipe { recipe: "state" })
                .expect("recipe"),
            tools.leptos_axum_recipe("state").expect("direct recipe")
        );

        let diagnostics = app
            .call_tool(ToolCall::DiagnoseLeptosCode {
                code: "fn App() -> impl IntoView { view! { <p/> } }",
            })
            .expect("diagnostics");
        assert!(matches!(
            diagnostics.structured,
            StructuredToolOutput::Diagnostics(_)
        ));
    }

    #[test]
    fn facade_resources_match_docs_module() {
        let app = LeptosApp::new();
        let resources = app.list_resources().resources;
        let sections = docs::list_sections().iter().collect::<Vec<_>>();

        assert_eq!(resources.len(), sections.len());
        for (resource, section) in resources.iter().zip(sections) {
            assert_eq!(resource.uri, docs::resource_uri(section));
            assert_eq!(resource.name, section.title);
            assert_eq!(resource.description, section.use_cases);
            assert_eq!(resource.mime_type, MARKDOWN_MIME_TYPE);
        }

        let content = app
            .read_resource("leptos://docs/axum")
            .expect("axum resource should resolve");
        let axum = docs::get_section_by_resource_uri("leptos://docs/axum")
            .expect("direct axum resource should resolve");
        assert_eq!(content.uri, "leptos://docs/axum");
        assert_eq!(content.mime_type, MARKDOWN_MIME_TYPE);
        assert_eq!(
            content.text,
            format!("# {}\n\n{}", axum.title, axum.content)
        );
    }

    #[test]
    fn facade_prompts_match_prompts_module() {
        let app = LeptosApp::new();
        let listed = app.list_prompts().prompts;
        let direct = prompts::all_prompts();

        assert_eq!(listed.len(), direct.len());
        for (listed, direct) in listed.iter().zip(direct) {
            assert_eq!(listed.name, direct.name);
            assert_eq!(listed.description, direct.description);
            assert_eq!(listed.arguments, direct.arguments);
        }

        let mut arguments = BTreeMap::new();
        arguments.insert("operation".to_string(), "create a user".to_string());
        let output = app
            .get_prompt("add-server-function", &arguments)
            .expect("prompt should render");
        let prompt = prompts::get_prompt("add-server-function").expect("direct prompt");
        assert_eq!(output.description, prompt.description);
        assert_eq!(output.messages.len(), 1);
        assert_eq!(output.messages[0].role, "user");
        assert_eq!(output.messages[0].content_type, "text");
        assert_eq!(
            output.messages[0].text,
            prompts::render_prompt_checked(prompt, &arguments).expect("direct render")
        );
    }
}
