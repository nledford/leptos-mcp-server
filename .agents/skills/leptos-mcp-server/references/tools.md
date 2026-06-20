# Tools, resources, prompts, schemas, and outputs

Protocol summary:

- Transport: stdio, newline-delimited JSON-RPC 2.0.
- MCP protocol version advertised by `initialize`: `2025-11-25`.
- Supported methods: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `resources/templates/list`, `completion/complete`, `prompts/list`, `prompts/get`.
- Tool arguments are strict; unknown fields are rejected.
- Invalid stdio JSON returns sanitized JSON-RPC parse errors. Valid JSON that is not a valid MCP client message returns sanitized invalid-request errors. This project no longer adds a custom 1 MiB request-line cap.

Successful tool calls return this outer shape:

```json
{
  "content": [{ "type": "text", "text": "..." }],
  "structuredContent": { "kind": "..." }
}
```

Prefer `structuredContent` for decisions and `content[0].text` for display.
SDK tool error results return `isError: true` with text content and no
`structuredContent`.

## Tools

### `list-sections`

Purpose: list all embedded documentation sections with canonical ids, aliases, versions, source metadata, task tags, related sections, and resource URIs.

Input schema:

```json
{
  "type": "object",
  "properties": {},
  "additionalProperties": false
}
```

Output shape:

```json
{
  "kind": "list-sections",
  "sections": [
    {
      "id": "signals",
      "title": "Signals",
      "path": "signals",
      "use_cases": "...",
      "aliases": ["signal", "reactivity", "state"],
      "leptos_version": "Leptos 0.8+",
      "source": "embedded project documentation",
      "source_path": "docs/signals.md",
      "reviewed_at": "2026-06-10",
      "resource_uri": "leptos://docs/signals",
      "crate_versions": [{ "name": "leptos", "version": "0.8.19", "docs_url": "..." }],
      "source_url": "...",
      "task_tags": ["reactivity"],
      "crate_apis": ["..."],
      "prerequisites": ["..."],
      "common_errors": ["..."],
      "related_sections": ["..."],
      "snippet_classification": "illustrative"
    }
  ]
}
```

Use first when you need exact section ids or aliases.

### `get-documentation`

Purpose: retrieve one embedded Markdown section by exact canonical id or declared alias. Partial substring lookup is intentionally rejected.

Input schema:

```json
{
  "type": "object",
  "required": ["section"],
  "additionalProperties": false,
  "properties": {
    "section": {
      "type": "string",
      "description": "Canonical section id or declared alias from list-sections"
    }
  }
}
```

Output shape:

```json
{
  "kind": "documentation",
  "section": { "id": "signals", "title": "Signals" },
  "content": "Markdown content..."
}
```

Common errors:

- `section must be a non-empty canonical id or alias`
- `Unknown documentation section`
- `Ambiguous documentation section. Matching sections: ...`

### `search-docs`

Purpose: search documentation by task, API, error, or workflow. Searches ids, titles, use cases, aliases, task tags, crate APIs, common errors, and content, then ranks matches.

Input schema:

```json
{
  "type": "object",
  "required": ["query"],
  "additionalProperties": false,
  "properties": {
    "query": {
      "type": "string",
      "description": "Task, API, error, or workflow to search for"
    }
  }
}
```

Output shape:

```json
{
  "kind": "search-docs",
  "query": "Axum state",
  "results": [
    {
      "section": { "id": "leptos-axum", "title": "Leptos Axum Integration" },
      "score": 42,
      "matched_fields": ["use_cases", "task_tags"],
      "why": "Matched Leptos Axum Integration for 'Axum state'",
      "next_actions": ["..."]
    }
  ]
}
```

Failure note: an empty query currently returns the section-empty message because the implementation reuses documentation lookup errors.

### `lookup-api`

Purpose: look up curated public API symbols, macro/attribute forms, aliases, and
concept entries for Leptos `0.8.19`, `leptos_axum` `0.8.9`, and Axum `0.8.9`.
Use it for exact symbols such as `leptos::prelude::Resource`, macro forms such
as `view!`, attribute forms such as `#[component]`, trait/type names such as
`IntoView`, and broad concepts such as `component` or `signal`.

Input schema:

```json
{
  "type": "object",
  "required": ["query"],
  "additionalProperties": false,
  "properties": {
    "query": {
      "type": "string",
      "description": "Symbol name, macro form, concept, or declared alias"
    },
    "crate": {
      "type": "string",
      "description": "Optional crate filter: leptos, leptos_axum, or axum"
    }
  }
}
```

Output shape:

```json
{
  "kind": "api-lookup",
  "query": "component",
  "lookup": {
    "query": "component",
    "normalized_query": "component",
    "crate_filter": null,
    "status": "found",
    "primary": {
      "match_kind": "concept",
      "score": 980,
      "matched": "component",
      "item": {
        "entry_type": "concept",
        "entry": {
          "id": "leptos-components",
          "title": "Leptos components",
          "crate_names": ["leptos"],
          "kind": "concept",
          "summary": "...",
          "aliases": ["component", "components", "props", "children"],
          "related_sections": ["components", "views"],
          "related_symbols": [
            "leptos::component",
            "leptos::prelude::IntoView",
            "leptos::prelude::view"
          ],
          "snippet": "..."
        }
      }
    },
    "matches": [{ "...": "same shape as primary" }],
    "suggestions": [],
    "guidance": ["..."]
  },
  "documentation_matches": [
    {
      "section": { "id": "suspense", "title": "Suspense and Transition" },
      "score": 42,
      "matched_fields": ["content"],
      "why": "...",
      "next_actions": ["..."]
    }
  ]
}
```

`lookup.status` values:

- `found` — one exact, alias, macro, concept, prefix, token, or summary match was
  selected. Use `lookup.primary`.
- `ambiguous` — multiple entries matched. Use `lookup.matches` and refine with a
  fully qualified symbol, macro/attribute form, or `crate` filter.
- `not-found` — no curated entry matched. Use `lookup.suggestions`,
  `lookup.guidance`, and `documentation_matches`; this is a successful tool
  result, not an SDK tool error.

`item.entry_type` is either `symbol` or `concept`. Symbol entries include
`name`, `crate_name`, `version`, `kind`, `url`, `summary`, `aliases`,
`related_sections`, `snippet`, and `snippet_classification`. Concept entries
include `id`, `title`, `crate_names`, `version_scope`, `kind`, `summary`,
`aliases`, `related_sections`, `related_symbols`, and `snippet`.

Curated symbols:

- `leptos::component`
- `leptos::prelude::view`
- `leptos::prelude::IntoView`
- `leptos::prelude::signal`
- `leptos::prelude::RwSignal`
- `leptos::prelude::Memo`
- `leptos::prelude::Resource`
- `leptos::server`
- `leptos::server_fn::ServerFnError`
- `leptos::form::ActionForm`
- `leptos_axum::LeptosRoutes`
- `leptos_axum::generate_route_list`
- `leptos_axum::handle_server_fns`
- `leptos_axum::file_and_error_handler`
- `leptos_axum::extract`
- `leptos_axum::extract_with_state`
- `leptos_axum::ResponseOptions`
- `axum::Router`
- `axum::extract::State`
- `axum::extract::Path`
- `axum::extract::Query`
- `axum::Json`
- `axum::response::IntoResponse`
- `axum::middleware`

Curated concepts:

- `leptos-components` via `component`, `components`, `props`, or `children`
- `leptos-signals` via `signal`, `signals`, `reactivity`, `reactive state`, or
  `state`

Useful query examples:

- Exact symbols: `leptos::prelude::Resource`,
  `leptos::server_fn::ServerFnError`, `leptos_axum::LeptosRoutes`,
  `axum::Router`
- Aliases: `Resource::new`, `server fn error`, `route_layer`
- Macro/attribute/function forms: `view!`, `#[component]`, `signal()`
- Concepts: `component`, `signal`
- Type/trait names: `IntoView`, `IntoResponse`

Common errors:

- `query must be a non-empty API symbol, macro, concept, or alias`

Unknown and ambiguous non-empty queries are represented in
`structuredContent.lookup.status` as `not-found` or `ambiguous`; they do not
return SDK tool errors. For `not-found` queries, `documentation_matches` contains
up to three embedded documentation search results for the query, so agents can
continue from local docs even when the exact API is not in the curated catalog.

### `leptos-axum-recipe`

Purpose: return a task-oriented recipe with steps, example files, related crates/sections/APIs, and validation checklist.

Input schema:

```json
{
  "type": "object",
  "required": ["recipe"],
  "additionalProperties": false,
  "properties": {
    "recipe": {
      "type": "string",
      "description": "Recipe id or alias such as ssr-app, server-functions, static-assets, custom-handler, state-context, database-query-patterns, or wasm-runtime"
    }
  }
}
```

Output shape:

```json
{
  "kind": "recipe",
  "recipe": {
    "id": "ssr-app",
    "title": "Wire a Leptos SSR app with Axum",
    "summary": "...",
    "aliases": ["ssr", "axum ssr"],
    "crates": ["leptos 0.8.19", "leptos_axum 0.8.9", "axum 0.8.9"],
    "related_sections": ["leptos-axum"],
    "related_apis": ["leptos_axum::LeptosRoutes"],
    "steps": ["..."],
    "files": [{ "path": "src/main.rs", "language": "rust", "content": "..." }],
    "validation": ["..."]
  }
}
```

Recipe ids:

- `ssr-app`
- `server-functions`
- `static-assets`
- `custom-handler`
- `state-context`
- `database-query-patterns`
- `wasm-runtime`

Common errors:

- `recipe must be a non-empty recipe id or alias`
- `Unknown Leptos Axum recipe`

### `leptos-diagnostics`

Purpose: run heuristic static diagnostics on supplied Leptos/Rust-like code. It does not compile or execute code.

Input schema:

```json
{
  "type": "object",
  "required": ["code"],
  "additionalProperties": false,
  "properties": {
    "code": {
      "type": "string",
      "description": "Leptos code to analyze",
      "maxLength": 262144
    }
  }
}
```

Output shape:

```json
{
  "kind": "diagnostics",
  "diagnostics": [
    {
      "rule_id": "leptos.signal-get-in-view",
      "severity": "warning",
      "message": "...",
      "span": { "line": 1, "column": 1 },
      "confidence": "medium",
      "suggested_fix": "..."
    }
  ],
  "summary": {
    "error_count": 0,
    "warning_count": 1,
    "info_count": 0
  }
}
```

Rules currently implemented:

- `leptos.signal-get-in-view`
- `leptos.signal-destructuring`
- `leptos.missing-component-attribute`
- `leptos.server-fn-error`
- `leptos.server-fn-async`
- `leptos.server-fn-generic`
- `leptos.server-fn-prefix`
- `leptos.server-fn-duplicate-path`
- `leptos-axum.extract-state`
- `leptos-axum.extract-body`
- `leptos.deprecated-create-signal`

Common errors:

- `code must be a non-empty string`
- `code must be at most 262144 bytes`

## Resources

`resources/list` exposes every embedded doc section as Markdown:

```json
{
  "resources": [
    {
      "uri": "leptos://docs/signals",
      "name": "Signals",
      "description": "state, reactivity, signals, derived, effects, get, set, read, write, update, always",
      "mimeType": "text/markdown"
    }
  ]
}
```

`resources/read` accepts:

```json
{ "uri": "leptos://docs/<section-id>" }
```

and returns:

```json
{
  "contents": [
    {
      "uri": "leptos://docs/signals",
      "mimeType": "text/markdown",
      "text": "# Signals\n\n..."
    }
  ]
}
```

Resource URIs:

- `leptos://docs/getting-started`
- `leptos://docs/components`
- `leptos://docs/signals`
- `leptos://docs/views`
- `leptos://docs/resources`
- `leptos://docs/actions`
- `leptos://docs/server-functions`
- `leptos://docs/routing`
- `leptos://docs/forms`
- `leptos://docs/error-handling`
- `leptos://docs/suspense`
- `leptos://docs/leptos-axum`
- `leptos://docs/axum`
- `leptos://docs/ssr-hydration-deployment`

`resources/templates/list` exposes the documentation URI template:

```json
{
  "resourceTemplates": [
    {
      "uriTemplate": "leptos://docs/{section}",
      "name": "leptos-doc-section",
      "description": "Leptos documentation section by canonical section id",
      "mimeType": "text/markdown"
    }
  ]
}
```

`completion/complete` supports canonical section completions for that resource
template:

```json
{
  "ref": { "type": "ref/resource", "uri": "leptos://docs/{section}" },
  "argument": { "name": "section", "value": "ax" }
}
```

Expected completion values include matching section ids such as `axum`.
Unsupported completion refs or argument names return method-not-found (`-32601`).

Common errors:

- `resources/read params are required`
- Unknown/wrong-prefix URIs return invalid params derived from documentation lookup errors.

## Prompts

`prompts/list` returns prompt names, descriptions, and argument metadata. `prompts/get` accepts:

```json
{
  "name": "debug-hydration",
  "arguments": { "symptom": "WASM 404" }
}
```

and returns:

```json
{
  "description": "...",
  "messages": [
    {
      "role": "user",
      "content": { "type": "text", "text": "rendered prompt" }
    }
  ]
}
```

Prompt names and arguments:

| Prompt | Use | Arguments |
| --- | --- | --- |
| `wire-leptos-axum-ssr` | Plan/review Axum 0.8.9-backed Leptos SSR wiring. | `app_name` optional, `state` optional |
| `add-server-function` | Add a Leptos server function and caller boundaries. | `operation` required, `data` optional |
| `review-sql-access` | Review sqlx or SeaQuery usage in Leptos/Axum code. | `code` required, `backend` optional |
| `debug-hydration` | Diagnose SSR/hydration or static asset failures. | `symptom` required, `environment` optional |
| `review-axum-integration` | Review Axum 0.8.9 routing/state/extractors/middleware/response handling in Leptos server code. | `code` required |

Prompt names normalize spaces/underscores to hyphens, so `review_sql_access` resolves to `review-sql-access`.

Required prompt arguments are enforced. Missing or blank required values and unknown prompt arguments return invalid-params prompt errors.

Common errors:

- `prompts/get params are required`
- `prompt name must be non-empty`
- `Unknown prompt`

## JSON-RPC error behavior

When the SDK emits a JSON-RPC error envelope, standard JSON-RPC error codes are
used:

- Parse error: `-32700`
- Invalid request: `-32600`
- Method not found: `-32601`
- Invalid params: `-32602`
- Internal error: `-32603`

Notifications without `id` generally produce no response.
Invalid raw JSON stdio frames return sanitized `-32700` parse-error envelopes
with `id: null`.
