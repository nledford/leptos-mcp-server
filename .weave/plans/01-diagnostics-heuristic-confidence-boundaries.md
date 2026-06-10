# Plan 01: Diagnostics Heuristic Confidence Boundaries

## TL;DR
> **Summary**: Reclassify Leptos diagnostics so heuristic rules cannot present speculative findings as hard errors, and add characterization tests before changing rule behavior.
> **Estimated Effort**: Medium

## Context
### Original Request
Create execution-ready Tapestry plans for the technical debt roadmap item **P1 diagnostics heuristic overconfidence**.

### Key Findings
- `src/diagnostics.rs` exposes only `Confidence::{High, Medium}` and several string/line-based rules report `Severity::Error` with high confidence despite heuristic parsing.
- `LeptosDiagnostics::analyze` runs lightweight detectors over sanitized text, not a Rust or Leptos AST.
- Protocol and tool tests currently assert presence of diagnostics but do not pin confidence/severity semantics.

## Objectives
### Core Objective
Make diagnostic output honest about heuristic uncertainty while preserving useful guidance for agents.

### Deliverables
- Characterization tests for current diagnostic rule IDs, messages, spans, severities, and confidence values.
- Explicit rule metadata/invariants separating compiler-like errors from heuristic warnings/info.
- Updated diagnostics rendering and structured output expectations for downgraded heuristic findings.
- Documentation/migration note for intentional breaking changes in diagnostic severity/confidence.

### Definition of Done
- `cargo test diagnostics` passes.
- `cargo test protocol::tests::diagnostics_tool_returns_structured_content` passes.
- A reviewer can identify which rules are heuristic and why they are not reported as high-confidence errors.

### Guardrails (Must NOT)
- Must not introduce a Rust parser dependency unless the implementation explicitly scopes and justifies it.
- Must not silently remove existing rule IDs without a breaking-change note.
- Must not claim compile correctness from text-only heuristics.

## TODOs

- [x] 1. Add characterization tests before behavior changes
  **What**: Pin representative true-positive and false-positive-prone inputs for each current rule, including `leptos.missing-component-attribute`, `leptos.server-fn-async`, `leptos.signal-get-in-view`, server function return errors, duplicate paths, body extractors, and deprecated signal APIs.
  **Files**: `src/diagnostics.rs`
  **Acceptance**: Tests fail only when rule IDs, spans, messages, severity, or confidence change unexpectedly.

- [x] 2. Define diagnostic confidence invariants
  **What**: Add a small rule metadata model or documented helper convention that states which detectors may emit `Severity::Error` and `Confidence::High`; add `Confidence::Low` if needed for speculative substring rules.
  **Files**: `src/diagnostics.rs`
  **Acceptance**: Every emitted diagnostic is created through the invariant-aware path or has an adjacent test explaining why it is high confidence.

- [x] 3. Downgrade heuristic-only findings
  **What**: Reclassify diagnostics that depend on substring/line scanning rather than structural certainty to warnings/info and medium/low confidence; keep truly protocol-independent compile blockers as errors only when the detector proves context.
  **Files**: `src/diagnostics.rs`
  **Acceptance**: False-positive-prone examples no longer emit high-confidence errors; useful text still appears in `render_diagnostics`.

- [x] 4. Update protocol/tool expectations
  **What**: Adjust tests that currently assume an error-level diagnostic from sample code so they assert stable rule presence plus the new severity/confidence contract.
  **Files**: `src/protocol.rs`, `src/tools.rs`
  **Acceptance**: Existing protocol structured output shape remains valid; changed severity/confidence values are intentional and tested.

- [x] 5. Add migration/release note
  **What**: Document that diagnostic severity/confidence changes are intentional breaking behavior for clients that gate on errors.
  **Files**: `CHANGELOG.md`
  **Acceptance**: Note names affected rule IDs and recommends clients treat diagnostics as advisory unless explicitly documented as compiler-equivalent.

## Verification
- [x] Run `cargo test diagnostics`.
- [x] Run `cargo test protocol`.
- [x] Run `cargo test tools` if tool rendering assertions are touched.
- [x] Run `cargo fmt -- --check`.
- [x] Run `cargo clippy --locked --all-targets -- -D warnings` if Rust source or tests are changed.
- [x] Run `cargo test` before merging.
- [x] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, or formatting failures in affected code before marking this plan complete.
- [x] No source behavior unrelated to diagnostics changes.

## Breaking-Change Notes
- Diagnostic severity/confidence changes are intentional breaking changes for consumers that fail builds on `error` severity.

## Migration/Docs Notes
- Tell consumers to key automation on `rule_id` plus documented confidence semantics, not severity alone.

## Risks
- Downgrading too much may reduce usefulness; mitigate with rule-specific tests that preserve actionable text.
- Adding metadata may feel heavier than current code; keep it local to diagnostics unless other plans need it.

## Rollback / Stop Conditions
- Stop if characterization tests reveal current diagnostics are relied on as protocol compatibility guarantees; split compatibility discussion into a separate release decision.
- Roll back by restoring previous severity/confidence values while keeping characterization tests.

## Dependencies
- Independent of all other plans.
- Prefer completing before Plan 06 if doc snippet diagnostics are used in compile-check triage.
