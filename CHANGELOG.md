# Changelog

All notable user-facing changes to this project are documented in this file.

This project uses [Semantic Versioning](https://semver.org/) and `release-plz`
to update this changelog from Conventional Commit history when a release pull
request is opened.

## Unreleased

### SDK migration scope

- The server now runs through `rust-mcp-sdk` 0.9.0 over stdio by default and
  advertises MCP protocol version `2025-11-25`.
- Implemented capabilities are tools, concrete documentation resources,
  `resources/templates/list` for `leptos://docs/{section}`, and prompts.
- Completion APIs are intentionally not implemented or advertised.
- Network transports (`streamable-http`/`http` and `sse`) remain deferred and
  disabled. Selecting them fails closed before server startup/listener creation;
  they are not Cargo features in this release.
- No authentication, authorization, CORS, request-limit, or network-timeout
  claims are made because no network transport is implemented.
- Protocol and validation errors now use SDK-native JSON-RPC/MCP behavior rather
  than the removed hand-rolled protocol layer.

### Migration notes from 0.1.0 to 0.2.0-alpha.*

- `0.2.0-alpha.*` is a pre-v1 alpha SDK migration release; clients that depend on
  exact wire-format details should test against it before upgrading production
  automation from `0.1.0`.
- The advertised MCP protocol version changes to `2025-11-25` through
  `rust-mcp-sdk` 0.9.0.
- Invalid requests, protocol failures, schema failures, and argument validation
  now use SDK-native JSON-RPC/MCP error envelopes and messages. Tool-domain
  failures are SDK tool error results, so custom `0.1.0` error envelope/text
  expectations may no longer match.
- Successful tool calls include text content plus SDK `structuredContent` objects
  for automation; tool error results do not include structured content.
- `resources/templates/list` exposes `leptos://docs/{section}` alongside concrete
  `leptos://docs/<section>` resources. Completion remains absent.
- The previous custom 1 MiB stdin line-limit semantics were removed with the
  hand-rolled line reader. Stdio framing and malformed-input behavior now come
  from the SDK transport, while `leptos-diagnostics` still enforces its 256 KiB
  code-input cap.
- Network transports remain deferred/disabled in this alpha and no auth, CORS,
  request-limit, or network-timeout behavior is implemented or claimed.

### Security-sensitive

- The SDK migration removes the previous hand-rolled stdio JSON-RPC line reader;
  the release no longer claims the custom 1 MiB stdin line bound as implemented
  behavior. The diagnostics tool still enforces its 256 KiB code-input cap.
- Residual risk for Warp review: stdio transport behavior is SDK-native and does
  not add a project-specific wall-clock read timeout.
- Release documentation now marks security-sensitive input validation,
  malformed-frame/frame-size behavior, dependency policy, release automation,
  release token, tag protection, and workflow permission changes as requiring
  Warp review before release.
- Release notes must now call out intentional breaking behavior, including
  security-sensitive malformed-client behavior inherited from the SDK stdio
  transport.

### Breaking diagnostic behavior

- Diagnostic severity/confidence changes are intentional breaking behavior for
  clients that gate workflows on `Error` severity or `High` confidence. Affected
  rule IDs: `leptos.signal-get-in-view` is now `Warning`/`Medium`,
  `leptos.missing-component-attribute` is now `Warning`/`Medium`,
  `leptos.server-fn-duplicate-path` is now `Warning`/`Medium`, and
  `leptos.deprecated-create-signal` confidence is now `Medium`.
- Clients should treat diagnostics as advisory unless a rule is explicitly
  documented as compiler-equivalent. Currently, high-confidence errors are
  limited to `leptos.server-fn-async` and `leptos.server-fn-generic`.

### Breaking prompt behavior

- `prompts/get` now enforces required prompt arguments and returns `-32602` when
  required values are missing or blank. Affected required arguments are
  `add-server-function.operation`, `review-sql-access.code`,
  `debug-hydration.symptom`, and `review-axum-integration.code`.
- Clients must send non-blank values for required prompt metadata. Unknown extra
  prompt arguments are also rejected; optional prompt arguments continue to render
  as empty strings when omitted.

### Breaking lookup/search behavior

- `search-docs` now ranks exact section IDs, resource URIs, declared aliases,
  all-token matches, and partial-token matches in explicit tiers, and suppresses
  noisy short/common matches. Queries that previously received a low-confidence
  broad match may now return no result unless they use a stable identifier.
- `lookup-api` now resolves exact symbols and declared aliases first, then uses
  controlled prefix/token summary phases. Vague queries or same-tier multi-match
  queries may now return `Ambiguous` or `Unknown` instead of a low-confidence
  substring match; JSON-RPC clients should expect these as `-32602` validation
  errors.
- For stable automation, prefer exact section IDs, resource URIs, declared
  aliases, and explicit crate filters such as `leptos`, `leptos_axum`, or `axum`.

## [0.1.0](https://github.com/nledford/leptos-mcp-server/releases/tag/v0.1.0) - 2026-06-10

### Other

- add release automation

Release entries are generated by `release-plz` in release pull requests.
