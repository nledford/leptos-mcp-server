# Leptos Axum Integration

`leptos_axum` 0.8.9 connects Leptos routing, rendering, server functions,
extractors, response options, and static file/error handling to Axum 0.8.9.

## SSR Route Wiring

Generate routes from the Leptos router tree and attach them to Axum:

```rust
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

let conf = get_configuration(None)?;
let leptos_options = conf.leptos_options;
let routes = generate_route_list(App);

let app = Router::new()
    .leptos_routes(&leptos_options, routes, {
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options.clone())
    });
```

Use `leptos_routes_with_context` when server functions or rendering need
context such as a database pool.

## Server Function Route

Server functions are public HTTP endpoints. Register a handler at the same API
prefix used by the `#[server]` macro.

```rust
use axum::{routing::post, Router};
use leptos_axum::handle_server_fns;

let app = Router::new()
    .route("/api/{*fn_name}", post(handle_server_fns));
```

## Extractors in Server Functions

Use `extract()` for Axum request-parts extractors such as `Query`, `Path`, and
headers. Use `extract_with_state()` for extractors that need Axum `State`.

```rust
use axum::extract::{Query, State};
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos_axum::{extract, extract_with_state};

#[server(Search)]
pub async fn search() -> Result<Vec<ItemDto>, ServerFnError> {
    let Query(params): Query<SearchParams> = extract().await?;
    Ok(run_search(params).await?)
}

#[server(UseState)]
pub async fn use_state() -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let State(app_state): State<AppState> = extract_with_state(&state).await?;
    Ok(())
}
```

Do not use `extract()` for body-consuming extractors such as JSON request
bodies. Server function arguments already consume the request body.

## Response Options

`ResponseOptions` lets SSR code and server functions set response status,
headers, and cookies.

```rust
use axum::http::StatusCode;
use leptos::prelude::*;
use leptos_axum::ResponseOptions;

let response = expect_context::<ResponseOptions>();
response.set_status(StatusCode::NOT_FOUND);
```

## Static Files and Error Fallback

`file_and_error_handler` is a convenience fallback for static files like
JS/WASM/CSS and 404 pages. Keep explicit API routes and Leptos routes ahead of
the fallback.

```rust
use leptos_axum::file_and_error_handler;

let app = Router::new()
    .route("/api/{*fn_name}", post(handle_server_fns))
    .leptos_routes(&leptos_options, routes, app)
    .fallback(file_and_error_handler(shell));
```

## Runtime Feature Choice

For normal Tokio/Axum servers, keep `leptos_axum` default features enabled.
Only use `default-features = false` together with `features = ["wasm"]` when
targeting JavaScript Fetch runtimes such as Deno or Workers.
