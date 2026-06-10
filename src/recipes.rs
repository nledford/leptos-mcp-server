//! Task-oriented recipes for Leptos application workflows.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RecipeFile {
    pub path: &'static str,
    pub language: &'static str,
    pub content: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Recipe {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub aliases: &'static [&'static str],
    pub crates: &'static [&'static str],
    pub related_sections: &'static [&'static str],
    pub related_apis: &'static [&'static str],
    pub steps: &'static [&'static str],
    pub files: &'static [RecipeFile],
    pub validation: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeLookupError {
    Empty,
    Unknown { recipe: String },
}

static RECIPES: &[Recipe] = &[
    Recipe {
        id: "ssr-app",
        title: "Wire a Leptos SSR app with Axum",
        summary: "Generate Leptos routes, attach them to an Axum 0.8.9 Router, and serve SSR pages through leptos_axum.",
        aliases: &["ssr", "axum ssr", "full-stack", "leptos axum"],
        crates: &["leptos 0.8.19", "leptos_axum 0.8.9", "axum 0.8.9"],
        related_sections: &["leptos-axum", "ssr-hydration-deployment", "routing"],
        related_apis: &[
            "leptos_axum::generate_route_list",
            "leptos_axum::LeptosRoutes",
            "axum::Router",
        ],
        steps: &[
            "Enable the server build with Leptos ssr features and the browser build with hydrate features.",
            "Generate the Leptos route list from the root app component.",
            "Attach generated routes to Axum with leptos_routes or leptos_routes_with_context.",
            "Serve generated package assets from /pkg and run the router with axum::serve.",
        ],
        files: &[RecipeFile {
            path: "src/main.rs",
            language: "rust",
            content: r#"use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}"#,
        }],
        validation: &[
            "cargo leptos watch starts both server and client builds.",
            "A browser request to a Leptos route returns HTML before hydration.",
            "The generated route list includes all Router routes that should SSR.",
        ],
    },
    Recipe {
        id: "server-functions",
        title: "Register and call Leptos server functions",
        summary: "Expose Leptos #[server] functions through Axum and call them from Resources or ServerActions.",
        aliases: &["server fn", "handle_server_fns", "api route"],
        crates: &["leptos 0.8.19", "leptos_axum 0.8.9", "axum 0.8.9"],
        related_sections: &["server-functions", "leptos-axum", "actions", "forms"],
        related_apis: &["leptos::server", "leptos_axum::handle_server_fns"],
        steps: &[
            "Keep server functions async and return Result<T, ServerFnError>.",
            "Register the server-function handler under the same API prefix used by the #[server] macro.",
            "Use Resource for reads and ServerAction or ActionForm for mutations.",
        ],
        files: &[RecipeFile {
            path: "src/main.rs",
            language: "rust",
            content: r#"use axum::{routing::post, Router};
use leptos_axum::handle_server_fns;

let app = Router::new()
    .route("/api/{*fn_name}", post(handle_server_fns));"#,
        }],
        validation: &[
            "A server function call reaches the Axum route configured for the API prefix.",
            "Server function DTOs derive Serialize and Deserialize.",
            "Failures surface through ServerFnError and are handled by the caller.",
        ],
    },
    Recipe {
        id: "static-assets",
        title: "Serve generated Leptos package assets",
        summary: "Mount generated WASM, JS, CSS, and other package assets from the Leptos site root.",
        aliases: &["assets", "pkg", "wasm", "css", "site root"],
        crates: &["leptos_axum 0.8.9", "axum 0.8.9"],
        related_sections: &["ssr-hydration-deployment", "leptos-axum"],
        related_apis: &["leptos_axum::file_and_error_handler"],
        steps: &[
            "Use the site root from LeptosOptions so the server and cargo-leptos agree on paths.",
            "Mount /pkg before the SSR fallback so assets are served directly.",
            "Verify the generated WASM and CSS files are reachable in the browser network panel.",
        ],
        files: &[RecipeFile {
            path: "src/main.rs",
            language: "rust",
            content: r#"use leptos_axum::file_and_error_handler;

let app = Router::new()
    .fallback(file_and_error_handler(shell));"#,
        }],
        validation: &[
            "GET /pkg/<app>.js returns JavaScript.",
            "GET /pkg/<app>_bg.wasm returns WebAssembly bytes.",
            "Missing assets do not fall through to an SSR page.",
        ],
    },
    Recipe {
        id: "custom-handler",
        title: "Reserve custom Axum handlers alongside Leptos routes",
        summary: "Exclude paths from generated Leptos routes when an Axum handler should own them.",
        aliases: &["custom axum handler", "excluded route", "api handler"],
        crates: &["leptos_axum 0.8.9", "axum 0.8.9"],
        related_sections: &["leptos-axum", "axum", "routing"],
        related_apis: &[
            "leptos_axum::generate_route_list_with_exclusions",
            "axum::Router",
        ],
        steps: &[
            "List the Axum-owned paths in Axum path format.",
            "Generate Leptos routes with exclusions.",
            "Attach the explicit Axum route before the Leptos route integration.",
        ],
        files: &[RecipeFile {
            path: "src/main.rs",
            language: "rust",
            content: r#"let excluded = ["/api/health"];
let routes = generate_route_list_with_exclusions(App, excluded);

let app = Router::new()
    .route("/api/health", get(health))
    .leptos_routes(&leptos_options, routes, app);"#,
        }],
        validation: &[
            "The custom Axum route responds without being captured by Leptos routing.",
            "Leptos pages outside excluded routes still SSR.",
        ],
    },
    Recipe {
        id: "state-context",
        title: "Share Axum state with server functions",
        summary: "Use Axum State for handlers and provide Leptos context for server functions that need app state.",
        aliases: &["state", "context", "extract_with_state", "database pool"],
        crates: &["leptos 0.8.19", "leptos_axum 0.8.9", "axum 0.8.9"],
        related_sections: &["server-functions", "leptos-axum", "axum"],
        related_apis: &[
            "axum::extract::State",
            "leptos_axum::extract_with_state",
            "leptos_axum::LeptosRoutes",
        ],
        steps: &[
            "Keep shared state cheap to clone, usually with Arc or cloneable pool handles.",
            "Use Router::with_state for Axum handlers.",
            "Use leptos_routes_with_context or expect_context for server functions.",
            "Use extract_with_state when a server function must run an Axum extractor that depends on State.",
        ],
        files: &[RecipeFile {
            path: "src/main.rs",
            language: "rust",
            content: r#"let app_state = AppState { pool };
let app = Router::new()
    .leptos_routes_with_context(
        &leptos_options,
        routes,
        {
            let app_state = app_state.clone();
            move || provide_context(app_state.pool.clone())
        },
        app,
    )
    .with_state(app_state);"#,
        }],
        validation: &[
            "Axum handlers can extract State<AppState>.",
            "Server functions can retrieve provided context with expect_context.",
        ],
    },
    Recipe {
        id: "wasm-runtime",
        title: "Configure leptos_axum for JS-hosted WebAssembly runtimes",
        summary: "Use the leptos_axum wasm feature only when targeting JS Fetch runtimes such as Deno or Workers.",
        aliases: &["workers", "deno", "wasm feature", "fetch runtime"],
        crates: &["leptos_axum 0.8.9"],
        related_sections: &["leptos-axum", "ssr-hydration-deployment"],
        related_apis: &["leptos_axum"],
        steps: &[
            "Use native default features for normal Tokio/Axum deployments.",
            "Set default-features = false only when enabling the wasm feature for JS Fetch runtimes.",
            "Keep the server runtime target explicit in Cargo features.",
        ],
        files: &[RecipeFile {
            path: "Cargo.toml",
            language: "toml",
            content: r#"[dependencies]
leptos_axum = { version = "0.8.9", default-features = false, features = ["wasm"] }"#,
        }],
        validation: &[
            "Native Axum builds do not disable leptos_axum default features.",
            "JS-hosted runtime builds enable the wasm feature explicitly.",
        ],
    },
];

pub fn all_recipes() -> &'static [Recipe] {
    RECIPES
}

pub fn get_recipe(query: &str) -> Result<&'static Recipe, RecipeLookupError> {
    let normalized = normalize(query);
    if normalized.is_empty() {
        return Err(RecipeLookupError::Empty);
    }

    RECIPES
        .iter()
        .find(|recipe| recipe.matches(&normalized))
        .ok_or_else(|| RecipeLookupError::Unknown {
            recipe: query.to_string(),
        })
}

impl Recipe {
    fn matches(&self, normalized_query: &str) -> bool {
        normalize(self.id) == normalized_query
            || normalize(self.title) == normalized_query
            || self
                .aliases
                .iter()
                .any(|alias| normalize(alias) == normalized_query)
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '_'], "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_static_asset_recipe_by_alias() {
        let recipe = get_recipe("pkg").expect("pkg alias should resolve");

        assert_eq!(recipe.id, "static-assets");
    }

    #[test]
    fn every_recipe_has_validation_guidance() {
        for recipe in all_recipes() {
            assert!(!recipe.steps.is_empty());
            assert!(!recipe.validation.is_empty());
            assert!(!recipe.files.is_empty());
        }
    }
}
