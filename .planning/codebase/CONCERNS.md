# Codebase Concerns

**Analysis Date:** 2026-10-21

Scope: full repo. Source is small (~1.9k lines Rust in `src/`, ~760 in `tests/`). Most risk is in spec/code drift, unverified external contracts, and Windows-specific startup checks, not in code volume. Checked against `AGENTS.md` constraints (section "AGENTS.md compliance" below).

## Spec / Code Contract Mismatches (stop conditions per AGENTS.md)

AGENTS.md says a contract mismatch is a stop condition: update the contract table and README, then stop for review. The v0.4.1 design/test specs (`specs/LIAM_DESIGN_SPEC_v0.4.1.md`, `specs/LIAM_TEST_SPEC_v0.4.1.md`) are uncommitted drafts and the code predates several of their ratified items. None of these gaps are recorded in `README.md` or `RESULTS.md`.

**D16: exit reasons from the service error code (not implemented):**
- Spec: design §5.4 and REQ-LIAM-20 require reason `hometown:<code>` taken from the `{"error": "<code>"}` body, falling back to `hometown:<status>`. Exit 21 for other 4xx.
- Code: `src/clients/hometown.rs` maps by HTTP status only. Only a `key_inactive` substring check on 401 bodies exists. 400/409/422 all become `Status(n)` and `src/main.rs` prints `hometown:{status}`, so the two 422 cases (`agent_not_registered`, `party_not_found`) are indistinguishable.
- Also: 503 `trace_not_saved`/`context_unavailable` and 504 are covered by `is_server_error()` and retried once (matches spec). 408/other timeouts go through `Unavailable` (matches).
- Olive: `src/clients/olive.rs` / `src/main.rs` always print `olive:unavailable` (exit 22) regardless of olive's error code; spec §7.4 requires reasons from olive's error code. Verify against spec §7.4 before changing.
- Fix approach: parse `{"error": string}` in both clients; extend `HometownError`/`OliveError` to carry a code string; add tests for TC-ANS-05/-07 and TC-ACT-08.

**D17: question length check (not implemented):**
- Spec: design §3.3 step 1 and exit-code table require `--question` of 1..4000 characters, else exit 11 `input:question_length`.
- Code: `required_arg("--question")` in `src/main.rs` accepts any length, including an empty string. No test exists.
- Fix approach: add check after argument parsing, before any network use; add process test.

**DSSE record integrity (TC-START-07):**
- Spec test expects a valid DSSE record with `record_public_key` to pass with `receipt_integrity=dsse`.
- Code: any non-empty `nono.record_public_key` in `liam.toml` exits `10 receipt:signature` (`src/main.rs`); `src/evidence.rs` hardcodes `receipt_integrity: "acl-only"`. This is a documented fail-closed deferral in `README.md` but the v0.4.1 test spec still lists it as `ci-elevated`.

**Version labels:**
- `Cargo.toml` and `liam.toml` `[agent] version` are `0.4.0` while specs are v0.4.1 and the amendment is ratified. `agent_version` in every run record reports `0.4.0`. `src/evidence.rs` test also hardcodes `0.4.0`.
- `profiles/liam.nono.toml` carries `profile_version = "0.3.0"` and a header saying it is blocked on the v0.4.3 receipt amendment, but `liam.toml` requires `profile_min_version = 4` (integer) and the agent record's `profile_version` is a `u32`. The profile file's version type/value does not match what Liam validates.

**Spec vs. working copy state:**
- `specs/LIAM_AMENDMENT_v0.4.1_proxy-transport.md` is modified (status changed to "Ratified 2026-10-20") and `specs/LIAM_DESIGN_SPEC_v0.4.1.md`, `specs/LIAM_TEST_SPEC_v0.4.1.md` are untracked. The ratification record exists only in the working tree until committed. Git also warns LF->CRLF for the amendment; `.gitattributes` only exempts `vendor/**`.

**TC-NET-03 static check (not automated):**
- Test spec revised TC-NET-03 requires a grep-style static check for `std::net`, `TcpStream`, `ToSocketAddrs`, second client constructor, gateway references. No such test exists in `tests/`. `src/stubs.rs` uses `tiny_http` and `reqwest` clients (feature-gated); a naive grep must exclude it.

**TC-NET-04 / TC-NET-08 (gateway trap, real nono):** no gateway-trap harness in `tests/`; `RESULTS.md` records these as not measured.

## AGENTS.md compliance

- Secrets: none found in `src/`, `liam.toml`, scripts. Test/fixture key IDs are UUID-shaped random values (`src/stubs.rs`, `src/evidence.rs`), not credentials. `stubs/records/` and `stubs/records/generated/` (gitignored) hold generated fixtures; keep it that way.
- Network: `src/clients/mod.rs` builds a single client with `no_proxy()`, `Proxy::all(..)`, and a `DenyAllResolver`. `validated_proxy` in `src/config.rs` accepts only `http://127.0.0.1:<port>` equal in both vars with empty `NO_PROXY`. No Liam-side allowlist. Compliant.
- No gateway call, no child processes in `src/main.rs`/`src/lib.rs`. `scripts/launch-under-nono.ps1` spawns `nono` but is operator tooling, not Liam. Compliant.
- Evidence content-free: `src/evidence.rs` writes only ids, status, hashes (party ref is SHA-256 hashed). The log writes fixed event names only. Compliant, but see Fragile Areas for the `event()` interpolation.
- Fail-closed one reason line: `fail()` in `src/main.rs` prints `liam: <code> <reason>`. Reasons from `HometownError::Status` interpolate a numeric status only. Compliant. Exception: when the evidence write fails after the task, the answer has already been printed to stdout, then exit 30 (documented in `RESULTS.md`).
- Reads only receipt and agent record from nono: compliant (`NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`).

## Tech Debt

**Hand-rolled vocabulary YAML parsing:**
- Issue: `vocabulary_contains_purpose` in `src/main.rs` line-scans for `purposes:` then `- <purpose>`. It breaks on quoted items, inline lists, comments after the value, tabs/CRLF variants, or reordered keys.
- Files: `src/main.rs`, `contracts/vendored/vocabulary.yaml`
- Impact: false refusals (fail-closed) or, if the format changes, false accepts of a substring-equivalent entry are unlikely but unverified; no unit tests for it.
- Fix approach: move into a testable function in `src/config.rs` and add cases; consider a YAML parser only if the vendored file format requires it (adds a vendored dependency).

**Placeholder vocabulary hash:**
- `liam.toml` pins `f3412d68...` (placeholder). The canonical hash `60a44f00...` is the POC exit criterion (`README.md`, `contracts/vendored/VENDORED.md`). Until swapped, TC-LVOC checks pass against the wrong file.

**Large `main.rs` with duplicated error arms:**
- `src/main.rs` (309 lines) repeats `finish_failure(&mut evidence, &session, &config, ...)` six times and calls `run(...)` twice with identical 9 arguments for the retry. `#[allow(clippy::too_many_arguments)]` is used on `run`, `Evidence::finish`, and `finish_failure`.
- Fix approach: a request-context struct and a mapping function from `TaskError` to `(outcome, code, reason)`. Retry should re-call only `hometown.answer`, not the whole `task::run` (the catalog is re-parsed each attempt).

**Config and path assumptions:**
- `liam.toml` and `catalog_path` are read relative to the current working directory (`src/main.rs`), not the executable directory. The launch script uses `.\target\release\liam.exe` from repo root; running elsewhere fails with `config:missing`. The profile comment says the executable directory is the intended read grant.
- `expand_local_app_data` silently leaves `%LOCALAPPDATA%` literal if the env var is missing, which would create a directory literally named `%LOCALAPPDATA%` relative to cwd (evidence written to an unintended location).
- `liam.toml` `base_url` values are `https://hometown.invalid` / `https://olive.invalid` placeholders (no real endpoint configured); service base URLs are not validated.

**Evidence writer:**
- `src/evidence.rs` `event()` builds JSON via `format!("{{\"event\":\"{event}\"}}")` with no escaping. Current callers pass literals; any future caller passing dynamic text could break the log or leak content.
- Writes are `fs::write` (non-atomic, overwrite). `run_id` is UUIDv7 so collisions are unlikely, but a crash between log and record writes leaves a log with no record.
- `receipt_integrity` is a hardcoded `"acl-only"`; the `profile_version` stored in the run record is `config.nono.profile_min_version`, not the agent record's actual `profile_version` (the validated value is not carried in `ValidatedSession`).

**Test-only stub hooks in production binary source:**
- `src/main.rs` honours `LIAM_STUBS=1` / `LIAM_STUB_PID` (overrides expected token arm to `WriteRestricted` and the process id checked against the receipt) under `cfg(feature = "stubs")`. Default builds exclude it, but CI runs `clippy --all-features` and tests with `--features stubs`. A release build with `--features stubs` would weaken the receipt PID/arm checks. Nothing prevents shipping that build; `scripts/launch-under-nono.ps1` expects `target/release/liam.exe` without verifying features.
- `StartupPolicy::for_stub` is the only constructor, also used in production code path (name is misleading).

## Known Bugs / Gaps

**Windows effective-ACL check is an open-for-write probe:**
- `ensure_not_writable` in `src/receipt.rs` treats `PermissionDenied` as "not writable" and any other error as `missing`. A sharing violation (file open elsewhere), a read-only attribute, or an ACL where write is denied for reasons other than the supervisor's intent are conflated. If the open succeeds, the handle is simply dropped (no modification, but it may update nothing). `README.md` states Windows effective-ACL checks remain outstanding; TC-START-06 is not measured. `windows-sys` ACL features are in `Cargo.toml` but unused in `src/` (verify before removal).

**TOCTOU between check and read:**
- `ensure_not_writable` then `fs::read_to_string` on a path from the environment. A path swap between the two calls is possible if the directory is writable by Liam; relies on nono directory ACLs.

**Retry semantics:**
- The retry covers `HometownError::Unavailable` only, which includes connection-level failures and timeouts (`send()` errors). A timed-out first request may already have been processed by Hometown (`409 run_conflict` possible on retry with the same `run_id`; spec table says no retry on 409 and exits 21). Behavior with a retry hitting `run_conflict` is untested.

**Receipt validation redundancy:**
- `validate_startup` in `src/receipt.rs` has a convoluted token-arm condition (matches a list then rejects three of them, then compares to the policy). Equivalent to "arm in {Null, WriteRestricted} and equals expected"; hard to reason about and easy to desync from design §4.3.
- `key_expiry` check uses `now_unix + 2 * max(timeouts)` with unchecked add; `profile_version` checks `<` min only.

## Security Considerations

**Unauthenticated proxy loopback port:**
- Risk: any local process can listen on `127.0.0.1:<port>` before nono-proxy; Liam only checks that the port matches the agent record's `proxy_port`. It sends the party reference and question in the clear (HTTP proxy, then TLS only through CONNECT to the service). With `Proxy::all` on an `http://` proxy and `https://` targets this is CONNECT-tunnelled; if `base_url` were set to `http://`, content would be sent in cleartext to the proxy. `base_url` scheme is not validated.
- Files: `src/clients/mod.rs`, `src/config.rs`, `liam.toml`
- Recommendation: reject non-`https` base URLs at config load (fail-closed, one reason line).

**Record integrity is ACL-only:**
- Receipt/agent record are trusted if parseable and non-writable by Liam. No signature verification (DSSE deferred). Documented as `receipt_integrity: "acl-only"`.

**Stdout contains the answer:**
- `src/main.rs` prints answer text and proposal id to stdout by design; stderr stays content-free. Operators capturing stdout into logs would capture content; documentation does not warn about this.

**Vendored dependency tree:**
- `vendor/` (thousands of files; most of the 8.7k tracked files) is trusted by checksum only. Recent commits (`16c4425`, `1d4aa2f`) fixed Windows checksum drift via `vendor/** -text` and force-added ignored `cc` sources. Risk: any normalization (autocrlf, editors) on `vendor/` breaks `cargo --locked --offline` builds, and a global ignore rule can silently drop vendored files again. `.gitignore` has no explicit vendor handling for patterns like `*.o`/`target`-style globals; verify with `git status --ignored vendor/` after vendor refreshes.
- `.cargo/config.toml` hard-sets `net.offline = true` and replaces crates.io; dependency updates require a manual bundle refresh (`liam-vendor-bundle-2.tar.gz` referenced in `README.md` is not in the repo). No `cargo audit`/`cargo deny` in CI.
- `reqwest 0.12` with `rustls-tls-native-roots` uses the Windows certificate store; the TLS trust for the proxy-tunnelled connection is OS-managed.

## Performance Bottlenecks

Not a concern at this scale (one task per process, at most 2 hometown requests + 1 olive request). Blocking `reqwest` creates an internal runtime per client; acceptable. Timeouts are 90s (hometown, times 2 with retry = up to 180s) and 30s (olive); the key-expiry check budgets `2 * max(timeouts)` seconds.

## Fragile Areas

**Stub harness (`src/stubs.rs`, 552 lines):**
- Files: `src/stubs.rs`
- Why fragile: heavy `expect()`/panics, three near-identical forwarding proxy implementations (lines ~36-170), hand-built receipt/agent JSON inline that must track the schemas in `contracts/agent-record.schema.json` and `specs/_agent_inputs/receipt.rs`. A schema change requires editing several copies. Lives in `src/` (public library module under a feature) rather than `tests/`.
- Safe modification: change all three proxy variants together; run `cargo test --locked --offline --features stubs -- --include-ignored`.

**Windows registry trap test:**
- `tests/proxy_contract.rs` `explicit_proxy_ignores_the_windows_system_proxy_trap` is `#[ignore]`, writes HKCU Internet Settings, and was denied on the dev machine (`RESULTS.md`). It mutates user state; a failed run can leave a modified system proxy. Ensure cleanup path is correct before running it on a real workstation.

**Schemas vs. code:**
- `contracts/agent-record.schema.json`, `contracts/receipt-fields.md` (marked "STOP FOR REVIEW", partly historical) and the serde structs in `src/receipt.rs` are three independent encodings. `receipt-fields.md` still describes the pre-v0.4.3 mismatch and may mislead.

## Test Coverage Gaps

**Process/ACL/real-nono paths:**
- Not measured per `RESULTS.md`: TC-START-06 (writable records), TC-START-07 (DSSE), TC-NET-02/-04/-08, TC-E2E-01/-02/-05/-07, TC-NET-01/-KEY-01 on `ci-elevated`. Priority: High for ACL check, since it is the only guard on the supervisor-owned files.

**Unit-untested logic:**
- Vocabulary parsing (`src/main.rs`), `expand_local_app_data` missing-env behavior, `Evidence::event` escaping, `HometownClient` status mapping edge cases (401 without body, non-JSON 5xx), `OliveClient` error handling (`src/clients/olive.rs`), catalog-version mismatch path in `src/task.rs`. Priority: Medium.
- `src/task.rs` proposal validation: `proposal.party_ref != party_ref` suppression, non-object parameters, extra/missing required parameters have partial coverage only (two tests: valid and not-in-catalog). Priority: Medium.

**CI:**
- `.github/workflows/ci.yml` uses a floating `dtolnay/rust-toolchain@stable` (no `rust-toolchain.toml`, no `rust-version` in `Cargo.toml`), so lints can change under `-D warnings`. Only one job on `windows-latest`; the `dev-standard-user` and `manual-hardware` tiers have no automation. `RESULTS.md` rows record "uncommitted" Liam commit and "not recorded" Windows build, so results are not reproducible against a revision.

## Missing Critical Features

**External contracts unconfirmed:**
- A1/A2: Hometown and Olive wire contracts are owner-authorized stubs (S-HT-1, S-OL-1) in `contracts/hometown-answer.S-HT-1.json` and `contracts/olive-proposal.S-OL-1.json`. A3/A4: Popeye key exchange and nono key injection/detection not confirmed implemented. A5: nono-proxy loopback behavior is an assumption (nono owner). Until confirmed, no live end-to-end claim is valid; any mismatch found later is a stop condition.
- `liam.toml` `expected_token_arm` is empty; outside stub mode every real run exits `10 receipt:token_arm` until it is configured (intentional fail-closed, TC-START-05).
- `scripts/make-stub-records.ps1` and `scripts/launch-under-nono.ps1` pass `--purpose`, `--party`, `--question` on the command line; command-line arguments carrying a party ref and question are visible to other local processes and in process listings (content-free evidence rule applies to logs, not process args).

## Dependencies at Risk

**`tiny_http 0.12` (optional, stubs only):** unmaintained-leaning crate used solely for test fixtures; keep it out of default features (currently correct).
**`windows-sys 0.59`:** declared with ACL/registry features; confirm each feature is used (`tests/proxy_contract.rs` uses Registry) to keep the vendored surface minimal.

---

*Concerns audit: 2026-10-21*
