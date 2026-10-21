# Phase 1: v0.4.1 Conformance - Research

**Researched:** 2026-10-21
**Domain:** Rust CLI (reqwest blocking, serde_json RawValue), Windows, local stub conformance against ratified v0.4.1 design/test specs
**Confidence:** HIGH on code gaps and spec text (read directly from the repo); MEDIUM on RawValue/ryu mechanics (vendored source checked, not yet compiled in this repo)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
Interpretation (locked): the HTTP status alone decides retry and exit code, as in v0.4 ("status handling ... unchanged", §5.4). The body code only supplies the reason. So 403 always exits 20, 401/400/409/422/other 4xx exit 21, and any hometown 5xx/timeout retries once and then gives `hometown:unavailable` whatever its body says. olive 5xx/timeout gives `olive:unavailable` with no retry.

- **D-01:** Carry the proposal as `serde_json::value::RawValue` (feature `raw_value`, already in vendored serde_json, no new crate). Keep each of the four fields as hometown's exact JSON bytes. Validation parses a separate read-only copy. The olive request body embeds the raw bytes, so the forwarded proposal is byte-identical to hometown's, which is stronger than JCS-identical.
- **D-02:** The JCS (RFC 8785) SHA-256 for TC-ACT-07 is computed by a small hand-written canonicaliser in test code only (`tests/` or `#[cfg(test)]`), using the vendored `ryu` for ECMAScript number formatting and checked against RFC 8785 vectors. Do not vendor a JCS crate. Liam does not compute JCS at runtime.
- **D-03:** Extend `contracts/catalog-0.4.json` with a per-action `parameters_schema` in a JSON-Schema subset: `type`, `properties`, `required`, `additionalProperties: false`, optionally `maxLength`. Validate it with a small hand-written validator in Liam. This makes TC-ACT-03's `"5"`-vs-integer case expressible, and the case fails as `schema` without being coerced. Record in README that this is a stub-catalog (A2) shape change; real olive's catalog shape is checked at the Phase 4 contract diff. Keep the `action_id_format` check (`^[a-z0-9_.]+@[0-9]+$` from the vocabulary).
- **D-04:** Keep the existing check that suppresses a proposal whose `party_ref` != `--party` (suppressed as `schema`). Record it in README as a deviation beyond §7.3. It only suppresses and never alters, so D15 holds.
- **D-05:** The four-key rule (§5.3/§7.3) applies to the raw proposal object. A proposal with other than exactly `action_type`, `party_ref`, `parameters`, `justification` is suppressed as `schema`. Extra top-level response keys are ignored (TC-SEC-03). Extra proposals in a non-conforming response are ignored and logged as `proposal.suppressed` reason `limit`.
- **D-06:** Parse `{"error": "<code>"}` from hometown and olive error bodies. Accept the code only if it matches `^[a-z][a-z0-9_]{0,63}$`. Otherwise (missing, unparseable, non-string or off-pattern) fall back to `<service>:<status>`. Unlisted but well-formed codes such as `rate_limited` pass through.
- **D-07:** Add `reason` to the run record's proposal object: `proposal: {proposal_id, status, reason}`. `reason` is null unless olive returned `refused_policy`, and is guarded by the same pattern as D-06. Record in README as an additive §8.1 field implied by TC-ACT-09.
- **D-08:** A `trace_proposal_mismatch` refusal is flagged by `"investigate": true` on the `proposal.refused` log event. Exit stays 0 and stderr stays empty.
- **D-09:** An idempotent replay (200 with an existing `proposal_id`) is recorded in the run record exactly as a fresh submission. The `proposal.submitted` log event carries `http_status` (200 or 202).
- **D-10:** Stub-only gateway attempt for TC-NET-04 is triggered by env var `LIAM_STUB_GATEWAY_ATTEMPT=1`, read only under `cfg(feature = "stubs")`.
- **D-11:** Attempt code and the stand-in gateway host (`gateway.invalid`) live only in `src/stubs.rs`, e.g. `stubs::gateway_attempt(&client)`, sending one request through Liam's real single proxy client. The test proxy refuses the gateway host the way nono's allowlist would, and a `gateway-trap` listener counts hits (must be zero). main logs a content-free `gateway.attempt` event with outcome `refused`. The run still completes through hometown. A normal run (flag unset) also gets zero trap hits.
- **D-12:** TC-NET-03 is a Rust integration test `tests/static_contract.rs`, run by `cargo test`. Reads `src/**/*.rs`, skips `src/stubs.rs`, allows exactly one client constructor in `src/clients/mod.rs`, asserting it calls `no_proxy()`, `Proxy::all(..)` and `dns_resolver(..)`. Asserts zero `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, Winsock calls and second client constructors.
- **D-13:** "Gateway reference" = gateway-shaped host (`gateway.` hostnames or the trap host) or a model API path (`/v1/chat/completions`, `/v1/messages`, `/v1/completions`) in a string literal. The word "gateway" in comments is allowed.
- **D-14:** Both `contracts/vendored/vocabulary.yaml` and `specs/_agent_inputs/vocabulary.yaml` hash to `f3412d68...`, not canonical `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`. **The user will supply the canonical vocabulary file.** CONF-11 is a blocking task waiting on that file; other CONF items proceed. On arrival: vendor unchanged, verify it hashes to `60a44f00...`, pin in `liam.toml`, update `VENDORED.md`/README, re-run TC-LVOC-01/02. If it does not hash to `60a44f00...`, stop for review and do not pin.
- **D-15:** In `profiles/liam.nono.toml` set `profile_version = 4` (integer). Update the header to say `NONO_v0.4.3` is ratified and the file is still a draft pending nono-owner review. Change no other profile content (§4 changes need re-ratification).
- **D-16:** Version labels move to 0.4.1: `Cargo.toml`, `liam.toml` `[agent] version`, run record `agent_version`, and the `src/evidence.rs` test that hardcodes 0.4.0. Add a line-ending policy for `specs/**` in `.gitattributes`.
- **D-17:** RESULTS.md gets one row per TC ID. Every locally runnable v0.4.1 TC gets its own row with n of n, tag, Liam commit, Windows build and limits. Existing grouped rows are split. Real-nono/E2E TCs keep `not measured` rows; TC-START-07 is marked `deferred` (DEF-01).
- **D-18:** Measurements run on a clean tree at a code commit. Each row records that short hash, the Windows build (OS version) and the true tag, `local development` (not `ci-elevated` unless it really ran there). The RESULTS.md update lands in a separate docs-only commit afterwards. A CI run is not required for Phase 1.

### Claude's Discretion
- Refactoring `src/main.rs` as needed for D16 (request-context struct, single `TaskError` -> (outcome, code, reason) mapping, retrying only `hometown.answer` rather than all of `task::run`). Behaviour must stay exactly as the spec says.
- Shape of the extended `HometownError`/`OliveError` enums carrying a code string.
- Where in the flow the gateway attempt fires, provided the run still completes through hometown.
- Updating the stub S-HT-1/S-OL-1 servers, `contracts/*.json` schemas and README for the v0.4.1 shapes: four-key proposal, new error codes, olive v1 202/200 responses, A1/A2/A5 status (CONF-05).
- Test layout and naming, following `<area>_contract.rs`.

### Deferred Ideas (OUT OF SCOPE)
- Refusing non-`https` `base_url` values at config load.
- Replacing the hand-rolled vocabulary YAML line-scanner (becomes in-scope only if the canonical file's format breaks it).
- Executable-directory-relative config paths and the missing `%LOCALAPPDATA%` fallback.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CONF-01 | Unchanged proposal forward, JCS hash equal | RawValue design (Pattern 1), JCS test canonicaliser (Pattern 6), stub catalog actions for nested/fractional params |
| CONF-02 | hometown error codes, retry rules | Error classification (Pattern 2), existing-test fixtures that must change |
| CONF-03 | olive errors, refused_policy, replay | Pattern 3, run record `reason`, answer-must-print-on-olive-failure gap |
| CONF-04 | Question length 1..4000 | Pattern 4 (`chars().count()`, `env::args` panic pitfall) |
| CONF-05 | Stubs, contracts, README match v0.4.1 | Contract file edits list, scripted stub design (Pattern 7) |
| CONF-06 | Version labels, profile, `.gitattributes` | Exact file/line inventory in "Version/label inventory" |
| CONF-07 | Static TC-NET-03 check | Pattern 5 (comment/string-aware scanner) |
| CONF-08 | Gateway-trap harness | Pattern 8 |
| CONF-09 | Command text / extra keys as data | RawValue + strict struct parse, TC-SEC-03 fixture |
| CONF-10 | RESULTS.md per-TC rows | TC runnable-locally classification table |
| CONF-11 | Canonical vocabulary pin | Blocked on user file; procedure and checks |
</phase_requirements>

## Summary

The repo is a small, healthy Rust crate (13 unit tests, 8 process tests, 5+1 proxy tests pass on `cargo test --locked --offline --features stubs` at HEAD `f0ea8d7`). All v0.4.1 work is local and offline: no new crate is needed. `serde_json` is already vendored with the `raw_value` feature available (only a `Cargo.toml` feature flag is needed; `Cargo.lock` should not change), and `ryu` 1.0.23 is vendored (as a transitive dep of serde_json) if a dev-dependency is wanted for the JCS test helper.

Reading the code against the specs shows these concrete gaps beyond what CONTEXT.md lists: (1) on any olive failure `task::run` returns `Err` and `main.rs` never prints the answer, but TC-ACT-08 and TC-FAIL-06 require "answer printed; exit 22"; (2) the run log only ever contains `run.started`, a bare `proposal.suppressed` (no reason), and `run.completed`, while TC-ACT-02/03/06/09 need reason-bearing events (`proposal.suppressed` with `not_in_catalog`/`schema`/`limit`, `proposal.refused`, `proposal.submitted`); (3) the existing stubs are one-shot, single-purpose servers with three near-identical proxy variants, so the ~25 new behaviours (error codes, sequences, 429, 503-then-200, replay, five-key proposal, command text, two proposals) need a scripted-response stub and a routing proxy rather than more copy-pasted constructors; (4) the existing proxy test sends `{"code":"key_inactive"}` where the spec body key is `error`; (5) `TC-EVD-01` references a `run-record.schema.json` that does not exist in `contracts/`; (6) the shape of the "two proposals" non-conforming response is not defined anywhere in the specs.

**Primary recommendation:** Do the work in this order: (a) test infrastructure (scripted stubs plus routing proxy plus trap) with existing tests migrated, (b) client/task/evidence changes (RawValue, error codes, events), (c) input check, version/profile/contracts/README, (d) static check and gateway trap, (e) measure and write RESULTS.md in a docs-only commit, with CONF-11 as an independent blocked task.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Question length / arg validation | Liam CLI (`main.rs`) | — | Must fail before any network or record |
| Proposal pass-through (unchanged bytes) | Liam clients (`hometown.rs` -> `olive.rs`) | `task.rs` (validation) | D15: validation read-only; forward raw bytes |
| Error-code to exit/reason mapping | Liam clients (classify) | `main.rs` (exit map) | Status decides exit; body only supplies guarded reason |
| Catalog / parameter validation | `task.rs` + small validator | `contracts/catalog-0.4.json` | Stub catalog gains `parameters_schema` |
| Run record / log content-freeness | `evidence.rs` | `main.rs` | Serde-built events; no string interpolation |
| Gateway-trap proof | `src/stubs.rs` + test proxy | `tests/` | Attempt code stub-only; release unchanged |
| Static egress conformance | `tests/static_contract.rs` | — | Source scan, no runtime |
| Canonical JCS hash | test code only | — | Liam never computes JCS at runtime |

## Standard Stack

No new runtime crates. Everything below is already in `vendor/` and `Cargo.lock`.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde_json | 1.0.154 vendored, feature `raw_value` | Byte-preserving proposal fields | `RawValue` implemented in vendored `src/raw.rs`, gated by `#[cfg(feature = "raw_value")]` [VERIFIED: vendor/serde_json/Cargo.toml line 71, src/lib.rs:440] |
| reqwest (blocking) | 0.12 (vendored 0.12.28) | Proxy-forced client (unchanged) | Existing; `.json()` serialises via serde_json so RawValue fields are emitted verbatim [ASSUMED: standard serde behaviour, confirm with the TC-ACT-07 test] |
| sha2 | 0.10 | SHA-256 for JCS hash in tests | Already a dependency |
| tiny_http | 0.12 (stubs feature) | Stub servers, routing proxy, trap listener | Already used |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| ryu | 1.0.23 vendored | Shortest-roundtrip digits for the test-only JCS number formatter (D-02) | Needs `[dev-dependencies] ryu = "1"`; integration tests cannot import transitive deps. After editing `Cargo.toml` run one `cargo build --offline --features stubs` (without `--locked`) so `Cargo.lock` records the new edge, then commit the lock; `ryu` is already in the lock so no vendor change [ASSUMED: lock-edge behaviour, verify no `vendor/` diff] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ryu dev-dep | std `format!("{:e}", f64)` | Rust's `{:e}` also yields shortest round-trip digits plus exponent, avoiding any manifest change. D-02 names ryu, so use ryu unless the lock edit proves awkward; either way the ES layout step below is required |
| hand-written catalog validator | `jsonschema` crate | Not vendored; D-03 locks hand-written |

**Installation:** none. Cargo.toml edit only: `serde_json = { version = "1", features = ["raw_value"] }` (and optionally the ryu dev-dependency).

## Package Legitimacy Audit

No new external packages are introduced. `serde_json`, `ryu`, `sha2` are already vendored and locked. slopcheck was not run because nothing is being installed from a registry; `ryu` is a long-established transitive dependency already present in `Cargo.lock` [VERIFIED: Cargo.lock line 822].

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
argv ──> parse args ──> question 1..4000 scalars? ──no──> exit 11 input:question_length (no net, no record)
              │yes
              v
 liam.toml, vocab hash+purpose, proxy env, receipt+agent record  ──fail──> exit 10/11 (no net)
              │ok
              v
 build_proxy_client (single, no_proxy, Proxy::all, DenyAllResolver)
              │
              ├─(stubs + LIAM_STUB_GATEWAY_ATTEMPT=1)─> stubs::gateway_attempt ──> test proxy ──refuse──> log gateway.attempt refused
              v
 hometown.answer  ──5xx/timeout──> retry once ──> still fail ──> exit 21 hometown:unavailable
   │ 403 ─> exit 20 hometown:<code|403>      │ other 4xx ─> exit 21 hometown:<code|status> (no retry)
   │ 200 -> parse: answer, trace_id, context_complete, proposal (raw)
   v
 proposal? ──none──> print answer ──> evidence ──> exit 0
   │yes
   v
 validate (read-only copy): exactly 4 keys; party_ref == --party; action in catalog + id format; parameters_schema
   │fail -> log proposal.suppressed{reason}; print answer; exit 0
   v
 olive.submit(raw 4 fields + run_id + trace_ref) ──> 202/200 ──> record {proposal_id,status,reason}; 200 refused_policy / replay exit 0
   │ 4xx ─> exit 22 olive:<code|status>, answer printed     │ 5xx/timeout ─> exit 22 olive:unavailable (no retry), answer printed
   v
 evidence (record + jsonl, serde-built events) ──fail──> exit 30
```

### Recommended Project Structure
```
src/
├── main.rs            # request-context struct; TaskError -> (outcome, code, reason); question check
├── task.rs            # validate raw proposal; return TaskResult or TaskError that CARRIES answer/trace for olive failure
├── catalog.rs         # (new, optional) Catalog + parameters_schema validator, unit-tested
├── clients/hometown.rs, olive.rs   # RawValue proposal; error_code() guard shared helper in clients/mod.rs
├── evidence.rs        # event(name, fields: serde_json::Value-ish); ProposalRecord.reason
└── stubs.rs           # ScriptedStub + routing TestProxy + GatewayTrap + gateway_attempt (stubs feature only)
tests/
├── process_contract.rs, proxy_contract.rs   # extend
├── static_contract.rs                       # new, TC-NET-03 (no stubs feature needed)
├── conformance_contract.rs                  # new: TC-ANS/ACT/FAIL/SEC matrix via process
└── common/jcs.rs                            # test-only RFC 8785 canonicaliser + vectors
```
A new `catalog.rs` module must not add anything the TC-NET-03 scan flags; keep it pure.

### Pattern 1: Byte-preserving proposal (CONF-01, CONF-09)
**What:** Deserialise the hometown 200 body as
`struct AnswerResponse { trace_id: String, answer: String, context_complete: bool, proposal: Option<Box<RawValue>> }` (extra top-level keys ignored, no `deny_unknown_fields` here per TC-SEC-03). Then parse the raw bytes a second time into a strict struct for validation:
`#[derive(Deserialize)] #[serde(deny_unknown_fields)] struct FourKeys { action_type: String, party_ref: String, parameters: Value, justification: String }`.
Missing, extra and *duplicate* keys all error, so "exactly four keys" holds even for `{"action_type":"a","action_type":"b",...}`, which a map-based `Value` check would silently collapse (hazard: olive could see a different object than Liam validated). To embed in the olive request, define a non-flatten struct with four `Box<RawValue>` fields extracted from a second parse of the same bytes, or parse the proposal once into `struct RawFour { action_type: Box<RawValue>, party_ref: Box<RawValue>, parameters: Box<RawValue>, justification: Box<RawValue> }` plus validation from `serde_json::from_str::<Value>` of each field. Serialise with a plain `#[derive(Serialize)]` struct: `{action_type, party_ref, parameters, justification, run_id, trace_ref}`.
**Do NOT use `#[serde(flatten)]` with RawValue.** The current `ProposalRequest` uses `#[serde(flatten)]`; flatten buffers through serde's private `Content` type, which RawValue cannot round-trip (known serde_json limitation, issue #599) [ASSUMED from training knowledge; the vendored docs mention RawValue limits but I did not find an explicit flatten note, so prove with a unit test first]. Replace flatten with explicit fields.
**Subtlety:** `RawValue` retains the exact input slice, including interior whitespace and key order; compact output does not strip it. That is intended (byte-identical).
**Also:** reqwest `response.json()` is fine for `Box<RawValue>`, but prefer `response.bytes()` then `serde_json::from_slice` so error classification is explicit and no body text can reach an error string.
**Test:** unit-test that `serde_json::to_string(&request)` contains the original field bytes verbatim, including `1.50`, `é` vs `é`, and reordered keys.

### Pattern 2: hometown error classification (CONF-02)
- `fn error_code(body: &[u8]) -> Option<String>`: parse `{"error": <string>}` only; accept if it matches `^[a-z][a-z0-9_]{0,63}$` (hand-written byte loop; no regex crate). Put this in `clients/mod.rs` and share with olive.
- `HometownError` becomes e.g. `PurposeRefused(String reason)`, `Rejected { status: u16, reason: String }` (4xx other), `Unavailable`, `InvalidResponse`. Outcome strings already used in evidence: `purpose_refused`, `hometown_no_injected_key`, `hometown_key_inactive`, `hometown_failed`, `hometown_unavailable`; TC-ANS-03 and TC-EVD-04 pin `purpose_refused` and `hometown_unavailable`. Keep those names.
- Decision flow is by status first: 403 -> exit 20; other 4xx (including 401, 429) -> exit 21, no retry, reason `hometown:<code|status>`; `is_server_error()` or transport/timeout -> `Unavailable` -> retry once.
- 401: drop the `body.contains("key_inactive")` substring check; the reason is simply the guarded `error` code (`no_injected_key` / `key_inactive`); outcome label can still be derived from the code for those two.
- 403 reason: the spec row is `hometown:purpose_refused` for body `purpose_refused`; under the generic rule a 403 with no usable body gives `hometown:403`. Existing unit test `(403, "{}", PurposeRefused)` still classifies to exit 20 but its expected reason changes. Flag for planner: confirm generic rule (recommended, consistent with D-06) rather than a hard-coded `purpose_refused`.
- Retry only the client call. Replace the double `run(...)` in `main.rs` with a retry wrapper around `hometown.answer(...)` so the catalog is parsed once and the same request is re-sent (spec: "a retry is a new step in the same run").
- Unparseable 200 body today maps to `InvalidResponse`, which falls into the `Err(Hometown(_))` arm and prints `hometown:unavailable` without retry. Keep that behaviour (spec silent) and note it in README.

### Pattern 3: olive responses (CONF-03)
- Success = 200 or 202 with `{proposal_id, status, reason?}`. Capture the HTTP status in `ProposalResponse` (needed for D-09 `http_status`). Guard `reason` with the D-06 pattern; off-pattern -> null (still record `status`).
- 4xx -> `OliveError::Rejected{status, reason}` -> exit 22 `olive:<code|status>`. 5xx/timeout/transport -> `olive:unavailable`. No retry anywhere in olive.
- **Gap:** `task::run` returns `Err(TaskError::Olive(..))` and drops the answer, so `main.rs` prints nothing. TC-ACT-08 and TC-FAIL-06 require the answer printed. Make the error carry `{answer, context_complete, trace_id, error}` or have `task::run` return a `TaskResult` with an `olive_failure: Option<..>` field. Recommended also to record `answer{trace_id, context_complete}` in the failed run record (join key); the spec is silent, so treat as discretion and document.
- Log events (serde-built, content-free): `answer.received{trace_id? no -- identifiers allowed, has_proposal, context_complete}`, `proposal.validated`, `proposal.suppressed{reason}`, `proposal.submitted{proposal_id? status, http_status}`, `proposal.refused{reason, investigate?}`. §8.2 allows identifiers, counts, status and reason codes. Never log a code that failed the guard.
- `trace_proposal_mismatch`: exit 0, run record `proposal.status=refused_policy`, `reason=trace_proposal_mismatch`, log `proposal.refused` with `"investigate":true`, stderr empty, one olive request.

### Pattern 4: question length (CONF-04)
`question.chars().count()` counts Unicode scalar values, which is what the spec asks (Rust `char` is a scalar value) [VERIFIED: Rust std semantics]. Check `1..=4000` immediately after args are parsed, before any file read that could write, with `fail(11, "input:question_length")`. Ordering between this and `vocab:*` is unconstrained as long as both are exit 11 and nothing is sent or written.
**Pitfalls:** `env::args()` panics on non-Unicode arguments (exit 101, not the one-line contract). Use `args_os()` and map non-UTF-8 to `fail(11, "args:required")`. `required_arg` uses `args.windows(2)` so a value equal to a flag name (e.g. `--question --party`) is misparsed; low risk, but test an empty `--question ""` explicitly (Windows `Command::args` quotes empty strings correctly). Test with 4000 and 4001 scalars including a multi-byte, non-BMP character (a 4-byte UTF-8 char counts 1; its UTF-16 length is 2, so do not use UTF-16 length).

### Pattern 5: TC-NET-03 static scan (CONF-07)
`tests/static_contract.rs`, no `stubs` feature gating. Walk `src/**/*.rs` (std `fs::read_dir` recursion), skip `src/stubs.rs`. **Do not strip comments with a naive `//` split**: string literals such as `"https://..."` contain `//`. Write a ~40 line scanner state machine handling line comments, nested block comments, normal strings, raw strings `r#"..."#`, and char literals; collect (a) code tokens text and (b) string-literal text separately. Then:
- forbidden in code text: `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, `UdpSocket`, `TcpListener`, `getaddrinfo`, `WSAStartup`, `windows_sys::Win32::Networking`, `Client::new`, `Client::builder`, `ClientBuilder`, `reqwest::Client`/`blocking::Client` constructors outside `src/clients/mod.rs`.
- `src/clients/mod.rs`: exactly one `Client::builder()`; same file contains `no_proxy()`, `Proxy::all(` and `dns_resolver(`.
- forbidden in string literals (D-13): regex-equivalent of `gateway\.`, `gateway.invalid`, `/v1/chat/completions`, `/v1/messages`, `/v1/completions`. Note existing literals `"direct_gateway"` in `src/lib.rs` and `src/receipt.rs` contain no `gateway.` and must pass; add a unit assertion for that. Also include a self-test (negative control): feed the scanner synthetic source containing each forbidden token and assert it is flagged, so the check cannot silently pass because the scanner is broken.
- Currently verified clean: grep of `src/` outside `stubs.rs` finds only one `Client::builder` (`src/clients/mod.rs:49`) and no `std::net`/`TcpStream`/`ToSocketAddrs`/`lookup_host` [VERIFIED: grep in this session].
- Also cheap to add to the same file (REQ-LIAM-03/ANS-04/ACT-05): scan for `Command::new`/`std::process::Command` (no child processes) and `Authorization`.

### Pattern 6: JCS test canonicaliser (CONF-01, D-02)
Test-only module (e.g. `tests/common/jcs.rs`). RFC 8785 rules:
- Object members sorted by **UTF-16 code-unit** order of the key, not UTF-8 byte or code point order. A key with U+10000 (surrogates D800 DC00) sorts before U+FFFD; byte order sorts the other way. Include this in the vectors.
- Strings: escape only `"` `\` and control chars: `\b \t \n \f \r` short forms, others `\u00xx` lowercase hex; no `\/`, no escaping of non-ASCII.
- Numbers: ECMAScript `Number::toString`. ryu gives shortest digits but its own layout (it switches to exponent form at different thresholds than ES: ES uses plain decimal for 1e-6 <= |x| < 1e21). Take digits + exponent from `ryu::Buffer::format_finite` (or std `{:e}`) and re-lay out per ES: -0 -> `0`, integers up to 1e21 without exponent, `1e+21`, `1e-7` form with explicit `+`.
- Validate against RFC 8785 sample numbers (Appendix B: e.g. `333333333.33333329`, `1E30` -> `1e+30`, `4.5`, `2e-3` -> `0.002`, `0.000001`, `1e-7` -> `1e-7`) and the Section 3.2.3 sorted-keys example.
- serde_json parses numbers to f64 without `float_roundtrip`; both sides of the hash comparison use the same parser, and the forwarded bytes are identical anyway, so the test is sound. Parse with `serde_json::Value` (enable nothing extra), keep key order irrelevant since canonicaliser sorts.
- TC-ACT-07 assertion: SHA-256(JCS(four fields received by olive stub capture)) == SHA-256(JCS(four fields of hometown fixture)); also assert the raw field bytes are byte-equal (the stronger D-01 property). The hometown fixture must contain nested objects, non-ASCII (e.g. `é`, a non-BMP emoji), a fractional number (`1.5`, `0.1`), and keys deliberately out of alphabetical order, in `parameters`.

### Pattern 7: Scripted stubs and routing proxy (CONF-05, all TC-ANS/ACT/FAIL)
Current state: `HometownStub` has six one-shot constructors, `OliveStub` two, `CaptureProxy` three variants with `forward_once`, `forward_two`, `forward_two_to` (each `expect`s/panics and spawns a fixed number of `recv`s). Adding ~25 behaviours this way will be unmaintainable. Recommended (still `stubs.rs`, still feature-gated):
- `ScriptedStub::new(Vec<Reply>)` where `Reply { status, body, delay: Option<Duration> }`; serves replies in order, records every request (method, path, header names with Authorization presence flag, body) in a `Vec`, exposes `requests()`/`count()`, and keeps serving until dropped (use `server.recv_timeout` loop with a stop flag so the thread ends and tests do not hang). The test spec says every stub writes each request to a capture file; the in-memory `Vec` plus optional file dump meets "retain capture files" for RESULTS.md.
- Provide named constructors for the spec's behaviours (`error(status, code)`, `answer_with(proposal_json)`, `five_key_proposal()`, `two_proposals()`, etc.) as thin wrappers over `ScriptedStub`, so process tests read like the TC table. Keep the existing constructors as wrappers to avoid churning working tests.
- One routing `TestProxy`: reads each request's absolute-form URL / `Host`, routes `hometown` vs `olive` base to the matching stub, injects the sentinel credential header on those two only, refuses any host in a refuse-list (e.g. `gateway.invalid`) with 403, counts per-route hits and `Authorization`-presence on inbound requests (TC-KEY-01 asserts at the proxy inbound side). It must not panic on failure paths; convert `expect` to logged errors, because the proxy thread panicking would turn failures into hangs.
- Forwarding from the proxy uses `reqwest::blocking::Client::new()`: this honours ambient `HTTP_PROXY` in the test process; set `.no_proxy()` there to avoid looping into itself when tests export proxy vars.
- Timeouts: process tests use `timeout_seconds = 2`. TC-FAIL-04 literally requires a 100 s delay with the 90 s timeout and completion within 190 s. Implement it as an `#[ignore = "long: ~190 s"]` test (CI already runs `-- --ignored`) with the real `liam.toml` timeout, and keep the fast 2 s variant as the default regression. Record n of n for the long run separately in RESULTS.md.
- Hometown 2xx bodies for the two-proposals case: see Open Question 1.

### Pattern 8: Gateway trap (CONF-08)
- `GatewayTrap`: a `tiny_http` listener on `127.0.0.1:0`; counts any request and any raw connection (use a `std::net::TcpListener` in `stubs.rs`, which is excluded from the static scan, so connection-only hits without an HTTP request are also counted).
- Routing proxy gets a refuse-list containing `gateway.invalid` and a **negative-control mode** that maps `gateway.invalid` to the trap address and forwards. A meta-test with the control enabled must show trap hits = 1, proving the trap can fire; the real test (refusal on) must show trap hits = 0 and proxy-recorded gateway attempts = 1 with a 403 reply.
- `stubs::gateway_attempt(client: &reqwest::blocking::Client) -> &'static str` sends `GET http://gateway.invalid/` (use plain `http://` so reqwest sends absolute-form to the proxy; `https://` would use `CONNECT`, which tiny_http does not serve) and returns `"refused"` on a non-2xx or transport error. main calls it only under `cfg(feature = "stubs")` when `LIAM_STUB_GATEWAY_ATTEMPT=1` and logs `{"event":"gateway.attempt","outcome":"refused"}`. If it unexpectedly succeeds, the event outcome should be `allowed` and the test fails; do not change the exit code (the spec's pass condition is run completes via hometown).
- Assertions for TC-NET-04: normal run: trap 0, no `gateway.attempt` event. Flagged run: exit 0, stderr empty, proxy saw the gateway attempt and refused it, trap 0, the hometown request still happened, `resolver_calls()` stays 0 (the DenyAll resolver is never invoked because the proxy URL is an IPv4 literal) [CITED: existing TC-NET-06 passing test], log contains the content-free event and no host name beyond what D-13 permits. The log must not record the gateway host string; the event is a fixed outcome only.
- Release build: `LIAM_STUB_GATEWAY_ATTEMPT` and `gateway.invalid` must not appear in the release binary (TC-BUILD-01). The static scan skips `stubs.rs`, so also keep the env-var name literal inside `stubs.rs` or a `cfg(feature="stubs")` block. Consider extending the release check with `strings`/`findstr` on `target/release/liam.exe`.

### Anti-Patterns to Avoid
- `serde(flatten)` with `RawValue` (see Pattern 1).
- Building events with `format!("{{\"event\":\"{event}\"}}")`; use `serde_json::json!`/a `Serialize` struct for every field (D-06/D-07 guard strings could otherwise break JSON).
- Echoing any response body, header or URL into stderr, the log or the record; only guarded codes and numeric statuses.
- Re-serialising the proposal through `Value` before forwarding (would reorder keys/reformat numbers, violating D15).
- A fourth CaptureProxy variant.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Raw JSON preservation | A custom byte slicer over the response | `serde_json::value::RawValue` | Handles escapes, nesting and validity |
| SHA-256 | Own hash | `sha2` (present) | — |
| Number shortest-digit | Own Grisu/Ryu | `ryu` (vendored) for digits only | The ES re-layout is the only hand-written part |
| Error-code regex | `regex` crate | 10-line byte loop | No crate; pattern is trivial |
| Windows version for RESULTS | Guessing | `cmd /c ver` or `[System.Environment]::OSVersion` plus UBR from `HKLM\...\CurrentVersion` | Test spec §4 wants "ver + UBR" |

**Key insight:** The hard parts here are not crates but invariants: bytes forwarded must equal bytes received, and nothing from a response body may reach stderr, the log or the record unguarded.

## Runtime State Inventory

Not a rename phase, but version-label changes touch persisted artefacts. Nothing found in these categories: Stored data: `runs/` and `%LOCALAPPDATA%\Fiskroad\liam\runs` hold prior run records with `agent_version` 0.4.0, which are historical and are not rewritten. Live service config: none. OS-registered state: none. Secrets/env vars: none (names `LIAM_STUBS`, `LIAM_STUB_PID` plus new `LIAM_STUB_GATEWAY_ATTEMPT`, stubs-only). Build artifacts: `target/` is gitignored; rebuild after the version bump.

## Common Pitfalls

### Pitfall 1: Answer lost on olive failure
**What goes wrong:** exit 22 prints no answer; TC-ACT-08/TC-FAIL-06 fail.
**Why:** `task::run` returns `Err` for olive errors, discarding the already-obtained answer.
**Avoid:** carry answer and trace in the error or result. **Warning sign:** process test stdout empty on 22.

### Pitfall 2: Existing tests encode old behaviour
`proxy_contract.rs` sends `{"code":"key_inactive"}` and expects `Status(429)`; `process_contract.rs` writes `version = "0.4.0"` and `sha256 = f3412d68...` into per-test `liam.toml`. These must be updated with the version bump (D-16) and again when the canonical hash arrives (D-14); factor a shared `write_test_config()` helper so the hash changes in one place. The same literal appears in `liam.toml`, `VENDORED.md`, `README.md` (and `RESULTS.md` text).

### Pitfall 3: Silent hang in test harness
Stub/proxy threads that `expect()` on failure leave Liam waiting until the 2 s timeout, which then looks like `hometown:unavailable`. Make harness failures loud (record and assert) and make threads exit when dropped.

### Pitfall 4: HTTPS via HTTP proxy
Real base URLs are `https://`; stubs are `http://`. `CONNECT` tunnelling is not exercised locally. That is a Phase 2/3 matter; note in README limits for TC-NET rows.

### Pitfall 5: Stub catalog needs more actions
`catalog-0.4.json` has only `crm.case_note@1` with a single `note` string. TC-ACT-03 (`"5"` where integer expected) and TC-ACT-07 (nested objects, fractional numbers, out-of-order keys) cannot be expressed. Add stub-only actions, e.g. `crm.follow_up@1` (`days: integer`) and a free-form `crm.case_update@1` (`details: {type: object}` with no inner restrictions; the validator must accept arbitrary nested content for `type: object` without `properties`). Define integer strictly: `Number::is_i64()||is_u64()` (so `5.0` fails; JSON Schema 2020-12 would accept `5.0`; document this stricter-than-schema choice as a stub-validator rule). `maxLength` counts scalar values, not bytes. Keep `required_parameters` readable for compatibility or remove it and update the two in-file unit tests in `task.rs` (which embed an inline catalog).

### Pitfall 6: Time and tag labelling
Spec tags: `ci-elevated`, `dev-standard-user`, `manual-hardware`; "a test is never marked verified from an environment with a different tag". D-18 chooses the honest tag `local development`, and existing rows already use it. Keep that; state in Limits which spec tag each TC is nominally and that it has not been run there.

### Pitfall 7: Profile type
`profile_version = 4` must be an integer in the profile; `liam.toml` `profile_min_version = 4` and the agent record's `profile_version` is a `u32`. No code reads the profile file, so this is a doc/consistency fix; edit only the header comment and that one line.

## Code Examples

### Guarded error code (shared helper)
```rust
// clients/mod.rs (no regex crate)
pub fn error_code(body: &[u8]) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Body { error: String }
    let code = serde_json::from_slice::<Body>(body).ok()?.error;
    let b = code.as_bytes();
    let ok = (1..=64).contains(&b.len())
        && b[0].is_ascii_lowercase()
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_');
    ok.then_some(code)
}
// reason: format!("hometown:{}", error_code(&body).unwrap_or_else(|| status.to_string()))
```
[ASSUMED shape; derived from D-06, not from an external source. A non-string `error` makes `from_slice` fail, which yields None as required.]

### Strict four-key parse
```rust
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FourKeys { action_type: String, party_ref: String, parameters: serde_json::Value, justification: String }
// from_str::<FourKeys>(raw.get()).is_err() => suppress as "schema" (missing, extra or duplicate key)
```
[ASSUMED; serde derive behaviour: duplicate field is an error for structs.]

### Verbatim forward
```rust
#[derive(serde::Serialize)]
struct OliveBody<'a> {
    action_type: &'a serde_json::value::RawValue,
    party_ref: &'a serde_json::value::RawValue,
    parameters: &'a serde_json::value::RawValue,
    justification: &'a serde_json::value::RawValue,
    run_id: &'a str,
    trace_ref: &'a str,
}
// client.post(url).header("Content-Type","application/json").body(serde_json::to_vec(&body)?)
```
[ASSUMED; prefer an explicit `.body(to_vec)` over `.json()` so the serialisation point is visible and testable.]

## State of the Art

| Old (v0.4 code) | v0.4.1 | Impact |
|-----------------|--------|--------|
| Reason `hometown:<status>`; olive always `olive:unavailable` | Reason from `{"error": code}`, fallback status | Pattern 2/3 |
| Typed `Proposal` re-serialised via flatten | Raw bytes forwarded (D15) | Pattern 1 |
| No input length check | 1..4000 scalars (D17) | Pattern 4 |
| TC-NET-03 manual grep | Automated scan test | Pattern 5 |

**Deprecated:** `{"code": ...}` error bodies in existing tests; the 0.4.0 labels.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `RawValue` cannot be used under `#[serde(flatten)]` | Pattern 1 | Low: explicit fields are correct regardless |
| A2 | Enabling `raw_value` and adding a `ryu` dev-dep need no `vendor/` change and only a `Cargo.lock` edge | Standard Stack | Medium: if cargo wants a refresh, the bundle must be re-supplied; check by building offline first in task 1 |
| A3 | A non-conforming "two proposals" response is represented by `proposal` as an array (first used, rest logged `limit`) | Open Question 1 | Medium: stub and parser shape only; no real service emits it |
| A4 | 403 reason follows the generic rule (`hometown:<code|403>`) rather than a fixed `purpose_refused` | Pattern 2 | Low: stubs send the code body |
| A5 | Unparseable 200 bodies keep mapping to `hometown:unavailable` (no retry); olive invalid 2xx bodies map to `olive:unavailable` | Patterns 2/3 | Low: spec silent; document in README |
| A6 | `http://` gateway URL via the HTTP proxy is sent in absolute-form and tiny_http exposes the host for routing | Pattern 8 | Medium: confirm in the first harness commit; fall back to the `Host` header |
| A7 | Recording `answer{trace_id, context_complete}` in failed olive runs is acceptable | Pattern 3 | Low: additive, content-free |

## Open Questions

1. **Shape of the "two proposals" non-conforming hometown response (TC-ACT-06, D-05)**
   - Known: spec says "hometown sends at most one"; "additional proposals in a non-conforming response are ignored and logged as `proposal.suppressed` reason `limit`"; test spec stub list says "200 with two proposals (non-conforming)". RESULTS already lists TC-ACT-06 as partial.
   - Unclear: JSON representation. Candidates: `proposal` as a two-element array, or an extra top-level `proposals` array.
   - Recommendation: treat `proposal` as array in the stub (A3); Liam handles `Option<Box<RawValue>>` whose first byte is `[` by taking element 0 and logging `limit` for the rest; also treat an extra top-level key `proposals` as ignored data. Record the choice in the README; one olive request must result either way.
2. **`run-record.schema.json` (TC-EVD-01)** is referenced but absent from `contracts/` (only `agent-record.schema.json`). Recommend adding `contracts/run-record.schema.json` for §8.1 plus the additive `proposal.reason`, and a test that validates a produced record structurally (no JSON-schema crate: assert keys/types by hand). If skipped, TC-EVD-01 must be recorded `partial`.
3. **Canonical vocabulary file** is not in the repo. The `vocab:purpose` scanner and `poc_tenants` placeholder in the current file suggest the canonical file differs in content; the line-scanner (`purposes:` then `- <purpose>`) may or may not handle it. Run TC-LVOC and a `servicing` purpose lookup immediately on arrival; a parser break promotes the deferred scanner replacement into scope.
4. **Which TCs count as "can run locally"** for CONF-10; proposed classification below.

## TC Coverage for RESULTS.md (CONF-10)

Runnable locally against stubs (own row each; spec tag shown, measured tag `local development`): TC-START-01..05, 08..16 (receipt fixtures; verify `make-stub-records`/`StubRecordPair` coverage, several mutation fixtures such as `census-13`, `keyid-upper`, `inject-gateway`, `stale`, `future` currently have only a few unit tests in `src/lib.rs`, so extra fixtures/tests may be needed for rows to be honest), TC-NET-01, -03, -04, -05, -06, -07 (the HKCU test is `#[ignore]` and was denied on this machine: row stays partial/not measured unless permission is available), TC-KEY-01..03 (TC-KEY-03 needs a key-shape scan of binary, config and logs), TC-META-01/02, TC-LVOC-01/02, TC-ANS-01..07 (ANS-04 static scan), TC-ACT-01..09 (ACT-05 static scan), TC-EVD-01..05, TC-SEC-03, TC-FAIL-01..04, -06, -07, -08, TC-BUILD-01/02 (dumpbin / strings scan on the release build; a prior row used `dumpbin /imports`).
Not locally measurable: TC-START-06 (dev-standard-user ACL harness), TC-FAIL-05 (`runs/` not writable, standard user; the existing evidence-path test covers it only partially), TC-NET-02, TC-NET-08, TC-SEC-01, TC-SEC-02, TC-E2E-01..07: keep as `not measured`. TC-START-07: `deferred` (DEF-01).
Existing grouped rows must be split and their `uncommitted`/`not recorded` fields replaced (D-17, D-18). Capture Windows build with `cmd /c ver` plus UBR from `HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion` (`UBR`); record Liam short hash from the code commit, not the docs commit.

## Version / Label Inventory (CONF-06)

| File | Change |
|------|--------|
| `Cargo.toml` | `version = "0.4.1"`; `serde_json` features; optional dev-dep ryu (`Cargo.lock` liam entry follows) |
| `liam.toml` | `[agent] version = "0.4.1"`; (later) vocabulary sha256 |
| `src/evidence.rs` test | `"0.4.0"` -> `"0.4.1"` |
| `tests/process_contract.rs` | per-test `liam.toml` strings `version = "0.4.0"` (two places seen, check all) |
| `profiles/liam.nono.toml` | `profile_version = 4`; header comment per D-15 |
| `.gitattributes` | add `specs/** -text` or `specs/** text eol=lf`; recommended `-text` (matches `vendor/**`, avoids spec hash churn); CONCERNS.md notes git LF/CRLF warnings on specs |
| `contracts/*.json`, `README.md` | S-HT-1: four-key proposal, error bodies; S-OL-1: 200/202 and error codes; catalog `parameters_schema`; README: A1/A2/A5 status (A5 unverified until Phase 2), D-04 deviation, D-07 field, stub catalog shape change, two-proposal shape, harness limits; stale "one-request fixtures" wording |
| `scripts/launch-under-nono.ps1` | check for a version string (not inspected) |
| `.github/workflows/ci.yml` | already runs stubs tests plus `--ignored`; no change required; the new static test runs under default features |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo (MSVC) | all | yes | cargo 1.98.1 | — |
| Vendored crates | build | yes | `--locked --offline` build passes | — |
| PowerShell | scripts, Windows build info | yes | — | — |
| dumpbin (VS tools) | TC-BUILD-02 | ASSUMED present (prior row used it) | — | `link /dump` or document as not re-run |
| HKCU write permission | TC-NET-07 | no (previous row: denied) | — | keep `#[ignore]`, row `partial` |
| Canonical vocabulary file | CONF-11 | no | — | CONF-11 stays blocked; rest proceeds |

## Security Domain

Security enforcement enabled, ASVS L1, block on high.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no (Liam holds no credential; proxy injects) | Assert no `Authorization` leaves Liam (TC-KEY-01) |
| V3 Session Management | no | — |
| V4 Access Control | limited | Egress limited to the single loopback proxy; static scan (TC-NET-03) |
| V5 Input Validation | yes | Strict-struct parse, guarded error code, read-only catalog validation, 1..4000 question check |
| V6 Cryptography | limited | SHA-256 via `sha2` only; no hand-rolled crypto |
| V7 Error handling/logging | yes | Content-free stderr/log/record; one reason line |
| V12 File/resources | limited | Static scan for `Command`/process spawn |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Hostile hometown/olive body injecting text into stderr or log | Tampering / Info disclosure | D-06 pattern guard for reasons; serde-built events; never echo bodies |
| Proposal altered between hometown and olive | Tampering | Raw byte forwarding; olive's `trace_proposal_mismatch` flagged `investigate` |
| Duplicate-key smuggling (Liam validates one object, olive reads another) | Tampering | `deny_unknown_fields` struct parse errors on duplicates |
| Model-output command text executed | Elevation | No process spawn (static scan), answer/justification printed as text only |
| Gateway bypass / direct egress | Info disclosure | Single proxy client, DenyAll resolver, static scan, trap harness |
| Oversized input / DoS via question | DoS | 4000-scalar cap before any request |
| Stubs shipped in release | Tampering | `cfg(feature="stubs")` gating; release strings scan; new env var only in stubs |
| Credential-shaped values in fixtures | Info disclosure | Use `proxy-sentinel` style labels; scan fixtures in TC-KEY-03 |

## Sources

### Primary (HIGH confidence)
- `specs/LIAM_DESIGN_SPEC_v0.4.1.md` §3.3, §5.2-§5.4, §7.2-§7.4, §8.1-§8.3, §10.3, exit codes, §13 (read in full this session)
- `specs/LIAM_TEST_SPEC_v0.4.1.md` §1, §2, §3.1-§3.12, §4, §5
- Repo source: `src/main.rs`, `src/task.rs`, `src/clients/*.rs`, `src/evidence.rs`, `src/stubs.rs`, `tests/*.rs`, `Cargo.toml`, `liam.toml`, `contracts/*`, `RESULTS.md`, `.planning/codebase/CONCERNS.md`
- `vendor/serde_json` (Cargo.toml feature `raw_value`, `src/raw.rs`), `vendor/ryu` 1.0.23
- Baseline `cargo test --locked --offline --features stubs`: all passing at HEAD

### Secondary (MEDIUM confidence)
- RFC 8785 rules recalled from training (UTF-16 key ordering, ES number serialisation); verify against the RFC's Appendix B vectors when writing the canonicaliser [ASSUMED]

### Tertiary (LOW confidence)
- serde flatten/RawValue incompatibility (A1)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH, no new crates, features verified in vendored manifests
- Architecture: HIGH for gaps (directly read), MEDIUM for harness details (A6)
- Pitfalls: HIGH for code-derived ones, MEDIUM for RFC/serde ones

**Research date:** 2026-10-21
**Valid until:** 30 days (specs are ratified and stable; the canonical vocabulary and any spec revision could change CONF-11)
