//! Curated public API catalog for Leptos application development.

use crate::docs::SnippetClassification;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const LEPTOS_VERSION: &str = "0.8.19";
pub const LEPTOS_AXUM_VERSION: &str = "0.8.9";
pub const AXUM_VERSION: &str = "0.8.9";
pub const LEPTOS_DOCS_URL: &str = "https://docs.rs/leptos/latest/leptos/";
pub const LEPTOS_AXUM_DOCS_URL: &str = "https://docs.rs/leptos_axum/latest/leptos_axum/";
pub const AXUM_DOCS_URL: &str = "https://docs.rs/axum/0.8.9/axum/";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ApiSymbol {
    pub name: &'static str,
    pub crate_name: &'static str,
    pub version: &'static str,
    pub kind: &'static str,
    pub url: &'static str,
    pub summary: &'static str,
    pub aliases: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub snippet: &'static str,
    pub snippet_classification: SnippetClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ApiConcept {
    pub id: &'static str,
    pub title: &'static str,
    pub crate_names: &'static [&'static str],
    pub version_scope: &'static str,
    pub kind: &'static str,
    pub summary: &'static str,
    pub aliases: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub related_symbols: &'static [&'static str],
    pub snippet: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "entry_type", content = "entry", rename_all = "kebab-case")]
pub enum ApiLookupItem {
    Symbol(ApiSymbol),
    Concept(ApiConcept),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiLookupStatus {
    Found,
    Ambiguous,
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiMatchKind {
    ExactSymbol,
    ExactAlias,
    Macro,
    AttributeMacro,
    FunctionCall,
    Concept,
    Prefix,
    Token,
    Summary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiLookupMatch {
    pub match_kind: ApiMatchKind,
    pub score: usize,
    pub matched: String,
    pub item: ApiLookupItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiLookupSuggestion {
    pub score: usize,
    pub reason: &'static str,
    pub item: ApiLookupItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiLookup {
    pub query: String,
    pub normalized_query: String,
    pub crate_filter: Option<String>,
    pub status: ApiLookupStatus,
    pub primary: Option<ApiLookupMatch>,
    pub matches: Vec<ApiLookupMatch>,
    pub suggestions: Vec<ApiLookupSuggestion>,
    pub guidance: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiRustSnippet {
    pub symbol_name: &'static str,
    pub classification: SnippetClassification,
    pub content: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiLookupError {
    Empty,
    Unknown {
        query: String,
        crate_name: Option<String>,
    },
    Ambiguous {
        query: String,
        matches: Vec<String>,
    },
}

static API_SYMBOLS: &[ApiSymbol] = &[
    ApiSymbol {
        name: "leptos::component",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "attribute-macro",
        url: "https://docs.rs/leptos/latest/leptos/attr.component.html",
        summary: "Marks a Rust function as a Leptos component with typed props and an IntoView return value.",
        aliases: &["#[component]", "component macro", "component attribute"],
        related_sections: &["components", "views"],
        snippet: "#[component]\nfn Greeting(name: String) -> impl IntoView {\n    view! { <p>\"Hello \" {name}</p> }\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::view",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "macro",
        url: "https://docs.rs/leptos/latest/leptos/macro.view.html",
        summary: "Creates Leptos view nodes using RSX-like syntax and supports reactive closures, components, attributes, classes, and styles.",
        aliases: &["view!", "view macro", "view template"],
        related_sections: &["views", "components", "signals"],
        snippet: "view! {\n    <button on:click=move |_| set_count.update(|n| *n += 1)>\n        {move || count.get()}\n    </button>\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::IntoView",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "trait",
        url: "https://docs.rs/leptos/latest/leptos/prelude/trait.IntoView.html",
        summary: "Trait implemented by values that can be converted into Leptos views; component functions commonly return impl IntoView.",
        aliases: &["IntoView", "into view", "component return type"],
        related_sections: &["components", "views"],
        snippet: "#[component]\nfn App() -> impl IntoView {\n    view! { <main>\"Hello\"</main> }\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::signal",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos/latest/leptos/prelude/fn.signal.html",
        summary: "Creates a read/write signal pair for reactive state in Leptos components and effects.",
        aliases: &["signal()", "signal function", "create signal"],
        related_sections: &["signals", "views"],
        snippet: "let (count, set_count) = signal(0);\nset_count.update(|count| *count += 1);",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::RwSignal",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "struct",
        url: "https://docs.rs/leptos/latest/leptos/prelude/struct.RwSignal.html",
        summary: "Copyable reactive value that can be read and written through get, set, read, write, update, and with.",
        aliases: &["RwSignal", "rw signal", "read write signal"],
        related_sections: &["signals"],
        snippet: "let count = RwSignal::new(0);\ncount.update(|value| *value += 1);",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::Memo",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "struct",
        url: "https://docs.rs/leptos/latest/leptos/prelude/struct.Memo.html",
        summary: "Memoized derived reactive value that recalculates when tracked dependencies change.",
        aliases: &["Memo", "memoized signal", "derived signal"],
        related_sections: &["signals"],
        snippet: "let double_count = Memo::new(move |_| count.get() * 2);",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::prelude::Resource",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "struct",
        url: "https://docs.rs/leptos/latest/leptos/prelude/struct.Resource.html",
        summary: "Reactive async data source used for SSR-aware reads; supports constructors such as Resource::new and Resource::new_blocking plus refetch().",
        aliases: &[
            "resource",
            "Resource",
            "Resource::new",
            "Resource::new_blocking",
            "refetch",
        ],
        related_sections: &["resources", "suspense", "server-functions"],
        snippet: "let data = Resource::new(move || id.get(), |id| load_item(id));\nlet blocking = Resource::new_blocking(|| (), |_| load_initial_state());",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::server",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "attribute-macro",
        url: "https://docs.rs/leptos/latest/leptos/attr.server.html",
        summary: "Marks an async server function that exposes a public HTTP endpoint and must return Result<T, ServerFnError>.",
        aliases: &["#[server]", "server", "server function", "server_fn"],
        related_sections: &["server-functions", "forms", "actions"],
        snippet: "#[server(GetUser)]\npub async fn get_user(id: String) -> Result<UserDto, ServerFnError> {\n    Ok(load_user(id).await?)\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::server_fn::ServerFnError",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "enum",
        url: "https://docs.rs/leptos/latest/leptos/server_fn/error/enum.ServerFnError.html",
        summary: "Standard error envelope for fallible server functions.",
        aliases: &["ServerFnError", "server fn error"],
        related_sections: &["server-functions", "error-handling"],
        snippet: "pub async fn save() -> Result<(), ServerFnError> {\n    fallible_work().await.map_err(ServerFnError::new)?;\n    Ok(())\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos::form::ActionForm",
        crate_name: "leptos",
        version: LEPTOS_VERSION,
        kind: "component",
        url: "https://docs.rs/leptos/latest/leptos/form/fn.ActionForm.html",
        summary: "Progressively enhanced form component that submits a ServerAction.",
        aliases: &["ActionForm", "action form", "server action form"],
        related_sections: &["forms", "actions", "server-functions"],
        snippet: "<ActionForm action=save_action>\n    <input name=\"title\" />\n    <button type=\"submit\">\"Save\"</button>\n</ActionForm>",
        snippet_classification: SnippetClassification::Ignore,
    },
    ApiSymbol {
        name: "leptos_axum::LeptosRoutes",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "trait",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/trait.LeptosRoutes.html",
        summary: "Extends Axum Router with Leptos route-list integration so app routes do not need wildcard duplication.",
        aliases: &[
            "LeptosRoutes",
            "leptos_routes",
            "leptos_routes_with_context",
        ],
        related_sections: &["leptos-axum", "ssr-hydration-deployment", "routing"],
        snippet: "let app = Router::new()\n    .leptos_routes_with_context(&leptos_options, routes, provide_context, app);",
        snippet_classification: SnippetClassification::CompileCandidate,
    },
    ApiSymbol {
        name: "leptos_axum::generate_route_list",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.generate_route_list.html",
        summary: "Generates Axum-compatible route paths from the Leptos Router tree.",
        aliases: &["generate_route_list", "route list", "routes"],
        related_sections: &["leptos-axum", "routing"],
        snippet: "let routes = generate_route_list(App);",
        snippet_classification: SnippetClassification::CompileCandidate,
    },
    ApiSymbol {
        name: "leptos_axum::handle_server_fns",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.handle_server_fns.html",
        summary: "Axum handler for Leptos server function requests.",
        aliases: &[
            "handle_server_fns",
            "server function handler",
            "server functions route",
        ],
        related_sections: &["leptos-axum", "server-functions"],
        snippet: "let app = Router::new()\n    .route(\"/api/{*fn_name}\", post(handle_server_fns));",
        snippet_classification: SnippetClassification::CompileCandidate,
    },
    ApiSymbol {
        name: "leptos_axum::file_and_error_handler",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.file_and_error_handler.html",
        summary: "Convenience Axum handler for serving static files such as JS/WASM/CSS and rendering 404 pages.",
        aliases: &[
            "file_and_error_handler",
            "static assets",
            "pkg assets",
            "wasm css",
        ],
        related_sections: &["ssr-hydration-deployment", "leptos-axum"],
        snippet: "let app = Router::new()\n    .fallback(file_and_error_handler(shell));",
        snippet_classification: SnippetClassification::CompileCandidate,
    },
    ApiSymbol {
        name: "leptos_axum::extract",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.extract.html",
        summary: "Runs Axum request-parts extractors inside Leptos server functions.",
        aliases: &["extract", "leptos_axum::extract", "axum extractors"],
        related_sections: &["server-functions", "leptos-axum", "axum"],
        snippet: "let Query(params): Query<SearchParams> = leptos_axum::extract().await?;",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos_axum::extract_with_state",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.extract_with_state.html",
        summary: "Runs Axum extractors that need State inside Leptos server functions.",
        aliases: &["extract_with_state", "state extractor in server fn"],
        related_sections: &["server-functions", "leptos-axum", "axum"],
        snippet: "let State(app_state): State<AppState> = leptos_axum::extract_with_state(&state).await?;",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "leptos_axum::ResponseOptions",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "struct",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/struct.ResponseOptions.html",
        summary: "Context object for setting response status, headers, and cookies from Leptos rendering or server functions.",
        aliases: &["ResponseOptions", "set_status", "headers", "cookies"],
        related_sections: &["leptos-axum", "error-handling", "ssr-hydration-deployment"],
        snippet: "let response = expect_context::<ResponseOptions>();\nresponse.set_status(StatusCode::NOT_FOUND);",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::Router",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "struct",
        url: "https://docs.rs/axum/0.8.9/axum/struct.Router.html",
        summary: "Axum's routing type for composing handlers, services, state, middleware, and Leptos routes.",
        aliases: &["Router", "axum router", "route", "fallback"],
        related_sections: &["axum", "leptos-axum", "routing"],
        snippet: "let app = Router::new()\n    .route(\"/api/health\", get(health))\n    .with_state(app_state);",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::extract::State",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "extractor",
        url: "https://docs.rs/axum/0.8.9/axum/extract/struct.State.html",
        summary: "Type-safe extractor for shared application state in Axum handlers.",
        aliases: &["State", "with_state", "FromRef", "substate"],
        related_sections: &["axum", "leptos-axum", "server-functions"],
        snippet: "async fn handler(State(state): State<AppState>) -> impl IntoResponse {\n    Json(state.health()).into_response()\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::extract::Path",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "extractor",
        url: "https://docs.rs/axum/0.8.9/axum/extract/struct.Path.html",
        summary: "Deserializes path parameters from Axum routes.",
        aliases: &["Path", "path params", "route params"],
        related_sections: &["axum", "routing"],
        snippet: "async fn show_user(Path(id): Path<Uuid>) -> impl IntoResponse { ... }",
        snippet_classification: SnippetClassification::Ignore,
    },
    ApiSymbol {
        name: "axum::extract::Query",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "extractor",
        url: "https://docs.rs/axum/0.8.9/axum/extract/struct.Query.html",
        summary: "Deserializes query string parameters into a typed value.",
        aliases: &["Query", "query params"],
        related_sections: &["axum", "server-functions"],
        snippet: "let Query(params): Query<SearchParams> = leptos_axum::extract().await?;",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::Json",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "extractor-response",
        url: "https://docs.rs/axum/0.8.9/axum/struct.Json.html",
        summary: "JSON extractor and response wrapper for Axum handlers.",
        aliases: &["Json", "json response", "json extractor"],
        related_sections: &["axum", "error-handling"],
        snippet: "async fn api() -> Json<ApiResponse> {\n    Json(ApiResponse::ok())\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::response::IntoResponse",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "trait",
        url: "https://docs.rs/axum/0.8.9/axum/response/trait.IntoResponse.html",
        summary: "Trait for converting handler return values and errors into HTTP responses.",
        aliases: &["IntoResponse", "response", "error response"],
        related_sections: &["axum", "error-handling"],
        snippet: "impl IntoResponse for AppError {\n    fn into_response(self) -> Response { (StatusCode::BAD_REQUEST, self.to_string()).into_response() }\n}",
        snippet_classification: SnippetClassification::Illustrative,
    },
    ApiSymbol {
        name: "axum::middleware",
        crate_name: "axum",
        version: AXUM_VERSION,
        kind: "module",
        url: "https://docs.rs/axum/0.8.9/axum/middleware/index.html",
        summary: "Axum middleware helpers built on Tower services and layers.",
        aliases: &["middleware", "tower layer", "route_layer"],
        related_sections: &["axum", "leptos-axum"],
        snippet: "Router::new().route_layer(axum::middleware::from_fn(auth_middleware))",
        snippet_classification: SnippetClassification::Illustrative,
    },
];

static API_CONCEPTS: &[ApiConcept] = &[
    ApiConcept {
        id: "leptos-components",
        title: "Leptos components",
        crate_names: &["leptos"],
        version_scope: LEPTOS_VERSION,
        kind: "concept",
        summary: "Component lookup entry for functions annotated with #[component], typed props, children, and impl IntoView return values.",
        aliases: &["component", "components", "props", "children"],
        related_sections: &["components", "views"],
        related_symbols: &[
            "leptos::component",
            "leptos::prelude::IntoView",
            "leptos::prelude::view",
        ],
        snippet: "#[component]\nfn Card(children: Children) -> impl IntoView {\n    view! { <section>{children()}</section> }\n}",
    },
    ApiConcept {
        id: "leptos-signals",
        title: "Leptos signals",
        crate_names: &["leptos"],
        version_scope: LEPTOS_VERSION,
        kind: "concept",
        summary: "Concept lookup entry for Leptos reactive state: signal(), RwSignal, Memo, get/set/read/write/update, and reactive reads in views.",
        aliases: &["signal", "signals", "reactivity", "reactive state", "state"],
        related_sections: &["signals", "views", "resources"],
        related_symbols: &[
            "leptos::prelude::signal",
            "leptos::prelude::RwSignal",
            "leptos::prelude::Memo",
        ],
        snippet: "let (count, set_count) = signal(0);\nview! { <p>{move || count.get()}</p> }",
    },
];

pub fn all_symbols() -> &'static [ApiSymbol] {
    API_SYMBOLS
}

pub fn all_concepts() -> &'static [ApiConcept] {
    API_CONCEPTS
}

pub fn rust_api_snippets() -> Vec<ApiRustSnippet> {
    API_SYMBOLS
        .iter()
        .filter(|symbol| symbol.snippet_classification != SnippetClassification::Ignore)
        .map(|symbol| ApiRustSnippet {
            symbol_name: symbol.name,
            classification: symbol.snippet_classification,
            content: symbol.snippet,
        })
        .collect()
}

pub fn lookup_api(query: &str, crate_name: Option<&str>) -> Result<ApiLookup, ApiLookupError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        if query.trim().is_empty() {
            return Err(ApiLookupError::Empty);
        }

        return Ok(not_found_lookup(query, crate_name, normalized_query));
    }

    let normalized_crate = crate_name.map(normalize);
    let exact_matches = exact_lookup_matches(query, normalized_crate.as_deref());
    if !exact_matches.is_empty() {
        return Ok(lookup_from_matches(
            query,
            normalized_query,
            normalized_crate,
            exact_matches,
            MatchOutcome::Exact,
        ));
    }

    let fuzzy_matches = fuzzy_lookup_matches(&normalized_query, normalized_crate.as_deref());
    if !fuzzy_matches.is_empty() {
        return Ok(lookup_from_matches(
            query,
            normalized_query,
            normalized_crate,
            fuzzy_matches,
            MatchOutcome::Fuzzy,
        ));
    }

    Ok(not_found_lookup(query, crate_name, normalized_query))
}

pub fn lookup_symbol(
    query: &str,
    crate_name: Option<&str>,
) -> Result<&'static ApiSymbol, ApiLookupError> {
    let lookup = lookup_api(query, crate_name)?;
    match lookup.status {
        ApiLookupStatus::Found => lookup
            .primary
            .as_ref()
            .and_then(|primary| primary.item.symbol_name())
            .and_then(find_symbol)
            .ok_or_else(|| ApiLookupError::Unknown {
                query: query.to_string(),
                crate_name: crate_name.map(str::to_string),
            }),
        ApiLookupStatus::Ambiguous => Err(ApiLookupError::Ambiguous {
            query: query.to_string(),
            matches: lookup
                .matches
                .iter()
                .map(|match_| match_.item.identity().to_string())
                .collect(),
        }),
        ApiLookupStatus::NotFound => Err(ApiLookupError::Unknown {
            query: query.to_string(),
            crate_name: crate_name.map(str::to_string),
        }),
    }
}

fn find_symbol(name: &str) -> Option<&'static ApiSymbol> {
    API_SYMBOLS.iter().find(|symbol| symbol.name == name)
}

fn find_item(identity: &str) -> Option<ApiLookupItem> {
    API_SYMBOLS
        .iter()
        .find(|symbol| symbol.name == identity)
        .map(|symbol| ApiLookupItem::Symbol(*symbol))
        .or_else(|| {
            API_CONCEPTS
                .iter()
                .find(|concept| concept.id == identity)
                .map(|concept| ApiLookupItem::Concept(*concept))
        })
}

impl ApiLookupItem {
    pub fn identity(&self) -> &'static str {
        match self {
            ApiLookupItem::Symbol(symbol) => symbol.name,
            ApiLookupItem::Concept(concept) => concept.id,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ApiLookupItem::Symbol(symbol) => symbol.name,
            ApiLookupItem::Concept(concept) => concept.title,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            ApiLookupItem::Symbol(symbol) => symbol.kind,
            ApiLookupItem::Concept(concept) => concept.kind,
        }
    }

    pub fn summary(&self) -> &'static str {
        match self {
            ApiLookupItem::Symbol(symbol) => symbol.summary,
            ApiLookupItem::Concept(concept) => concept.summary,
        }
    }

    pub fn related_sections(&self) -> &'static [&'static str] {
        match self {
            ApiLookupItem::Symbol(symbol) => symbol.related_sections,
            ApiLookupItem::Concept(concept) => concept.related_sections,
        }
    }

    pub fn symbol_name(&self) -> Option<&'static str> {
        match self {
            ApiLookupItem::Symbol(symbol) => Some(symbol.name),
            ApiLookupItem::Concept(_) => None,
        }
    }

    pub fn concept_id(&self) -> Option<&'static str> {
        match self {
            ApiLookupItem::Symbol(_) => None,
            ApiLookupItem::Concept(concept) => Some(concept.id),
        }
    }
}

impl ApiLookup {
    pub fn primary_symbol(&self) -> Option<&ApiSymbol> {
        self.primary
            .as_ref()
            .and_then(|primary| match &primary.item {
                ApiLookupItem::Symbol(symbol) => Some(symbol),
                ApiLookupItem::Concept(_) => None,
            })
    }

    pub fn primary_concept(&self) -> Option<&ApiConcept> {
        self.primary
            .as_ref()
            .and_then(|primary| match &primary.item {
                ApiLookupItem::Symbol(_) => None,
                ApiLookupItem::Concept(concept) => Some(concept),
            })
    }
}

#[derive(Debug)]
struct ApiIndex {
    exact_terms: BTreeMap<String, Vec<ApiIndexedTerm>>,
}

#[derive(Debug, Clone, Copy)]
struct ApiIndexedTerm {
    item: ApiLookupItem,
    term_kind: ApiTermKind,
    term: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApiTermKind {
    CanonicalSymbol,
    SymbolAlias,
    MacroAlias,
    AttributeAlias,
    FunctionCallAlias,
    ConceptIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchOutcome {
    Exact,
    Fuzzy,
}

static API_INDEX: OnceLock<ApiIndex> = OnceLock::new();

fn api_index() -> &'static ApiIndex {
    API_INDEX.get_or_init(build_api_index)
}

fn build_api_index() -> ApiIndex {
    let mut exact_terms: BTreeMap<String, Vec<ApiIndexedTerm>> = BTreeMap::new();

    for symbol in API_SYMBOLS {
        let item = ApiLookupItem::Symbol(*symbol);
        index_exact_term(
            &mut exact_terms,
            item,
            symbol.name,
            ApiTermKind::CanonicalSymbol,
        );
        for alias in symbol.aliases {
            index_exact_term(&mut exact_terms, item, alias, symbol_alias_term_kind(alias));
        }
    }

    for concept in API_CONCEPTS {
        let item = ApiLookupItem::Concept(*concept);
        for term in [concept.id, concept.title] {
            index_exact_term(&mut exact_terms, item, term, ApiTermKind::ConceptIdentity);
        }
        for alias in concept.aliases {
            index_exact_term(&mut exact_terms, item, alias, ApiTermKind::ConceptIdentity);
        }
    }

    ApiIndex { exact_terms }
}

fn index_exact_term(
    exact_terms: &mut BTreeMap<String, Vec<ApiIndexedTerm>>,
    item: ApiLookupItem,
    term: &'static str,
    term_kind: ApiTermKind,
) {
    for key in exact_term_keys(term) {
        exact_terms.entry(key).or_default().push(ApiIndexedTerm {
            item,
            term_kind,
            term,
        });
    }
}

fn exact_lookup_matches(query: &str, normalized_crate: Option<&str>) -> Vec<ApiLookupMatch> {
    let mut matches = Vec::new();
    for key in exact_query_keys(query) {
        if let Some(indexed_terms) = api_index().exact_terms.get(&key) {
            matches.extend(
                indexed_terms
                    .iter()
                    .filter(|indexed| item_matches_crate(indexed.item, normalized_crate))
                    .map(indexed_term_match),
            );
        }
    }

    rank_matches(matches)
}

fn indexed_term_match(indexed: &ApiIndexedTerm) -> ApiLookupMatch {
    ApiLookupMatch {
        match_kind: indexed.term_kind.match_kind(),
        score: indexed.term_kind.exact_score(),
        matched: indexed.term.to_string(),
        item: indexed.item,
    }
}

fn fuzzy_lookup_matches(
    normalized_query: &str,
    normalized_crate: Option<&str>,
) -> Vec<ApiLookupMatch> {
    if normalized_query.len() < MIN_QUERY_TOKEN_LEN {
        return Vec::new();
    }

    let tokens = query_tokens(normalized_query);
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for item in catalog_items() {
        if !item_matches_crate(item, normalized_crate) {
            continue;
        }

        if let Some(match_) = best_fuzzy_match(item, normalized_query, &tokens) {
            matches.push(match_);
        }
    }

    rank_matches(matches)
}

fn best_fuzzy_match(
    item: ApiLookupItem,
    normalized_query: &str,
    normalized_tokens: &[String],
) -> Option<ApiLookupMatch> {
    let mut best = None;
    for field in searchable_fields(item) {
        consider_api_match(
            &mut best,
            prefix_field_match(item, normalized_query, field.name, field.value),
        );
        consider_api_match(
            &mut best,
            token_field_match(item, normalized_tokens, field.name, field.value),
        );
    }

    consider_api_match(
        &mut best,
        summary_field_match(item, normalized_tokens, item.summary()),
    );

    best
}

#[derive(Debug, Clone, Copy)]
struct SearchField {
    name: &'static str,
    value: &'static str,
}

fn searchable_fields(item: ApiLookupItem) -> Vec<SearchField> {
    match item {
        ApiLookupItem::Symbol(symbol) => {
            let mut fields = vec![
                SearchField {
                    name: "symbol",
                    value: symbol.name,
                },
                SearchField {
                    name: "kind",
                    value: symbol.kind,
                },
            ];
            fields.extend(symbol.aliases.iter().map(|alias| SearchField {
                name: "alias",
                value: alias,
            }));
            fields
        }
        ApiLookupItem::Concept(concept) => {
            let mut fields = vec![
                SearchField {
                    name: "concept",
                    value: concept.id,
                },
                SearchField {
                    name: "title",
                    value: concept.title,
                },
                SearchField {
                    name: "kind",
                    value: concept.kind,
                },
            ];
            fields.extend(concept.aliases.iter().map(|alias| SearchField {
                name: "alias",
                value: alias,
            }));
            fields.extend(concept.related_symbols.iter().map(|symbol| SearchField {
                name: "related-symbol",
                value: symbol,
            }));
            fields
        }
    }
}

fn prefix_field_match(
    item: ApiLookupItem,
    normalized_query: &str,
    field_name: &'static str,
    field: &str,
) -> Option<ApiLookupMatch> {
    field_matches_prefix(normalized_query, field).then(|| ApiLookupMatch {
        match_kind: ApiMatchKind::Prefix,
        score: 700 + field_weight(field_name),
        matched: field.to_string(),
        item,
    })
}

fn token_field_match(
    item: ApiLookupItem,
    normalized_tokens: &[String],
    field_name: &'static str,
    field: &str,
) -> Option<ApiLookupMatch> {
    matches_field_tokens(normalized_tokens, field).then(|| ApiLookupMatch {
        match_kind: ApiMatchKind::Token,
        score: 500 + field_weight(field_name),
        matched: field.to_string(),
        item,
    })
}

fn summary_field_match(
    item: ApiLookupItem,
    normalized_tokens: &[String],
    summary: &str,
) -> Option<ApiLookupMatch> {
    (normalized_tokens.len() >= 2 && matches_field_tokens(normalized_tokens, summary)).then(|| {
        ApiLookupMatch {
            match_kind: ApiMatchKind::Summary,
            score: 300,
            matched: "summary".to_string(),
            item,
        }
    })
}

fn field_weight(field_name: &str) -> usize {
    match field_name {
        "symbol" | "concept" => 90,
        "title" => 85,
        "alias" => 80,
        "related-symbol" => 70,
        "kind" => 50,
        _ => 0,
    }
}

fn consider_api_match(best: &mut Option<ApiLookupMatch>, candidate: Option<ApiLookupMatch>) {
    let Some(candidate) = candidate else {
        return;
    };

    if best
        .as_ref()
        .is_none_or(|current| api_match_is_stronger(&candidate, current))
    {
        *best = Some(candidate);
    }
}

fn rank_matches(matches: Vec<ApiLookupMatch>) -> Vec<ApiLookupMatch> {
    let mut by_identity: BTreeMap<&'static str, ApiLookupMatch> = BTreeMap::new();
    for match_ in matches {
        let identity = match_.item.identity();
        match by_identity.get(identity) {
            Some(current) if !api_match_is_stronger(&match_, current) => {}
            _ => {
                by_identity.insert(identity, match_);
            }
        }
    }

    let mut matches: Vec<ApiLookupMatch> = by_identity.into_values().collect();
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.item.identity().cmp(right.item.identity()))
    });
    matches
}

fn api_match_is_stronger(candidate: &ApiLookupMatch, current: &ApiLookupMatch) -> bool {
    candidate
        .score
        .cmp(&current.score)
        .then_with(|| current.item.identity().cmp(candidate.item.identity()))
        .is_gt()
}

fn lookup_from_matches(
    query: &str,
    normalized_query: String,
    normalized_crate: Option<String>,
    matches: Vec<ApiLookupMatch>,
    outcome: MatchOutcome,
) -> ApiLookup {
    let status = match matches.as_slice() {
        [_] => ApiLookupStatus::Found,
        _ => ApiLookupStatus::Ambiguous,
    };
    let primary = (status == ApiLookupStatus::Found).then(|| matches[0].clone());
    let guidance = match status {
        ApiLookupStatus::Found => found_guidance(primary.as_ref().expect("primary match")),
        ApiLookupStatus::Ambiguous => ambiguous_guidance(&matches),
        ApiLookupStatus::NotFound => unreachable!("matches produced a found or ambiguous lookup"),
    };
    let suggestions = if outcome == MatchOutcome::Fuzzy && status == ApiLookupStatus::Ambiguous {
        matches
            .iter()
            .take(MAX_SUGGESTIONS)
            .map(match_as_suggestion)
            .collect()
    } else {
        Vec::new()
    };

    ApiLookup {
        query: query.trim().to_string(),
        normalized_query,
        crate_filter: normalized_crate,
        status,
        primary,
        matches,
        suggestions,
        guidance,
    }
}

fn not_found_lookup(query: &str, crate_name: Option<&str>, normalized_query: String) -> ApiLookup {
    let normalized_crate = crate_name.map(normalize);
    let suggestions = default_suggestions(normalized_crate.as_deref());
    ApiLookup {
        query: query.trim().to_string(),
        normalized_query,
        crate_filter: normalized_crate.clone(),
        status: ApiLookupStatus::NotFound,
        primary: None,
        matches: Vec::new(),
        suggestions,
        guidance: not_found_guidance(normalized_crate.as_deref()),
    }
}

fn found_guidance(match_: &ApiLookupMatch) -> Vec<String> {
    match match_.item {
        ApiLookupItem::Symbol(_) => vec![
            "This is a curated exact API entry. Use related_sections for deeper local documentation."
                .to_string(),
        ],
        ApiLookupItem::Concept(concept) => vec![format!(
            "This is a concept entry. Use related_symbols ({}) for exact API entries and related_sections ({}) for documentation.",
            concept.related_symbols.join(", "),
            concept.related_sections.join(", ")
        )],
    }
}

fn ambiguous_guidance(matches: &[ApiLookupMatch]) -> Vec<String> {
    vec![format!(
        "Multiple curated API entries matched. Use a fully qualified symbol, a macro/attribute form, or a crate filter. Matches: {}.",
        matches
            .iter()
            .map(|match_| match_.item.identity())
            .collect::<Vec<_>>()
            .join(", ")
    )]
}

fn not_found_guidance(normalized_crate: Option<&str>) -> Vec<String> {
    let mut guidance = vec![
        "No curated API entry matched. Try an exact symbol, a declared alias, a macro such as view!, or a concept such as component or signal."
            .to_string(),
    ];

    if normalized_crate.is_some_and(|crate_name| !SUPPORTED_CRATES.contains(&crate_name)) {
        guidance.push(format!(
            "The crate filter is outside the curated crates. Supported crates: {}.",
            SUPPORTED_CRATES.join(", ")
        ));
    }

    guidance
}

fn match_as_suggestion(match_: &ApiLookupMatch) -> ApiLookupSuggestion {
    ApiLookupSuggestion {
        score: match_.score,
        reason: "matched the query but needs disambiguation",
        item: match_.item,
    }
}

const MAX_SUGGESTIONS: usize = 5;
const DEFAULT_SUGGESTION_IDENTITIES: &[&str] = &[
    "leptos-signals",
    "leptos-components",
    "leptos::prelude::view",
    "leptos::prelude::Resource",
    "leptos_axum::LeptosRoutes",
    "axum::Router",
];
const SUPPORTED_CRATES: &[&str] = &["leptos", "leptos_axum", "axum"];

fn default_suggestions(normalized_crate: Option<&str>) -> Vec<ApiLookupSuggestion> {
    DEFAULT_SUGGESTION_IDENTITIES
        .iter()
        .filter_map(|identity| find_item(identity))
        .filter(|item| item_matches_crate(*item, normalized_crate))
        .take(MAX_SUGGESTIONS)
        .enumerate()
        .map(|(index, item)| ApiLookupSuggestion {
            score: MAX_SUGGESTIONS - index,
            reason: "common curated lookup entry",
            item,
        })
        .collect()
}

fn catalog_items() -> impl Iterator<Item = ApiLookupItem> {
    API_SYMBOLS
        .iter()
        .map(|symbol| ApiLookupItem::Symbol(*symbol))
        .chain(
            API_CONCEPTS
                .iter()
                .map(|concept| ApiLookupItem::Concept(*concept)),
        )
}

fn item_matches_crate(item: ApiLookupItem, normalized_crate: Option<&str>) -> bool {
    normalized_crate.is_none_or(|crate_name| match item {
        ApiLookupItem::Symbol(symbol) => normalize(symbol.crate_name) == crate_name,
        ApiLookupItem::Concept(concept) => concept
            .crate_names
            .iter()
            .any(|item_crate| normalize(item_crate) == crate_name),
    })
}

impl ApiTermKind {
    fn match_kind(self) -> ApiMatchKind {
        match self {
            ApiTermKind::CanonicalSymbol => ApiMatchKind::ExactSymbol,
            ApiTermKind::SymbolAlias => ApiMatchKind::ExactAlias,
            ApiTermKind::MacroAlias => ApiMatchKind::Macro,
            ApiTermKind::AttributeAlias => ApiMatchKind::AttributeMacro,
            ApiTermKind::FunctionCallAlias => ApiMatchKind::FunctionCall,
            ApiTermKind::ConceptIdentity => ApiMatchKind::Concept,
        }
    }

    fn exact_score(self) -> usize {
        match self {
            ApiTermKind::CanonicalSymbol => 1_000,
            ApiTermKind::MacroAlias => 990,
            ApiTermKind::AttributeAlias => 990,
            ApiTermKind::FunctionCallAlias => 985,
            ApiTermKind::ConceptIdentity => 980,
            ApiTermKind::SymbolAlias => 960,
        }
    }
}

fn symbol_alias_term_kind(alias: &str) -> ApiTermKind {
    if rust_macro_key(alias).is_some() {
        ApiTermKind::MacroAlias
    } else if rust_attribute_key(alias).is_some() {
        ApiTermKind::AttributeAlias
    } else if rust_call_key(alias).is_some() {
        ApiTermKind::FunctionCallAlias
    } else {
        ApiTermKind::SymbolAlias
    }
}

fn exact_query_keys(query: &str) -> Vec<String> {
    if let Some(key) = rust_attribute_key(query) {
        return vec![key];
    }
    if let Some(key) = rust_macro_key(query) {
        return vec![key];
    }
    if let Some(key) = rust_call_key(query) {
        return vec![key];
    }

    let normalized = normalize(query);
    if normalized.is_empty() {
        Vec::new()
    } else {
        vec![normalized]
    }
}

fn exact_term_keys(term: &str) -> Vec<String> {
    exact_query_keys(term)
}

fn rust_macro_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let macro_name = trimmed.strip_suffix('!')?;
    let normalized = normalize(macro_name);
    (!normalized.is_empty()).then(|| format!("macro:{normalized}"))
}

fn rust_attribute_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let attribute_name = trimmed.strip_prefix("#[")?.strip_suffix(']')?;
    let normalized = normalize(attribute_name);
    (!normalized.is_empty()).then(|| format!("attribute:{normalized}"))
}

fn rust_call_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let function_name = trimmed.strip_suffix("()")?;
    let normalized = normalize(function_name);
    (!normalized.is_empty()).then(|| format!("call:{normalized}"))
}

fn matches_field_tokens(normalized_tokens: &[String], field: &str) -> bool {
    let field_tokens = query_tokens(field);

    !field_tokens.is_empty()
        && normalized_tokens.iter().all(|query_token| {
            field_tokens
                .iter()
                .any(|field_token| field_token.starts_with(query_token))
        })
}

fn field_matches_prefix(normalized_query: &str, field: &str) -> bool {
    if !normalize(field).starts_with(normalized_query) {
        return false;
    }

    let query_token_count = query_tokens_with_min_len(normalized_query, 1).len();
    let field_token_count = query_tokens_with_min_len(field, 1).len();

    query_token_count > 1 || field_token_count == 1
}

pub const MIN_QUERY_TOKEN_LEN: usize = 3;

pub fn normalize_query(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = true;
    let mut alphanumeric_run = String::new();

    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            alphanumeric_run.push(character);
        } else {
            push_alphanumeric_run(
                &mut normalized,
                &mut previous_was_separator,
                &alphanumeric_run,
            );
            alphanumeric_run.clear();
            push_separator(&mut normalized, &mut previous_was_separator);
        }
    }
    push_alphanumeric_run(
        &mut normalized,
        &mut previous_was_separator,
        &alphanumeric_run,
    );

    normalized.trim_matches('-').to_string()
}

pub fn query_tokens(value: &str) -> Vec<String> {
    query_tokens_with_min_len(value, MIN_QUERY_TOKEN_LEN)
}

pub fn query_tokens_with_min_len(value: &str, min_len: usize) -> Vec<String> {
    normalize_query(value)
        .split('-')
        .filter(|token| token.len() >= min_len)
        .map(str::to_string)
        .collect()
}

fn push_separator(value: &mut String, previous_was_separator: &mut bool) {
    if !*previous_was_separator {
        value.push('-');
        *previous_was_separator = true;
    }
}

fn push_alphanumeric_run(normalized: &mut String, previous_was_separator: &mut bool, run: &str) {
    if run.is_empty() {
        return;
    }

    let split_camel_case = !is_pathological_mixed_case(run);
    let mut previous_was_lowercase_or_digit = false;

    for character in run.chars() {
        if split_camel_case && character.is_ascii_uppercase() && previous_was_lowercase_or_digit {
            push_separator(normalized, previous_was_separator);
        }

        normalized.push(character.to_ascii_lowercase());
        *previous_was_separator = false;
        previous_was_lowercase_or_digit =
            character.is_ascii_lowercase() || character.is_ascii_digit();
    }
}

fn is_pathological_mixed_case(value: &str) -> bool {
    let mut uppercase_count = 0usize;
    let mut lowercase_count = 0usize;
    let mut current_lowercase_run = 0usize;
    let mut max_lowercase_run = 0usize;
    let mut case_transitions = 0usize;
    let mut previous_was_uppercase = None;

    for character in value
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
    {
        let is_uppercase = character.is_ascii_uppercase();
        if is_uppercase {
            uppercase_count += 1;
            current_lowercase_run = 0;
        } else {
            lowercase_count += 1;
            current_lowercase_run += 1;
            max_lowercase_run = max_lowercase_run.max(current_lowercase_run);
        }

        if previous_was_uppercase.is_some_and(|previous| previous != is_uppercase) {
            case_transitions += 1;
        }
        previous_was_uppercase = Some(is_uppercase);
    }

    uppercase_count >= 2 && lowercase_count >= 2 && max_lowercase_run <= 1 && case_transitions >= 3
}

pub fn normalize(value: &str) -> String {
    normalize_query(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(query: &str, crate_name: Option<&str>) -> ApiLookup {
        lookup_api(query, crate_name)
            .unwrap_or_else(|error| panic!("query '{query}' should resolve: {error:?}"))
    }

    fn assert_primary_symbol(
        query: &str,
        crate_name: Option<&str>,
        expected_name: &str,
        expected_match_kind: ApiMatchKind,
    ) {
        let lookup = lookup(query, crate_name);

        assert_eq!(lookup.status, ApiLookupStatus::Found);
        let primary = lookup
            .primary
            .as_ref()
            .unwrap_or_else(|| panic!("query '{query}' should have a primary match"));
        assert_eq!(primary.match_kind, expected_match_kind);
        assert_eq!(
            primary.item.symbol_name(),
            Some(expected_name),
            "query '{query}' should resolve to symbol '{expected_name}'"
        );
    }

    fn assert_primary_concept(
        query: &str,
        crate_name: Option<&str>,
        expected_id: &str,
        expected_related_symbol: &str,
    ) {
        let lookup = lookup(query, crate_name);

        assert_eq!(lookup.status, ApiLookupStatus::Found);
        let concept = lookup
            .primary_concept()
            .unwrap_or_else(|| panic!("query '{query}' should resolve to a concept"));
        assert_eq!(concept.id, expected_id);
        assert!(concept.related_symbols.contains(&expected_related_symbol));
    }

    fn assert_ambiguous_identities(query: &str, crate_name: Option<&str>, expected: &[&str]) {
        let lookup = lookup(query, crate_name);
        let identities = lookup
            .matches
            .iter()
            .map(|match_| match_.item.identity())
            .collect::<Vec<_>>();

        assert_eq!(lookup.status, ApiLookupStatus::Ambiguous);
        assert_eq!(identities, expected);
    }

    fn assert_not_found_with_suggestions(query: &str, crate_name: Option<&str>) {
        let lookup = lookup(query, crate_name);

        assert_eq!(lookup.status, ApiLookupStatus::NotFound);
        assert!(lookup.primary.is_none());
        assert!(!lookup.suggestions.is_empty());
        assert!(!lookup.guidance.is_empty());
    }

    #[test]
    fn normalization_tokenizes_rust_symbols_macros_and_phrases() {
        assert_eq!(
            normalize("leptos_axum::ResponseOptions"),
            "leptos-axum-response-options"
        );
        assert_eq!(normalize("#[server]"), "server");
        assert_eq!(
            normalize("server function handler"),
            "server-function-handler"
        );
        assert_eq!(normalize("aXuM StAtE"), "axum-state");
        assert_eq!(normalize("ServerFnError"), "server-fn-error");

        assert_eq!(
            query_tokens("#[server] leptos_axum::ResponseOptions as"),
            vec!["server", "leptos", "axum", "response", "options"]
        );
        assert_eq!(query_tokens_with_min_len("a api fn", 3), vec!["api"]);
    }

    #[test]
    fn lookup_builds_index_once_with_expected_exact_keys() {
        let first = api_index();
        let second = api_index();

        assert!(std::ptr::eq(first, second));
        assert!(first.exact_terms.contains_key("macro:view"));
        assert!(first.exact_terms.contains_key("attribute:component"));
        assert!(first.exact_terms.contains_key("call:signal"));
    }

    #[test]
    fn lookup_finds_existing_exact_fully_qualified_symbols() {
        assert_primary_symbol(
            "leptos::prelude::Resource",
            None,
            "leptos::prelude::Resource",
            ApiMatchKind::ExactSymbol,
        );
        assert_primary_symbol(
            "leptos::server_fn::ServerFnError",
            None,
            "leptos::server_fn::ServerFnError",
            ApiMatchKind::ExactSymbol,
        );
        assert_primary_symbol(
            "leptos_axum::LeptosRoutes",
            None,
            "leptos_axum::LeptosRoutes",
            ApiMatchKind::ExactSymbol,
        );
        assert_primary_symbol(
            "axum::Router",
            None,
            "axum::Router",
            ApiMatchKind::ExactSymbol,
        );
    }

    #[test]
    fn lookup_finds_symbols_by_aliases_with_normalized_spelling() {
        assert_primary_symbol(
            "#[server]",
            None,
            "leptos::server",
            ApiMatchKind::AttributeMacro,
        );
        assert_primary_symbol(
            "server fn error",
            None,
            "leptos::server_fn::ServerFnError",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_symbol(
            "route_layer",
            None,
            "axum::middleware",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_symbol(
            "signal()",
            Some("leptos"),
            "leptos::prelude::signal",
            ApiMatchKind::FunctionCall,
        );
    }

    #[test]
    fn lookup_resolves_recent_test_drive_queries_usefully() {
        assert_primary_concept("component", None, "leptos-components", "leptos::component");
        assert_primary_concept("signal", None, "leptos-signals", "leptos::prelude::signal");
        assert_primary_symbol("view!", None, "leptos::prelude::view", ApiMatchKind::Macro);
        assert_primary_symbol(
            "IntoView",
            None,
            "leptos::prelude::IntoView",
            ApiMatchKind::ExactAlias,
        );
    }

    #[test]
    fn lookup_normalizes_case_and_rust_marker_forms() {
        assert_primary_symbol("VIEW!", None, "leptos::prelude::view", ApiMatchKind::Macro);
        assert_primary_symbol(
            "#[Component]",
            None,
            "leptos::component",
            ApiMatchKind::AttributeMacro,
        );
        assert_primary_symbol(
            "into view",
            None,
            "leptos::prelude::IntoView",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_concept(
            "Reactive State",
            Some("leptos"),
            "leptos-signals",
            "leptos::prelude::signal",
        );
    }

    #[test]
    fn lookup_applies_crate_filter_before_exact_and_fuzzy_matching() {
        assert_primary_symbol(
            "State",
            Some("axum"),
            "axum::extract::State",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_symbol(
            "extract",
            Some("leptos_axum"),
            "leptos_axum::extract",
            ApiMatchKind::ExactAlias,
        );

        let lookup = lookup("Path", Some("leptos"));
        assert_eq!(lookup.status, ApiLookupStatus::NotFound);
        assert_eq!(lookup.crate_filter.as_deref(), Some("leptos"));
    }

    #[test]
    fn lookup_keeps_prefix_like_terms_distinct_from_short_ambiguous_terms() {
        assert_primary_symbol(
            "Resource::new",
            None,
            "leptos::prelude::Resource",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_symbol(
            "extract",
            None,
            "leptos_axum::extract",
            ApiMatchKind::ExactAlias,
        );
        assert_primary_symbol(
            "leptos_axum::extract_with",
            None,
            "leptos_axum::extract_with_state",
            ApiMatchKind::Prefix,
        );
        assert_ambiguous_identities(
            "extracto",
            None,
            &[
                "axum::extract::Path",
                "axum::extract::Query",
                "axum::extract::State",
                "axum::Json",
                "leptos_axum::extract",
                "leptos_axum::extract_with_state",
            ],
        );
    }

    #[test]
    fn lookup_reports_ambiguous_terms_as_structured_matches() {
        assert_ambiguous_identities(
            "extractor",
            None,
            &[
                "axum::extract::Path",
                "axum::extract::Query",
                "axum::extract::State",
                "axum::Json",
                "leptos_axum::extract",
                "leptos_axum::extract_with_state",
            ],
        );

        assert_primary_symbol(
            "response",
            None,
            "axum::response::IntoResponse",
            ApiMatchKind::ExactAlias,
        );
        assert_ambiguous_identities(
            "error",
            None,
            &[
                "leptos::server_fn::ServerFnError",
                "leptos_axum::file_and_error_handler",
                "axum::response::IntoResponse",
            ],
        );
    }

    #[test]
    fn lookup_returns_actionable_not_found_for_unknown_or_noisy_queries() {
        assert_not_found_with_suggestions("a", None);
        assert_not_found_with_suggestions("???", None);
        assert_not_found_with_suggestions("nope", Some("leptos"));
    }

    #[test]
    fn lookup_rejects_empty_query() {
        let error = lookup_symbol(" ", None).expect_err("empty query should fail");

        assert_eq!(error, ApiLookupError::Empty);
    }

    #[test]
    fn api_symbol_urls_use_owned_docs_urls_for_shared_crates() {
        for symbol in all_symbols() {
            let expected_docs_url = match symbol.crate_name {
                "leptos" => LEPTOS_DOCS_URL,
                "leptos_axum" => LEPTOS_AXUM_DOCS_URL,
                "axum" => AXUM_DOCS_URL,
                other => panic!("unexpected curated crate '{other}'"),
            };

            assert!(
                symbol.url.starts_with(expected_docs_url),
                "API symbol '{}' URL should be under owned docs URL for crate '{}'",
                symbol.name,
                symbol.crate_name
            );
        }
    }

    #[test]
    fn concept_related_symbols_reference_curated_symbols() {
        let symbol_names = all_symbols()
            .iter()
            .map(|symbol| symbol.name)
            .collect::<Vec<_>>();

        for concept in all_concepts() {
            assert!(
                !concept.related_symbols.is_empty(),
                "concept '{}' should guide agents toward exact API entries",
                concept.id
            );
            for related_symbol in concept.related_symbols {
                assert!(
                    symbol_names.contains(related_symbol),
                    "concept '{}' references missing symbol '{}'",
                    concept.id,
                    related_symbol
                );
            }
        }
    }

    #[test]
    fn owned_docs_urls_follow_catalog_pin_policy() {
        let expectations = [
            (
                "leptos",
                LEPTOS_VERSION,
                LEPTOS_DOCS_URL,
                "/latest/leptos/".to_string(),
            ),
            (
                "leptos_axum",
                LEPTOS_AXUM_VERSION,
                LEPTOS_AXUM_DOCS_URL,
                "/latest/leptos_axum/".to_string(),
            ),
            (
                "axum",
                AXUM_VERSION,
                AXUM_DOCS_URL,
                format!("/{AXUM_VERSION}/axum/"),
            ),
        ];

        for (crate_name, version, docs_url, expected_path) in expectations.iter() {
            assert!(
                docs_url.contains(expected_path.as_str()),
                "owned docs URL for crate '{crate_name}' must follow the catalog pin policy for version '{version}'; expected URL containing '{expected_path}', got '{docs_url}'"
            );
        }
    }

    #[test]
    fn api_rust_snippet_inventory_exposes_current_classifications() {
        let snippets = rust_api_snippets();

        assert_eq!(snippets.len(), 22);
        assert_eq!(
            snippets
                .iter()
                .filter(|snippet| snippet.classification == SnippetClassification::CompileCandidate)
                .count(),
            4,
            "only complete API examples supported by the shared harness should be compile candidates"
        );
        assert_eq!(
            snippets
                .iter()
                .filter(|snippet| snippet.classification == SnippetClassification::Illustrative)
                .count(),
            18
        );
    }
}
