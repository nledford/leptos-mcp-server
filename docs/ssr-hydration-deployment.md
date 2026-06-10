# SSR, Hydration, and Deployment

Leptos full-stack apps usually build two targets: a server binary with `ssr`
enabled and a browser WASM package with `hydrate` enabled.

## Feature Flags

Keep server and browser rendering modes separate.

```toml
[features]
hydrate = ["leptos/hydrate"]
ssr = ["leptos/ssr", "leptos_axum/default"]
```

The server binary should compile with `ssr`. The browser package should compile
with `hydrate`. Do not enable both modes in one target unless you have a very
specific reason and tests for it.

## Hydration Contract

The HTML generated on the server must match what the browser expects during
hydration.

Common causes of hydration mismatch:

1. Reading browser-only APIs during SSR.
2. Rendering time, random values, or environment-specific content directly.
3. Using `LocalResource` where SSR serialization is required.
4. Missing generated JS/WASM/CSS assets.

## Static Assets

Generated package assets must be served before the app fallback. With
`leptos_axum`, use the documented file/error fallback or a custom equivalent
when deployment needs more control.

```rust
use leptos_axum::file_and_error_handler;

let app = Router::new()
    .route("/api/{*fn_name}", post(handle_server_fns))
    .leptos_routes(&leptos_options, routes, app)
    .fallback(file_and_error_handler(shell));
```

## Deployment Checks

Before deploying:

1. Run the server binary with the same feature flags used in production.
2. Confirm generated `/pkg` assets are reachable.
3. Confirm one SSR route returns HTML before hydration.
4. Confirm one server function call reaches the configured API prefix.
5. Confirm 404 and error pages set appropriate status codes.

## Debugging Hydration Failures

When hydration fails, inspect in this order:

1. Browser console for asset or WASM load failures.
2. Network tab for `/pkg` and server function route status.
3. Server logs for panics during route render.
4. Resource constructors: use `Resource` for SSR-serialized data and
   `LocalResource` for browser-only data.
5. Feature flags: server target should use `ssr`; browser target should use
   `hydrate`.
