# Error handling and troubleshooting

## Server does not start

- Confirm `cargo build --release --locked` completed successfully.
- Confirm the MCP config `command` is an absolute path to `target/release/leptos-mcp-server`.
- Restart the MCP client after config changes.
- Remember that server logs go to stderr. stdout must contain only JSON-RPC responses.

## Manual smoke test has no useful output

- Send one complete JSON object per line.
- Redirect stderr if you need raw JSON on stdout: `2>/dev/null`.
- Use `tools/list` first to confirm the binary starts and lists tools.

## JSON-RPC or MCP errors

- Invalid JSON returns parse error `-32700`.
- Missing/invalid `jsonrpc`, method, or params returns SDK-native JSON-RPC/MCP errors.
- Unknown method returns `-32601`.
- Unknown tool names or bad tool params return SDK tool error results where applicable.
- Stdio framing and malformed input are handled by `rust-mcp-sdk`; this project no longer adds a custom 1 MiB request-line cap.

## Tool argument errors

- Tool arguments are deserialized with unknown fields denied. Remove extra fields.
- `get-documentation` does not perform arbitrary substring matching. Call `list-sections` or `search-docs`, then pass a canonical id or declared alias.
- `leptos-diagnostics` requires non-empty `code` and rejects inputs larger than 262144 bytes.
- `lookup-api` only covers the curated symbols and concepts listed in
  [tools.md](tools.md), not every symbol in the crates. Non-empty unknown
  queries return `lookup.status: not-found` with suggestions and guidance rather
  than a tool error; blank queries are still rejected.

## Prompt pitfalls

- `prompts/get` requires a non-empty prompt name.
- Prompt names normalize spaces/underscores to hyphens.
- Required prompt arguments are enforced; missing or blank required values and unknown arguments are rejected.

## Scope failures

- If a task needs live upstream docs, this MCP server may be stale because it uses embedded documentation reviewed at a fixed date.
- If a task needs project-specific context, read the project files directly.
- If a task needs authoritative compiler behavior, run Rust tooling. `leptos-diagnostics` is heuristic only.
- If a task needs database state, migrations, schemas, or query execution, use appropriate database/project tools. This MCP server only provides SQL guidance.

## Security and privacy reminders

- Do not send secrets, tokens, credentials, or confidential code into prompt/diagnostics inputs unless the MCP client transcript/log handling is trusted.
- Do not assume the server made network, filesystem, or database changes; it exposes read-only embedded docs, recipes, prompts, resources, and in-memory diagnostics.
