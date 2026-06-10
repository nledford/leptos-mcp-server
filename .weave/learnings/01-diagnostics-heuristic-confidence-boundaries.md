# Learnings: 01 Diagnostics Heuristic Confidence Boundaries

## Task 1: Add characterization tests before behavior changes
- **Discrepancy**: The start-work validation warnings claimed referenced files such as `src/diagnostics.rs`, `src/protocol.rs`, and `src/tools.rs` did not exist, but they are present in the repository.
- **Resolution**: Ignored the stale file-reference warnings and verified against the actual repository files.
- **Suggestion**: Plan validation should resolve paths relative to the repository root before warning that source files are missing.
