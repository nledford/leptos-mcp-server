//! MCP prompt catalog for common Leptos workflows.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptRenderError {
    MissingRequiredArguments {
        prompt_name: &'static str,
        names: Vec<&'static str>,
    },
    BlankRequiredArgument {
        prompt_name: &'static str,
        name: &'static str,
    },
    UnknownArgument {
        prompt_name: &'static str,
        name: String,
    },
}

impl fmt::Display for PromptRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptRenderError::MissingRequiredArguments { prompt_name, names } => {
                write!(
                    f,
                    "prompt `{prompt_name}` is missing required argument(s): {}",
                    names.join(", ")
                )
            }
            PromptRenderError::BlankRequiredArgument { prompt_name, name } => write!(
                f,
                "prompt `{prompt_name}` has blank required argument: {name}"
            ),
            PromptRenderError::UnknownArgument { prompt_name, .. } => {
                write!(f, "prompt `{prompt_name}` has unknown argument")
            }
        }
    }
}

impl std::error::Error for PromptRenderError {}

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
        name: "review-sql-access",
        description: "Review sqlx or SeaQuery usage in a Leptos/Axum application.",
        arguments: &[
            PromptArgument {
                name: "code",
                description: "Leptos server function, Axum handler, repository, or query-builder code to review.",
                required: true,
            },
            PromptArgument {
                name: "backend",
                description: "Database backend if known, such as PostgreSQL, SQLite, or MySQL.",
                required: false,
            },
        ],
        related_tools: &["search-docs", "leptos-axum-recipe", "leptos-diagnostics"],
        related_sections: &[
            "server-functions",
            "actions",
            "leptos-axum",
            "axum",
            "error-handling",
        ],
        template: "Review this Leptos/Axum database access code for safe and maintainable SQL usage. Check sqlx pool ownership through Axum State and Leptos context, fixed SQL with query! or query_as! where practical, SeaQuery only for genuinely dynamic query shapes, bind parameters or SeaQuery values for all user input, transaction boundaries for multi-step writes, user-safe error mapping, DTO boundaries, async/Send behavior, and database-backed tests. Database backend: {backend}.\n\nCode:\n{code}",
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
        arguments: &[PromptArgument {
            name: "code",
            description: "Axum or Leptos server code to review.",
            required: true,
        }],
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

pub fn validate_prompt_arguments(
    template: &PromptTemplate,
    arguments: &BTreeMap<String, String>,
) -> Result<(), PromptRenderError> {
    for name in arguments.keys() {
        if !template
            .arguments
            .iter()
            .any(|argument| argument.name == name)
        {
            return Err(PromptRenderError::UnknownArgument {
                prompt_name: template.name,
                name: name.clone(),
            });
        }
    }

    let missing_names: Vec<_> = template
        .arguments
        .iter()
        .filter(|argument| argument.required && !arguments.contains_key(argument.name))
        .map(|argument| argument.name)
        .collect();
    if !missing_names.is_empty() {
        return Err(PromptRenderError::MissingRequiredArguments {
            prompt_name: template.name,
            names: missing_names,
        });
    }

    for argument in template.arguments {
        if !argument.required {
            continue;
        }

        let value = arguments
            .get(argument.name)
            .expect("missing required arguments were checked first");

        if value.trim().is_empty() {
            return Err(PromptRenderError::BlankRequiredArgument {
                prompt_name: template.name,
                name: argument.name,
            });
        }
    }

    Ok(())
}

pub fn render_prompt_checked(
    template: &PromptTemplate,
    arguments: &BTreeMap<String, String>,
) -> Result<String, PromptRenderError> {
    validate_prompt_arguments(template, arguments)?;
    Ok(render_prompt(template, arguments))
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
        assert!(rendered.contains("Environment: ."));
    }

    #[test]
    fn renders_sql_access_review_prompt() {
        let prompt = get_prompt("review_sql_access").expect("SQL review prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("backend".to_string(), "SQLite".to_string());
        arguments.insert("code".to_string(), "sqlx::query!(\"SELECT 1\")".to_string());

        let rendered = render_prompt(prompt, &arguments);

        assert!(rendered.contains("SQLite"));
        assert!(rendered.contains("bind parameters"));
        assert!(rendered.contains("sqlx::query!"));
    }

    #[test]
    fn renders_prompt_when_required_argument_is_present() {
        let prompt = get_prompt("add-server-function").expect("prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("operation".to_string(), "create a user".to_string());

        let rendered =
            render_prompt_checked(prompt, &arguments).expect("arguments should validate");

        assert!(rendered.contains("create a user"));
        assert!(rendered.contains("Data context: ."));
    }

    #[test]
    fn rejects_missing_required_prompt_argument() {
        let prompt = get_prompt("add-server-function").expect("prompt should exist");
        let arguments = BTreeMap::new();

        let error =
            validate_prompt_arguments(prompt, &arguments).expect_err("missing required arg");

        assert_eq!(
            error,
            PromptRenderError::MissingRequiredArguments {
                prompt_name: "add-server-function",
                names: vec!["operation"]
            }
        );
        assert_eq!(
            error.to_string(),
            "prompt `add-server-function` is missing required argument(s): operation"
        );
    }

    #[test]
    fn rejects_blank_required_prompt_argument() {
        let prompt = get_prompt("add-server-function").expect("prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("operation".to_string(), " \t\n ".to_string());

        let error = validate_prompt_arguments(prompt, &arguments).expect_err("blank required arg");

        assert_eq!(
            error,
            PromptRenderError::BlankRequiredArgument {
                prompt_name: "add-server-function",
                name: "operation"
            }
        );
        assert_eq!(
            error.to_string(),
            "prompt `add-server-function` has blank required argument: operation"
        );
    }

    #[test]
    fn renders_prompt_when_optional_argument_is_missing() {
        let prompt = get_prompt("review-sql-access").expect("prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("code".to_string(), "sqlx::query!(\"SELECT 1\")".to_string());

        let rendered =
            render_prompt_checked(prompt, &arguments).expect("arguments should validate");

        assert!(rendered.contains("Database backend: ."));
        assert!(rendered.contains("sqlx::query!"));
    }

    #[test]
    fn rejects_unknown_extra_prompt_argument() {
        let prompt = get_prompt("debug-hydration").expect("prompt should exist");
        let mut arguments = BTreeMap::new();
        arguments.insert("symptom".to_string(), "WASM 404".to_string());
        arguments.insert("extra".to_string(), "ignored?".to_string());

        let error = validate_prompt_arguments(prompt, &arguments).expect_err("unknown extra arg");

        assert_eq!(
            error,
            PromptRenderError::UnknownArgument {
                prompt_name: "debug-hydration",
                name: "extra".to_string()
            }
        );
        assert_eq!(
            error.to_string(),
            "prompt `debug-hydration` has unknown argument"
        );
    }

    #[test]
    fn every_prompt_points_to_related_tools_and_sections() {
        for prompt in all_prompts() {
            assert!(!prompt.related_tools.is_empty());
            assert!(!prompt.related_sections.is_empty());
        }
    }
}
