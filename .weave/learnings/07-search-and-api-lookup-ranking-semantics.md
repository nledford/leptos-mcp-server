# Learnings: 07-search-and-api-lookup-ranking-semantics

## Task 1: Add search characterization tests
- **Discrepancy**: Start-work validation warned that `src/docs.rs` did not exist, but the file exists in the repository.
- **Resolution**: Ignored the stale warning and verified the actual repository file before delegating and verifying the task.
- **Suggestion**: Resolve plan file references relative to the repository root before emitting missing-file warnings.

## Task 2: Add API lookup characterization tests
- **Discrepancy**: Start-work validation warned that `src/api.rs` did not exist, but the file exists in the repository.
- **Resolution**: Ignored the stale warning and verified the actual repository file before delegating and verifying the task.
- **Suggestion**: Resolve plan file references relative to the repository root before emitting missing-file warnings.
