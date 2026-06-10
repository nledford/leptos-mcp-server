//! Curated public API catalog for Leptos application development.

use serde::Serialize;

pub const LEPTOS_VERSION: &str = "0.8.19";
pub const LEPTOS_AXUM_VERSION: &str = "0.8.9";
pub const AXUM_VERSION: &str = "0.8.9";

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
        aliases: &["resource", "Resource", "Resource::new", "Resource::new_blocking", "refetch"],
        related_sections: &["resources", "suspense", "server-functions"],
        snippet: "let data = Resource::new(move || id.get(), |id| load_item(id));\nlet blocking = Resource::new_blocking(|| (), |_| load_initial_state());",
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
    },
    ApiSymbol {
        name: "leptos_axum::LeptosRoutes",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "trait",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/trait.LeptosRoutes.html",
        summary: "Extends Axum Router with Leptos route-list integration so app routes do not need wildcard duplication.",
        aliases: &["LeptosRoutes", "leptos_routes", "leptos_routes_with_context"],
        related_sections: &["leptos-axum", "ssr-hydration-deployment", "routing"],
        snippet: "Router::new()\n    .leptos_routes_with_context(&leptos_options, routes, provide_context, app)",
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
    },
    ApiSymbol {
        name: "leptos_axum::handle_server_fns",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.handle_server_fns.html",
        summary: "Axum handler for Leptos server function requests.",
        aliases: &["handle_server_fns", "server function handler", "server functions route"],
        related_sections: &["leptos-axum", "server-functions"],
        snippet: ".route(\"/api/{*fn_name}\", post(handle_server_fns))",
    },
    ApiSymbol {
        name: "leptos_axum::file_and_error_handler",
        crate_name: "leptos_axum",
        version: LEPTOS_AXUM_VERSION,
        kind: "function",
        url: "https://docs.rs/leptos_axum/latest/leptos_axum/fn.file_and_error_handler.html",
        summary: "Convenience Axum handler for serving static files such as JS/WASM/CSS and rendering 404 pages.",
        aliases: &["file_and_error_handler", "static assets", "pkg assets", "wasm css"],
        related_sections: &["ssr-hydration-deployment", "leptos-axum"],
        snippet: ".fallback(file_and_error_handler(shell))",
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
    },
];

pub fn all_symbols() -> &'static [ApiSymbol] {
    API_SYMBOLS
}

pub fn lookup_symbol(
    query: &str,
    crate_name: Option<&str>,
) -> Result<&'static ApiSymbol, ApiLookupError> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Err(ApiLookupError::Empty);
    }
    let normalized_crate = crate_name.map(normalize);

    let exact_matches: Vec<&ApiSymbol> = API_SYMBOLS
        .iter()
        .filter(|symbol| crate_matches(symbol, normalized_crate.as_deref()))
        .filter(|symbol| symbol.matches_exact(&normalized_query))
        .collect();

    match exact_matches.as_slice() {
        [symbol] => Ok(*symbol),
        [] => {
            let fuzzy_matches: Vec<&ApiSymbol> = API_SYMBOLS
                .iter()
                .filter(|symbol| crate_matches(symbol, normalized_crate.as_deref()))
                .filter(|symbol| symbol.matches_fuzzy(&normalized_query))
                .collect();

            match fuzzy_matches.as_slice() {
                [symbol] => Ok(*symbol),
                [] => Err(ApiLookupError::Unknown {
                    query: query.to_string(),
                    crate_name: crate_name.map(str::to_string),
                }),
                multiple => Err(ApiLookupError::Ambiguous {
                    query: query.to_string(),
                    matches: multiple
                        .iter()
                        .map(|symbol| symbol.name.to_string())
                        .collect(),
                }),
            }
        }
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

    fn matches_fuzzy(&self, normalized_query: &str) -> bool {
        normalize(self.name).contains(normalized_query)
            || normalize(self.summary).contains(normalized_query)
            || self
                .aliases
                .iter()
                .any(|alias| normalize(alias).contains(normalized_query))
    }
}

pub fn normalize(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace("::", "-")
        .replace([' ', '_', '#', '[', ']'], "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_exact_symbol_with_crate_filter() {
        let symbol = lookup_symbol("file_and_error_handler", Some("leptos_axum"))
            .expect("static asset helper should resolve");

        assert_eq!(symbol.name, "leptos_axum::file_and_error_handler");
    }

    #[test]
    fn lookup_reports_ambiguous_short_query() {
        let error = lookup_symbol("extractor", None).expect_err("extractor is ambiguous");

        assert!(matches!(error, ApiLookupError::Ambiguous { .. }));
    }

    #[test]
    fn lookup_rejects_empty_query() {
        let error = lookup_symbol(" ", None).expect_err("empty query should fail");

        assert_eq!(error, ApiLookupError::Empty);
    }
}
