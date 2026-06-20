# Recommended workflows and examples

These examples show agent decisions. Use your MCP client's native tool-call UI when available; raw JSON-RPC examples are only for smoke testing.

## Workflow: implement or review Leptos signal/view code

1. Call `search-docs` with a task query such as `signal read in view`.
2. Call `get-documentation` for the best section, commonly `signals` or `views`.
3. Call `lookup-api` for broad or exact terms as needed: `signal` for the
   concept entry, `signal()` for the function entry, `view!` for the macro, and
   `IntoView` for component return types.
4. If the user supplied a snippet, call `leptos-diagnostics` with the relevant code only.
5. Apply code changes with normal repo-editing tools.
6. Verify with project-local Rust checks/tests.

Example diagnostic input:

```json
{
  "code": "fn App() -> impl IntoView { view! { <p>{count.get()}</p> } }"
}
```

Expected use: treat diagnostics as heuristic guidance. If it flags `leptos.signal-get-in-view`, change the view to read reactively through a closure, then run compiler/tests.

## Workflow: wire Leptos SSR with Axum

1. Call `leptos-axum-recipe` with `ssr-app`.
2. Call `lookup-api` for `generate_route_list`, `LeptosRoutes`, and `Router` if you need API snippets.
3. Optionally call `prompts/get` for `wire-leptos-axum-ssr` with `app_name` and `state` to produce a review checklist.
4. Edit the project using normal repo tools.
5. Verify by building/running the app and checking SSR HTML plus hydration assets.

Raw tool call shape:

```json
{
  "name": "leptos-axum-recipe",
  "arguments": { "recipe": "ssr-app" }
}
```

## Workflow: add a server function

1. Call `search-docs` with `server function ActionForm Resource ServerFnError`.
2. Call `get-documentation` for `server-functions`, then `actions` or `forms` as needed.
3. Call `lookup-api` for `#[server]`, `ServerFnError`, and `ActionForm` or `Resource`.
4. Render `add-server-function` prompt with a concrete `operation` and optional DTO/validation notes.
5. Implement server/client boundaries; never leak server-only data to client DTOs.
6. Run project checks.

Prompt example:

```json
{
  "name": "add-server-function",
  "arguments": {
    "operation": "create a todo item",
    "data": "title is required, return TodoDto"
  }
}
```

## Workflow: create or review components

1. Call `lookup-api` with `component` for the concept entry. Use
   `related_sections` to retrieve `components` and `views`.
2. Call `lookup-api` with `#[component]` when you need the attribute macro, and
   `IntoView` when you need the return trait.
3. Call `lookup-api` with `view!` when checking template syntax or reactive view
   behavior.
4. If `lookup.status` is `ambiguous`, inspect `matches` and refine with a fully
   qualified symbol, macro form, or `crate` filter. If it is `not-found`,
   inspect `suggestions`, `guidance`, and `documentation_matches`; use the
   matched docs before falling back to live upstream references.

## Workflow: review SQL access in Leptos/Axum

Use this server for guidance and review framing only; it cannot connect to the database.

1. Call `leptos-axum-recipe` with `database-query-patterns`.
2. Call `get-documentation` for `server-functions`, `leptos-axum`, `axum`, and `error-handling` as needed.
3. Render `review-sql-access` with the code under review and backend.
4. Check for cloneable pool handles in Axum state/Leptos context, bind parameters, SeaQuery only for dynamic shapes, transactions for multi-step writes, and user-safe `ServerFnError` mapping.
5. Verify with project database tests or sqlx preparation checks when available.

Prompt example:

```json
{
  "name": "review-sql-access",
  "arguments": {
    "backend": "SQLite",
    "code": "#[server] pub async fn get_item(id: i64) -> Result<...> { ... }"
  }
}
```

## Workflow: debug hydration or static assets

1. Call `search-docs` for the symptom, such as `WASM 404` or `hydration mismatch`.
2. Call `get-documentation` for `ssr-hydration-deployment`, `leptos-axum`, or `resources`.
3. Render `debug-hydration` with `symptom` and deployment/build environment.
4. Inspect project config, generated `/pkg` assets, Axum fallback ordering, SSR/hydrate feature flags, and browser/network errors with normal tools.

Prompt example:

```json
{
  "name": "debug-hydration",
  "arguments": {
    "symptom": "GET /pkg/app_bg.wasm returns HTML",
    "environment": "Axum SSR behind reverse proxy"
  }
}
```

## Workflow: decide not to use the MCP server

Use normal tools instead when:

- The user asks to find a project-specific component, route, migration, or test file.
- The task is to run `cargo test`, inspect build errors, or change repository code.
- The issue is in a non-Leptos framework or an unsupported library.
- The answer requires current upstream API changes beyond the embedded docs snapshot.

In those cases, inspect the repo or live docs directly, and only return to this MCP server for Leptos/Axum-specific reference guidance.
