//! Leptos documentation catalog.

use crate::api::{
    AXUM_DOCS_URL, AXUM_VERSION, LEPTOS_AXUM_DOCS_URL, LEPTOS_AXUM_VERSION, LEPTOS_DOCS_URL,
    LEPTOS_VERSION,
};
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
    /// A Rust snippet intended to be compiled by an automated harness as-is or
    /// with only the standard repository-provided wrapper/import prelude.
    ///
    /// Use this only when the snippet is complete enough to type-check without
    /// application-specific state, routes, database schemas, generated files,
    /// feature flags, or omitted surrounding functions. If a snippet needs a
    /// custom wrapper, add that wrapper to the harness before promoting it.
    CompileCandidate,
    /// A Rust snippet intended to teach an API or pattern but not to compile on
    /// its own in this repository.
    ///
    /// Use this for fragments, excerpts, pseudo-application code, examples that
    /// omit surrounding components/functions/imports, and examples that depend
    /// on user project state such as database pools, route trees, or schemas.
    /// This is the default classification until a compile harness exists.
    Illustrative,
    /// A fenced Rust block that should be excluded from snippet inventory and
    /// compile checks.
    ///
    /// Use this only for non-example Rust-like text, intentionally invalid code,
    /// expected compiler diagnostics, or placeholders where treating the block as
    /// a snippet would mislead contributors. Prefer `Illustrative` for real
    /// fragments that still communicate useful application code.
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

/// Single-row documentation catalog model.
///
/// The catalog row is the chosen source for section summary generation: it pairs
/// each `DocSection` with its `SectionMetadata` up front, so callers can render
/// normal summaries by iterating `list_catalog_sections()` instead of joining a
/// section id back to `SECTION_METADATA`. The legacy section and metadata slices
/// remain during the migration for existing accessors; Task 3 can derive those
/// accessors from this paired catalog surface without changing public ids or
/// resource URIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogSection {
    pub section: &'static DocSection,
    pub metadata: &'static SectionMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionList;

impl SectionList {
    pub fn iter(self) -> impl Iterator<Item = &'static DocSection> {
        list_catalog_sections().iter().map(catalog_section_doc)
    }

    pub fn len(self) -> usize {
        list_catalog_sections().len()
    }

    pub fn is_empty(self) -> bool {
        list_catalog_sections().is_empty()
    }
}

impl IntoIterator for SectionList {
    type Item = &'static DocSection;
    type IntoIter = std::iter::Map<
        std::slice::Iter<'static, CatalogSection>,
        fn(&'static CatalogSection) -> &'static DocSection,
    >;

    fn into_iter(self) -> Self::IntoIter {
        list_catalog_sections().iter().map(catalog_section_doc)
    }
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
    docs_url: LEPTOS_DOCS_URL,
}];

static LEPTOS_AXUM_CRATES: &[CrateVersion] = &[
    CrateVersion {
        name: "leptos",
        version: LEPTOS_VERSION,
        docs_url: LEPTOS_DOCS_URL,
    },
    CrateVersion {
        name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        docs_url: LEPTOS_AXUM_DOCS_URL,
    },
    CrateVersion {
        name: "axum",
        version: AXUM_VERSION,
        docs_url: AXUM_DOCS_URL,
    },
];

static SQL_GUIDANCE_CRATES: &[CrateVersion] = &[
    CrateVersion {
        name: "leptos",
        version: LEPTOS_VERSION,
        docs_url: LEPTOS_DOCS_URL,
    },
    CrateVersion {
        name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        docs_url: LEPTOS_AXUM_DOCS_URL,
    },
    CrateVersion {
        name: "axum",
        version: AXUM_VERSION,
        docs_url: AXUM_DOCS_URL,
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
        source_url: LEPTOS_DOCS_URL,
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
        source_url: LEPTOS_AXUM_DOCS_URL,
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
        source_url: AXUM_DOCS_URL,
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
        source_url: LEPTOS_DOCS_URL,
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
        snippet_classification: SnippetClassification::CompileCandidate,
    },
];

static CATALOG_SECTIONS: &[CatalogSection] = &[
    CatalogSection {
        section: &SECTIONS[0],
        metadata: &SECTION_METADATA[0],
    },
    CatalogSection {
        section: &SECTIONS[1],
        metadata: &SECTION_METADATA[1],
    },
    CatalogSection {
        section: &SECTIONS[2],
        metadata: &SECTION_METADATA[2],
    },
    CatalogSection {
        section: &SECTIONS[3],
        metadata: &SECTION_METADATA[3],
    },
    CatalogSection {
        section: &SECTIONS[4],
        metadata: &SECTION_METADATA[4],
    },
    CatalogSection {
        section: &SECTIONS[5],
        metadata: &SECTION_METADATA[5],
    },
    CatalogSection {
        section: &SECTIONS[6],
        metadata: &SECTION_METADATA[6],
    },
    CatalogSection {
        section: &SECTIONS[7],
        metadata: &SECTION_METADATA[7],
    },
    CatalogSection {
        section: &SECTIONS[8],
        metadata: &SECTION_METADATA[8],
    },
    CatalogSection {
        section: &SECTIONS[9],
        metadata: &SECTION_METADATA[9],
    },
    CatalogSection {
        section: &SECTIONS[10],
        metadata: &SECTION_METADATA[10],
    },
    CatalogSection {
        section: &SECTIONS[11],
        metadata: &SECTION_METADATA[11],
    },
    CatalogSection {
        section: &SECTIONS[12],
        metadata: &SECTION_METADATA[12],
    },
    CatalogSection {
        section: &SECTIONS[13],
        metadata: &SECTION_METADATA[13],
    },
];

pub fn list_sections() -> SectionList {
    SectionList
}

pub fn list_catalog_sections() -> &'static [CatalogSection] {
    CATALOG_SECTIONS
}

pub fn get_metadata(section_id: &str) -> Option<&'static SectionMetadata> {
    CATALOG_SECTIONS
        .iter()
        .find(|catalog_section| catalog_section.section.id == section_id)
        .map(|catalog_section| catalog_section.metadata)
}

pub fn resource_uri(section: &DocSection) -> String {
    format!("{DOC_RESOURCE_PREFIX}{}", section.id)
}

pub fn get_section_by_resource_uri(uri: &str) -> Result<&'static DocSection, SectionLookupError> {
    get_catalog_section_by_resource_uri(uri).map(|catalog_section| catalog_section.section)
}

pub fn get_catalog_section_by_resource_uri(
    uri: &str,
) -> Result<&'static CatalogSection, SectionLookupError> {
    let section_id =
        uri.strip_prefix(DOC_RESOURCE_PREFIX)
            .ok_or_else(|| SectionLookupError::Unknown {
                query: uri.to_string(),
            })?;

    get_catalog_section(section_id)
}

pub fn get_section(query: &str) -> Result<&'static DocSection, SectionLookupError> {
    get_catalog_section(query).map(|catalog_section| catalog_section.section)
}

pub fn get_catalog_section(query: &str) -> Result<&'static CatalogSection, SectionLookupError> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return Err(SectionLookupError::Empty);
    }

    let matches: Vec<&CatalogSection> = CATALOG_SECTIONS
        .iter()
        .filter(|catalog_section| catalog_section.section.matches(&normalized))
        .collect();

    match matches.as_slice() {
        [catalog_section] => Ok(*catalog_section),
        [] => Err(SectionLookupError::Unknown {
            query: query.to_string(),
        }),
        multiple => Err(SectionLookupError::Ambiguous {
            query: query.to_string(),
            matches: multiple
                .iter()
                .map(|catalog_section| catalog_section.section.id.to_string())
                .collect(),
        }),
    }
}

pub fn search_sections(query: &str) -> Result<Vec<SectionSearchMatch>, SectionLookupError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Err(SectionLookupError::Empty);
    }

    let mut matches: Vec<SectionSearchMatch> = CATALOG_SECTIONS
        .iter()
        .filter_map(|catalog_section| {
            let section = catalog_section.section;
            let metadata = catalog_section.metadata;
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

    for catalog_section in CATALOG_SECTIONS {
        let section = catalog_section.section;
        let classification = catalog_section.metadata.snippet_classification;
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

fn catalog_section_doc(catalog_section: &'static CatalogSection) -> &'static DocSection {
    catalog_section.section
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
    use std::collections::{HashMap, HashSet};

    const ALLOWED_SELF_RELATED_SECTIONS: &[&str] = &[];

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
    fn normalized_lookup_terms_do_not_collide_across_sections() {
        let mut lookup_terms: HashMap<String, &str> = HashMap::new();

        for section in list_sections() {
            let terms = [section.id, section.path, section.title]
                .into_iter()
                .chain(section.aliases.iter().copied());

            for term in terms {
                let normalized = normalize(term);
                if let Some(existing_section_id) =
                    lookup_terms.insert(normalized.clone(), section.id)
                {
                    assert_eq!(
                        existing_section_id, section.id,
                        "normalized lookup term '{normalized}' is used by both '{existing_section_id}' and '{}'",
                        section.id
                    );
                }
            }
        }
    }

    #[test]
    fn sections_have_provenance_metadata() {
        for section in list_sections() {
            assert!(
                !section.leptos_version.is_empty(),
                "section '{}' has empty leptos_version",
                section.id
            );
            assert!(
                !section.source.is_empty(),
                "section '{}' has empty source",
                section.id
            );
            assert!(
                !section.source_path.is_empty(),
                "section '{}' has empty source_path",
                section.id
            );
            assert!(
                !section.reviewed_at.is_empty(),
                "section '{}' has empty reviewed_at",
                section.id
            );
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
        let mut metadata_ids = HashSet::new();

        for metadata in SECTION_METADATA {
            assert!(
                metadata_ids.insert(metadata.id),
                "duplicate metadata record for section '{}'",
                metadata.id
            );
            assert!(
                section_ids.contains(metadata.id),
                "orphan metadata record for unknown section '{}'",
                metadata.id
            );
        }

        for section_id in &section_ids {
            assert!(
                metadata_ids.contains(section_id),
                "missing metadata record for section '{section_id}'"
            );
        }

        assert_eq!(
            SECTION_METADATA.len(),
            list_sections().len(),
            "expected exactly one metadata record per section"
        );
        for metadata in SECTION_METADATA {
            assert!(
                !metadata.crate_versions.is_empty(),
                "metadata for '{}' has no crate versions",
                metadata.id
            );
            assert!(
                !metadata.source_url.is_empty(),
                "metadata for '{}' has empty source_url",
                metadata.id
            );
            assert!(
                !metadata.task_tags.is_empty(),
                "metadata for '{}' has no task tags",
                metadata.id
            );
            assert!(
                !metadata.related_sections.is_empty(),
                "metadata for '{}' has no related sections",
                metadata.id
            );
        }
    }

    #[test]
    fn related_sections_reference_known_non_self_sections() {
        let section_ids: HashSet<&str> = list_sections().iter().map(|section| section.id).collect();
        let allowed_self_relations: HashSet<&str> =
            ALLOWED_SELF_RELATED_SECTIONS.iter().copied().collect();
        let mut known_section_ids: Vec<&str> = section_ids.iter().copied().collect();
        known_section_ids.sort_unstable();

        let mut failures = Vec::new();

        for metadata in SECTION_METADATA {
            for related in metadata.related_sections {
                if !section_ids.contains(related) {
                    failures.push(format!(
                        "metadata for '{}' has related_sections entry '{}' but no section with that id exists; add the missing section or use one of: {}",
                        metadata.id,
                        related,
                        known_section_ids.join(", ")
                    ));
                }

                if *related == metadata.id && !allowed_self_relations.contains(metadata.id) {
                    failures.push(format!(
                        "metadata for '{}' relates to itself via related_sections; remove '{}' or add it to ALLOWED_SELF_RELATED_SECTIONS with an explicit rationale",
                        metadata.id,
                        related
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "invalid related_sections entries:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn catalog_sections_pair_sections_with_metadata_for_summary_generation() {
        assert_eq!(
            list_catalog_sections().len(),
            list_sections().len(),
            "catalog rows should cover every section"
        );

        for catalog_section in list_catalog_sections() {
            assert_eq!(
                catalog_section.section.id, catalog_section.metadata.id,
                "catalog row must pair section '{}' with its own metadata",
                catalog_section.section.id
            );
            assert!(
                !catalog_section.metadata.task_tags.is_empty(),
                "catalog row for '{}' should expose summary metadata without an id join",
                catalog_section.section.id
            );
        }
    }

    #[test]
    fn metadata_crate_versions_have_source_fields() {
        for metadata in SECTION_METADATA {
            for crate_version in metadata.crate_versions {
                assert!(
                    !crate_version.name.is_empty(),
                    "metadata for '{}' has crate version with empty name",
                    metadata.id
                );
                assert!(
                    !crate_version.version.is_empty(),
                    "metadata for '{}' has crate '{}' with empty version",
                    metadata.id,
                    crate_version.name
                );
                assert!(
                    !crate_version.docs_url.is_empty(),
                    "metadata for '{}' has crate '{}' with empty docs_url",
                    metadata.id,
                    crate_version.name
                );
            }
        }
    }

    #[test]
    fn metadata_versions_match_api_symbol_versions_for_shared_crates() {
        let expected_docs_urls: HashMap<&str, &str> = HashMap::from([
            ("leptos", LEPTOS_DOCS_URL),
            ("leptos_axum", LEPTOS_AXUM_DOCS_URL),
            ("axum", AXUM_DOCS_URL),
        ]);

        let mut api_versions_by_crate: HashMap<&str, HashSet<&str>> = HashMap::new();
        for symbol in crate::api::all_symbols() {
            if expected_docs_urls.contains_key(symbol.crate_name) {
                api_versions_by_crate
                    .entry(symbol.crate_name)
                    .or_default()
                    .insert(symbol.version);
            }
        }

        for (crate_name, versions) in &api_versions_by_crate {
            assert_eq!(
                versions.len(),
                1,
                "API symbols for crate '{crate_name}' should share one owned version"
            );
        }

        for metadata in SECTION_METADATA {
            for crate_version in metadata.crate_versions {
                let Some(expected_docs_url) = expected_docs_urls.get(crate_version.name) else {
                    continue;
                };
                let api_versions = api_versions_by_crate
                    .get(crate_version.name)
                    .expect("shared crate should have API symbols");
                let api_version = api_versions
                    .iter()
                    .next()
                    .expect("shared crate should have API symbol version");

                assert_eq!(
                    crate_version.version, *api_version,
                    "metadata for section '{}' has crate '{}' version that drifts from API symbols",
                    metadata.id, crate_version.name
                );
                assert_eq!(
                    crate_version.docs_url, *expected_docs_url,
                    "metadata for section '{}' has crate '{}' docs_url that drifts from owned source URL",
                    metadata.id, crate_version.name
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
    fn resource_uris_round_trip_for_all_sections() {
        for section in list_sections() {
            let uri = resource_uri(section);
            assert_eq!(
                uri,
                format!("{DOC_RESOURCE_PREFIX}{}", section.id),
                "resource URI for '{}' should use canonical section id",
                section.id
            );

            let resolved = get_section_by_resource_uri(&uri)
                .unwrap_or_else(|_| panic!("resource URI '{uri}' should resolve"));
            assert_eq!(
                resolved.id, section.id,
                "resource URI '{uri}' resolved to '{}' instead of '{}'",
                resolved.id, section.id
            );
        }
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

    #[test]
    fn rust_snippet_inventory_exposes_current_classifications() {
        let blocks = rust_code_blocks();

        let mut inventory: HashMap<&str, (SnippetClassification, usize)> = HashMap::new();
        for block in &blocks {
            let entry = inventory
                .entry(block.section_id)
                .or_insert((block.classification, 0));
            assert_eq!(
                entry.0, block.classification,
                "section '{}' should have one snippet classification",
                block.section_id
            );
            entry.1 += 1;
        }

        assert_eq!(
            inventory,
            HashMap::from([
                ("getting-started", (SnippetClassification::Illustrative, 1)),
                ("components", (SnippetClassification::Illustrative, 4)),
                ("signals", (SnippetClassification::Illustrative, 10)),
                ("views", (SnippetClassification::Illustrative, 9)),
                ("resources", (SnippetClassification::Illustrative, 10)),
                ("actions", (SnippetClassification::Illustrative, 9)),
                (
                    "server-functions",
                    (SnippetClassification::Illustrative, 11),
                ),
                ("routing", (SnippetClassification::Illustrative, 8)),
                ("forms", (SnippetClassification::Illustrative, 4)),
                ("suspense", (SnippetClassification::Illustrative, 6)),
                ("error-handling", (SnippetClassification::Illustrative, 5)),
                ("leptos-axum", (SnippetClassification::Illustrative, 6)),
                ("axum", (SnippetClassification::Illustrative, 5)),
                (
                    "ssr-hydration-deployment",
                    (SnippetClassification::CompileCandidate, 1),
                ),
            ])
        );
        assert_eq!(
            blocks
                .iter()
                .filter(|block| block.classification == SnippetClassification::CompileCandidate)
                .count(),
            1,
            "only docs snippets supported by the shared harness should be compile candidates"
        );
        assert_eq!(
            blocks
                .iter()
                .filter(|block| block.classification == SnippetClassification::Illustrative)
                .count(),
            88
        );
    }
}
