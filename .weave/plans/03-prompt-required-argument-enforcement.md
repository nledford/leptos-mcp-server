# Plan 03: Prompt Required Argument Enforcement

## TL;DR
> **Summary**: Enforce prompt `required` arguments at `prompts/get` time so clients cannot receive partially rendered templates with silent empty placeholders.
> **Estimated Effort**: Short

## Context
### Original Request
Create execution-ready Tapestry plans for **P3 prompt required arguments not enforced**.

### Key Findings
- `src/prompts.rs` marks `PromptArgument { required: true }` for prompts such as `add-server-function`, `review-sql-access`, `debug-hydration`, and `review-axum-integration`.
- `render_prompt` replaces missing arguments with `""`, so required fields are metadata only.
- `src/protocol.rs::handle_get_prompt` calls `render_prompt` directly after lookup and returns success for missing required arguments.

## Objectives
### Core Objective
Make prompt metadata executable by rejecting missing or blank required prompt arguments.

### Deliverables
- Characterization/regression test showing missing required prompt arguments are currently accepted, then updated to require rejection.
- Prompt validation API returning structured missing-argument errors.
- Protocol-level `-32602 InvalidParams` response for missing required prompt arguments.
- Tests for optional arguments still rendering as empty strings.

### Definition of Done
- `cargo test prompts` passes.
- `cargo test protocol::tests::prompts_get_rejects_missing_required_arguments` passes.
- `prompts/list` metadata and `prompts/get` behavior are consistent.

### Guardrails (Must NOT)
- Must not make optional prompt arguments required.
- Must not silently trim meaningful user whitespace inside supplied prompt content beyond blank detection.
- Must not change prompt names or template text unless required for validation messaging.

## TODOs

- [x] 1. Add prompt-domain tests first
  **What**: Cover required present, required missing, required blank/whitespace, optional missing, and unknown extra arguments if the implementation chooses to reject extras.
  **Files**: `src/prompts.rs`
  **Acceptance**: Tests define the domain behavior independent of JSON-RPC.

- [x] 2. Introduce prompt validation error type
  **What**: Extend `PromptLookupError` or add `PromptRenderError` with missing required argument names; expose `render_prompt` as fallible or add `validate_arguments` used before rendering.
  **Files**: `src/prompts.rs`
  **Acceptance**: Missing required arguments produce deterministic, testable error messages containing prompt name and argument names.

- [x] 3. Wire validation into protocol
  **What**: Update `handle_get_prompt` to map prompt validation failures to `ProtocolError::InvalidParams` (`-32602`).
  **Files**: `src/protocol.rs`
  **Acceptance**: `prompts/get` with `review-sql-access` and missing `code` returns an error, not a partial prompt.

- [x] 4. Preserve optional placeholder semantics
  **What**: Keep optional missing values rendering as empty strings for templates that intentionally include context placeholders.
  **Files**: `src/prompts.rs`, `src/protocol.rs`
  **Acceptance**: Existing tests for `debug-hydration` with only `symptom` and SQL review with both required/optional args still pass.

- [x] 5. Add breaking-change note
  **What**: Document that `prompts/get` now enforces required arguments and clients must send non-blank values for required metadata.
  **Files**: `CHANGELOG.md`
  **Acceptance**: Release note lists affected prompt names and argument names.

## Verification
- [x] Run `cargo test prompts`.
- [x] Run `cargo test protocol::tests::prompts_list_and_get_render_workflow_prompt`.
- [x] Run `cargo test protocol::tests::prompts_get_rejects_missing_required_arguments`.
- [x] Run `cargo test --test stdio` if prompt protocol snapshots are added there.
- [x] Run `cargo fmt -- --check`.
- [x] Run `cargo clippy --locked --all-targets -- -D warnings`.
- [x] Run `cargo test` before merging.
- [x] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, or formatting failures in affected code before marking this plan complete.

## Breaking-Change Notes
- Intentional breaking change: previously accepted missing required prompt arguments now return `-32602`.

## Migration/Docs Notes
- Clients should read `prompts/list` and supply every argument where `required: true`.

## Risks
- Existing clients may rely on partial prompts; mitigate with clear release note and actionable error messages.
- Extra argument policy can expand scope; defer extra-argument rejection unless explicitly needed.

## Rollback / Stop Conditions
- Stop if MCP prompt spec requires a different error shape; align with spec before coding.
- Roll back by allowing missing required args only behind an explicit compatibility decision, not silently.

## Dependencies
- Independent of other plans.
- If Plan 04 introduces capability builders, keep prompt validation in `prompts.rs` rather than transport/capability code.
