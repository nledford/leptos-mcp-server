# Learnings: 03 Prompt Required Argument Enforcement

## Task 1: Add prompt-domain tests first
- **Discrepancy**: The start-work validation warnings claimed referenced files such as `src/prompts.rs` and `src/protocol.rs` did not exist, but they are present in the repository.
- **Resolution**: Ignored the stale file-reference warnings and verified against the actual repository files.
- **Suggestion**: Plan validation should resolve paths relative to the repository root before warning that source files are missing.
