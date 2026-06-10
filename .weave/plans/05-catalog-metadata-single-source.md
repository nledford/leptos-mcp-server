# Plan 05: Catalog Metadata Single Source

## TL;DR
> **Summary**: Remove duplicated documentation/catalog metadata by creating explicit single-source catalog records with validation for section and metadata consistency.
> **Estimated Effort**: Large

## Context
### Original Request
Create execution-ready Tapestry plans for **P5 catalog metadata duplication**.

### Key Findings
- `src/docs.rs` has separate `SECTIONS` and `SECTION_METADATA` arrays linked by `id`, with panic-on-missing behavior in `tools::section_summary`.
- Version constants live in `src/api.rs` and are reused by docs metadata, while some docs URLs still use `latest` and some hard-code versions.
- Tests verify some catalog fields exist but do not enforce one-to-one section/metadata coverage or duplicate aliases.

## Objectives
### Core Objective
Make catalog metadata coherent, validated, and maintainable without ID-based duplication drift.

### Deliverables
- [ ] Catalog consistency tests before refactor.
- [ ] Single-source representation or generated view that prevents missing metadata for a section.
- [ ] Validation for unique section IDs, aliases, resource URIs, related section IDs, and metadata coverage.
- [ ] Clear ownership of version constants and docs URLs.

### Definition of Done
- [ ] `cargo test docs` passes.
- [ ] `cargo test tools::tests::list_sections`-style tests pass or equivalent coverage exists.
- [ ] Adding a section in one place cannot compile or cannot pass tests without metadata.

### Guardrails (Must NOT)
- [ ] Must not change public section IDs/resource URIs unless explicitly marked breaking.
- [ ] Must not hide catalog errors behind runtime panics where tests can catch them.
- [ ] Must not mix broad content rewrites into metadata normalization.

## TODOs

- [ ] 1. Add catalog invariant tests
  **What**: Assert unique section IDs, unique normalized aliases within a lookup scope, one metadata record per section, no orphan metadata, valid `related_sections`, valid resource URI round trips, and non-empty version/source fields.
  **Files**: `src/docs.rs`
  **Acceptance**: Tests fail on duplicate/missing catalog metadata.

- [ ] 2. Choose and document single-source model
  **What**: Decide between embedding `SectionMetadata` directly in `DocSection`, creating a `CatalogSection { section, metadata }`, or a static table that owns both with derived accessors.
  **Files**: `src/docs.rs`
  **Acceptance**: The chosen model eliminates ID joins for normal section summary generation.

- [ ] 3. Refactor catalog accessors
  **What**: Update `list_sections`, `get_section`, `get_metadata`, `search_sections`, `rust_code_blocks`, and resource lookup to use the new model without changing external outputs.
  **Files**: `src/docs.rs`, `src/tools.rs`, `src/protocol.rs`
  **Acceptance**: Existing tool/resource outputs remain semantically identical.

- [ ] 4. Normalize version/source ownership
  **What**: Centralize crate/version/docs URL definitions so `LEPTOS_VERSION`, `LEPTOS_AXUM_VERSION`, `AXUM_VERSION`, and associated docs URLs cannot drift across docs/api metadata.
  **Files**: `src/api.rs`, `src/docs.rs`
  **Acceptance**: Tests assert docs metadata versions match API symbol versions for shared crates.

- [ ] 5. Add related-section validation coverage
  **What**: Ensure every `related_sections` entry points to a known section ID and no section relates to itself unless explicitly allowed.
  **Files**: `src/docs.rs`
  **Acceptance**: Invalid relation IDs fail tests with actionable messages.

- [ ] 6. Add behavior-preserving release note if needed
  **What**: If no public IDs change, add no user-facing note; if any duplicate/ambiguous aliases are removed, document as breaking.
  **Files**: `CHANGELOG.md`
  **Acceptance**: Any lookup behavior change is explicitly documented.

## Verification
- [ ] Run `cargo test docs`.
- [ ] Run `cargo test tools`.
- [ ] Run `cargo test protocol::tests::resources_list_and_read_expose_documentation_sections`.
- [ ] Run `cargo test` before merging.

## Breaking-Change Notes
- [ ] Intended as behavior-preserving for public catalog IDs.
- [ ] Removing or changing aliases/resource URIs is breaking and must be called out if it happens.

## Migration/Docs Notes
- [ ] No docs needed if behavior-preserving; otherwise list changed aliases and replacement IDs.

## Risks
- [ ] Large static table edits are error-prone; rely on invariant tests and small commits.
- [ ] Centralizing version URLs may expose existing `latest` vs pinned URL policy drift; coordinate with Plan 08.

## Rollback / Stop Conditions
- [ ] Stop if refactor changes search ranking unexpectedly; either preserve old order or split ranking changes into Plan 07.
- [ ] Roll back single-source representation if it worsens readability; keep invariant tests regardless.

## Dependencies
- [ ] Plan 04 can make catalog builders cleaner but is not required.
- [ ] Coordinate with Plan 08 for version/release policy constants.
