# Axum 0.8.9 for Leptos Servers

Axum provides the HTTP routing, handlers, extractors, responses, middleware, and
shared state around a Leptos SSR app.

## Router and Handlers

Build explicit API routes beside generated Leptos routes.

```rust
use axum::{routing::get, Router};

async fn health() -> &'static str {
    "ok"
}

let app = Router::new()
    .route("/api/health", get(health));
```

Handlers are async functions that receive zero or more extractors and return a
type that implements `IntoResponse`.

## Extractors

Use `Path`, `Query`, and `Json` for ordinary Axum API handlers.

```rust
use axum::{
    extract::{Path, Query},
    Json,
};

async fn get_user(
    Path(id): Path<String>,
    Query(params): Query<UserQuery>,
) -> Json<UserDto> {
    Json(load_user(id, params).await)
}
```

Inside Leptos server functions, prefer `leptos_axum::extract()` for request
parts and avoid body-consuming extractors because server function arguments
already use the request body.

## Shared State

Use `Router::with_state` and the `State` extractor for type-safe shared state.
Use `FromRef` when handlers need a smaller substate.

```rust
use axum::{
    extract::{FromRef, State},
    routing::get,
    Router,
};

#[derive(Clone)]
struct AppState {
    pool: DbPool,
}

#[derive(Clone)]
struct DbPool;

impl FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

async fn handler(State(pool): State<DbPool>) {
    let _ = pool;
}

let app = Router::new()
    .route("/api/items", get(handler))
    .with_state(AppState { pool });
```

## Responses and Errors

Return values that implement `IntoResponse`. Model API errors explicitly so
HTTP status codes are not hidden in handler bodies.

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

enum ApiError {
    NotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found").into_response(),
        }
    }
}
```

## Middleware

Axum middleware is Tower-based. Use `route_layer` for middleware that should
only affect existing routes, or `layer` when it should wrap all routes and
fallbacks.

```rust
use axum::{middleware, Router};

let app = Router::new()
    .route_layer(middleware::from_fn(auth));
```

## Testing

Use `tower::ServiceExt` in application tests to exercise the router without
opening a network socket.
