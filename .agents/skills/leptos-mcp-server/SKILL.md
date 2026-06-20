---
name: leptos-mcp-server
description: Use the Leptos MCP server for Leptos 0.8, leptos_axum, Axum 0.8 SSR, server functions, hydration, forms/actions, SQL guidance, and heuristic Leptos diagnostics when an agent needs curated local MCP docs, recipes, API lookups, resources, or workflow prompts.
---

# Leptos MCP Server

## Overview

Use this skill when a coding task involves Leptos application development and an MCP client can connect to `leptos-mcp-server`. The server is a local stdio MCP server that embeds curated Leptos, `leptos_axum`, and Axum documentation and exposes it through MCP tools, resources, and prompts. It does not fetch live docs, run code, inspect projects, or connect to databases.

Detailed reference:

- [Setup and connection](references/setup.md)
- [Tools, resources, and prompts](references/tools.md)
- [Recommended workflows and examples](references/examples.md)
- [Troubleshooting](references/troubleshooting.md)

## When to use this MCP server

Use it for Leptos 0.8+ work when you need one of these grounded, local aids:

- Find or retrieve curated docs for signals, components, views, resources, actions, server functions, routing, forms, error handling, suspense, Leptos Axum, Axum 0.8.9, or SSR/hydration/deployment.
- Look up curated API symbols, macro forms, aliases, and Leptos concepts for
  `leptos`, `leptos_axum`, or `axum` before coding.
- Get task recipes for Leptos + Axum SSR, server functions, static assets, custom Axum handlers, shared state/context, database query patterns, or JS-hosted wasm runtimes.
- Run heuristic diagnostics on pasted Leptos/Rust-like code for common Leptos and Leptos Axum issues.
- Render workflow prompts for SSR wiring, server functions, SQL access review, hydration debugging, or Axum integration review.

## When not to use it

Do not use this MCP server as a substitute for:

- Normal repo/file tools for project-specific search, edits, test discovery, or reading user code.
- `cargo check`, `cargo test`, `cargo clippy`, formatters, or real compiler diagnostics.
- Live/authoritative documentation when the embedded snapshot may be stale.
- Database introspection, migrations, query execution, schema management, or secret retrieval.
- General Rust, frontend, or non-Leptos framework questions outside the curated catalog.

## Prerequisites and setup

- Rust 1.96+ and Cargo.
- Build the binary with `cargo build --release --locked`.
- Configure the MCP client to launch the absolute path to `target/release/leptos-mcp-server`.
- No project-specific environment variables, credentials, tokens, API keys, local services, or databases are required.
- Optional `RUST_LOG` controls stderr logging; stdout is reserved for JSON-RPC.

See [setup.md](references/setup.md) for Claude Desktop, Antigravity, OpenCode, smoke tests, and skill-install notes.

## Available capabilities

Tools:

- `list-sections` — list embedded documentation sections and metadata.
- `get-documentation` — fetch one section by canonical id or declared alias.
- `search-docs` — search docs by task, API, or failure mode.
- `lookup-api` — look up curated Leptos, `leptos_axum`, or Axum public API
  symbols, macro forms, aliases, and concepts.
- `leptos-axum-recipe` — return task recipes with steps, example files, and validation.
- `leptos-diagnostics` — analyze provided code and return structured heuristic diagnostics.

Resources use `leptos://docs/<section-id>` URIs and expose the template `leptos://docs/{section}`. Prompts include `wire-leptos-axum-ssr`, `add-server-function`, `review-sql-access`, `debug-hydration`, and `review-axum-integration`.

Use [tools.md](references/tools.md) for exact names, schemas, output shapes, resources, prompts, and known error messages.

## Recommended agent workflows

1. Start broad with `search-docs` for task/error wording, or `list-sections` when you need canonical ids.
2. Use `get-documentation` or `resources/read` for the selected section before changing code.
3. Use `lookup-api` before writing Leptos, `leptos_axum`, or Axum API calls
   from memory. It accepts exact symbols such as `leptos::prelude::Resource`,
   aliases such as `Resource::new`, macro/attribute forms such as `view!` and
   `#[component]`, trait/type names such as `IntoView`, and concepts such as
   `component` or `signal`.
4. Use `leptos-axum-recipe` for multi-step SSR/server-function/state/static-asset workflows.
5. Use prompts to frame reviews or debugging plans; provide code/symptoms explicitly.
6. Use `leptos-diagnostics` only on relevant snippets and still verify with normal Rust tooling.

## Input/output guidance

- Tool schemas are strict; omit extra fields.
- `get-documentation` requires a canonical section id or declared alias; arbitrary substring matching is intentionally rejected.
- `leptos-diagnostics.code` must be non-empty and no more than 262144 bytes.
- Successful tool calls return text plus `structuredContent`; SDK tool error results return `isError: true` with text only.
- `lookup-api` returns successful structured results for exact matches,
  concept matches, ambiguous matches, and unknown queries. Check
  `structuredContent.lookup.status`, which is one of `found`, `ambiguous`, or
  `not-found`. Unknown queries include `suggestions`, `guidance`, and, when the
  term appears in embedded docs, `documentation_matches` rather than a bare
  `Unknown API symbol` tool error. A blank query is still a tool error.
- Prompt calls enforce required arguments; missing or blank required values and unknown prompt arguments return prompt argument errors.

## Security and privacy considerations

- The server is documentation-only/read-only: no filesystem writes, command execution, runtime network fetches, database connections, migrations, or query execution.
- User-supplied code/text passed to diagnostics or prompts is echoed in MCP responses and may be logged by the MCP client. Do not send secrets, tokens, credentials, or sensitive proprietary code unless the client environment is trusted.
- SQL guidance is for application code patterns only; it does not grant database access.
- Stdio framing and malformed-input behavior are SDK-native; diagnostics code is capped at 256 KiB.

## Verification checklist

Before relying on this MCP server in a task:

- Confirm the MCP client lists the six tools above.
- Call `list-sections` or `resources/list` to verify the embedded docs are discoverable.
- For the target task, retrieve at least one relevant doc section or recipe before coding.
- If generated code changes behavior, run project-local checks/tests after applying changes.
- If the needed answer is outside the embedded Leptos/Axum scope or needs latest upstream docs, use an appropriate live-docs/source tool instead.
