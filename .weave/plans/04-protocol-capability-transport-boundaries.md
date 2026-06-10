# Plan 04: Protocol Capability and Transport Boundaries

## TL;DR
> **Summary**: Decouple MCP capability construction and JSON-RPC method handling from stdio transport so protocol behavior is testable without transport coupling.
> **Estimated Effort**: Large

## Context
### Original Request
Create execution-ready Tapestry plans for **P4 protocol capability/transport coupling**.

### Key Findings
- `src/protocol.rs` owns JSON-RPC parsing, MCP capability payloads, tool/resource/prompt routing, stdio read/write loops, and low-level line framing.
- `McpServer::run` hardcodes stdio, while tests exercise both `handle_line` and private read helpers in the same module.
- Capabilities in `initialize`, `tools/list`, resources, and prompts are constructed inline with protocol routing.

## Objectives
### Core Objective
Create clean boundaries among transport framing, JSON-RPC protocol dispatch, and MCP domain capability catalogs.

### Deliverables
- Characterization tests for initialize, tools/list, resources/list/read, prompts/list/get, and error behavior before refactor.
- Extracted capability/catalog construction that can be tested without JSON-RPC line handling.
- Extracted stdio transport/framing module with bounded read tests.
- Refactored `McpServer` protocol dispatch that remains behavior-compatible except where prior plans intentionally changed behavior.

### Definition of Done
- `cargo test protocol` passes.
- `cargo test --test stdio` passes.
- Public behavior for valid JSON-RPC requests is unchanged except intentional breaking changes from Plans 02/03 if already applied.

### Guardrails (Must NOT)
- Must not combine this structural refactor with unrelated capability changes.
- Must not alter tool names, resource URIs, or prompt names as part of the boundary refactor.
- Must not make stdio async unless separately justified.

## TODOs

- [ ] 1. Add behavior characterization tests
  **What**: Pin `initialize` capability keys, tool schemas, resources shape, prompt shape, notification behavior, and JSON-RPC error codes before moving code.
  **Files**: `src/protocol.rs`, `tests/stdio.rs`
  **Acceptance**: Tests would catch accidental capability or wire-format drift.

- [ ] 2. Extract capability/catalog builders
  **What**: Move inline JSON builders for initialize capabilities, tools/list schemas, resources/list, and prompts/list into a focused module or structs with pure functions.
  **Files**: `src/protocol.rs`, `src/lib.rs`, `src/tools.rs`, `src/docs.rs`, `src/prompts.rs`
  **Acceptance**: Builders are callable from tests without constructing stdio transport.

- [ ] 3. Extract transport/framing code
  **What**: Move `LineRead`, `read_limited_line`, `discard_until_newline` if still needed, and `decode_line` into a transport-focused module.
  **Files**: `src/protocol.rs`, `src/lib.rs`
  **Acceptance**: Framing tests live with transport code; protocol dispatch no longer directly owns low-level buffer mechanics.

- [ ] 4. Simplify protocol dispatcher
  **What**: Keep `McpServer::handle_line`/`handle_request` focused on JSON-RPC validation, notification semantics, and domain routing.
  **Files**: `src/protocol.rs`
  **Acceptance**: `handle_request` reads as method dispatch only; transport concerns are absent.

- [ ] 5. Preserve process entrypoint
  **What**: Keep `src/main.rs` minimal and update imports if module names change.
  **Files**: `src/main.rs`, `src/lib.rs`
  **Acceptance**: Binary still starts the stdio MCP server and logs only to stderr.

- [ ] 6. Document boundary decisions
  **What**: Add concise module-level comments explaining protocol vs transport vs domain responsibilities.
  **Files**: `src/protocol.rs`, `src/lib.rs`
  **Acceptance**: Future capability additions have an obvious home.

## Verification
- [ ] Run `cargo test protocol`.
- [ ] Run `cargo test --test stdio`.
- [ ] Run `cargo test tools` if tool schema construction changes touch tool constants/output.
- [ ] Run `cargo fmt -- --check`.
- [ ] Run `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Run `cargo test` before merging.
- [ ] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, or formatting failures in affected code before marking this plan complete.

## Breaking-Change Notes
- This plan should be behavior-preserving unless Plans 02 or 03 have already introduced intentional changes.

## Migration/Docs Notes
- No user-facing docs required for a pure internal boundary refactor.

## Risks
- Moving private helpers can expose accidental public API; keep new modules private unless tests require crate-level access.
- Snapshot-style JSON tests can become brittle; assert semantically important keys rather than full formatting.

## Rollback / Stop Conditions
- Stop if more than one behavior change appears during refactor; isolate it in a prerequisite plan.
- Roll back structural moves if tests cannot prove wire compatibility.

## Dependencies
- Prefer Plan 02 first so transport extraction moves hardened code once.
- Can run after Plan 03; keep prompt validation in prompt domain.
- Plan 05 may reuse extracted catalog builders.
