//! Leptos documentation catalog.

use serde::Serialize;

const LEPTOS_VERSION_SCOPE: &str = "Leptos 0.8+";
const DOCUMENTATION_SOURCE: &str = "embedded project documentation";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DocSection {
    pub id: &'static str,
    pub title: &'static str,
    pub path: &'static str,
    pub use_cases: &'static str,
    pub content: &'static str,
    pub aliases: &'static [&'static str],
    pub leptos_version: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionLookupError {
    Empty,
    Unknown { query: String },
    Ambiguous { query: String, matches: Vec<String> },
}

static SECTIONS: &[DocSection] = &[
    DocSection {
        id: "getting-started",
        title: "Getting Started",
        path: "getting-started",
        use_cases: "new project, setup, installation, basics, hello world",
        content: include_str!("../docs/getting-started.md"),
        aliases: &["start", "setup", "installation"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "components",
        title: "Components",
        path: "components",
        use_cases: "UI, view, component, props, children, #[component], always",
        content: include_str!("../docs/components.md"),
        aliases: &["component", "props", "children"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "signals",
        title: "Signals",
        path: "signals",
        use_cases: "state, reactivity, signals, derived, effects, get, set, read, write, update, always",
        content: include_str!("../docs/signals.md"),
        aliases: &["signal", "reactivity", "state"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "views",
        title: "Views",
        path: "views",
        use_cases: "view macro, dynamic classes, dynamic styles, attributes, class:, style:, events, always",
        content: include_str!("../docs/views.md"),
        aliases: &["view", "view-macro", "classes", "styles"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "resources",
        title: "Resources",
        path: "resources",
        use_cases: "async, data loading, Resource, LocalResource, OnceResource, fetch, API",
        content: include_str!("../docs/resources.md"),
        aliases: &["resource", "async-data", "data-loading"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "actions",
        title: "Actions",
        path: "actions",
        use_cases: "mutations, POST, forms, ActionForm, ServerAction, submit, create, update, delete",
        content: include_str!("../docs/actions.md"),
        aliases: &["action", "mutations", "server-action"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "server-functions",
        title: "Server Functions",
        path: "server-functions",
        use_cases: "backend, API, database, server, SSR, #[server], extractors, Axum",
        content: include_str!("../docs/server-functions.md"),
        aliases: &["server", "server-fn", "server-functions"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "routing",
        title: "Routing",
        path: "routing",
        use_cases: "navigation, pages, routes, params, nested routes, Router",
        content: include_str!("../docs/routing.md"),
        aliases: &["routes", "router", "navigation"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "forms",
        title: "Forms",
        path: "forms",
        use_cases: "form, input, validation, submit, controlled input, prop:value",
        content: include_str!("../docs/forms.md"),
        aliases: &["form", "inputs", "validation"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "error-handling",
        title: "Error Handling",
        path: "error-handling",
        use_cases: "errors, ErrorBoundary, Result, ServerFnError, try",
        content: include_str!("../docs/error-handling.md"),
        aliases: &["errors", "error-boundary", "server-fn-error"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
    DocSection {
        id: "suspense",
        title: "Suspense",
        path: "suspense",
        use_cases: "loading, async, Suspense, Transition, streaming, fallback",
        content: include_str!("../docs/suspense.md"),
        aliases: &["transition", "loading", "streaming"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
    },
];

pub fn list_sections() -> &'static [DocSection] {
    SECTIONS
}

pub fn get_section(query: &str) -> Result<&'static DocSection, SectionLookupError> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return Err(SectionLookupError::Empty);
    }

    let matches: Vec<&DocSection> = SECTIONS
        .iter()
        .filter(|section| section.matches(&normalized))
        .collect();

    match matches.as_slice() {
        [section] => Ok(*section),
        [] => Err(SectionLookupError::Unknown {
            query: query.to_string(),
        }),
        multiple => Err(SectionLookupError::Ambiguous {
            query: query.to_string(),
            matches: multiple
                .iter()
                .map(|section| section.id.to_string())
                .collect(),
        }),
    }
}

impl DocSection {
    fn matches(&self, normalized_query: &str) -> bool {
        normalize(self.id) == normalized_query
            || normalize(self.path) == normalized_query
            || normalize(self.title) == normalized_query
            || self
                .aliases
                .iter()
                .any(|alias| normalize(alias) == normalized_query)
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn resolves_canonical_section_id() {
        let section = get_section("signals").expect("signals should resolve");

        assert_eq!(section.id, "signals");
    }

    #[test]
    fn resolves_declared_alias() {
        let section = get_section("server-fn").expect("server-fn alias should resolve");

        assert_eq!(section.id, "server-functions");
    }

    #[test]
    fn rejects_empty_section_query() {
        let error = get_section("   ").expect_err("empty query must be invalid");

        assert_eq!(error, SectionLookupError::Empty);
    }

    #[test]
    fn rejects_partial_ambiguous_or_unknown_queries() {
        let error = get_section("s").expect_err("partial substring lookup is not allowed");

        assert_eq!(
            error,
            SectionLookupError::Unknown {
                query: "s".to_string()
            }
        );
    }

    #[test]
    fn section_ids_are_unique() {
        let mut ids = HashSet::new();

        for section in list_sections() {
            assert!(
                ids.insert(section.id),
                "duplicate section id {}",
                section.id
            );
        }
    }
}
