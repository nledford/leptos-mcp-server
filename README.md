# Leptos MCP Server

An MCP (Model Context Protocol) server providing comprehensive Leptos documentation and code analysis tools for AI agents.

It runs as a stdio MCP server and speaks newline-delimited JSON-RPC 2.0. The
server embeds curated Leptos, `leptos_axum`, and Axum documentation, exposes it
as tools/resources, and provides workflow prompts for common Leptos + Axum work.

## Features

| Tool                  | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `list-sections`       | List all available documentation sections with task metadata      |
| `get-documentation`   | Retrieve documentation by canonical section id or declared alias  |
| `search-docs`         | Search documentation by task, API, or failure mode                |
| `lookup-api`          | Look up curated Leptos, leptos_axum, or Axum public API symbols   |
| `leptos-axum-recipe`  | Return task recipes for common Leptos + Axum workflows            |
| `leptos-diagnostics`  | Analyze Leptos code and return structured diagnostics             |

This server also exposes embedded documentation as MCP resources and provides
workflow prompts for SSR wiring, server functions, hydration debugging, and
Axum integration review.

## Architecture

- `src/main.rs` initializes tracing and runs the stdio MCP server.
- `src/protocol.rs` implements MCP/JSON-RPC request handling, schemas,
  resources, prompts, error responses, and request limits.
- `src/tools.rs` contains the tool handlers and response models.
- `src/docs.rs` indexes the embedded Markdown documentation in `docs/` and maps
  it to `leptos://docs/<section>` resources.
- `src/api.rs` contains a curated API catalog for Leptos, `leptos_axum`, and
  Axum symbols.
- `src/diagnostics.rs` provides heuristic Leptos/Axum diagnostics.
- `src/recipes.rs` and `src/prompts.rs` provide workflow recipes and MCP prompt
  templates.
- `tests/stdio.rs` exercises the compiled binary over stdio.

## Documentation Sections

| Section              | Topics                                                             |
| -------------------- | ------------------------------------------------------------------ |
| **Getting Started**  | Project setup, installation, hello world                           |
| **Components**       | `#[component]`, props, children                                    |
| **Signals**          | `get()`, `set()`, `read()`, `write()`, `update()`, derived signals |
| **Views**            | `view!` macro, dynamic classes/styles/attributes                   |
| **Resources**        | `Resource`, `LocalResource`, `OnceResource`, async data loading    |
| **Actions**          | `ServerAction`, `ActionForm`, mutations                            |
| **Server Functions** | `#[server]`, extractors, Axum integration                          |
| **Routing**          | Router, routes, params, nested routing                             |
| **Forms**            | Controlled inputs, `prop:value`, validation                        |
| **Error Handling**   | `ErrorBoundary`, `ServerFnError`                                   |
| **Suspense**         | `<Suspense>`, `<Transition>`, loading states                       |
| **Leptos Axum**      | `LeptosRoutes`, `handle_server_fns`, extractors, response options  |
| **Axum 0.8.9**       | `Router`, `State`, extractors, middleware, `IntoResponse`          |
| **SSR/Hydration**    | Feature flags, static files, deployment, hydration debugging       |

## Prerequisites

- Rust 1.96 or newer. `Cargo.toml` declares `rust-version = "1.96"`, and CI
  checks Rust 1.96.0 plus the current stable toolchain.
- Cargo, included with Rust.
- Optional CI-style tools for local parity: `cargo-llvm-cov`, `cargo-audit`, and
  `cargo-deny`.

## Installation

```bash
cd leptos-mcp-server
cargo build --release --locked
```

The release binary is written to `target/release/leptos-mcp-server`.

## Configuration

No project-specific environment variables are required.

Logging is controlled with `RUST_LOG`. If unset, the server defaults to
`leptos_mcp_server=info`. Logs are written to stderr so stdout remains reserved
for JSON-RPC responses.

## Usage with Claude Desktop / Antigravity

Add to your MCP config file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Antigravity:** `~/.gemini/antigravity/mcp_config.json`

```json
{
  "mcpServers": {
    "leptos": {
      "command": "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server"
    }
  }
}
```

Use an absolute path to the release binary you built in the installation step.

## Usage with OpenCode

OpenCode configures MCP servers in an `mcp` object. Add a local server entry to
`opencode.json` in your project root, or to `~/.config/opencode/opencode.json`
for a user-wide configuration:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "leptos": {
      "type": "local",
      "command": [
        "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server"
      ],
      "enabled": true
    }
  }
}
```

Use the absolute path to the release binary produced by
`cargo build --release --locked`. To adjust server logging while OpenCode runs
it, add an `environment` object:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "leptos": {
      "type": "local",
      "command": [
        "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server"
      ],
      "environment": {
        "RUST_LOG": "leptos_mcp_server=warn"
      }
    }
  }
}
```

Restart OpenCode after updating the config so it can launch the MCP server and
discover the Leptos tools.

## MCP Smoke Tests

The examples below assume `cargo build --release --locked` has already produced
`./target/release/leptos-mcp-server`.

```bash
# Test tools/list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test list-sections
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections","arguments":{}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test get-documentation with a canonical section id
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test diagnostics
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"leptos-diagnostics","arguments":{"code":"fn App() -> impl IntoView { view! { <p>{count.get()}</p> } }"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Search documentation
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search-docs","arguments":{"query":"Axum state"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Look up an API symbol
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"lookup-api","arguments":{"query":"ResponseOptions","crate":"leptos_axum"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Get a Leptos + Axum recipe
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"leptos-axum-recipe","arguments":{"recipe":"ssr-app"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# List resources
echo '{"jsonrpc":"2.0","id":1,"method":"resources/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Read an embedded documentation resource
echo '{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"leptos://docs/signals"}}' | ./target/release/leptos-mcp-server 2>/dev/null

# List prompts
echo '{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Render a prompt
echo '{"jsonrpc":"2.0","id":1,"method":"prompts/get","params":{"name":"debug-hydration","arguments":{"symptom":"WASM 404"}}}' | ./target/release/leptos-mcp-server 2>/dev/null
```

Available recipe ids are `ssr-app`, `server-functions`, `static-assets`,
`custom-handler`, `state-context`, and `wasm-runtime`.

Available prompt names are `wire-leptos-axum-ssr`, `add-server-function`,
`debug-hydration`, and `review-axum-integration`.

## Development

```bash
# Run in development
cargo run

# Check for errors using the locked dependency graph
cargo check --locked

# Check the declared MSRV, matching CI
cargo +1.96.0 check --locked

# Format check
cargo fmt -- --check

# Run tests
cargo test --locked

# Lint all targets
cargo clippy --locked --all-targets -- -D warnings

# Build release
cargo build --release --locked

# Optional CI parity checks, if the cargo subcommands are installed
cargo llvm-cov --locked --summary-only --fail-under-lines 70
cargo audit
cargo deny check
```

## Protocol

This server implements MCP over stdio using newline-delimited JSON-RPC 2.0.
It advertises MCP protocol version `2024-11-05` and supports `initialize`,
`tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`,
and `prompts/get`.

Invalid JSON-RPC requests return standard JSON-RPC error codes. Documentation
lookup requires a canonical section id or declared alias from `list-sections`;
partial substring lookup is intentionally rejected to avoid returning plausible
but incorrect documentation.

Tool arguments are validated strictly against the advertised schemas. Extra
fields are rejected. Individual JSON-RPC request lines are limited to 1 MiB,
and `leptos-diagnostics` accepts code payloads up to 256 KiB.

Documentation responses include the embedded source path, source URL, reviewed
date, target crate versions, related sections, task tags, common errors,
relevant APIs, and snippet classification for each section.

Leptos API references target Leptos 0.8.19. `leptos_axum` references target
0.8.9, and Axum references target Axum 0.8.9.

## Troubleshooting

- If an MCP client cannot start the server, confirm the configured `command` is
  an absolute path to `target/release/leptos-mcp-server` and that the release
  build completed successfully.
- JSON-RPC requests must be newline-delimited. When testing manually, send one
  complete JSON object per line.
- stdout is reserved for JSON-RPC responses. Use stderr or `RUST_LOG` for logs;
  redirect stderr when you need raw JSON output in shell pipelines.
- Tool arguments are strict. Extra fields or unknown tool names return
  JSON-RPC errors.
- `get-documentation` requires a canonical section id or alias from
  `list-sections`; arbitrary substrings are rejected.
- Individual JSON-RPC request lines are limited to 1 MiB, and
  `leptos-diagnostics` accepts code payloads up to 256 KiB.

## License

MIT. See [LICENSE](LICENSE).
