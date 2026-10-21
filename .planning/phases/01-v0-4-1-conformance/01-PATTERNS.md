# Phase 1: v0.4.1 Conformance - Pattern Map

**Mapped:** 2026-10-21
**Files analyzed:** 25 (new or modified)
**Analogs found:** 23 / 25 (two have no in-repo analog)

All analogs are in-repo, and most files are modified in place, so the "analog" is usually the file's own current shape. Line numbers refer to HEAD.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/main.rs` (question check, TaskError map, retry wrapper) | CLI orchestrator | request-response | itself (`main.rs:80-279`) | self |
| `src/task.rs` (raw proposal, 4-key, schema validator, answer-on-olive-fail) | service | transform | itself (`task.rs:31-131`) | self |
| `src/catalog.rs` (new, optional) | utility (pure validator) | transform | `src/task.rs` `Catalog` structs (`task.rs:8-17`) + `main.rs:62-78` hand scanner | role-match |
| `src/clients/mod.rs` (`error_code` helper) | utility | transform | `src/config.rs` `Result<_, &'static str>` validators; `clients/mod.rs:47-55` | role-match |
| `src/clients/hometown.rs` (RawValue, codes) | client | request-response | itself (`hometown.rs:55-87`) | self |
| `src/clients/olive.rs` (explicit RawValue body, 200/202, codes) | client | request-response | itself (`olive.rs:42-56`) | self |
| `src/evidence.rs` (serde events, `reason`) | service | file-I/O | itself (`evidence.rs:9-121`) | self |
| `src/stubs.rs` (ScriptedStub, routing proxy, trap, `gateway_attempt`) | test fixture | request-response | `HometownStub::server_error_then_answer` (`stubs.rs:377-412`), `CaptureProxy::forward_two_to` (`stubs.rs:124-170`) | role-match |
| `tests/static_contract.rs` (new, TC-NET-03) | test | batch (file scan) | none in repo (shape: `tests/process_contract.rs` for layout only) | no analog |
| `tests/common/jcs.rs` (new, test-only) | test utility | transform | none | no analog |
| `tests/conformance_contract.rs` (new, TC-ANS/ACT/FAIL/SEC matrix) | test | request-response | `tests/process_contract.rs:7-92`, `:95-167` | exact |
| `tests/process_contract.rs` (extend, dedupe config) | test | request-response | itself | self |
| `tests/proxy_contract.rs` (migrate `code` -> `error`) | test | request-response | itself (`proxy_contract.rs:264-293`) | self |
| `contracts/catalog-0.4.json` | config/contract | n/a | itself | self |
| `contracts/hometown-answer.S-HT-1.json`, `olive-proposal.S-OL-1.json` | contract | n/a | themselves | self |
| `contracts/run-record.schema.json` (new, open question) | contract | n/a | `contracts/agent-record.schema.json` | role-match |
| `Cargo.toml` | config | n/a | itself (`:3`, `:11`) | self |
| `liam.toml` | config | n/a | itself | self |
| `profiles/liam.nono.toml` | config | n/a | itself (`:1-4`) | self |
| `.gitattributes` | config | n/a | itself (`:3`) | self |
| `README.md`, `contracts/vendored/VENDORED.md` | docs | n/a | `VENDORED.md` text | self |
| `RESULTS.md` | docs | n/a | itself (`RESULTS.md:7-21`) | self |
| `tests/` shared `write_test_config()` helper | test utility | n/a | the six duplicated `liam.toml` blocks in `process_contract.rs` | exact |
| `contracts/vendored/vocabulary.yaml` (CONF-11, blocked) | contract | n/a | none (user-supplied) | n/a |

## Pattern Assignments

### `src/clients/mod.rs` - `error_code` guard (utility, transform)

**Analog:** the existing `build_proxy_client` module plus the research snippet. Add next to `pub mod hometown; pub mod olive;` (`mod.rs:1-2`). This is the only place `clients/mod.rs` should gain code. The single `Client::builder()` at `mod.rs:47-55` must stay untouched, because the TC-NET-03 scan asserts exactly one constructor with `no_proxy()`, `Proxy::all(`, `dns_resolver(`.

**Existing constructor to leave alone** (`mod.rs:47-55`):
```rust
pub fn build_proxy_client(proxy: &str) -> Result<ProxyClient, reqwest::Error> {
    let resolver = Arc::new(DenyAllResolver::default());
    let client = Client::builder()
        .no_proxy()
        .proxy(Proxy::all(proxy)?)
        .dns_resolver(Arc::clone(&resolver))
        .build()?;
    Ok(ProxyClient { client, resolver })
}
```
**New helper shape** (derived from RESEARCH Code Examples, D-06; no regex crate): `pub fn error_code(body: &[u8]) -> Option<String>` parses `{"error": <string>}`, then accepts it only if the byte loop matches `^[a-z][a-z0-9_]{0,63}$`. Callers build the reason as `format!("hometown:{}", error_code(&body).unwrap_or_else(|| status.to_string()))`. Put a `#[cfg(test)]` unit test in the same file, following the `mod tests` convention at `mod.rs:57-89`.

---

### `src/clients/hometown.rs` (client, request-response)

**Analog:** itself. Replace `Proposal`/typed deserialise with RawValue and extend the error enum.

**Current types to change** (`hometown.rs:15-37`):
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AnswerResponse {
    pub trace_id: String,
    pub answer: String,
    pub context_complete: bool,
    pub proposal: Option<Proposal>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum HometownError {
    NoInjectedKey, KeyInactive, PurposeRefused, Status(u16), Unavailable, InvalidResponse,
}
```
Target: `proposal: Option<Box<RawValue>>` (extra top-level keys are ignored, so no `deny_unknown_fields` on `AnswerResponse`, per TC-SEC-03). The error variants carry a guarded reason string: `PurposeRefused(String)`, `Rejected { status: u16, reason: String }`, with `Unavailable` and `InvalidResponse` kept. Note `AnswerResponse` derives `Clone` and the task.rs tests clone it (`task.rs:145`); `Box<RawValue>` is `Clone`, so this still works.

**Core request pattern to keep** (`hometown.rs:56-66`): `.post(&self.endpoint).timeout(Duration::from_secs(self.timeout_seconds)).json(request).send().map_err(|_| HometownError::Unavailable)?`. Keep the request side unchanged. On the response side, replace `response.json()` with `response.bytes()` then `serde_json::from_slice`, so no body text can reach an error string.

**Status ladder to rewrite** (`hometown.rs:68-85`). Keep the order (status decides), drop the `body.contains("key_inactive")` substring check (`:69-77`):
```rust
if status.as_u16() == 403 { return Err(HometownError::PurposeRefused); }
if status.is_server_error() { return Err(HometownError::Unavailable); }
Err(HometownError::Status(status.as_u16()))
```
Locked interpretation: 403 -> exit 20, any other 4xx (401, 429 included) -> exit 21 no retry, 5xx/transport -> `Unavailable` (retry once in main).

**Test migration:** `tests/proxy_contract.rs:264-293` uses `{"code":"key_inactive"}` and the `Status(429)` expectation. Change the body key to `error` and the expected values to the new variants.

---

### `src/clients/olive.rs` (client, request-response)

**Analog:** itself.

**Current anti-pattern to remove** (`olive.rs:5-11`): `#[serde(flatten)] pub proposal: &'a Proposal`. Replace it with four explicit `&'a RawValue` fields (no flatten with RawValue):
```rust
#[derive(Serialize)]
pub struct ProposalRequest<'a> {
    pub action_type: &'a RawValue, pub party_ref: &'a RawValue,
    pub parameters: &'a RawValue, pub justification: &'a RawValue,
    pub run_id: &'a str, pub trace_ref: &'a str,
}
```
Send with an explicit `.body(serde_json::to_vec(&body)?)` plus `Content-Type: application/json` instead of `.json(request)` (`olive.rs:48`), so the serialisation point is visible.

**Response** (`olive.rs:12-18`): `ProposalResponse` keeps `proposal_id`, `status`, `#[serde(default)] reason: Option<String>`. Add the HTTP status (200 vs 202) for D-09 `http_status`. The reason is guarded by `error_code`'s pattern, off-pattern becomes `None`.

**Errors** (`olive.rs:19-24`): `Status(u16)` becomes a code-carrying variant, for example `Rejected { status, reason }`. 5xx/transport -> `olive:unavailable`, no retry. Status check at `olive.rs:51-53` currently maps every non-2xx to `Status(code)`; split 4xx (rejected with guarded code) from 5xx (unavailable).

**Test migration:** `tests/proxy_contract.rs:220-233` builds a `Proposal` struct and passes `proposal: &proposal`; rebuild it from raw JSON.

---

### `src/task.rs` (service, transform)

**Analog:** itself. Keep the suppression structure, but change its inputs.

**Keep this early-return shape for each suppression path** (`task.rs:67-75`, repeated at `:76-88`, `:89-100`, `:101-116`):
```rust
return Ok(TaskResult { answer: answer.answer, context_complete: answer.context_complete,
    trace_id: answer.trace_id, proposal: None, suppressed: Some("schema") });
```
Collapse the repetition with a small local helper closure or fn. Reasons stay `schema`, `not_in_catalog`, `limit`.

**Replace** the typed `proposal.party_ref` / `proposal.action_type` / `proposal.parameters` access with a read-only parse of the raw bytes:
```rust
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FourKeys { action_type: String, party_ref: String, parameters: serde_json::Value, justification: String }
// from_str::<FourKeys>(raw.get()).is_err() => suppress as "schema"
```
(from RESEARCH; duplicate keys error for derived structs.) Keep the `party_ref != party_ref` check (`task.rs:67`, D-04).

**Catalog** (`task.rs:8-17`): the `Catalog`/`CatalogAction` structs with `required_parameters` become `parameters_schema` (D-03). Keep `catalog.version != catalog_version` mapped to an error (`task.rs:45-47`) and the `action_id_format` check. The two inline-catalog unit tests (`task.rs:175`, `:197`) embed `required_parameters` and must be updated or the field kept readable.

**Gap to fix (Pitfall 1):** `task.rs:117-123` `.map_err(TaskError::Olive)?` drops the answer, so `main.rs` prints nothing on exit 22. Make `TaskError::Olive` carry `{answer, context_complete, trace_id, error}`, or return a `TaskResult` with an `olive_failure` field. TC-ACT-08 and TC-FAIL-06 require the answer printed.

**Test fakes pattern to keep** (`task.rs:140-158`): in-memory `Hometown`/`Olive` structs implementing the traits, with `assert_eq!(request.trace_ref, "trace-1")` inside the fake. Add new cases (4-key, extra key, `"5"` vs integer, two proposals) in this `mod tests` block.

---

### `src/evidence.rs` (service, file-I/O)

**Analog:** itself.

**Fix the unescaped `event()`** (`evidence.rs:59-61`):
```rust
pub fn event(&mut self, event: &str) {
    self.lines.push(format!("{{\"event\":\"{event}\"}}"));
}
```
Replace it with a serde-built line (`serde_json::json!({"event": name, ...fields})` or a `Serialize` struct) so the new fields (`reason`, `http_status`, `investigate`, `outcome`) cannot break JSON. The initial `run.started` line at `evidence.rs:55` is the same pattern; build it the same way.

**Add `reason`** to `ProposalRecord` (`evidence.rs:38-42`): `reason: Option<&'a str>` serialised as `null` when absent. `finish` takes `proposal: Option<(&str, &str)>` (`:73`); widen it to a triple or small struct. Existing call at `main.rs:207-211` adapts.

**Content-free test pattern to copy** (`evidence.rs:127-157`): the `CANARY-7f3a` sentinel asserted absent from both record and log. Change the `"0.4.0"` literal at `:144` to `"0.4.1"` (D-16).

---

### `src/main.rs` (CLI orchestrator, request-response)

**Analog:** itself. Keep the `fail(code, reason) -> !` and `.unwrap_or_else(|_| fail(CODE, "area:detail"))` conventions.

**`fail`** (`main.rs:18-21`) is the only exit path:
```rust
fn fail(code: i32, reason: &str) -> ! {
    eprintln!("liam: {code} {reason}");
    std::process::exit(code);
}
```
**Question check (CONF-04):** insert right after `required_arg("--question")` (`main.rs:86`), before the `vocabulary` read:
`if !(1..=4000).contains(&question.chars().count()) { fail(11, "input:question_length") }`. Pitfall: `required_arg` uses `env::args()` (`main.rs:44`), which panics on non-Unicode; switch to `args_os()` and map non-UTF-8 to `fail(11, "args:required")`.

**Retry** (`main.rs:160-184`) currently re-runs all of `task::run`. Replace with a retry wrapper around `hometown.answer(...)` only (catalog parsed once), same request re-sent.

**Error mapping** (`main.rs:218-277`): the eight near-identical `finish_failure(&mut evidence, &session, &config, &purpose, &party_ref, outcome, code, reason)` arms collapse into one `TaskError -> (outcome, code, reason)` function. Keep outcome names `purpose_refused`, `hometown_no_injected_key`, `hometown_key_inactive`, `hometown_failed`, `hometown_unavailable`, `olive_failed`.

**Stubs-only env gating pattern to copy for `LIAM_STUB_GATEWAY_ATTEMPT`** (`main.rs:29-42` and `:117`):
```rust
#[cfg(feature = "stubs")]
fn launch_process_id() -> u32 {
    if env::var("LIAM_STUBS").as_deref() == Ok("1") { ... }
    std::process::id()
}
let stub_arm = if cfg!(feature = "stubs") && env::var("LIAM_STUBS").as_deref() == Ok("1") { ... }
```
Put the env-var literal inside a `#[cfg(feature = "stubs")]` function or in `src/stubs.rs`, so it never appears in the release binary (TC-BUILD-01).

**Success path** (`main.rs:186-217`): prints the Notice, answer and `Proposal {id}: {status}`; emits `proposal.suppressed` via `evidence.event`. Extend it to emit `proposal.suppressed{reason}`, `proposal.submitted{http_status}`, `proposal.refused{reason, investigate}` (D-08/D-09) with exit 0 and empty stderr.

---

### `src/stubs.rs` (test fixture, request-response)

**Analog:** `HometownStub::server_error_then_answer` (`stubs.rs:377-412`) for ordered replies; `CaptureProxy::forward_two_to` (`stubs.rs:124-170`) for routing. Do not add a fourth copy-pasted `CaptureProxy` variant (`forward_once` `:35`, `forward_two` `:79`, `forward_two_to` `:124`); build `ScriptedStub` + one routing `TestProxy` and keep existing constructors as thin wrappers (so the existing tests keep working).

**Server bootstrap boilerplate (all stubs use this, `stubs.rs:238-245`):**
```rust
let server = Server::http("127.0.0.1:0").expect("start hometown stub");
let base_url = format!("http://{}", server.server_addr().to_ip().expect("IP listener"));
let capture = Arc::new(Mutex::new(HometownCapture::default()));
let thread_capture = Arc::clone(&capture);
```
**Request capture (header + body), repeated in every stub (`stubs.rs:248-261`):**
```rust
let authorization = request.headers().iter()
    .find(|header| header.field.equiv("Authorization"))
    .map(|header| header.value.as_str().to_owned());
let mut body = String::new();
request.as_reader().read_to_string(&mut body).expect("read hometown body");
```
**Proxy forward + sentinel injection (`stubs.rs:60-71`):**
```rust
let response = reqwest::blocking::Client::new()
    .post(target)
    .header("Authorization", "Bearer proxy-sentinel")
    .header("Content-Type", "application/json")
    .body(body).send().expect("forward request");
let status = response.status().as_u16();
let body = response.bytes().expect("read forwarded response");
request.respond(Response::from_data(body).with_status_code(status)).expect("respond to Liam");
```
Changes the research calls for: forwarding client uses `.no_proxy()` so it does not loop through ambient proxy vars; replace `expect` with recorded errors (proxy-thread panics turn into 2 s timeouts that look like `hometown:unavailable`); loop with a stop flag so threads end.

**Existing delay-based stub to copy for timeout cases** (`stubs.rs:434-458` `timeout_twice`, `thread::sleep(Duration::from_secs(2))`).

**Existing stub-record fixture to reuse as-is** (`stubs.rs:195-219` `StubRecordPair::for_proxy`; layers list at `:200-202`, `direct_gateway: false` at `:208`). The `inject-gateway`, `census-13`, `keyid-upper`, `stale`, `future` mutations for TC-START rows will need mutated variants of this builder.

**Gateway trap (CONF-08).** Closest existing trap: the Windows registry test at `tests/proxy_contract.rs:115-129` (nonblocking `TcpListener` with an `AtomicBool` hit flag):
```rust
let trap = TcpListener::bind("127.0.0.1:0").expect("start trap listener");
trap.set_nonblocking(true).expect("make trap nonblocking");
...
if trap.accept().is_ok() { trap_capture.store(true, Ordering::Relaxed); return; }
```
Move this into `stubs.rs` as `GatewayTrap` with a hit counter (`stubs.rs` is excluded from the static scan, so `std::net` is permitted there). Add `gateway_attempt(client: &reqwest::blocking::Client) -> &'static str`, sending plain `http://gateway.invalid/` through the real proxy client; the test proxy refuses the host (403) the way nono's allowlist would, plus a negative-control mode mapping the host to the trap.

---

### `tests/static_contract.rs` (test, batch file scan) - NO ANALOG

No existing source-scan test. Use RESEARCH Pattern 5. Layout conventions to copy from `tests/process_contract.rs`: file named `<area>_contract.rs`, but do **not** add `#![cfg(feature = "stubs")]` (line 1 of the other two); this test must run under default features. Resolve source paths via `std::env::current_dir()` as `process_contract.rs:13-18` does (`std::env::current_dir().unwrap().join("contracts/...")`), here `.join("src")`. Required: comment/string-aware scanner (a naive `//` split breaks on `"https://..."`), self-test with synthetic source for each forbidden token, skip `src/stubs.rs`. Existing literals `"direct_gateway"` (`src/lib.rs`, `src/receipt.rs`) must not be flagged.

---

### `tests/conformance_contract.rs` and `tests/process_contract.rs` (test, request-response)

**Analog:** `tests/process_contract.rs:95-167` (two-service flow) and `:276-308` (error-exit flow). Copy the process-test skeleton:
```rust
#![cfg(feature = "stubs")]
let fixture = StubRecordPair::for_proxy(proxy.port());
fs::write(fixture.directory().join("liam.toml"), format!(r#"[agent] ..."#, ...)).unwrap();
let output = Command::new(env!("CARGO_BIN_EXE_liam"))
    .current_dir(fixture.directory())
    .args(["--purpose","servicing","--party","opaque-party","--question","question"])
    .env("LIAM_STUBS","1").env("LIAM_STUB_PID","0")
    .env("HTTPS_PROXY", proxy.url()).env("HTTP_PROXY", proxy.url()).env("NO_PROXY","")
    .env("NONO_RECEIPT_PATH", fixture.receipt_path())
    .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
    .output().unwrap();
assert_eq!(output.status.code(), Some(21));
assert_eq!(String::from_utf8_lossy(&output.stderr).trim_end(), "liam: 21 hometown:unavailable");
```
**Dedupe first (Pitfall 2):** the per-test `liam.toml` is copy-pasted six times (`process_contract.rs:20-49`, `108-138`, `182-212`, `282`, `317`, `354`, `390`), each with `version = "0.4.0"` and `sha256 = "f3412d68..."`. Extract `write_test_config(...)` into `tests/common/mod.rs` so the version and (later, CONF-11) hash change in one place. Include `tests/common/mod.rs` from each test crate with `mod common;`.

**Content-free assertions to reuse** (`process_contract.rs:80-91`): read the single `.json` from the runs dir, hash `.jsonl`, compare `record["log_sha256"]`, assert canary absent.

**Request-count assertion** (`process_contract.rs:164`, `:307`): `proxy.captured().request_count`.

---

### `tests/common/jcs.rs` (test utility, transform) - NO ANALOG

Hand-written RFC 8785 canonicaliser per RESEARCH Pattern 6 (UTF-16 key order, ES number layout via `ryu`, escaping rules, RFC 8785 Appendix B vectors). Hashing reuses `sha2::{Digest, Sha256}` exactly as `process_contract.rs:4,89` (`format!("{:x}", Sha256::digest(&log))`). Requires `[dev-dependencies] ryu = "1"` (see Cargo note below).

---

### Config and contract files

**`Cargo.toml`** (`:3` `version = "0.4.0"`, `:11` `serde_json = "1"`): set `version = "0.4.1"`, `serde_json = { version = "1", features = ["raw_value"] }`, optional `[dev-dependencies] ryu = "1"`. Build once without `--locked` to update `Cargo.lock`, and check there is no `vendor/` diff (A2).

**`liam.toml`** (`[agent] version = "0.4.0"`, sha256 line): `version = "0.4.1"`; sha256 stays `f3412d68...` until CONF-11 arrives.

**`profiles/liam.nono.toml`** (header `:1-4`, current `profile_version = "0.3.0"` at `:4`): set `profile_version = 4` (integer) and rewrite only the header comment (NONO_v0.4.3 ratified; draft pending nono-owner review). Change nothing else.

**`.gitattributes`** (currently only `vendor/** -text`, `:3`): add a matching line `specs/** -text` with a comment in the same style as lines 1-2.

**`contracts/catalog-0.4.json`** (current, 4 lines of content):
```json
{ "version": "0.4", "actions": [ {"action_type": "crm.case_note@1", "required_parameters": ["note"]} ] }
```
Add per-action `parameters_schema` (`type`, `properties`, `required`, `additionalProperties: false`, optional `maxLength`) and stub-only actions such as `crm.follow_up@1` (`days: integer`) and a free-form `crm.case_update@1` (`details: {type: object}`). Integer means `is_i64()||is_u64()` (stricter than JSON Schema 2020-12; document).

**`contracts/hometown-answer.S-HT-1.json` / `olive-proposal.S-OL-1.json`**: single-line-per-section style (`"response": {"required": [...], "proposal": [...]}`, `"success_statuses": [200, 202]`). Add error-body shape `{"error": "<code>"}` and the guard pattern, and four-key rule wording; match the existing compact-JSON style.

**`contracts/vendored/VENDORED.md`**: existing text states the placeholder deviation (`f3412d68...` vs canonical `60a44f00...`). On CONF-11 delivery, rewrite to verified, only after the supplied file hashes to `60a44f00...` (otherwise stop; do not pin).

**`RESULTS.md`**: the existing table format (`RESULTS.md:7-21`) is the row template: `| Claim | TC | n of n | Tag | Windows build | Liam commit | nono commit | Limits / not measured | Status |`. Split grouped rows (`:9-17`) into one row per TC ID; replace `not recorded`/`uncommitted` with Windows build (`cmd /c ver` + UBR) and the short hash of the code commit; keep `local development` as the tag; keep `not measured` rows for real-nono/E2E (existing examples `:18-21`) and mark TC-START-07 `deferred`.

## Shared Patterns

### Fail-closed single reason line
**Source:** `src/main.rs:18-21` and every prerequisite in `main.rs:81-139`.
**Apply to:** question-length check, every new prerequisite.
```rust
.unwrap_or_else(|_| fail(11, "vocab:missing"))
```
Never `unwrap()`/`expect()` outside tests and `stubs.rs`.

### Content-free output
**Source:** `src/evidence.rs:1-2,127-157`.
**Apply to:** every new log event, reason string, record field.
Only guarded codes (`error_code`), numeric statuses, identifiers and fixed outcome strings. Never echo a body, header or URL. Build events with serde, not `format!`.

### Stubs-only gating
**Source:** `src/main.rs:29-42,117`; `tests/*.rs:1` `#![cfg(feature = "stubs")]`.
**Apply to:** `LIAM_STUB_GATEWAY_ATTEMPT`, `gateway.invalid`, `gateway_attempt`, trap.
All of it lives in `src/stubs.rs` or `cfg(feature = "stubs")` blocks. `static_contract.rs` is the one test that is not feature-gated.

### Typed client errors, status decides
**Source:** `src/clients/hometown.rs:29-37,68-85`.
**Apply to:** hometown and olive clients and the `main.rs` exit mapping.
Status picks retry and exit code; the guarded body code only supplies the reason, with fallback `<service>:<status>`.

### Test harness skeleton
**Source:** `tests/process_contract.rs:95-167` (stub + proxy + `StubRecordPair` + `Command` with `LIAM_STUBS=1`, `LIAM_STUB_PID=0`, loopback proxy env, record paths).
**Apply to:** all new process tests. Pull the `liam.toml` text into one `write_test_config()` helper.

### Library `Result` conventions
**Source:** `clients/mod.rs` (`Result<ProxyClient, reqwest::Error>`) and CLAUDE.md (`Result<_, &'static str>` or typed enums). Pure validators take inputs as arguments so they are unit-testable (`task.rs:31-42`, `validate_startup`).

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `tests/static_contract.rs` | test | source scan | No existing test reads `src/`; TC-NET-03 was a manual grep. Use RESEARCH Pattern 5. |
| `tests/common/jcs.rs` | test utility | transform | No canonicaliser in repo, and D-02 forbids vendoring one. Use RESEARCH Pattern 6 and the RFC 8785 vectors. |

Also without an in-repo source: `contracts/vendored/vocabulary.yaml` (CONF-11, user-supplied) and `contracts/run-record.schema.json` (TC-EVD-01 references it, but it does not exist; model structure on `contracts/agent-record.schema.json`, which was not read in this pass).

## Notes for the planner

- Existing tests encode old behaviour and will fail until migrated: `proxy_contract.rs:264-293` (`code` vs `error`, `Status(429)`), `proxy_contract.rs:220-233,296-343` (`Proposal` struct, `OliveError::Status`), `process_contract.rs` (`version = "0.4.0"` x7), `evidence.rs:144`, `task.rs` tests (typed `Proposal`, `required_parameters`).
- The unparseable-200 behaviour (`InvalidResponse` -> `hometown:unavailable`, no retry) is current behaviour; keep and document it (A5).
- Open question 1 (two-proposal shape): RESEARCH recommends `proposal` as an array, first used, rest logged `limit`. Planner must record the choice in README.
- The final ordering from RESEARCH holds: test infra and migration, then client/task/evidence, then input check and labels, then static check and trap, then measure and RESULTS in a docs-only commit; CONF-11 independent and blocked.

## Metadata

**Analog search scope:** `src/`, `tests/`, `contracts/`, `profiles/`, root config files, `RESULTS.md`
**Files scanned:** 17 read in full or in part
**Pattern extraction date:** 2026-10-21
