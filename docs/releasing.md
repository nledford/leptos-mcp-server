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
- **Credential gate:** the tag/GitHub Release job targets the protected
  `release` GitHub Environment and must not receive release credentials until
  the dry-run/manual verification gate below has been completed.

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
6. The tag/GitHub Release job is bound to the protected `release` Environment,
   so maintainers can require manual approval before any write-scoped release
   token is exposed to the job.

## Release token and workflow safety policy

Release credentials are narrowly scoped and are only exposed to jobs that need
them:

- The workflow default is `permissions: {}`. Each job declares only the minimum
  permissions it needs.
- `preflight` uses `contents: read` only.
- `release-plz-release` may use `contents: write` to create `vX.Y.Z` tags and
  GitHub Releases, plus `pull-requests: read` for release-plz metadata checks.
- `release-plz-pr` may use `contents: write` and `pull-requests: write` only to
  create or update the release PR.
- While `release-plz.toml` keeps `git_only = true`, no crates.io,
  GitHub Packages, Docker, binary-distribution, or deployment publishing token is
  allowed in the workflow.
- Do not add `id-token: write`, `packages: write`, `CARGO_REGISTRY_TOKEN`, or
  other package-publishing credentials unless publishing is intentionally added
  in a reviewed release-policy change.
- If `RELEASE_PLZ_TOKEN` is configured, it must be a fine-grained
  repository-scoped PAT or GitHub App token limited to Contents and Pull Requests
  write access for this repository only. Prefer the default `GITHUB_TOKEN` when
  it satisfies branch-protection and release-PR requirements.
- Never print tokens, secret names with values, authorization headers, `env`,
  GitHub context dumps, or release-plz debug output that could include
  credentials. Do not enable shell tracing (`set -x`) in release jobs.

Release publication is limited to protected refs:

- The Release workflow must remain triggered by `workflow_run` for a successful
  `CI` run from a `push` to `main`; it must not publish from `pull_request`, tag,
  scheduled, or arbitrary branch triggers.
- The `preflight` job and each write-capable job must verify that the
  CI-validated SHA is still the current `main` SHA immediately before any
  write-scoped release command runs.
- Protect `main` and require the `CI success` check before merging release PRs.
- Protect release tags matching `v*`/`vX.Y.Z` so maintainers review who or what
  can create/update release refs. Normal releases should be created only by the
  approved Release workflow; manual tag creation is emergency-only.

Before release credentials can create tags, GitHub Releases, or future published
artifacts, maintainers must complete this concrete gate:

1. Run `release-plz release --dry-run` and the binary smoke check from
   [Maintainer release steps](#maintainer-release-steps) on the exact release
   commit, and confirm the dry-run proposes only expected tag/release actions.
2. Review release token scopes, workflow permissions, protected `main`, and
   `v*` tag protection.
3. Approve the protected `release` GitHub Environment for the
   `release-plz-release` job only after those checks pass. Configure the
   Environment with required reviewers and do not store unrelated publishing
   secrets in it.

## Maintainer release steps

Use this section as the maintainer checklist for the `0.2.0-alpha.*` migration
line, the final `0.2.0` release, and any decision to defer v1. The goal is to
avoid reconstructing the SDK migration context at release time: `0.2.0-alpha.1`
and later alphas are pre-v1 validation releases for the `rust-mcp-sdk` stdio
migration; `0.2.0` is the first non-alpha release of that migration after client
validation is complete; v1 remains deferred unless maintainers explicitly decide
the criteria below are met.

### Current CI parity commands

Run or confirm the matching CI jobs for the exact commit being released. The
commands below mirror `.github/workflows/ci.yml` as of the `0.2.0-alpha.*`
release line:

```bash
cargo +1.96.0 fmt --check
cargo +1.96.0 check --locked
cargo +1.96.0 test --locked
cargo +1.96.0 clippy --locked --all-targets -- -D warnings

cargo +1.96.0 check --locked
cargo +1.96.0 test --locked
cargo +1.96.0 check --locked --no-default-features --features stdio
cargo +1.96.0 test --locked --no-default-features --features stdio
cargo +1.96.0 check --locked --all-features
cargo +1.96.0 test --locked --all-features

cargo +1.96.0 tree --locked -e features --features stdio --depth 1
cargo +1.96.0 build --release --locked
cargo llvm-cov --locked --summary-only --fail-under-lines 70
cargo audit
cargo deny check
```

Also run the feature-policy assertions from CI, not just the `cargo tree`
command: default features must remain `default = ["stdio"]`, `stdio` must map to
`rust-mcp-sdk/stdio`, `rust-mcp-sdk` must stay on `default-features = false` with
only `server` and `macros` declared directly, and no public HTTP/SSE/WebSocket,
TCP, auth, or OAuth features may appear. If `cargo-llvm-cov`, `cargo-audit`,
`cargo-deny`, or a named toolchain is not installed locally, install it or cite
the passing CI job in the release PR notes.

For documentation, recipe, and API example changes, also run:

```bash
cargo test snippets
```

### Manual MCP validation checklist

Before approving a release PR or release Environment job, validate the release
binary manually over stdio and through at least one real MCP client configuration.

Raw stdin smoke flow:

```bash
cargo build --release --locked

echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"release-smoke","version":"0"}}}' \
  | ./target/release/leptos-mcp-server \
  > /tmp/leptos-mcp-initialize.json
python3 -m json.tool /tmp/leptos-mcp-initialize.json >/dev/null

echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | ./target/release/leptos-mcp-server \
  > /tmp/leptos-mcp-tools.json
python3 -m json.tool /tmp/leptos-mcp-tools.json >/dev/null

echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list-sections","arguments":{}}}' \
  | ./target/release/leptos-mcp-server \
  > /tmp/leptos-mcp-list-sections.json
python3 -m json.tool /tmp/leptos-mcp-list-sections.json >/dev/null

echo '{"jsonrpc":"2.0","id":4,"method":"resources/templates/list","params":{}}' \
  | ./target/release/leptos-mcp-server \
  > /tmp/leptos-mcp-resource-templates.json
python3 -m json.tool /tmp/leptos-mcp-resource-templates.json >/dev/null

echo '{"jsonrpc":"2.0","id":5,"method":"prompts/list","params":{}}' \
  | ./target/release/leptos-mcp-server \
  > /tmp/leptos-mcp-prompts.json
python3 -m json.tool /tmp/leptos-mcp-prompts.json >/dev/null
```

Confirm the responses advertise only the implemented capabilities: tools,
concrete documentation resources, `resources/templates/list`, and prompts.
Completion must remain absent. Confirm stderr contains logs only and stdout is
valid JSON-RPC.

Client smoke flow:

- Configure a local/stdio MCP client such as Claude Desktop, Antigravity, or
  OpenCode with an absolute path to `target/release/leptos-mcp-server` and
  `--transport stdio`; do not configure a URL.
- Restart the client and confirm it discovers the six tools documented in
  `README.md`: `list-sections`, `get-documentation`, `search-docs`,
  `lookup-api`, `leptos-axum-recipe`, and `leptos-diagnostics`.
- Exercise at least one docs lookup (`get-documentation` for `signals`), one
  search (`search-docs` for `Axum state`), one API lookup (`ResponseOptions` in
  `leptos_axum`), one recipe (`ssr-app`), resource listing/reading, and prompt
  listing/rendering.
- Test an invalid tool argument or unknown lookup and confirm the client handles
  SDK-native JSON-RPC/MCP errors without assuming `0.1.0` custom error text.
- Record the client name/version, binary path, commit SHA, and any observed
  migration incompatibilities in the release PR or release notes.

### `0.2.0-alpha.1` checklist

- Confirm the release is explicitly labeled pre-v1 and alpha in the release PR,
  GitHub Release notes, and changelog entry.
- Confirm `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and the release PR title
  agree on `0.2.0-alpha.1` / `v0.2.0-alpha.1`.
- Run or cite the current CI parity commands above, including feature matrix,
  feature policy, build-smoke, coverage, and dependency-policy jobs.
- Complete the raw stdin and MCP client smoke flows above against the release
  binary.
- Review docs for migration context: README capability snapshot, migration notes,
  protocol section, performance/input limits, MCP smoke tests, and this release
  document must all agree that stdio is the only implemented transport,
  completion is absent, and errors are SDK-native.
- Review `CHANGELOG.md` for SDK migration scope, migration notes from `0.1.0`,
  security-sensitive notes, breaking diagnostic behavior, breaking prompt
  behavior, and breaking lookup/search behavior.
- Confirm network transports remain disabled/deferred: no Cargo features for
  HTTP/SSE/network/auth/CORS, unsupported transport selections fail closed before
  tracing/server startup/listener creation, and no network security claims are
  made.
- Run `release-plz release-pr --dry-run` when checking the generated release PR
  locally, and run `release-plz release --dry-run` before approving tag/GitHub
  Release creation. The dry-runs must propose only the expected alpha release
  actions.
- Do not enable crates.io, package registry, Docker, binary artifact, or hosted
  endpoint publishing; `release-plz.toml` must remain `git_only = true`.

### Subsequent `0.2.0-alpha.N` checklist

- Use another alpha when client validation, docs wording, SDK-native protocol
  behavior, or release automation still needs field feedback before `0.2.0`.
- Preserve the same release gates as `0.2.0-alpha.1`; do not weaken CI parity,
  manual stdio validation, client validation, docs review, dependency review, or
  token/workflow safety checks.
- Re-test any MCP client or automation that reported an incompatibility in a
  previous alpha, and document whether the issue is fixed, accepted as a breaking
  migration behavior, or still open.
- Update `CHANGELOG.md` with every user-facing alpha delta, especially breaking
  SDK-native error/result behavior, stdio framing/malformed-input observations,
  diagnostics severity/confidence changes, prompt argument validation changes,
  lookup/search behavior changes, and docs/capability corrections.
- Keep network transports deferred unless the separate network security review
  below has been completed and the implementation has its own tests and release
  notes. An alpha tag by itself is not approval to expose a network listener.

### `0.2.0` checklist

- Cut `0.2.0` only after at least one `0.2.0-alpha.*` has passed CI, raw stdio
  smoke testing, and real MCP client validation without unresolved release-blocking
  migration issues.
- Confirm all known `0.1.0` to `0.2.0` migration differences are documented in
  README and `CHANGELOG.md`, including SDK-native errors, structured content,
  resources/templates, absent completion, removed custom 1 MiB stdin line-limit
  semantics, and deferred network transports.
- Confirm `CHANGELOG.md` no longer leaves release-critical migration context only
  under `Unreleased`; the generated `release-plz` release PR must move it into the
  `0.2.0` release entry or otherwise produce equivalent GitHub Release notes.
- Re-run or cite the full current CI parity commands, raw stdin smoke flow, and
  MCP client smoke flow on the exact `0.2.0` commit.
- Confirm `release-plz.toml` remains consistent with intended publishing status:
  `git_only = true`, package-level `git_tag_enable = true`, and package-level
  `git_release_enable = true` for Git tags/GitHub Releases only.
- Confirm no v1 claims are made in docs, release notes, or client guidance.

### Network transport security review gate

Network support must not be enabled, documented as available, exposed behind a
Cargo feature, or made publicly reachable until a separate security review has
approved the implementation and release notes. That review must cover at least:

- explicit opt-in transport selection and safe defaults (`stdio` remains default;
  any host default is loopback such as `127.0.0.1`; ports are explicit);
- authentication/authorization strategy, including a decision for local-only,
  private network, and public exposure cases;
- CORS policy with no wildcard default for credentialed/public use;
- HTTP request body limits, JSON-RPC/message size limits, malformed-input
  handling, read/request/handler timeouts, connection limits, and backpressure;
- logging and error sanitization for malformed requests, credentials, headers,
  request bodies, and panics;
- tests proving unsupported or disabled transports fail closed before listener
  creation, plus positive tests for any enabled network listener and negative
  tests for auth/CORS/limit failures;
- release documentation that distinguishes stdio support from network support and
  avoids public-service security claims unless the controls are implemented.

Any PR that enables HTTP, streamable HTTP, SSE, WebSocket, TCP, auth, OAuth,
CORS, or network timeout/limit behavior is security-sensitive and requires Warp
review before release.

### v1 deferral criteria

Do not cut `1.0.0` as part of the `0.2.0-alpha.*` or `0.2.0` process unless
maintainers explicitly decide all v1 criteria are met. Defer v1 when any of the
following are true:

- the SDK migration has not completed at least one alpha and one non-alpha
  release cycle with successful MCP client validation;
- supported capability boundaries are still changing, including completion,
  resources/templates, prompts, diagnostics contracts, or network transport
  posture;
- malformed-input, stdio framing, SDK-native error/result shapes, or structured
  output behavior still need compatibility feedback from downstream clients;
- network transports are still disabled/deferred or have not completed the
  separate security review gate if maintainers want network support in v1;
- release automation, token scope, tag protection, changelog generation, or
  publishing/distribution policy is still being revised;
- README, `CHANGELOG.md`, embedded docs/catalog metadata, Agent Skill guidance,
  and release notes are not aligned on supported behavior; or
- maintainers are not ready to treat the current tool/resource/prompt schemas and
  MCP behavior as stable public contracts.

If all v1 criteria are met later, open a dedicated v1 release-policy PR instead
of promoting an alpha or `0.2.0` release opportunistically.

1. Merge user-facing changes to `main` using Conventional Commit messages.
2. Wait for CI to pass on `main`.
3. Before reviewing or merging a release PR, run the commands in
   [Current CI parity commands](#current-ci-parity-commands), including the
   feature matrix, feature-policy assertions, build-smoke, coverage, and
   dependency-policy checks.

   If `cargo-audit`, `cargo-deny`, or a named toolchain is not installed locally,
   either install it and rerun the command or confirm the matching CI job passed
   and document the local skip in the PR.
4. For documentation, recipe, and API example changes, run the snippet validation
   harness locally even if the full gate above is skipped for a docs-only PR:

   ```bash
   cargo test snippets
   ```

   If any Rust docs, recipe, or API example was added, removed, or changed from a
   complete example to an illustrative/ignored example (or back), update its
   snippet classification before publishing.
5. Align the embedded catalog versions in one pass; see
   [Embedded catalog version alignment](#embedded-catalog-version-alignment).
6. Perform the required human review before merging the generated `release-plz`
   PR:

   - Confirm `Cargo.toml`, `Cargo.lock`, and the PR title/body agree on the next
     `vX.Y.Z` release.
   - Review `CHANGELOG.md` for every user-facing change, breaking-change note,
     dependency/security note, and snippet/docs behavior change included since the
     previous tag.
   - Confirm `README.md`, public API metadata, embedded docs, recipe text,
     `CHANGELOG.md`, and the catalog target-version statements remain aligned.
   - If catalog targets changed, review the docs URL policy at the same time:
     Axum URLs are pinned to the reviewed patch version, while Leptos and
     leptos_axum may intentionally stay on docs.rs `latest` only when every owned
     symbol URL still resolves under that policy.
   - Review any `Cargo.lock` diff for unexpected transitive upgrades and rerun or
     confirm `cargo audit` and `cargo deny check`.
   - Request Warp review for every security-sensitive release item listed in
     [Security-sensitive Warp review triggers](#security-sensitive-warp-review-triggers).
7. Before any tag or GitHub Release is allowed to be created, perform a dry-run
   and manual verification gate on the exact release commit:

   ```bash
   release-plz release --dry-run
   echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
     | ./target/release/leptos-mcp-server \
     > /tmp/leptos-mcp-smoke.json
   python3 -m json.tool /tmp/leptos-mcp-smoke.json >/dev/null
   ```

   The dry-run must show only the expected tag/release actions, and the manual
   smoke response must be valid JSON-RPC containing the advertised tool list. When
   checking release-PR generation locally, use:

   ```bash
   release-plz release-pr --dry-run
   ```
8. Confirm release workflow and token safety before merging the release PR:

   - `.github/workflows/release.yml` must still run only after successful `CI` on
     a push to `main`, verify the validated SHA is still current `main`, and keep
     `concurrency.group` scoped to `release-main`.
   - Job permissions must stay limited to the documented `contents` and
     `pull-requests` access needed by each release-plz command.
   - While `release-plz.toml` has `git_only = true`, no `CARGO_REGISTRY_TOKEN`,
     crates.io trusted-publishing `id-token: write`, package registry token, or
     binary-distribution secret should be present or required.
   - If `RELEASE_PLZ_TOKEN` is configured, it must be repository-scoped and limited
     to Contents and Pull Requests write access; otherwise the workflow should use
     the default `GITHUB_TOKEN`.
9. Merge the release PR after required checks, human reviews, Warp review when
   triggered, dry-run/manual verification, and token/workflow safety checks pass.
10. Wait for CI and the Release workflow to complete. The workflow creates the
   `vX.Y.Z` tag and GitHub Release.

Do not manually create release tags during the normal process. For emergency
recovery, run the full CI command set locally first, ensure `Cargo.toml`,
`Cargo.lock`, `CHANGELOG.md`, and the intended tag agree, and then push exactly
one `vX.Y.Z` tag.

## Dependency version and lockfile policy

This repository is an application-style MCP server that is built and tested from
`Cargo.lock`, but its direct dependency declarations in `Cargo.toml` may use
broad compatible SemVer ranges for mature Rust ecosystem libraries. Keep broad
caret ranges such as `tokio = "1"`, `serde = "1"`, or `tracing = "0.1"` when all
of the following are true:

- the dependency has a stable public compatibility policy and frequent patch
  releases;
- the dependency is exercised by CI with `--locked` on Rust 1.96.0;
- the current range does not admit a known security, MSRV, license, or runtime
  behavior problem; and
- the resolved version in `Cargo.lock` is reviewed as part of release and
  dependency-update PRs.

Pin or narrow a direct dependency range when compatibility risk is higher: a
pre-1.0 dependency, a dependency without a clear compatibility policy, a known
bad release within the broad range, an MSRV increase beyond Rust 1.96, a security
or license restriction, a generated-code/build-tool dependency that affects
reproducibility, or a runtime/protocol dependency whose minor releases can alter
observable MCP behavior. Document the reason in the PR when narrowing or pinning.

Update `Cargo.lock` only when the resolved dependency graph is intentionally
changing:

- dependency declarations, features, or build tooling changed;
- a security, yanked-release, MSRV, or compatibility update is being applied;
- `release-plz` updates the lockfile in a release PR; or
- CI fails because the locked graph is no longer valid.

Do not refresh `Cargo.lock` for docs-only or formatting-only changes. When the
lockfile changes, review the diff for unexpected transitive upgrades, confirm the
package version still matches the intended release state, and run the locked CI
checks relevant to the change.

Run dependency audits with `cargo audit` and `cargo deny check` before merging a
dependency-update PR, before merging a release PR that changes `Cargo.lock`, when
a RustSec or license advisory applies to the dependency graph, and at least once
per release cycle even if no dependency PR was opened. If the tools are not
installed locally, rely on CI or document the skipped local audit in the PR.

Warp review is required before merging dependency changes that pin, narrow, or
loosen direct dependency ranges; introduce new direct dependencies or features;
change runtime/protocol, build, or release tooling dependencies; apply security
advisory fixes; or raise the effective MSRV. Warp review is not required for a
release-plz PR whose only dependency-related change is the expected lockfile
metadata/version update and whose CI and audits pass.

## Security-sensitive Warp review triggers

Request Warp review before merging or releasing any security-sensitive change in
these categories:

- input validation or malformed-client handling, including JSON-RPC validation,
  prompt argument validation, and stdio framing errors;
- frame-size, line-bound, oversized-input, unterminated-frame, or rejection-timing
  behavior for MCP stdin/stdout transport;
- dependency policy, direct dependency range/feature changes, lockfile policy,
  audit/deny policy, security advisory handling, release tooling dependencies, or
  effective MSRV changes;
- release automation behavior that can create release PRs, tags, GitHub Releases,
  artifacts, or future package publishes;
- release token, publishing secret, GitHub App, PAT, `GITHUB_TOKEN`,
  `RELEASE_PLZ_TOKEN`, `CARGO_REGISTRY_TOKEN`, or trusted-publishing
  configuration changes;
- tag protection, protected refs/environments, manual approval gates, or branch
  protection requirements for release refs; and
- GitHub Actions workflow permission changes, especially `contents`,
  `pull-requests`, `id-token`, package, or environment access.

Security-sensitive malformed input and frame-size behavior changes must be called
out in the release notes. For the `0.2.0-alpha.*` SDK migration line, this means
calling out that the previous project-specific stdio JSON-RPC line reader and 1
MiB line-bound semantics were removed, and that stdio framing and malformed-input
behavior are now inherited from `rust-mcp-sdk`. Security-sensitive dependency,
release policy, token, tag-protection, and workflow-permission changes must also
be visible in the release PR review checklist and changelog before publishing.

## Breaking-change release note policy

Backward compatibility is not prioritized over correctness, security, or clearer
MCP protocol behavior. When an intentional user-visible incompatibility is
merged, maintainers must record it in the release PR and final release notes even
if the change fixes previously invalid, malformed, ambiguous, or undocumented
client behavior.

Use a dedicated changelog subsection named for the affected behavior, such as
`### Breaking stdio framing behavior`, `### Breaking diagnostic behavior`,
`### Breaking prompt behavior`, or `### Breaking lookup/search behavior`. Each
entry must answer the following questions:

```markdown
### Breaking <area> behavior

- What changed: <specific old behavior> now <specific new behavior>.
- Affected clients: <which clients, automations, tests, or malformed requests may
  observe the difference>.
- Error/result contract: <JSON-RPC error code, diagnostic severity/confidence,
  response ID behavior, Ambiguous/Unknown result, or no-result behavior>.
- Required client action: <how clients should update requests, validation,
  assertions, or query strings>.
- Security/reliability note: <why backward compatibility was intentionally not
  preserved, especially for malformed-client or resource-exhaustion behavior>.
```

Security-sensitive malformed-client behavior changes are release-note material.
This includes changes to when invalid input is rejected, how much invalid input is
read before rejection, how oversized or unterminated stdio frames are classified,
whether a response can safely echo the request ID, and which JSON-RPC error code
is returned. For oversized-frame changes, explicitly state the rejection timing,
for example that the server rejects once the hard-cap violation byte is read, and
state whether the response uses `id: null` because the request ID cannot be
trusted after frame-bound failure.

Known intentional breaking-change families that must be documented when present
in a release include:

- SDK migration stdio behavior: removal of the hand-rolled JSON-RPC line reader,
  removal of the project-specific 1 MiB stdin line-bound contract, and adoption
  of SDK-native stdio framing and malformed-input behavior.
- Diagnostic behavior: changed rule severities, confidence levels, and
  any client guidance about treating diagnostics as advisory versus
  compiler-equivalent.
- Prompt behavior: required prompt argument enforcement, blank-value
  rejection, unknown-argument rejection, and the resulting `-32602` errors.
- Lookup/search behavior: revised search ranking, short/common-token
  suppression, exact ID/alias preferences, and `Ambiguous` or `Unknown` outcomes
  replacing broad substring matches.

Do not hide these changes under generic `Fixed`, `Changed`, or `Security`
headings alone. If a security-sensitive fix also breaks malformed or invalid
client behavior, include both the security-sensitive context and the breaking
contract in the release note.

## Embedded catalog version alignment

The embedded documentation/API catalog has its own curated target versions. These
targets are policy data, not the server package version and not necessarily the
same as dependency versions resolved in `Cargo.lock`. Update them only when the
catalog is intentionally re-reviewed for a new Leptos/leptos_axum/Axum target.

Make catalog version changes as a single alignment step so public API lookups,
section metadata, docs URLs, and recipes cannot drift. In the same PR, review and
update all of the following together:

- `src/api.rs`: `LEPTOS_VERSION`, `LEPTOS_AXUM_VERSION`, `AXUM_VERSION`, and
  the owned docs URL constants (`LEPTOS_DOCS_URL`, `LEPTOS_AXUM_DOCS_URL`,
  `AXUM_DOCS_URL`). Keep each curated API symbol's `version` and `url` aligned
  with those constants.
- `src/docs.rs`: section titles/use cases/aliases that mention target versions,
  `LEPTOS_VERSION_SCOPE`, crate-version metadata arrays, `source_url` values,
  and version-specific task tags/common guidance. Shared Leptos, leptos_axum,
  and Axum metadata must continue to use the API constants. Third-party SQL
  guidance (`sqlx`, `sea-query`, `sea-query-sqlx`) may remain `latest` unless
  that guidance is being pinned and re-reviewed too.
- `src/recipes.rs`: recipe `crates` strings, summaries, validation text, and
  embedded dependency examples that mention Leptos, leptos_axum, or Axum
  versions.
- User-facing docs such as `README.md` when they state the catalog target
  versions.

Expected examples for an Axum patch-target update from `0.8.9` to `0.8.10`:

- `AXUM_VERSION` becomes `"0.8.10"` and `AXUM_DOCS_URL` becomes
  `https://docs.rs/axum/0.8.10/axum/`.
- Axum API symbol URLs move from `/axum/0.8.9/axum/...` to
  `/axum/0.8.10/axum/...`.
- The Axum section title, use cases, aliases, and task tags that say `0.8.9`
  say `0.8.10`.
- Recipe crate strings such as `"leptos 0.8.19", "leptos_axum 0.8.9",
  "axum 0.8.9"` become the reviewed target set, for example
  `"leptos 0.8.19", "leptos_axum 0.8.9", "axum 0.8.10"`.

Expected examples for a Leptos/leptos_axum catalog refresh:

- `LEPTOS_VERSION` and/or `LEPTOS_AXUM_VERSION` change with their docs URL
  policy reviewed at the same time. If the catalog intentionally tracks
  docs.rs `latest` for Leptos/leptos_axum, keep the URL constants on `latest`
  and verify every symbol URL still resolves under the corresponding owned URL.
- `LEPTOS_VERSION_SCOPE` and any section guidance that names the supported
  Leptos family are updated if the target scope changes.
- Recipe `crates` strings and any embedded dependency examples are updated to
  the same reviewed target versions.

After any catalog alignment, run the relevant focused tests for changed Rust
catalog files, plus `cargo test snippets` if snippets or snippet
classifications changed. For docs-only policy changes, a diff review is
sufficient.

## Required GitHub settings

- Repository Actions workflow permissions must allow the Release workflow's
  job-level permissions: `contents: write` and `pull-requests: write`.
- Enable **Allow GitHub Actions to create and approve pull requests** if using
  the default `GITHUB_TOKEN` for release PRs.
- Protect `main` and require the `CI success` check before merging application
  or release PRs.
- Create a protected GitHub Environment named `release` with required reviewers.
  The `release-plz-release` job must wait for this approval before write-scoped
  credentials can create tags or GitHub Releases.
- Protect release tags matching `v*`/`vX.Y.Z`; restrict who can create/update
  them and prefer the approved Release workflow over manual tag pushes.

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
