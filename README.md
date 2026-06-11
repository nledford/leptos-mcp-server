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
Axum integration review, including guidance for sqlx and SeaQuery usage in
Leptos/Axum applications.

## Implemented capability snapshot

- MCP implementation: `rust-mcp-sdk` 0.9.0 with advertised MCP protocol
  version `2025-11-25`.
- Transport: stdio is the default and only implemented transport. The default
  Cargo feature set is `default = ["stdio"]`; `stdio` enables
  `rust-mcp-sdk/stdio`.
- Deferred transports: `--transport streamable-http`, `--transport http`, and
  `--transport sse` are accepted as transport selections but intentionally fail
  closed before tracing, server startup, or any listener creation. They are not
  Cargo features in this release.
- Tools: `list-sections`, `get-documentation`, `search-docs`, `lookup-api`,
  `leptos-axum-recipe`, and `leptos-diagnostics`.
- Resources: concrete embedded Markdown documentation resources are exposed as
  `leptos://docs/<section>` URIs and the resource template
  `leptos://docs/{section}` is exposed through `resources/templates/list`.
- Prompts: `wire-leptos-axum-ssr`, `add-server-function`, `review-sql-access`,
  `debug-hydration`, and `review-axum-integration` are supported.
- Errors: protocol and validation errors use SDK-native JSON-RPC/MCP error
  behavior. Tool-domain failures are returned as SDK tool error results.
- Intentionally unsupported: completion APIs are not implemented or advertised,
  and no authentication, authorization, CORS, or network security claims are made
  because network transports remain disabled.

## Architecture

- `src/main.rs` parses transport selection, initializes tracing, and runs the
  default stdio MCP server.
- `src/sdk.rs` adapts the application facade to `rust-mcp-sdk` tools,
  resources, resource templates, prompts, initialization metadata, and
  SDK-native error behavior.
- `src/tools.rs` contains the tool handlers and response models.
- `src/docs.rs` indexes the embedded Markdown documentation in `docs/` and maps
  it to `leptos://docs/<section>` resources.
- `src/api.rs` contains a curated API catalog for Leptos, `leptos_axum`, and
  Axum symbols.
- `src/diagnostics.rs` provides heuristic Leptos/Axum diagnostics.
- `src/recipes.rs` and `src/prompts.rs` provide workflow recipes and MCP prompt
  templates.
- `tests/stdio.rs` exercises the compiled binary over stdio.
- `.agents/skills/leptos-mcp-server/` contains an optional Agent Skill that
  teaches compatible agents when and how to use this MCP server.

## Documentation Sections

| Section              | Topics                                                             |
| -------------------- | ------------------------------------------------------------------ |
| **Getting Started**  | Project setup, installation, hello world                           |
| **Components**       | `#[component]`, props, children                                    |
| **Signals**          | `get()`, `set()`, `read()`, `write()`, `update()`, derived signals |
| **Views**            | `view!` macro, dynamic classes/styles/attributes                   |
| **Resources**        | `Resource`, `LocalResource`, `OnceResource`, async data loading    |
| **Actions**          | `ServerAction`, `ActionForm`, mutations, transactions              |
| **Server Functions** | `#[server]`, extractors, Axum integration, sqlx, SeaQuery          |
| **Routing**          | Router, routes, params, nested routing                             |
| **Forms**            | Controlled inputs, `prop:value`, validation                        |
| **Error Handling**   | `ErrorBoundary`, `ServerFnError`, database errors                  |
| **Suspense**         | `<Suspense>`, `<Transition>`, loading states                       |
| **Leptos Axum**      | `LeptosRoutes`, `handle_server_fns`, extractors, database context  |
| **Axum 0.8.9**       | `Router`, `State`, extractors, middleware, database pools          |
| **SSR/Hydration**    | Feature flags, static files, deployment, hydration debugging       |

## Snippet Classification Policy

Rust snippets in `docs/` and Rust recipe files are inventoried as one of three
classifications:

- `CompileCandidate`: complete enough for an automated compile harness. The
  snippet may rely only on wrappers/import prelude that the repository harness
  supplies uniformly. If it needs a custom component, `main`, async runtime,
  route tree, feature flag, database schema, generated file, or fixture, add
  that support to the harness before using this classification.
- `Illustrative`: real example code that teaches an API or pattern but is not
  expected to compile by itself. Use this for fragments, excerpts, omitted
  imports, app-specific state, database examples, and code that needs wrappers
  the repository does not yet provide. This is the default for new snippets.
- `Ignore`: fenced Rust-like text that should not count as a snippet or compile
  target, such as intentionally invalid code, expected compiler diagnostics, or
  placeholders. Prefer `Illustrative` when the block is useful example code even
  if it is incomplete.

When adding a new Rust block, classify it by asking: can the current shared
harness compile it without guessing about missing surroundings? If yes, mark it
`CompileCandidate`; if no but it is useful example code, mark it `Illustrative`;
if it is not example code, mark it `Ignore`.

Run the snippet validation harness before publishing documentation changes:

```bash
cargo test snippets
```

Reclassify snippets whenever a Rust docs, recipe, or API example is added,
removed, or changes between a complete compile candidate and an illustrative or
ignored example. The snippet test verifies the inventory/classification filters
and compiles the examples currently classified as compile candidates.

## Prerequisites

- Rust 1.96 or newer. `Cargo.toml` declares `rust-version = "1.96"`, and CI
  runs check and test jobs on Rust 1.96.0.
- Cargo, included with Rust.
- Optional CI-style tools for local parity: `cargo-llvm-cov`, `cargo-audit`, and
  `cargo-deny`.

## Installation

Build from a local checkout; this project does not currently publish a binary,
crate, npm package, Docker image, or hosted MCP endpoint.

```bash
git clone https://github.com/nledford/leptos-mcp-server.git
cd leptos-mcp-server
cargo build --release --locked
```

The release binary is written to `target/release/leptos-mcp-server`.

For development, you can run the stdio server directly with Cargo:

```bash
cargo run --locked -- --transport stdio
```

For normal MCP client use, configure the release binary as a local/stdio server.
The default invocation already uses stdio, so `--transport stdio` is optional but
shown below to make the transport explicit.

## Configuration

No project-specific environment variables are required.

Logging is controlled with `RUST_LOG`. If unset, the server defaults to
`leptos_mcp_server=info`. Logs are written to stderr so stdout remains reserved
for JSON-RPC responses.

Transport selection can be made with either CLI flags or environment variables:

```bash
# Equivalent stdio invocations
./target/release/leptos-mcp-server
./target/release/leptos-mcp-server --transport stdio
LEPTOS_MCP_TRANSPORT=stdio ./target/release/leptos-mcp-server
```

`LEPTOS_MCP_HOST`/`LEPTOS_MCP_PORT` and `--host`/`--port` are reserved for future
network transports and are rejected with stdio. Do not set them in stdio MCP
client configuration.

### Cargo features

The default feature set is:

```toml
default = ["stdio"]
stdio = ["rust-mcp-sdk/stdio"]
```

Build normally with `cargo build --release --locked`. If you disable default
features, the stdio transport is not compiled in; rebuild with `--features stdio`
to produce a usable MCP server. There are no Cargo features for HTTP, SSE,
authentication, CORS, or other network support in this release.

### Network transports deferred

Only stdio is implemented and it is the default/current transport. Network
transports (`streamable-http`/`http` and `sse`) are feature-deferred and
unsupported: those transport selections fail closed before tracing or server startup,
and no network listener is started. They are not exposed as Cargo
features in this release. The documented future safe host default is
`127.0.0.1`; a network port would need to be provided explicitly.
Network use is never implicit: a future network transport would require an
explicit `--transport streamable-http`/`--transport sse` or
`LEPTOS_MCP_TRANSPORT` selection, and stdio remains the default.

Do not expose this server as a public network service without adding and testing
separate production controls. Because network transports are disabled, this
project does not currently configure a network authentication layer, CORS
allowlist policy, HTTP request body/message limits, or read/request/handler
timeouts. There is no default wildcard CORS policy. Network support remains
deferred until those guardrails, including sanitized malformed-input handling,
can be configured and verified.

### Performance and input limits

- Stdio transport framing, message-size behavior, malformed-input handling, and
  read timing are SDK-native. This project no longer adds the previous custom
  1 MiB stdin line cap from the removed hand-rolled reader.
- `leptos-diagnostics` rejects empty code and code larger than 256 KiB before
  running heuristic analysis. This is the remaining project-specific tool input
  size guard.
- Embedded documentation and API/recipe catalogs are compiled into the binary as
  static strings. `resources/read` and documentation tools allocate a response
  string for the selected section and do not stream or paginate section content;
  resource descriptors currently advertise no `size` field. The checked-in
  corpus is intentionally small, but adding substantially larger docs/resources
  increases binary size and per-request response allocation.
- No project-specific wall-clock handler timeout is configured for stdio tool,
  resource, or prompt calls. Network read/request/handler timeouts remain absent
  because network transports are disabled.

## Agent Skill

This repository includes an optional Agent Skill for agents that support the
`SKILL.md` format:

```text
.agents/skills/leptos-mcp-server/SKILL.md
```

Install it directly from GitHub with the `skills` CLI:

```bash
npx skills add https://github.com/nledford/leptos-mcp-server --skill leptos-mcp-server
```

For a local checkout, use `npx skills add ./.agents/skills/leptos-mcp-server`
from the repository root.

The skill documents when agents should use this MCP server, its exact tools,
resources, prompts, schemas, setup steps, workflows, troubleshooting, and
security/privacy considerations. It does not replace building and configuring
the MCP binary; use the installation and client configuration sections below for
that.

## Usage with Claude Desktop / Antigravity

Add to your MCP config file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Antigravity:** `~/.gemini/antigravity/mcp_config.json`

```json
{
  "mcpServers": {
    "leptos": {
      "command": "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server",
      "args": ["--transport", "stdio"]
    }
  }
}
```

Use an absolute path to the release binary you built in the installation step.
Configure this as a local/stdio MCP server, not as a URL. The server writes MCP
JSON-RPC responses on stdout and logs on stderr.

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
        "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server",
        "--transport",
        "stdio"
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
        "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server",
        "--transport",
        "stdio"
      ],
      "environment": {
        "LEPTOS_MCP_TRANSPORT": "stdio",
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

# List resource templates, including leptos://docs/{section}
echo '{"jsonrpc":"2.0","id":1,"method":"resources/templates/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# List prompts
echo '{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Render a prompt
echo '{"jsonrpc":"2.0","id":1,"method":"prompts/get","params":{"name":"debug-hydration","arguments":{"symptom":"WASM 404"}}}' | ./target/release/leptos-mcp-server 2>/dev/null
```

Available recipe ids are `ssr-app`, `server-functions`, `static-assets`,
`custom-handler`, `state-context`, `database-query-patterns`, and
`wasm-runtime`.

Available prompt names are `wire-leptos-axum-ssr`, `add-server-function`,
`review-sql-access`, `debug-hydration`, and `review-axum-integration`.

## Development

```bash
# Run in development
cargo run

# Check for errors using the locked dependency graph
cargo check --locked

# Check the declared MSRV, matching CI
cargo +1.96.0 check --locked
cargo +1.96.0 test --locked

# Format check
cargo fmt -- --check

# Run tests
cargo test --locked

# Validate Rust snippet classification and compile candidates
cargo test snippets

# Lint all targets
cargo clippy --locked --all-targets -- -D warnings

# Build release
cargo build --release --locked

# Optional CI parity checks, if the cargo subcommands are installed
cargo llvm-cov --locked --summary-only --fail-under-lines 70
cargo audit
cargo deny check
```

## Releases

Release automation uses Semantic Versioning, Conventional Commits, `release-plz`,
and `vX.Y.Z` Git tags. See [docs/releasing.md](docs/releasing.md) for the
maintainer process, dependency version policy, lockfile update rules, audit
timing, Warp review criteria, GitHub settings, and publishing status.

## Migration notes for `0.2.0-alpha.*`

The `0.2.0-alpha.*` line is a pre-v1 SDK migration release. Existing `0.1.0`
users should test it with their MCP client before relying on exact wire-format or
error behavior in automation.

- Protocol/runtime: the server now uses `rust-mcp-sdk` 0.9.0 and advertises MCP
  protocol version `2025-11-25` instead of the previous hand-rolled protocol
  adapter behavior.
- Error envelopes: invalid JSON-RPC, protocol, schema, and argument validation
  failures now use SDK-native JSON-RPC/MCP error shapes and messages. Tool-domain
  failures are returned as SDK tool error results, so clients should not depend
  on `0.1.0` custom error text or envelope details.
- Structured output: successful tool calls still include human-readable text, but
  clients should prefer SDK `structuredContent` objects for automation. Tool
  error results do not include structured content.
- Resources/templates: concrete docs remain available as `leptos://docs/<section>`
  resources, and `resources/templates/list` now exposes the template
  `leptos://docs/{section}`. Completion remains intentionally absent.
- Stdio limits: the custom 1 MiB stdin line-limit semantics from the manual
  reader are removed; stdio framing and malformed-input behavior are inherited
  from the SDK transport. The `leptos-diagnostics` tool still enforces its
  256 KiB code-input guard.
- Network transports: `streamable-http`/`http` and `sse` remain deferred. Selecting
  them fails closed before tracing, server startup, or listener creation; no
  authentication, CORS, HTTP request-limit, or network-timeout behavior is
  implemented or advertised.

## Protocol

This server implements MCP over stdio using newline-delimited JSON-RPC 2.0.
It uses `rust-mcp-sdk` 0.9.0, advertises MCP protocol version `2025-11-25`,
and supports `initialize`, `tools/list`, `tools/call`, `resources/list`,
`resources/read`, `resources/templates/list`, `prompts/list`, and
`prompts/get`.

Invalid JSON-RPC requests return SDK-native JSON-RPC/MCP error envelopes.
Documentation lookup requires a canonical section id or declared alias from
`list-sections`; partial substring lookup is intentionally rejected to avoid
returning plausible but incorrect documentation.

Tool arguments are validated strictly against the advertised schemas. Extra
fields are rejected. The previous hand-rolled 1 MiB stdio line cap is not part
of the SDK stdio transport; `leptos-diagnostics` still accepts code payloads up
to 256 KiB.

Completion is intentionally absent from advertised capabilities. Authentication,
authorization, CORS policy, HTTP request limits, and network timeouts are also
absent because no network transport is implemented.

Documentation responses include the embedded source path, source URL, reviewed
date, target crate versions, related sections, task tags, common errors,
relevant APIs, and snippet classification for each section.

Leptos API references target Leptos 0.8.19. `leptos_axum` references target
0.8.9, and Axum references target Axum 0.8.9.

SQL guidance is documentation-only. It references current `sqlx`, SeaQuery, and
`sea-query-sqlx` APIs for application code examples but does not add database
connectivity, query execution, schema inspection, or migration management to the
MCP server.

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
- The SDK stdio transport replaced the previous custom JSON-RPC line reader; the
  server still caps `leptos-diagnostics` code payloads at 256 KiB.

## License

MIT. See [LICENSE](LICENSE).
