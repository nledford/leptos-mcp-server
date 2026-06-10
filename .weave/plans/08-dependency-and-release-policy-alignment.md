# Plan 08: Dependency and Release Policy Alignment

## TL;DR
> **Summary**: Align dependency versions, embedded catalog versions, release checklist, and security review expectations so policy drift is visible before publishing.
> **Estimated Effort**: Medium

## Context
### Original Request
Create execution-ready Tapestry plans for **P8 dependency/release policy drift**, a security-sensitive maintenance item requiring Warp review.

### Key Findings
- `Cargo.toml` uses broad dependency ranges (`tokio = "1"`, `serde = "1"`, `serde_json = "1"`, `anyhow = "1"`, `tracing = "0.1"`, `tracing-subscriber = "0.3"`) and Rust `edition = "2024"`, `rust-version = "1.96"`.
- Embedded catalog constants in `src/api.rs` pin Leptos/Axum versions (`0.8.19`, `0.8.9`) while some docs URLs use `latest` and release docs exist in `docs/releasing.md`.
- CI already runs `cargo audit`, `cargo deny`, format, clippy, tests, coverage, and release smoke checks; `deny.toml` exists.
- Remaining gaps are policy/enforcement clarity: no targeted tests/checks enforce catalog version alignment, no explicit breaking-change release-note gate for security-sensitive behavior changes, and release-token/workflow safety is not tied to a concrete pre-publish gate.

## Objectives
### Core Objective
Make dependency and release policy explicit, testable where practical, and aligned with embedded docs/API metadata.

### Deliverables
- Written policy for dependency ranges, MSRV/rust-version, lockfile handling, catalog version updates, and release checklist.
- Tests or lightweight checks that catch version drift between `Cargo.toml`, API constants, docs metadata, and release docs where feasible.
- Security-sensitive review checklist for input limits, dependency updates, and generated artifacts.
- Release note process for intentional breaking changes from Plans 01, 02, 03, and 07.
- Release token/workflow safety requirements covering least privilege, logging, protected refs, workflow permissions, and dry-run/manual verification before publishing.

### Definition of Done
- `cargo test release_policy` or equivalent tests/checks pass if implemented in Rust.
- `docs/releasing.md` tells maintainers exactly how to update dependency/catalog versions and what to verify.
- Warp review checklist is explicit for security-sensitive dependency/input validation changes.
- Release publishing requires least-privilege tokens, restricted workflow permissions, no token logging, protected release/tag workflows, and dry-run/manual verification before any real publish.

### Guardrails (Must NOT)
- Must not update dependencies or lockfiles as part of planning/initial policy work unless explicitly implementing this plan later.
- Must not invent an unsupported MSRV; verify against actual toolchain requirements during execution.
- Must not promise automated security scanning unless a real command/tool is added.

## TODOs

- [ ] 1. Inventory current policy and drift points
  **What**: Review `Cargo.toml`, `Cargo.lock`, `src/api.rs`, `src/docs.rs`, `docs/releasing.md`, `README.md`, and `CHANGELOG.md` for version and release statements.
  **Files**: `Cargo.toml`, `Cargo.lock`, `src/api.rs`, `src/docs.rs`, `docs/releasing.md`, `README.md`, `CHANGELOG.md`
  **Acceptance**: Drift list identifies dependency range policy, MSRV, catalog crate versions, docs URL pinning/latest policy, and release checklist gaps.

- [ ] 2. Define dependency version policy
  **What**: Decide whether broad semver ranges remain acceptable for library-like dependencies or whether selected dependencies should be pinned/narrowed; document criteria and who reviews updates.
  **Files**: `docs/releasing.md`, `README.md`
  **Acceptance**: Maintainers can answer when to update `Cargo.lock`, when to run audits, and when Warp review is required.

- [ ] 3. Define embedded catalog version policy
  **What**: State how `LEPTOS_VERSION`, `LEPTOS_AXUM_VERSION`, `AXUM_VERSION`, docs URLs, section metadata, and recipe crate strings are updated together.
  **Files**: `src/api.rs`, `src/docs.rs`, `src/recipes.rs`, `docs/releasing.md`
  **Acceptance**: Release checklist includes a single step for catalog version alignment and examples of expected changes.

- [ ] 4. Add version alignment tests/checks
  **What**: Add tests that compare API constants with docs metadata and recipe crate strings for Leptos/leptos_axum/Axum versions; optionally validate docs URLs are pinned or intentionally `latest` per policy.
  **Files**: `src/api.rs`, `src/docs.rs`, `src/recipes.rs`
  **Acceptance**: A future catalog version mismatch fails tests with a targeted message.

- [ ] 5. Add release checklist gates
  **What**: Update `docs/releasing.md` with commands for full tests, snippet checks from Plan 06 if available, changelog review, dependency review/audit command if adopted, Warp review triggers, and release token/workflow safety checks.
  **Files**: `docs/releasing.md`
  **Acceptance**: Checklist names the exact commands maintainers should run, which changes need human review, and the required pre-publish dry-run/manual verification step.

- [ ] 6. Add security-sensitive review marker
  **What**: Mark input validation, frame size changes, dependency policy changes, release automation changes, release token changes, tag protection changes, and workflow permission changes as requiring Warp review.
  **Files**: `docs/releasing.md`, `CHANGELOG.md`
  **Acceptance**: Security-sensitive items from Plan 02 and this plan are visible in release docs.

- [ ] 7. Capture breaking-change policy
  **What**: Define how to record intentional breaking changes when backward compatibility is not prioritized, including Plan 02's earlier rejection of invalid oversized frames, diagnostic severity changes, prompt argument enforcement, and search semantics.
  **Files**: `docs/releasing.md`, `CHANGELOG.md`
  **Acceptance**: Release process has a concrete breaking-change section template that treats security-sensitive malformed-client behavior changes, including oversized frame rejection timing, as release-note material.

- [ ] 8. Define release token and workflow safety policy
  **What**: Document and, where workflows create real tags/GitHub Releases or publish artifacts, enforce least-privilege release token requirements, no token/secret logging, minimum workflow permissions, protected release/tag expectations, and a concrete dry-run/manual approval gate before release credentials can publish.
  **Files**: `docs/releasing.md`, `.github/workflows/*`
  **Acceptance**: Maintainers can verify tokens are scoped only to publishing needs, workflows cannot publish from unprotected refs/tags, secrets are never printed, default permissions are restricted, and the workflow or documented release process includes a concrete protected-environment/manual-approval or dry-run verification gate before credentials can create tags, GitHub Releases, or published artifacts.

## Verification
- [ ] Run `cargo test api`.
- [ ] Run `cargo test docs`.
- [ ] Run `cargo test recipes`.
- [ ] Run any newly documented dependency audit/check command if adopted.
- [ ] Run `cargo fmt -- --check` if Rust source/tests or workflow-adjacent generated files are changed.
- [ ] Run `cargo clippy --locked --all-targets -- -D warnings` if Rust source/tests are changed.
- [ ] Run `cargo test` before merging.
- [ ] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, formatting failures, audit/deny failures, or workflow validation failures in affected areas before marking this plan complete.

## Breaking-Change Notes
- Policy documentation itself is behavior-preserving.
- Future dependency range changes may be breaking operationally and must be called out in release notes.
- Plan 02's earlier rejection of invalid oversized frames is security-sensitive and behavior-changing for malformed clients; it must be included in breaking-change/release-note review.

## Migration/Docs Notes
- Maintainers should follow the new release checklist for every release and every catalog/dependency update.
- Maintainers should verify release token scope, workflow permissions, protected tag/release controls, and dry-run/manual publish evidence before every release.

## Risks
- Policy without tests can drift; add lightweight alignment tests wherever data is static.
- Overly strict URL pinning can conflict with docs.rs latest links; decide policy explicitly rather than mixing styles accidentally.

## Rollback / Stop Conditions
- Stop if selected audit/check tooling requires network or credentials in normal tests; keep it as a manual release command.
- Roll back only the disputed policy text, not version alignment tests that catch real drift.

## Dependencies
- Coordinate with Plan 05 for single-source catalog version metadata.
- Coordinate with Plan 06 for snippet validation release gates.
- Should be completed before publishing changes from Plans 01, 02, 03, and 07.
