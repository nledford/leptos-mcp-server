# Learnings: 04 Protocol Capability and Transport Boundaries

## Task 1: Add behavior characterization tests
- **Discrepancy**: The start-work validation warnings claimed referenced files such as `src/protocol.rs`, `tests/stdio.rs`, `src/lib.rs`, `src/tools.rs`, `src/docs.rs`, and `src/prompts.rs` did not exist, but they are present in the repository.
- **Resolution**: Ignored the stale file-reference warnings and verified against the actual repository files.
- **Suggestion**: Plan validation should resolve paths relative to the repository root before warning that source files are missing.
