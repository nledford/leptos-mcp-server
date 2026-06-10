# Plan 02: Stdin Size and Framing Hardening

## TL;DR
> **Summary**: Close the oversized unterminated stdin denial-of-service gap by enforcing bounded reads even when clients never send a newline.
> **Estimated Effort**: Short

## Context
### Original Request
Create execution-ready Tapestry plans for **P2 oversized unterminated stdin DoS**, a security-sensitive input validation issue requiring Warp review.

### Key Findings
- `src/protocol.rs` reads newline-delimited JSON-RPC over stdio with `MAX_JSON_RPC_LINE_BYTES = 1024 * 1024`.
- `read_limited_line` discards until newline after oversize, but an attacker can keep stdin open with an unterminated oversized line and force unbounded waiting/work depending on pipe behavior.
- Existing tests cover oversized strings passed to `handle_line`, max-sized newline input, and stdio happy paths; they do not cover unterminated oversized process input.

## Objectives
### Core Objective
Guarantee bounded memory and bounded response behavior for oversized, malformed, or unterminated stdio frames.

### Deliverables
- Regression test for oversized unterminated stdin input before remediation.
- Hardened line reader behavior that returns an error response without waiting for newline once the byte limit is exceeded.
- Instrumented reader test proving the byte-limit branch returns immediately at `MAX_JSON_RPC_LINE_BYTES + 1` without discard or extra `fill_buf` reads.
- Process-level stdio test proving a live client receives a JSON-RPC error before newline, EOF, or stdin close.
- Security review note for Warp covering limits, error semantics, and residual risks.

### Definition of Done
- `cargo test stdio_process_rejects_oversized_unterminated_live_input_before_stdin_close` passes with a bounded timeout.
- `cargo test protocol::tests::oversized_unterminated_stdin_line_is_rejected` passes.
- `cargo test protocol::tests::read_limited_line_stops_at_hard_cap_without_discard` passes and proves no `discard_until_newline` call or post-cap `fill_buf` read occurs.
- Oversized unterminated live input receives a JSON-RPC `-32600` response before newline/EOF/stdin close.
- Oversized input never allocates more than `MAX_JSON_RPC_LINE_BYTES` plus bounded overhead.
- Memory remains bounded without post-limit discard buffering for malicious clients that keep stdin open.

### Guardrails (Must NOT)
- Must not raise `MAX_JSON_RPC_LINE_BYTES` to hide the issue.
- Must not buffer an entire malicious line while searching for newline.
- Must not emit non-JSON text on stdout.

## TODOs

- [ ] 1. Add unit regression for unterminated oversized reads
  **What**: Test `read_limited_line` with input longer than `MAX_JSON_RPC_LINE_BYTES` and no trailing newline; assert it returns `LineRead::Oversized` promptly.
  **Files**: `src/protocol.rs`
  **Acceptance**: The test captures the intended behavior before implementation and passes after the reader is hardened.

- [ ] 2. Add instrumented hard-cap reader regression
  **What**: Add an instrumented `BufRead`/reader test that feeds exactly `MAX_JSON_RPC_LINE_BYTES + 1` bytes without a newline and records reader operations.
  **Files**: `src/protocol.rs`
  **Acceptance**: The test proves `read_limited_line` returns `LineRead::Oversized` immediately after observing byte `MAX_JSON_RPC_LINE_BYTES + 1`, does not call `discard_until_newline`, and performs no further `fill_buf` reads beyond the hard cap.

- [ ] 3. Add process-level live-client stdio regression
  **What**: Add an integration test that starts the server process, writes oversized unterminated JSON-RPC-ish input, keeps stdin open, and waits with a bounded timeout for stdout.
  **Files**: `tests/stdio.rs`
  **Acceptance**: The test observes one JSON-RPC `-32600` response with `id: null` before stdin is closed and fails fast on timeout, proving the live-client DoS is fixed rather than only EOF behavior.

- [ ] 4. Harden bounded read implementation
  **What**: Ensure `read_limited_line` consumes at most the bounded frame, returns `Oversized` as soon as the limit is exceeded, and cannot loop indefinitely waiting for a newline after EOF/oversize.
  **Files**: `src/protocol.rs`
  **Acceptance**: Reader behavior is deterministic for newline, EOF, exact-limit, limit-plus-one, and invalid UTF-8 inputs.

- [ ] 5. Review JSON-RPC error semantics
  **What**: Confirm oversized frames return `InvalidRequest` with `id: null`; document why request ID cannot be trusted/read after rejecting oversized input.
  **Files**: `src/protocol.rs`
  **Acceptance**: Unit tests assert `-32600` and `id == null` for oversized read-path errors.

- [ ] 6. Prepare Warp security review note
  **What**: Add a concise release/changelog note identifying the DoS class, fixed bound, and residual absence of wall-clock read timeouts for clients that keep pipes open without exceeding the limit.
  **Files**: `CHANGELOG.md`
  **Acceptance**: Note is marked security-sensitive and ready for Warp review.

## Verification
- [ ] Run `cargo test protocol::tests::max_sized_stdin_line_with_newline_is_accepted`.
- [ ] Run `cargo test protocol::tests::oversized_unterminated_stdin_line_is_rejected`.
- [ ] Run `cargo test protocol::tests::read_limited_line_stops_at_hard_cap_without_discard`.
- [ ] Run `cargo test stdio_process_rejects_oversized_unterminated_live_input_before_stdin_close`.
- [ ] Run `cargo test --test stdio`.
- [ ] Run `cargo fmt -- --check`.
- [ ] Run `cargo clippy --locked --all-targets -- -D warnings`.
- [ ] Run `cargo test` before merging.
- [ ] Fix any discovered or introduced test failures, compilation/type errors, Clippy warnings, or formatting failures in affected code before marking this plan complete.

## Breaking-Change Notes
- Behavior-preserving for valid clients.
- Invalid clients sending oversized unterminated frames may receive earlier rejection than before.

## Migration/Docs Notes
- Document the exact maximum frame size and newline-delimited framing expectation if user-facing docs are updated.

## Risks
- A naive discard loop can still block on live malicious clients; prefer returning after bounded consumption instead of waiting for delimiter.
- Integration tests can hang if cleanup is wrong; keep child stdin open until the expected `-32600` response is observed or the bounded timeout fires, then close stdin only during cleanup.

## Rollback / Stop Conditions
- Stop if a fix requires async stdio/timeouts; create a separate transport plan rather than mixing runtimes.
- Roll back to prior reader only if bounded-memory tests remain in place and issue is escalated.

## Dependencies
- Independent of other plans.
- Should be prioritized before release and before Plan 04 transport abstraction if both are active.
