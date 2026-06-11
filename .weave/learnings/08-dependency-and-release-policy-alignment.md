# Learnings: 08-dependency-and-release-policy-alignment

## Task 1: Inventory current policy and drift points
- **Discrepancy**: Start-work validation warned that referenced files such as `src/api.rs`, `src/docs.rs`, `docs/releasing.md`, `README.md`, and `CHANGELOG.md` might not exist, but the files exist in the repository.
- **Resolution**: Ignored the stale warnings and verified actual repository files during the inventory task.
- **Suggestion**: Resolve plan file references relative to the repository root before emitting missing-file warnings.
