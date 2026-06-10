//! Leptos documentation catalog.

use crate::api::{AXUM_VERSION, LEPTOS_AXUM_VERSION, LEPTOS_VERSION};
use serde::Serialize;

const LEPTOS_VERSION_SCOPE: &str = "Leptos 0.8+";
const DOCUMENTATION_SOURCE: &str = "embedded project documentation";
const REVIEWED_AT: &str = "2026-06-10";
pub const DOC_RESOURCE_PREFIX: &str = "leptos://docs/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CrateVersion {
    pub name: &'static str,
    pub version: &'static str,
    pub docs_url: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnippetClassification {
    CompileCandidate,
    Illustrative,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SectionMetadata {
    pub id: &'static str,
    pub crate_versions: &'static [CrateVersion],
    pub source_url: &'static str,
    pub task_tags: &'static [&'static str],
    pub crate_apis: &'static [&'static str],
    pub prerequisites: &'static [&'static str],
    pub common_errors: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub snippet_classification: SnippetClassification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionSearchMatch {
    pub section: &'static DocSection,
    pub metadata: &'static SectionMetadata,
    pub score: usize,
    pub matched_fields: Vec<&'static str>,
    pub why: String,
    pub next_actions: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustCodeBlock {
    pub section_id: &'static str,
    pub classification: SnippetClassification,
    pub content: String,
}

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
    pub source_path: &'static str,
    pub reviewed_at: &'static str,
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
        source_path: "docs/getting-started.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/components.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/signals.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/views.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/resources.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "actions",
        title: "Actions",
        path: "actions",
        use_cases: "mutations, POST, forms, ActionForm, ServerAction, submit, create, update, delete, sqlx, transactions",
        content: include_str!("../docs/actions.md"),
        aliases: &["action", "mutations", "server-action", "transaction"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/actions.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "server-functions",
        title: "Server Functions",
        path: "server-functions",
        use_cases: "backend, API, database, SQL, sqlx, SeaQuery, server, SSR, #[server], extractors, Axum",
        content: include_str!("../docs/server-functions.md"),
        aliases: &[
            "server",
            "server-fn",
            "server-functions",
            "sqlx",
            "sea-query",
            "SeaQuery",
        ],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/server-functions.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/routing.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/forms.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "error-handling",
        title: "Error Handling",
        path: "error-handling",
        use_cases: "errors, ErrorBoundary, Result, ServerFnError, try, sqlx, database errors",
        content: include_str!("../docs/error-handling.md"),
        aliases: &["errors", "error-boundary", "server-fn-error"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/error-handling.md",
        reviewed_at: REVIEWED_AT,
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
        source_path: "docs/suspense.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "leptos-axum",
        title: "Leptos Axum Integration",
        path: "leptos-axum",
        use_cases: "Axum integration, SSR routes, server function handler, extractors, ResponseOptions, database pool, sqlx",
        content: include_str!("../docs/leptos-axum.md"),
        aliases: &[
            "axum-integration",
            "leptos_axum",
            "leptos axum",
            "database-pool",
        ],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/leptos-axum.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "axum",
        title: "Axum 0.8.9 for Leptos Servers",
        path: "axum",
        use_cases: "Axum 0.8.9, Router, extractors, State, middleware, IntoResponse, testing, database pool, sqlx",
        content: include_str!("../docs/axum.md"),
        aliases: &["axum-0.8.9", "axum-router", "axum-state", "db-pool"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/axum.md",
        reviewed_at: REVIEWED_AT,
    },
    DocSection {
        id: "ssr-hydration-deployment",
        title: "SSR, Hydration, and Deployment",
        path: "ssr-hydration-deployment",
        use_cases: "SSR, hydration, cargo-leptos, feature flags, static files, deployment, WASM assets",
        content: include_str!("../docs/ssr-hydration-deployment.md"),
        aliases: &["ssr", "hydrate", "hydration", "deployment", "static-assets"],
        leptos_version: LEPTOS_VERSION_SCOPE,
        source: DOCUMENTATION_SOURCE,
        source_path: "docs/ssr-hydration-deployment.md",
        reviewed_at: REVIEWED_AT,
    },
];

static LEPTOS_CRATE: &[CrateVersion] = &[CrateVersion {
    name: "leptos",
    version: LEPTOS_VERSION,
    docs_url: "https://docs.rs/leptos/latest/leptos/",
}];

static LEPTOS_AXUM_CRATES: &[CrateVersion] = &[
    CrateVersion {
        name: "leptos",
        version: LEPTOS_VERSION,
        docs_url: "https://docs.rs/leptos/latest/leptos/",
    },
    CrateVersion {
        name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        docs_url: "https://docs.rs/leptos_axum/latest/leptos_axum/",
    },
    CrateVersion {
        name: "axum",
        version: AXUM_VERSION,
        docs_url: "https://docs.rs/axum/0.8.9/axum/",
    },
];

static SQL_GUIDANCE_CRATES: &[CrateVersion] = &[
    CrateVersion {
        name: "leptos",
        version: LEPTOS_VERSION,
        docs_url: "https://docs.rs/leptos/latest/leptos/",
    },
    CrateVersion {
        name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        docs_url: "https://docs.rs/leptos_axum/latest/leptos_axum/",
    },
    CrateVersion {
        name: "axum",
        version: AXUM_VERSION,
        docs_url: "https://docs.rs/axum/0.8.9/axum/",
    },
    CrateVersion {
        name: "sqlx",
        version: "latest",
        docs_url: "https://docs.rs/sqlx/latest/sqlx/",
    },
    CrateVersion {
        name: "sea-query",
        version: "latest",
        docs_url: "https://docs.rs/sea-query/latest/sea_query/",
    },
    CrateVersion {
        name: "sea-query-sqlx",
        version: "latest",
        docs_url: "https://docs.rs/sea-query-sqlx/latest/sea_query_sqlx/",
    },
];

static SECTION_METADATA: &[SectionMetadata] = &[
    SectionMetadata {
        id: "getting-started",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/",
        task_tags: &["project-setup", "cargo-leptos", "hello-world"],
        crate_apis: &["leptos::prelude::*", "view!"],
        prerequisites: &[
            "Rust toolchain",
            "cargo-leptos",
            "wasm32-unknown-unknown target",
        ],
        common_errors: &["Missing wasm target", "cargo-leptos not installed"],
        related_sections: &["components", "ssr-hydration-deployment", "leptos-axum"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "components",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/attr.component.html",
        task_tags: &["component-modeling", "props", "children"],
        crate_apis: &["#[component]", "Children", "IntoView"],
        prerequisites: &["leptos::prelude::*"],
        common_errors: &["Component name is not PascalCase", "Missing #[component]"],
        related_sections: &["views", "signals", "forms"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "signals",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/reactive/signal/index.html",
        task_tags: &["reactivity", "state", "derived-values"],
        crate_apis: &["signal", "RwSignal", "Memo", "Effect"],
        prerequisites: &["Leptos reactive ownership basics"],
        common_errors: &[
            "Signal read in view without move closure",
            "Effect writes derived state",
        ],
        related_sections: &["views", "resources", "components"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "views",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/macro.view.html",
        task_tags: &["view-macro", "attributes", "events", "styling"],
        crate_apis: &["view!", "class:", "style:", "on:"],
        prerequisites: &["Components", "signals"],
        common_errors: &["Unescaped inner_html", "Non-reactive attributes"],
        related_sections: &["components", "signals", "forms"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "resources",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/prelude/struct.Resource.html",
        task_tags: &["async-data", "ssr", "suspense", "refetch"],
        crate_apis: &[
            "Resource",
            "Resource::new",
            "Resource::new_blocking",
            "refetch",
        ],
        prerequisites: &["Signals", "async Rust"],
        common_errors: &[
            "Fetcher accidentally tracks signals",
            "Using LocalResource for SSR data",
        ],
        related_sections: &["suspense", "server-functions", "actions"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "actions",
        crate_versions: SQL_GUIDANCE_CRATES,
        source_url: "https://docs.rs/leptos/latest/leptos/prelude/struct.Action.html",
        task_tags: &[
            "mutations",
            "forms",
            "progressive-enhancement",
            "sqlx",
            "transactions",
        ],
        crate_apis: &[
            "Action",
            "ServerAction",
            "ActionForm",
            "sqlx::query!",
            "sqlx::Transaction",
        ],
        prerequisites: &[
            "Server functions for server mutations",
            "Shared database pool context",
        ],
        common_errors: &[
            "Missing input names in ActionForm",
            "Not handling action.value errors",
            "Missing transaction around multi-step database mutation",
        ],
        related_sections: &["forms", "server-functions", "resources"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "server-functions",
        crate_versions: SQL_GUIDANCE_CRATES,
        source_url: "https://docs.rs/leptos/latest/leptos/attr.server.html",
        task_tags: &[
            "server-functions",
            "public-api",
            "dto",
            "extractors",
            "sqlx",
            "sea-query",
            "database-queries",
            "parameter-binding",
            "transactions",
            "testing-database-code",
        ],
        crate_apis: &[
            "#[server]",
            "ServerFnError",
            "leptos_axum::extract",
            "sqlx::query!",
            "sqlx::query_as!",
            "sqlx::Pool",
            "sqlx::Transaction",
            "sea_query::Query",
            "sea_query_sqlx::SqlxBinder",
        ],
        prerequisites: &[
            "Serializable DTOs",
            "Axum server integration",
            "Shared database pool context",
            "DATABASE_URL or sqlx offline metadata for checked query macros",
        ],
        common_errors: &[
            "Server function is not async",
            "Return type is not Result<T, ServerFnError>",
            "Leaking server-only data",
            "Formatting user input into SQL",
            "Missing sqlx prepare data in CI",
        ],
        related_sections: &["leptos-axum", "forms", "actions", "axum"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "routing",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos_router/latest/leptos_router/",
        task_tags: &["routing", "params", "navigation", "layouts"],
        crate_apis: &["Router", "Routes", "Route", "ParentRoute", "Outlet"],
        prerequisites: &["Component structure"],
        common_errors: &["Missing Outlet in parent route", "Mismatched route params"],
        related_sections: &["leptos-axum", "ssr-hydration-deployment"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "forms",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/form/index.html",
        task_tags: &["forms", "validation", "controlled-inputs", "ActionForm"],
        crate_apis: &["ActionForm", "event_target_value", "prop:value"],
        prerequisites: &["Signals", "actions for server submit"],
        common_errors: &[
            "Using value instead of prop:value",
            "Missing prevent_default for manual forms",
        ],
        related_sections: &["actions", "server-functions", "views"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "error-handling",
        crate_versions: SQL_GUIDANCE_CRATES,
        source_url: "https://docs.rs/leptos/latest/leptos/error/index.html",
        task_tags: &[
            "errors",
            "ErrorBoundary",
            "ServerFnError",
            "IntoResponse",
            "sqlx",
            "database-errors",
        ],
        crate_apis: &[
            "ErrorBoundary",
            "ServerFnError",
            "ResponseOptions",
            "IntoResponse",
            "sqlx::Error",
        ],
        prerequisites: &["Result-returning resources and server functions"],
        common_errors: &[
            "Showing internal errors to users",
            "Not setting HTTP status for SSR errors",
            "Exposing SQL text or driver details to the browser",
        ],
        related_sections: &["server-functions", "leptos-axum", "axum"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "suspense",
        crate_versions: LEPTOS_CRATE,
        source_url: "https://docs.rs/leptos/latest/leptos/suspense/index.html",
        task_tags: &["async-ui", "loading", "streaming", "transition"],
        crate_apis: &["Suspense", "Transition", "Await"],
        prerequisites: &["Resource or async data source"],
        common_errors: &["One large Suspense blocks unrelated page regions"],
        related_sections: &["resources", "ssr-hydration-deployment"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "leptos-axum",
        crate_versions: SQL_GUIDANCE_CRATES,
        source_url: "https://docs.rs/leptos_axum/latest/leptos_axum/",
        task_tags: &[
            "leptos_axum",
            "ssr",
            "server-functions",
            "extractors",
            "route-list",
            "database-pool",
            "sqlx",
        ],
        crate_apis: &[
            "LeptosRoutes",
            "generate_route_list",
            "handle_server_fns",
            "extract",
            "extract_with_state",
            "ResponseOptions",
            "sqlx::Pool",
        ],
        prerequisites: &[
            "Axum Router",
            "Leptos Router",
            "LeptosOptions",
            "Database pool initialized during server startup",
        ],
        common_errors: &[
            "API prefix does not match server function route",
            "Using body extractors with leptos_axum::extract",
            "Disabling default features outside wasm runtimes",
            "Providing pool to Axum state but not to Leptos context",
        ],
        related_sections: &[
            "ssr-hydration-deployment",
            "server-functions",
            "axum",
            "routing",
        ],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "axum",
        crate_versions: SQL_GUIDANCE_CRATES,
        source_url: "https://docs.rs/axum/0.8.9/axum/",
        task_tags: &[
            "axum-0.8.9",
            "Router",
            "extractors",
            "State",
            "middleware",
            "IntoResponse",
            "database-pool",
            "sqlx",
        ],
        crate_apis: &[
            "Router",
            "State",
            "Path",
            "Query",
            "Json",
            "IntoResponse",
            "middleware",
            "sqlx::Pool",
        ],
        prerequisites: &[
            "Tokio runtime",
            "HTTP handler basics",
            "Database pool initialized before Router::with_state",
        ],
        common_errors: &[
            "Router<S> still missing state",
            "Extension extraction fails at runtime",
            "Middleware applied to the wrong route scope",
            "Opening a new database pool per request",
        ],
        related_sections: &["leptos-axum", "server-functions", "error-handling"],
        snippet_classification: SnippetClassification::Illustrative,
    },
    SectionMetadata {
        id: "ssr-hydration-deployment",
        crate_versions: LEPTOS_AXUM_CRATES,
        source_url: "https://docs.rs/leptos/latest/leptos/",
        task_tags: &[
            "ssr",
            "hydrate",
            "deployment",
            "static-assets",
            "feature-flags",
        ],
        crate_apis: &["csr", "hydrate", "ssr", "islands", "file_and_error_handler"],
        prerequisites: &["cargo-leptos build pipeline", "Axum route wiring"],
        common_errors: &[
            "Server and client features enabled together",
            "Generated /pkg assets not served",
            "Hydration mismatch from environment-specific rendering",
        ],
        related_sections: &["leptos-axum", "resources", "suspense", "routing"],
        snippet_classification: SnippetClassification::Illustrative,
    },
];

pub fn list_sections() -> &'static [DocSection] {
    SECTIONS
}

pub fn get_metadata(section_id: &str) -> Option<&'static SectionMetadata> {
    SECTION_METADATA
        .iter()
        .find(|metadata| metadata.id == section_id)
}

pub fn resource_uri(section: &DocSection) -> String {
    format!("{DOC_RESOURCE_PREFIX}{}", section.id)
}

pub fn get_section_by_resource_uri(uri: &str) -> Result<&'static DocSection, SectionLookupError> {
    let section_id =
        uri.strip_prefix(DOC_RESOURCE_PREFIX)
            .ok_or_else(|| SectionLookupError::Unknown {
                query: uri.to_string(),
            })?;

    get_section(section_id)
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

pub fn search_sections(query: &str) -> Result<Vec<SectionSearchMatch>, SectionLookupError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Err(SectionLookupError::Empty);
    }

    let mut matches: Vec<SectionSearchMatch> = SECTIONS
        .iter()
        .filter_map(|section| {
            let metadata = get_metadata(section.id)?;
            let mut score = 0;
            let mut matched_fields = Vec::new();

            score += score_field("id", section.id, &normalized_query, &mut matched_fields, 30);
            score += score_field(
                "title",
                section.title,
                &normalized_query,
                &mut matched_fields,
                25,
            );
            score += score_field(
                "use_cases",
                section.use_cases,
                &normalized_query,
                &mut matched_fields,
                15,
            );
            score += score_slice(
                "aliases",
                section.aliases,
                &normalized_query,
                &mut matched_fields,
                20,
            );
            score += score_slice(
                "task_tags",
                metadata.task_tags,
                &normalized_query,
                &mut matched_fields,
                20,
            );
            score += score_slice(
                "crate_apis",
                metadata.crate_apis,
                &normalized_query,
                &mut matched_fields,
                15,
            );
            score += score_slice(
                "common_errors",
                metadata.common_errors,
                &normalized_query,
                &mut matched_fields,
                10,
            );
            score += score_field(
                "content",
                section.content,
                &normalized_query,
                &mut matched_fields,
                3,
            );

            (score > 0).then(|| SectionSearchMatch {
                section,
                metadata,
                score,
                matched_fields,
                why: format!("Matched {} for '{}'", section.title, query.trim()),
                next_actions: next_actions_for(section.id),
            })
        })
        .collect();

    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.section.id.cmp(right.section.id))
    });

    Ok(matches)
}

pub fn rust_code_blocks() -> Vec<RustCodeBlock> {
    let mut blocks = Vec::new();

    for section in SECTIONS {
        let classification = get_metadata(section.id)
            .map(|metadata| metadata.snippet_classification)
            .unwrap_or(SnippetClassification::Ignore);
        let mut in_rust_block = false;
        let mut current = Vec::new();

        for line in section.content.lines() {
            if line.trim_start().starts_with("```rust") {
                in_rust_block = true;
                current.clear();
                continue;
            }

            if in_rust_block && line.trim_start().starts_with("```") {
                blocks.push(RustCodeBlock {
                    section_id: section.id,
                    classification,
                    content: current.join("\n"),
                });
                in_rust_block = false;
                continue;
            }

            if in_rust_block {
                current.push(line);
            }
        }
    }

    blocks
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

fn score_field(
    field_name: &'static str,
    value: &str,
    normalized_query: &str,
    matched_fields: &mut Vec<&'static str>,
    points: usize,
) -> usize {
    let normalized_value = normalize(value);
    if normalized_value == normalized_query {
        push_unique(matched_fields, field_name);
        points * 2
    } else if normalized_value.contains(normalized_query) {
        push_unique(matched_fields, field_name);
        points
    } else {
        0
    }
}

fn score_slice(
    field_name: &'static str,
    values: &[&str],
    normalized_query: &str,
    matched_fields: &mut Vec<&'static str>,
    points: usize,
) -> usize {
    let matches = values
        .iter()
        .filter(|value| normalize(value).contains(normalized_query))
        .count();
    if matches > 0 {
        push_unique(matched_fields, field_name);
    }
    matches * points
}

fn push_unique(values: &mut Vec<&'static str>, value: &'static str) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn next_actions_for(section_id: &str) -> Vec<&'static str> {
    match section_id {
        "leptos-axum" => vec![
            "Call leptos-axum-recipe with recipe=ssr-app",
            "Lookup leptos_axum::LeptosRoutes",
        ],
        "axum" => vec![
            "Lookup axum::Router or axum::extract::State",
            "Review Axum 0.8.9 handler and middleware boundaries",
        ],
        "ssr-hydration-deployment" => vec![
            "Check ssr/hydrate feature flags",
            "Use the debug-hydration prompt for runtime failures",
        ],
        "server-functions" => vec![
            "Run leptos-diagnostics on server function code",
            "Lookup leptos::server",
        ],
        _ => vec!["Call get-documentation with the matched section id"],
    }
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

    #[test]
    fn sections_have_provenance_metadata() {
        for section in list_sections() {
            assert!(!section.leptos_version.is_empty());
            assert!(!section.source.is_empty());
            assert!(!section.source_path.is_empty());
            assert!(!section.reviewed_at.is_empty());
        }
    }

    #[test]
    fn new_full_stack_sections_are_available() {
        assert_eq!(
            get_section("leptos_axum")
                .expect("leptos_axum alias should resolve")
                .id,
            "leptos-axum"
        );
        assert_eq!(
            get_section("axum-0.8.9")
                .expect("axum alias should resolve")
                .id,
            "axum"
        );
        assert_eq!(
            get_section("hydrate")
                .expect("hydration alias should resolve")
                .id,
            "ssr-hydration-deployment"
        );
    }

    #[test]
    fn every_section_has_extended_metadata() {
        let section_ids: HashSet<&str> = list_sections().iter().map(|section| section.id).collect();
        let metadata_ids: HashSet<&str> = SECTION_METADATA
            .iter()
            .map(|metadata| metadata.id)
            .collect();

        assert_eq!(section_ids, metadata_ids);
        for metadata in SECTION_METADATA {
            assert!(!metadata.crate_versions.is_empty());
            assert!(!metadata.source_url.is_empty());
            assert!(!metadata.task_tags.is_empty());
            assert!(!metadata.related_sections.is_empty());
            for related in metadata.related_sections {
                assert!(
                    section_ids.contains(related),
                    "unknown related section {related}"
                );
            }
        }
    }

    #[test]
    fn search_ranks_task_relevant_sections() {
        let matches = search_sections("hydration").expect("search should succeed");

        assert_eq!(
            matches.first().expect("expected match").section.id,
            "ssr-hydration-deployment"
        );
        assert!(
            matches[0].matched_fields.contains(&"aliases")
                || matches[0].matched_fields.contains(&"task_tags")
        );
    }

    #[test]
    fn search_finds_sql_guidance_sections() {
        let matches = search_sections("sqlx").expect("search should succeed");
        let ids: Vec<&str> = matches.iter().map(|match_| match_.section.id).collect();

        assert!(ids.contains(&"server-functions"));
        assert!(ids.contains(&"axum"));
        assert!(
            matches[0].matched_fields.contains(&"aliases")
                || matches[0].matched_fields.contains(&"task_tags")
                || matches[0].matched_fields.contains(&"crate_apis")
        );
    }

    #[test]
    fn search_finds_sea_query_guidance() {
        let matches = search_sections("sea-query").expect("search should succeed");

        assert_eq!(
            matches.first().expect("expected match").section.id,
            "server-functions"
        );
    }

    #[test]
    fn resource_uri_resolves_to_documentation_section() {
        let section =
            get_section_by_resource_uri("leptos://docs/axum").expect("resource URI should resolve");

        assert_eq!(section.id, "axum");
    }

    #[test]
    fn rust_snippets_are_classified_for_drift_review() {
        let blocks = rust_code_blocks();

        assert!(!blocks.is_empty());
        assert!(
            blocks
                .iter()
                .all(|block| block.classification != SnippetClassification::Ignore)
        );
    }
}
