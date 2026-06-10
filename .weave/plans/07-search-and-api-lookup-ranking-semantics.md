# Plan 07: Search and API Lookup Ranking Semantics

## TL;DR
> **Summary**: Replace primitive substring matching with explicit token/exact/fuzzy ranking semantics for docs search and API lookup, backed by regression tests.
> **Estimated Effort**: Medium

## Context
### Original Request
Create execution-ready Tapestry plans for **P7 search/API lookup primitive substring behavior**.

### Key Findings
- `src/docs.rs::search_sections` normalizes the whole query and uses `contains` scoring across fields; multi-word and short-token behavior can produce noisy matches.
- `src/api.rs::lookup_symbol` tries exact matches, then substring fuzzy matches over name, summary, and aliases; ambiguous short queries are tested only minimally.
- Search output includes `score`, `matched_fields`, `why`, and `next_actions`, so ranking changes are user-visible.

## Objectives
### Core Objective
Make search and API lookup predictable, explainable, and less noisy while preserving useful aliases.

### Deliverables
- [ ] Characterization tests for current noisy/desired search and lookup cases before algorithm changes.
- [ ] Query normalization/tokenization model with explicit minimum token handling.
- [ ] Ranked docs search with exact ID/alias/title boosts, token intersection, and deterministic tie-breaking.
- [ ] API lookup behavior that distinguishes exact symbol/alias, prefix, token, and summary matches.
- [ ] Breaking-change notes for changed ambiguous/unknown outcomes.

### Definition of Done
- [ ] `cargo test docs::tests::search`-style tests pass.
- [ ] `cargo test api::tests::lookup`-style tests pass.
- [ ] Short/common queries produce deterministic ambiguous or unknown outcomes rather than arbitrary substring matches.

### Guardrails (Must NOT)
- [ ] Must not remove declared aliases without catalog migration notes.
- [ ] Must not add external search dependencies for this small static catalog unless current requirements expand.
- [ ] Must not change docs content to game ranking; fix the algorithm/metadata instead.

## TODOs

- [ ] 1. Add search characterization tests
  **What**: Cover exact section ID, alias, title, multi-token query (`server function`), short/noisy token (`as`, `get`, `api`), common error phrases, and deterministic ordering.
  **Files**: `src/docs.rs`
  **Acceptance**: Tests describe both preserved and intentionally changed ranking outcomes.

- [ ] 2. Add API lookup characterization tests
  **What**: Cover exact symbols, aliases, crate-filtered lookups, prefix-like terms, ambiguous terms (`extractor`, `response`), and too-short/noisy queries.
  **Files**: `src/api.rs`
  **Acceptance**: Tests fail if fuzzy substring behavior returns an overconfident single symbol for ambiguous input.

- [ ] 3. Implement shared query tokenization helpers
  **What**: Introduce normalization/tokenization helpers that split punctuation/case consistently, drop or reject tokens below a minimum length where appropriate, and preserve exact phrase matching where valuable.
  **Files**: `src/docs.rs`, `src/api.rs`
  **Acceptance**: Unit tests pin normalization of `leptos_axum::ResponseOptions`, `#[server]`, and multi-word phrases.

- [ ] 4. Refactor docs scoring
  **What**: Score exact ID/alias/title matches highest, then all-token field matches, then partial token matches; update `matched_fields` and `why` to explain the strongest match.
  **Files**: `src/docs.rs`, `src/tools.rs`
  **Acceptance**: Results are deterministic and less noisy for short/common inputs.

- [ ] 5. Refactor API lookup phases
  **What**: Keep exact symbol/alias resolution, then add controlled prefix/token matching; return `Ambiguous` when multiple same-tier symbols match rather than falling through to broad summary substring matches.
  **Files**: `src/api.rs`, `src/tools.rs`
  **Acceptance**: API lookup no longer returns a single low-quality match for vague queries.

- [ ] 6. Update protocol tests if output changes
  **What**: Adjust tests that assert specific lookup/search responses to match the new ranking/error contract.
  **Files**: `src/protocol.rs`
  **Acceptance**: Protocol still returns `-32602` for ambiguous/unknown lookups and valid structured content for successful searches.

- [ ] 7. Document breaking lookup/search changes
  **What**: Add release notes for queries that may now return ambiguous/unknown instead of a low-confidence result.
  **Files**: `CHANGELOG.md`
  **Acceptance**: Clients are told to use exact IDs, aliases, or crate filters for stable automation.

## Verification
- [ ] Run `cargo test docs`.
- [ ] Run `cargo test api`.
- [ ] Run `cargo test tools`.
- [ ] Run `cargo test protocol::tests::api_lookup_tool_returns_symbol_metadata`.
- [ ] Run `cargo test` before merging.

## Breaking-Change Notes
- [ ] Intentional breaking change: some vague substring queries may become `Ambiguous` or `Unknown`.
- [ ] Exact section IDs, resource URIs, and declared aliases should remain stable.

## Migration/Docs Notes
- [ ] Recommend clients use `list-sections` and exact `lookup-api` aliases, plus `crate` filters for API lookup stability.

## Risks
- [ ] Ranking changes can surprise interactive users; mitigate with `why`/`matched_fields` clarity.
- [ ] Overengineering search is possible; keep algorithm static-catalog appropriate.

## Rollback / Stop Conditions
- [ ] Stop if tests reveal ranking changes overlap catalog duplication issues; complete Plan 05 first.
- [ ] Roll back broad fuzzy matching changes if they significantly reduce known useful results without replacement aliases.

## Dependencies
- [ ] Plan 05 is recommended first for catalog alias/metadata invariants.
- [ ] Independent of transport/protocol Plan 04 except for protocol assertion updates.
