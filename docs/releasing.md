# Release process

This repository is a Rust/Cargo MCP server. It currently ships source code and a
locally built `leptos-mcp-server` binary; it does not publish to crates.io, npm,
Docker, GitHub Packages, or a deployment target.

## Release automation strategy

- **Version source:** `Cargo.toml` (`package.version`) and `Cargo.lock`.
  Runtime server metadata reads the Cargo version with `CARGO_PKG_VERSION`.
- **Versioning policy:** Semantic Versioning.
- **Commit convention:** Conventional Commits drive release bumps.
- **Tag format:** `vX.Y.Z`.
- **Changelog:** `CHANGELOG.md`, updated by `release-plz` release PRs.
- **Release trigger:** successful push CI on `main` runs the Release workflow.
- **Publishing:** disabled for now with `release-plz` `git_only = true`.

Default bump policy:

| Commit type | Release bump |
| --- | --- |
| `feat!:` or `BREAKING CHANGE:` footer | major |
| `feat:` | minor |
| `fix:`, `perf:`, runtime-affecting security or dependency fixes | patch |
| `docs:`, `test:`, `chore:`, `ci:`, `style:`, pure internal refactors | no release by default |

## Workflow overview

1. Pull requests and pushes to `main` run `.github/workflows/ci.yml`.
2. After CI succeeds for a push to the current `main` commit,
   `.github/workflows/release.yml` runs `release-plz`.
3. `release-plz release` creates missing `vX.Y.Z` tags and GitHub Releases after
   a release PR has been merged.
4. `release-plz release-pr` creates or updates the release PR containing the next
   Cargo version, `Cargo.lock`, and `CHANGELOG.md` changes.
5. The release workflow verifies the CI-validated commit is still current `main`
   before creating tags or release PRs, which avoids releasing stale commits.

## Maintainer release steps

1. Merge user-facing changes to `main` using Conventional Commit messages.
2. Wait for CI to pass on `main`.
3. Before publishing documentation or release changes, run the snippet validation
   harness locally:

   ```bash
   cargo test snippets
   ```

   If any Rust docs, recipe, or API example was added, removed, or changed from a
   complete example to an illustrative/ignored example (or back), update its
   snippet classification before publishing.
4. Review the generated release PR from `release-plz`.
5. Confirm the version bump and changelog match the included commits.
6. Merge the release PR after required checks and reviews pass.
7. Wait for CI and the Release workflow to complete. The workflow creates the
   `vX.Y.Z` tag and GitHub Release.

Do not manually create release tags during the normal process. For emergency
recovery, run the full CI command set locally first, ensure `Cargo.toml`,
`Cargo.lock`, `CHANGELOG.md`, and the intended tag agree, and then push exactly
one `vX.Y.Z` tag.

## Required GitHub settings

- Repository Actions workflow permissions must allow the Release workflow's
  job-level permissions: `contents: write` and `pull-requests: write`.
- Enable **Allow GitHub Actions to create and approve pull requests** if using
  the default `GITHUB_TOKEN` for release PRs.
- Protect `main` and require the `CI success` check before merging application
  or release PRs.

No publishing secret is required while `release-plz.toml` uses `git_only = true`.

Optional secret:

- `RELEASE_PLZ_TOKEN`: a fine-grained, repository-scoped PAT or GitHub App token
  with Contents and Pull Requests write access only. It does not need package
  publishing permissions while `git_only = true`. Use it if release PRs created
  by the default `GITHUB_TOKEN` do not trigger the checks required by branch
  protection.

Future crates.io publishing would require intentionally removing `git_only =
true`, adding package metadata, and configuring either `CARGO_REGISTRY_TOKEN` or
crates.io trusted publishing with `id-token: write`.

Future binary artifact distribution can be layered on with `cargo-dist` after
tag creation is stable.
