//! Curated public API catalog for Leptos application development.

use crate::docs::SnippetClassification;
use serde::Serialize;

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

pub fn all_symbols() -> &'static [ApiSymbol] {
    API_SYMBOLS
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

pub fn lookup_symbol(
    query: &str,
    crate_name: Option<&str>,
) -> Result<&'static ApiSymbol, ApiLookupError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        if !query.trim().is_empty() {
            return Err(ApiLookupError::Unknown {
                query: query.to_string(),
                crate_name: crate_name.map(str::to_string),
            });
        }

        return Err(ApiLookupError::Empty);
    }
    let normalized_crate = crate_name.map(normalize);
    let candidate_symbols: Vec<&ApiSymbol> = API_SYMBOLS
        .iter()
        .filter(|symbol| crate_matches(symbol, normalized_crate.as_deref()))
        .collect();

    let exact_matches: Vec<&ApiSymbol> = candidate_symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.matches_exact(&normalized_query))
        .collect();

    if !exact_matches.is_empty() {
        return resolve_lookup_tier(query, exact_matches);
    }

    if normalized_query.len() < MIN_QUERY_TOKEN_LEN {
        return Err(ApiLookupError::Unknown {
            query: query.to_string(),
            crate_name: crate_name.map(str::to_string),
        });
    }

    let prefix_matches: Vec<&ApiSymbol> = candidate_symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.matches_prefix(&normalized_query))
        .collect();

    if !prefix_matches.is_empty() {
        return resolve_lookup_tier(query, prefix_matches);
    }

    let tokens = query_tokens(&normalized_query);
    if tokens.is_empty() {
        return Err(ApiLookupError::Unknown {
            query: query.to_string(),
            crate_name: crate_name.map(str::to_string),
        });
    }

    let token_matches: Vec<&ApiSymbol> = candidate_symbols
        .iter()
        .copied()
        .filter(|symbol| symbol.matches_tokens(&tokens))
        .collect();

    if !token_matches.is_empty() {
        return resolve_lookup_tier(query, token_matches);
    }

    let summary_matches: Vec<&ApiSymbol> = if tokens.len() >= 2 {
        candidate_symbols
            .iter()
            .copied()
            .filter(|symbol| symbol.matches_summary_tokens(&tokens))
            .collect()
    } else {
        Vec::new()
    };

    if !summary_matches.is_empty() {
        return resolve_lookup_tier(query, summary_matches);
    }

    Err(ApiLookupError::Unknown {
        query: query.to_string(),
        crate_name: crate_name.map(str::to_string),
    })
}

fn resolve_lookup_tier(
    query: &str,
    matches: Vec<&'static ApiSymbol>,
) -> Result<&'static ApiSymbol, ApiLookupError> {
    match matches.as_slice() {
        [symbol] => Ok(*symbol),
        multiple => Err(ApiLookupError::Ambiguous {
            query: query.to_string(),
            matches: multiple
                .iter()
                .map(|symbol| symbol.name.to_string())
                .collect(),
        }),
    }
}

fn crate_matches(symbol: &ApiSymbol, normalized_crate: Option<&str>) -> bool {
    normalized_crate.is_none_or(|crate_name| normalize(symbol.crate_name) == crate_name)
}

impl ApiSymbol {
    fn matches_exact(&self, normalized_query: &str) -> bool {
        normalize(self.name) == normalized_query
            || self
                .aliases
                .iter()
                .any(|alias| normalize(alias) == normalized_query)
    }

    fn matches_prefix(&self, normalized_query: &str) -> bool {
        self.field_matches_prefix(normalized_query, self.name)
            || self
                .aliases
                .iter()
                .any(|alias| self.field_matches_prefix(normalized_query, alias))
    }

    fn matches_tokens(&self, normalized_tokens: &[String]) -> bool {
        self.matches_field_tokens(normalized_tokens, self.name)
            || self
                .aliases
                .iter()
                .any(|alias| self.matches_field_tokens(normalized_tokens, alias))
    }

    fn matches_summary_tokens(&self, normalized_tokens: &[String]) -> bool {
        self.matches_field_tokens(normalized_tokens, self.summary)
    }

    fn matches_field_tokens(&self, normalized_tokens: &[String], field: &str) -> bool {
        let field_tokens = query_tokens(field);

        !field_tokens.is_empty()
            && normalized_tokens.iter().all(|query_token| {
                field_tokens
                    .iter()
                    .any(|field_token| field_token.starts_with(query_token))
            })
    }

    fn field_matches_prefix(&self, normalized_query: &str, field: &str) -> bool {
        if !normalize(field).starts_with(normalized_query) {
            return false;
        }

        let query_token_count = query_tokens_with_min_len(normalized_query, 1).len();
        let field_token_count = query_tokens_with_min_len(field, 1).len();

        query_token_count > 1 || field_token_count == 1
    }
}

pub const MIN_QUERY_TOKEN_LEN: usize = 3;

pub fn normalize_query(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = true;
    let mut previous_was_lowercase_or_digit = false;

    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && previous_was_lowercase_or_digit {
                push_separator(&mut normalized, &mut previous_was_separator);
            }

            normalized.push(character.to_ascii_lowercase());
            previous_was_separator = false;
            previous_was_lowercase_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else {
            push_separator(&mut normalized, &mut previous_was_separator);
            previous_was_lowercase_or_digit = false;
        }
    }

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

pub fn normalize(value: &str) -> String {
    normalize_query(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_lookup_name(query: &str, crate_name: Option<&str>, expected_name: &str) {
        let symbol = lookup_symbol(query, crate_name)
            .unwrap_or_else(|error| panic!("query '{query}' should resolve: {error:?}"));

        assert_eq!(symbol.name, expected_name);
    }

    fn assert_ambiguous_names(query: &str, crate_name: Option<&str>, expected_names: &[&str]) {
        let error = lookup_symbol(query, crate_name)
            .expect_err(&format!("query '{query}' should be ambiguous"));

        let ApiLookupError::Ambiguous { matches, .. } = error else {
            panic!("query '{query}' should be ambiguous, got {error:?}");
        };

        assert_eq!(matches, expected_names);
    }

    fn assert_unknown(query: &str, crate_name: Option<&str>) {
        let error = lookup_symbol(query, crate_name)
            .expect_err(&format!("query '{query}' should be unknown"));

        assert_eq!(
            error,
            ApiLookupError::Unknown {
                query: query.to_string(),
                crate_name: crate_name.map(str::to_string),
            }
        );
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

        assert_eq!(
            query_tokens("#[server] leptos_axum::ResponseOptions as"),
            vec!["server", "leptos", "axum", "response", "options"]
        );
        assert_eq!(query_tokens_with_min_len("a api fn", 3), vec!["api"]);
    }

    #[test]
    fn lookup_finds_exact_symbol_with_crate_filter() {
        let symbol = lookup_symbol("file_and_error_handler", Some("leptos_axum"))
            .expect("static asset helper should resolve");

        assert_eq!(symbol.name, "leptos_axum::file_and_error_handler");
    }

    #[test]
    fn lookup_finds_exact_fully_qualified_symbols() {
        assert_lookup_name(
            "leptos::prelude::Resource",
            None,
            "leptos::prelude::Resource",
        );
        assert_lookup_name(
            "leptos_axum::extract_with_state",
            None,
            "leptos_axum::extract_with_state",
        );
        assert_lookup_name(
            "axum::response::IntoResponse",
            None,
            "axum::response::IntoResponse",
        );
    }

    #[test]
    fn lookup_finds_aliases_with_normalized_spelling() {
        assert_lookup_name("#[server]", None, "leptos::server");
        assert_lookup_name("server fn error", None, "leptos::server_fn::ServerFnError");
        assert_lookup_name("route_layer", None, "axum::middleware");
    }

    #[test]
    fn lookup_applies_crate_filter_before_exact_and_fuzzy_matching() {
        assert_lookup_name("State", Some("axum"), "axum::extract::State");
        assert_lookup_name("extract", Some("leptos_axum"), "leptos_axum::extract");

        let error =
            lookup_symbol("State", Some("leptos")).expect_err("crate filter excludes State");
        assert_eq!(
            error,
            ApiLookupError::Unknown {
                query: "State".to_string(),
                crate_name: Some("leptos".to_string()),
            }
        );
    }

    #[test]
    fn lookup_keeps_prefix_like_terms_distinct_from_short_ambiguous_terms() {
        assert_lookup_name("Resource::new", None, "leptos::prelude::Resource");
        assert_lookup_name("extract", None, "leptos_axum::extract");
        assert_lookup_name(
            "leptos_axum::extract_with",
            None,
            "leptos_axum::extract_with_state",
        );
        assert_ambiguous_names(
            "extracto",
            None,
            &[
                "leptos_axum::extract",
                "leptos_axum::extract_with_state",
                "axum::Json",
            ],
        );
    }

    #[test]
    fn lookup_reports_ambiguous_short_query() {
        let error = lookup_symbol("extractor", None).expect_err("extractor is ambiguous");

        assert!(matches!(error, ApiLookupError::Ambiguous { .. }));
    }

    #[test]
    fn lookup_reports_expected_ambiguous_terms_without_single_fuzzy_winner() {
        assert_ambiguous_names(
            "extractor",
            None,
            &[
                "leptos_axum::extract",
                "leptos_axum::extract_with_state",
                "axum::Json",
            ],
        );

        assert_lookup_name("response", None, "axum::response::IntoResponse");
        assert_ambiguous_names(
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
    fn lookup_handles_too_short_and_noisy_queries_without_overconfident_results() {
        assert_unknown("a", None);

        let error = lookup_symbol("???", None).expect_err("punctuation noise should not resolve");
        assert_eq!(
            error,
            ApiLookupError::Unknown {
                query: "???".to_string(),
                crate_name: None,
            }
        );
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
    fn api_rust_snippet_inventory_exposes_current_classifications() {
        let snippets = rust_api_snippets();

        assert_eq!(snippets.len(), 16);
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
            12
        );
    }
}
