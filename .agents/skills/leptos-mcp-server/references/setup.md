# Setup and connection reference

This MCP server is implemented as a Rust binary that speaks newline-delimited JSON-RPC 2.0 over stdio.

## Build requirements

- Rust 1.96 or newer.
- Cargo.
- Optional local CI parity tools: `cargo-llvm-cov`, `cargo-audit`, `cargo-deny`.

Build from the server repository root:

```bash
cargo build --release --locked
```

The binary is written to:

```text
target/release/leptos-mcp-server
```

## Environment and credentials

- No project-specific environment variables are required.
- No credentials, tokens, API keys, database URLs, or local services are required by the server.
- `RUST_LOG` controls logging. If unset, the server defaults to `leptos_mcp_server=info`.
- Logs are written to stderr. stdout is reserved for JSON-RPC responses.

## Claude Desktop / Antigravity config

Use an absolute path to the release binary:

```json
{
  "mcpServers": {
    "leptos": {
      "command": "/absolute/path/to/leptos-mcp-server/target/release/leptos-mcp-server"
    }
  }
}
```

Known config locations from the repository README:

- Claude Desktop macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Antigravity: `~/.gemini/antigravity/mcp_config.json`

## OpenCode config

Add a local MCP server entry in project `opencode.json` or user-wide `~/.config/opencode/opencode.json`:

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

Optional logging override:

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

Restart the MCP client after config changes.

## JSON-RPC smoke tests

These examples assume the release binary has been built.

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | ./target/release/leptos-mcp-server 2>/dev/null

echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list-sections","arguments":{}}}' \
  | ./target/release/leptos-mcp-server 2>/dev/null

echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get-documentation","arguments":{"section":"signals"}}}' \
  | ./target/release/leptos-mcp-server 2>/dev/null
```

JSON-RPC requests must be one complete JSON object per line.

## Development verification commands

Repository-supported checks:

```bash
cargo check --locked
cargo +1.96.0 check --locked
cargo fmt -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
cargo llvm-cov --locked --summary-only --fail-under-lines 70
cargo audit
cargo deny check
```

## Installing this Agent Skill

The `skills` npm package supports SKILL.md skills with YAML frontmatter and local path installs. From this repository root, install this local skill with:

```bash
npx skills add ./.agents/skills/leptos-mcp-server
```

Manual installation is also valid: copy `.agents/skills/leptos-mcp-server` into an agent skill directory such as `~/.agents/skills/leptos-mcp-server`.

This repository is not an npm package and has no `package.json`, so no npm publishing metadata was added. Current `skills` CLI docs do not show stable `npx skills add <npm-package-name>` behavior for arbitrary npm packages; npm-package-distributed skills appear to rely on already-installed `node_modules` scanning that is marked experimental in the CLI source. If this repository later becomes an npm package, include `.agents/skills/**` in `package.json.files` so the skill files are present in the published tarball.
