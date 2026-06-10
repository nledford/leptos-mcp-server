//! MCP prompt catalog for common Leptos workflows.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PromptArgument {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PromptTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: &'static [PromptArgument],
    pub related_tools: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub template: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptLookupError {
    Empty,
    Unknown { name: String },
}

static PROMPTS: &[PromptTemplate] = &[
    PromptTemplate {
        name: "wire-leptos-axum-ssr",
        description: "Plan or review an Axum 0.8.9 backed Leptos SSR application.",
        arguments: &[
            PromptArgument {
                name: "app_name",
                description: "Name of the application or crate being wired.",
                required: false,
            },
            PromptArgument {
                name: "state",
                description: "Shared server state such as database pools or config.",
                required: false,
            },
        ],
        related_tools: &["leptos-axum-recipe", "lookup-api", "search-docs"],
        related_sections: &["leptos-axum", "ssr-hydration-deployment", "axum"],
        template: "Given a Leptos app named {app_name}, design the Axum 0.8.9 SSR wiring. Include route-list generation, LeptosRoutes integration, server function handling, static file/error handling, state/context flow, and validation steps. Shared state: {state}.",
    },
    PromptTemplate {
        name: "add-server-function",
        description: "Add a Leptos server function and caller with correct public API boundaries.",
        arguments: &[
            PromptArgument {
                name: "operation",
                description: "Read or mutation behavior the server function should perform.",
                required: true,
            },
            PromptArgument {
                name: "data",
                description: "DTOs, parameters, or validation rules involved.",
                required: false,
            },
        ],
        related_tools: &["lookup-api", "leptos-diagnostics", "search-docs"],
        related_sections: &["server-functions", "actions", "forms"],
        template: "Add a Leptos #[server] function for {operation}. Keep it async, return Result<T, ServerFnError>, define serializable DTOs, avoid leaking server-only data, and choose Resource for reads or ServerAction/ActionForm for mutations. Data context: {data}.",
    },
    PromptTemplate {
        name: "debug-hydration",
        description: "Diagnose SSR/hydration or static asset failures in a Leptos app.",
        arguments: &[
            PromptArgument {
                name: "symptom",
                description: "Observed browser/server symptom or error message.",
                required: true,
            },
            PromptArgument {
                name: "environment",
                description: "Runtime, deployment target, or feature flags.",
                required: false,
            },
        ],
        related_tools: &["search-docs", "lookup-api", "leptos-axum-recipe"],
        related_sections: &["ssr-hydration-deployment", "leptos-axum", "resources"],
        template: "Debug this Leptos hydration/static asset issue: {symptom}. Check ssr/hydrate feature flags, generated /pkg assets, Axum fallback ordering, Resource serialization expectations, and deployment paths. Environment: {environment}.",
    },
    PromptTemplate {
        name: "review-axum-integration",
        description: "Review Axum 0.8.9 routing, state, extractors, middleware, and response handling in a Leptos server.",
        arguments: &[
            PromptArgument {
                name: "code",
                description: "Axum or Leptos server code to review.",
                required: true,
            },
        ],
        related_tools: &["lookup-api", "leptos-diagnostics", "search-docs"],
        related_sections: &["axum", "leptos-axum", "server-functions"],
        template: "Review this Axum 0.8.9 + Leptos integration code for routing, State/FromRef, middleware placement, IntoResponse errors, server function routes, extractor use, and SSR/static fallback ordering:\n\n{code}",
    },
];

pub fn all_prompts() -> &'static [PromptTemplate] {
    PROMPTS
}

pub fn get_prompt(name: &str) -> Result<&'static PromptTemplate, PromptLookupError> {
    let normalized = normalize(name);
    if normalized.is_empty() {
        return Err(PromptLookupError::Empty);
    }

    PROMPTS
        .iter()
        .find(|prompt| normalize(prompt.name) == normalized)
        .ok_or_else(|| PromptLookupError::Unknown {
            name: name.to_string(),
        })
}

pub fn render_prompt(template: &PromptTemplate, arguments: &BTreeMap<String, String>) -> String {
    let mut rendered = template.template.to_string();

    for argument in template.arguments {
        let value = arguments
            .get(argument.name)
            .map(String::as_str)
            .unwrap_or("");
        rendered = rendered.replace(&format!("{{{}}}", argument.name), value);
    }

    rendered
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '_'], "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_prompt_with_arguments() {
        let prompt = get_prompt("debug-hydration").expect("prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("symptom".to_string(), "WASM 404".to_string());

        let rendered = render_prompt(prompt, &arguments);

        assert!(rendered.contains("WASM 404"));
        assert!(rendered.contains("feature flags"));
    }

    #[test]
    fn every_prompt_points_to_related_tools_and_sections() {
        for prompt in all_prompts() {
            assert!(!prompt.related_tools.is_empty());
            assert!(!prompt.related_sections.is_empty());
        }
    }
}
