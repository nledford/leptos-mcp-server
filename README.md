# Leptos MCP Server

An MCP (Model Context Protocol) server providing comprehensive Leptos documentation and code analysis tools for AI agents.

## Features

| Tool                | Description                                                     |
| ------------------- | --------------------------------------------------------------- |
| `list-sections`     | List all available Leptos documentation sections with use cases |
| `get-documentation` | Retrieve documentation by canonical section id or declared alias|
| `leptos-diagnostics`| Analyze Leptos code and return structured diagnostics           |

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

## Installation

```bash
cd leptos-mcp-server
cargo build --release
```

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

## Testing

```bash
# Test tools/list
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test list-sections
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections","arguments":{}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test get-documentation with a canonical section id
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals"}}}' | ./target/release/leptos-mcp-server 2>/dev/null

# Test diagnostics
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"leptos-diagnostics","arguments":{"code":"fn App() -> impl IntoView { view! { <p>{count.get()}</p> } }"}}}' | ./target/release/leptos-mcp-server 2>/dev/null
```

## Development

```bash
# Run in development
cargo run

# Check for errors
cargo check

# Build release
cargo build --release
```

## Protocol

This server implements MCP over stdio using newline-delimited JSON-RPC 2.0.

Invalid JSON-RPC requests return standard JSON-RPC error codes. Documentation
lookup requires a canonical section id or declared alias from `list-sections`;
partial substring lookup is intentionally rejected to avoid returning plausible
but incorrect documentation.

Tool arguments are validated strictly against the advertised schemas. Extra
fields are rejected. Individual JSON-RPC request lines are limited to 1 MiB,
and `leptos-diagnostics` accepts code payloads up to 256 KiB.

Documentation responses include the embedded source path, reviewed date, and
target Leptos version scope for each section.

## License

MIT
