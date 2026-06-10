# Plan 06: Docs and Recipe Snippet Compile Checking

## TL;DR
> **Summary**: Build a sustainable snippet validation workflow so documentation and recipe Rust examples are classified, extracted, and compile-checked where promised.
> **Estimated Effort**: XL

## Context
### Original Request
Create execution-ready Tapestry plans for **P6 docs/recipe snippets not compile-checked**.

### Key Findings
- `src/docs.rs` already has `SnippetClassification::{CompileCandidate, Illustrative, Ignore}` and `rust_code_blocks()`, but current metadata marks many sections `Illustrative`.
- `src/recipes.rs` embeds Rust snippets as static strings with `language: "rust"`, including partial fragments that will not compile as standalone files.
- There is no compile-check harness, fixture crate, or CI command for snippet validation.

## Objectives
### Core Objective
Ensure snippets labeled compile-checkable are actually compiled, while illustrative snippets are intentionally marked and tested for classification.

### Deliverables
- Inventory test for every Rust code block and recipe Rust file classification.
- Snippet extraction harness or fixture crate for compile candidates.
- Clear classification policy for `CompileCandidate`, `Illustrative`, and `Ignore`.
- CI/local validation command documented for snippet checks.

### Definition of Done
- `cargo test snippet` or an equivalent named test command passes.
- Every compile-candidate docs/recipe snippet is checked by a compile harness.
- Every non-compiling snippet is explicitly classified as illustrative or ignore with rationale.

### Guardrails (Must NOT)
- Must not force all partial teaching snippets to become standalone programs.
- Must not add heavyweight dependencies unless necessary for reliable compile checks.
- Must not weaken examples by deleting useful context solely to make them compile.

## TODOs

- [ ] 1. Add snippet inventory tests first
  **What**: Test that `docs::rust_code_blocks()` returns expected section IDs/classifications and add equivalent recipe snippet enumeration for `RecipeFile { language: "rust" }`.
  **Files**: `src/docs.rs`, `src/recipes.rs`
  **Acceptance**: Tests expose which snippets are compile candidates versus illustrative before remediation.

- [ ] 2. Define classification policy
  **What**: Document in code or docs what makes a snippet compile-candidate, illustrative, or ignore, including whether fragments need wrappers.
  **Files**: `src/docs.rs`, `src/recipes.rs`, `README.md`
  **Acceptance**: Contributors can classify new snippets without guessing.

- [ ] 3. Create compile-check fixture strategy
  **What**: Add a test-only fixture approach that wraps compile candidates with required imports/stubs or writes temporary files under a test-controlled target/temp directory.
  **Files**: `tests/snippets.rs`, `src/docs.rs`, `src/recipes.rs`
  **Acceptance**: At least one docs snippet and one recipe snippet marked `CompileCandidate` are compiled in tests.

- [ ] 4. Convert high-value snippets to compile candidates
  **What**: Start with stable, complete examples such as API catalog snippets or recipe files that can compile with minimal wrappers; leave partial router fragments illustrative.
  **Files**: `src/docs.rs`, `src/recipes.rs`, `src/api.rs`
  **Acceptance**: The compile-candidate set is small but real, and every candidate passes the harness.

- [ ] 5. Add failure diagnostics
  **What**: Make snippet test failures print section/recipe ID, file path, classification, and generated fixture location.
  **Files**: `tests/snippets.rs`
  **Acceptance**: A failed compile points directly to the source snippet.

- [ ] 6. Document validation command
  **What**: Add or update contributor/release docs with the command to run snippet checks and when to reclassify snippets.
  **Files**: `README.md`, `docs/releasing.md`
  **Acceptance**: Release checklist includes snippet validation before publishing.

## Verification
- [ ] Run `cargo test snippet` or the final snippet-specific test name.
- [ ] Run `cargo test docs`.
- [ ] Run `cargo test recipes`.
- [ ] Run `cargo fmt -- --check`.
- [ ] Run `cargo clippy --locked --all-targets -- -D warnings` if Rust source, tests, or fixture crates are changed.
- [ ] Run `cargo test` before merging.
- [ ] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, or formatting failures in affected code before marking this plan complete.

## Breaking-Change Notes
- Behavior-preserving unless snippets/output text changes.
- Any changed snippet guidance that affects users should be listed in release notes.

## Migration/Docs Notes
- Add contributor guidance for classifying snippets and running the validation command.

## Risks
- Compile harness complexity can grow quickly; begin with a minimal curated candidate set.
- Leptos examples may require feature flags or generated app context; avoid overpromising full app compilation.

## Rollback / Stop Conditions
- Stop if compile harness requires network access or full cargo-leptos app generation; reduce scope to extraction/classification tests first.
- Roll back snippet text changes if they reduce instructional clarity.

## Dependencies
- Plan 01 may improve diagnostics used to triage snippet examples but is not required.
- Plan 05 catalog consistency helps snippet classification coverage.
- Plan 08 may define release checklist hooks for snippet validation.
