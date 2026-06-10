# Learnings: 05 Catalog Metadata Single Source

## Task 1: Add catalog invariant tests
- **Discrepancy**: The start-work validation warnings claimed referenced files such as `src/docs.rs`, `src/tools.rs`, `src/protocol.rs`, and `src/api.rs` did not exist, but they are present in the repository.
- **Resolution**: Ignored the stale file-reference warnings and verified against the actual repository files.
- **Suggestion**: Plan validation should resolve paths relative to the repository root before warning that source files are missing.
