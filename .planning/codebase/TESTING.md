# Testing Patterns

**Analysis Date:** 2026-10-21

## Test Framework

**Runner:** built-in Rust `cargo test` (libtest); no external test crates. Test specs: `specs/LIAM_TEST_SPEC_v0.4.md`, `specs/LIAM_TEST_SPEC_v0.4.1.md`.

**Assertion Library:** std `assert!`, `assert_eq!`, `matches!`.

**Run Commands:**
```bash
cargo test --locked --offline --features stubs              # all non-ignored tests
cargo test --locked --offline --features stubs -- --ignored # Windows registry proxy-trap test
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
cargo fmt -- --check
```
CI (`.github/workflows/ci.yml`, `windows-latest`) runs exactly these plus a release build. Integration tests require `--features stubs` (they carry `#![cfg(feature = "stubs")]`). Tests must never make external network requests; only loopback stubs.

## Test File Organization

**Location:** unit tests inline in `#[cfg(test)] mod tests` at the bottom of `src/config.rs`, `src/evidence.rs`, `src/task.rs`, `src/lib.rs`, `src/clients/mod.rs`; integration tests in `tests/`.

**Naming:** long descriptive sentence-style snake_case, e.g. `failed_startup_writes_no_evidence`, `hometown_terminal_statuses_have_the_documented_classification`, `non_loopback_proxy_environment_fails_before_any_request_or_evidence`.

**Structure:**
```
src/<module>.rs           # + #[cfg(test)] mod tests
src/stubs.rs              # feature-gated fixtures (CaptureProxy, HometownStub, OliveStub, StubRecordPair)
tests/process_contract.rs # spawns the real binary, checks exit codes, stderr, evidence
tests/proxy_contract.rs   # client-level checks through CaptureProxy
```

## Test Structure

```rust
#[test]
fn failed_startup_writes_no_evidence() {
    let fixture = StubRecordPair::for_proxy(50413);
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .args(["--purpose", "servicing", "--party", "opaque-party", "--question", "question"])
        .env("LIAM_STUBS", "1").env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", "http://127.0.0.1:50413")
        .env("NONO_RECEIPT_PATH", fixture.directory().join("missing-receipt.json"))
        .output().unwrap();
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim_end(), "liam: 10 receipt:missing");
}
```

**Patterns:**
- Process tests spawn `env!("CARGO_BIN_EXE_liam")`, set `LIAM_STUBS=1`/`LIAM_STUB_PID`, proxy env vars, and `NONO_*_PATH`, write a per-test `liam.toml` into the fixture temp dir (paths with forward slashes, vocabulary sha256 pinned) and set `current_dir` to it.
- Assert exact exit code and exact stderr line `liam: {code} {reason}` for fail-closed paths; assert no request reached stubs and no evidence written.
- `unwrap()`/`expect()` are acceptable in tests only.
- Temp dirs come from `std::env::temp_dir().join(format!("liam-...-{}", Uuid::now_v7()))`.

## Mocking

**Framework:** none; hand-written fakes.

**Patterns:**
- Trait fakes for unit tests (`struct Hometown { response }` implementing `HometownClient`; `struct Olive` implementing `OliveClient`, with asserts inside the fake) in `src/task.rs`.
- Real loopback HTTP via `tiny_http` stubs in `src/stubs.rs`: `HometownStub::{answer_without_proposal, answer_with_proposal, server_error, server_error_then_answer, server_error_twice, timeout_twice, status(..)}`, `OliveStub::{awaiting_approval, status(..)}`, `CaptureProxy::{forward_once_to, forward_two, ...}` which simulates nono-proxy key injection at its outbound boundary. Read results with `.captured()` (request counts, bodies).

**What to Mock:** the Hometown and Olive services and the nono proxy.
**What NOT to Mock:** Liam's own client (use real `build_proxy_client`), the binary entry point, the receipt/agent-record parsing (use `StubRecordPair`).

## Fixtures and Factories

```rust
let hometown = HometownStub::answer_without_proposal();
let proxy = CaptureProxy::forward_once_to(hometown.base_url());
let fixture = StubRecordPair::for_proxy(proxy.port());
```
`StubRecordPair` writes read-only receipt/agent records (14 enforcement layers) into a temp dir. Fixtures use opaque canaries (e.g. `CANARY-7f3a`) to prove content-freeness. Never put credential-shaped values in fixtures. Contract fixtures: `contracts/`, `stubs/records/README.md`.

## Coverage

**Requirements:** none enforced; no coverage tooling. Contract behavior is covered by spec-driven tests.

## Test Types

**Unit Tests:** pure logic (proxy env validation, sha256 vector, evidence content-freeness, receipt validation in `src/lib.rs`, catalog gating in `src/task.rs`, resolver behavior in `src/clients/mod.rs`).
**Integration Tests:** binary-level and client-level through stub proxy (`tests/`); cover retry-once on 5xx, two-request cap, timeouts, Olive refusal, evidence-write failure (exit 30), non-loopback proxy rejection, no Authorization header before proxy injection.
**E2E Tests:** not used. Windows-only `#[ignore]`d test `explicit_proxy_ignores_the_windows_system_proxy_trap` in `tests/proxy_contract.rs` temporarily edits HKCU Internet Settings (restored in `Drop`); run with `-- --ignored`.

## Common Patterns

**Async Testing:** none; blocking `reqwest`. The resolver test polls the future manually with `Waker::noop()` (`src/clients/mod.rs`).

**Error Testing:**
```rust
assert_eq!(validated_proxy(value, value, ""), Err("env:proxy_not_loopback"));
```
Also check content-freeness: `assert!(!record.contains("CANARY-7f3a") && !log.contains("CANARY-7f3a"));` and `log_sha256` equals `Sha256::digest(&log)`.

---

*Testing analysis: 2026-10-21*
