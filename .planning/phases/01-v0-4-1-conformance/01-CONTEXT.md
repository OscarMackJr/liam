# Phase 1: v0.4.1 Conformance - Context

**Gathered:** 2026-10-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Against stubs only, bring Liam's code, S-HT-1/S-OL-1 stubs, `contracts/`, README, version labels and RESULTS.md into line with the ratified v0.4.1 design and test specs (CONF-01..CONF-11). Every TC that can run locally gets a measured RESULTS.md row. No real nono, no live services, no §4 changes.

Already fixed by the ratified spec and not re-decided here: the §5.4 and §7.4 error tables, `--question` 1..4000 Unicode scalar values (exit 11 `input:question_length`, no request, no run record), read-only validation with no coercion (§7.3), the exit codes, DSSE staying deferred (fail closed on `record_public_key`).

Interpretation (locked): the HTTP status alone decides retry and exit code, as in v0.4 ("status handling … unchanged", §5.4). The body code only supplies the reason. So 403 always exits 20, 401/400/409/422/other 4xx exit 21, and any hometown 5xx/timeout retries once and then gives `hometown:unavailable` whatever its body says. olive 5xx/timeout gives `olive:unavailable` with no retry.

</domain>

<decisions>
## Implementation Decisions

### Unchanged pass-through (D15 / CONF-01, CONF-09)
- **D-01:** Carry the proposal as `serde_json::value::RawValue` (feature `raw_value`, already in vendored serde_json, no new crate). Keep each of the four fields as hometown's exact JSON bytes. Validation parses a separate read-only copy. The olive request body embeds the raw bytes, so the forwarded proposal is byte-identical to hometown's, which is stronger than JCS-identical.
- **D-02:** The JCS (RFC 8785) SHA-256 for TC-ACT-07 is computed by a small hand-written canonicaliser in test code only (`tests/` or `#[cfg(test)]`), using the vendored `ryu` for ECMAScript number formatting and checked against RFC 8785 vectors. Do not vendor a JCS crate. Liam does not compute JCS at runtime.
- **D-03:** Extend `contracts/catalog-0.4.json` with a per-action `parameters_schema` in a JSON-Schema subset: `type`, `properties`, `required`, `additionalProperties: false`, optionally `maxLength`. Validate it with a small hand-written validator in Liam. This makes TC-ACT-03's `"5"`-vs-integer case expressible, and the case fails as `schema` without being coerced. Record in README that this is a stub-catalog (A2) shape change; real olive's catalog shape is checked at the Phase 4 contract diff. Keep the `action_id_format` check (`^[a-z0-9_.]+@[0-9]+$` from the vocabulary).
- **D-04:** Keep the existing check that suppresses a proposal whose `party_ref` ≠ `--party` (suppressed as `schema`). Record it in README as a deviation beyond §7.3. It only suppresses and never alters, so D15 holds.
- **D-05:** The four-key rule (§5.3/§7.3) applies to the raw proposal object. A proposal with other than exactly `action_type`, `party_ref`, `parameters`, `justification` is suppressed as `schema`. Extra top-level response keys are ignored (TC-SEC-03). Extra proposals in a non-conforming response are ignored and logged as `proposal.suppressed` reason `limit`.

### Error codes and run record (D16 / CONF-02, CONF-03)
- **D-06:** Parse `{"error": "<code>"}` from hometown and olive error bodies. Accept the code only if it matches `^[a-z][a-z0-9_]{0,63}$`. Otherwise (missing, unparseable, non-string or off-pattern) fall back to `<service>:<status>`. This keeps stderr and evidence content-free even if a service misbehaves. Unlisted but well-formed codes such as `rate_limited` pass through.
- **D-07:** Add `reason` to the run record's proposal object: `proposal: {proposal_id, status, reason}`. `reason` is null unless olive returned `refused_policy`, and is guarded by the same pattern as D-06. Record it in README as an additive §8.1 field implied by TC-ACT-09.
- **D-08:** A `trace_proposal_mismatch` refusal is flagged by `"investigate": true` on the `proposal.refused` log event. Exit stays 0 and stderr stays empty.
- **D-09:** An idempotent replay (200 with an existing `proposal_id`) is recorded in the run record exactly as a fresh submission. The `proposal.submitted` log event carries `http_status` (200 or 202) so the two can be told apart.

### Gateway trap and static check (CONF-07, CONF-08)
- **D-10:** The stub-only gateway attempt for TC-NET-04 is triggered by env var `LIAM_STUB_GATEWAY_ATTEMPT=1`, read only under `cfg(feature = "stubs")`. This matches the existing `LIAM_STUBS`/`LIAM_STUB_PID` pattern and leaves the release CLI unchanged.
- **D-11:** The attempt code and the stand-in gateway host (`gateway.invalid`) live only in `src/stubs.rs`. It exposes something like `stubs::gateway_attempt(&client)`, which sends one request through Liam's real single proxy client. The test proxy refuses the gateway host the way nono's allowlist would, and a `gateway-trap` listener counts hits, which must be zero. main logs a content-free `gateway.attempt` event with outcome `refused`. The run still completes through hometown. A normal run (flag unset) also gets zero trap hits.
- **D-12:** TC-NET-03 is a Rust integration test, `tests/static_contract.rs`, run by `cargo test`. It reads `src/**/*.rs`, skips `src/stubs.rs`, and allows exactly one client constructor, in `src/clients/mod.rs`, asserting that it calls `no_proxy()`, `Proxy::all(..)` and `dns_resolver(..)`. It also asserts zero `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, Winsock calls and second client constructors.
- **D-13:** A "gateway reference" means a gateway-shaped host (`gateway.` hostnames or the trap host) or a model API path (`/v1/chat/completions`, `/v1/messages`, `/v1/completions`) in a string literal. The word "gateway" in comments is allowed.

### Vocabulary hash, profile, RESULTS (CONF-06, CONF-10, CONF-11)
- **D-14:** Both `contracts/vendored/vocabulary.yaml` and `specs/_agent_inputs/vocabulary.yaml` hash to `f3412d68…`, not the canonical `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`. **The user will supply the canonical vocabulary file.** The plan treats CONF-11 as a blocking task waiting on that file, and the other CONF items proceed meanwhile. When the file arrives: vendor it unchanged, verify it hashes to `60a44f00…`, pin that hash in `liam.toml`, update `VENDORED.md`/README, and re-run TC-LVOC-01/02. If the supplied file does not hash to `60a44f00…`, stop for review and do not pin it.
- **D-15:** In `profiles/liam.nono.toml`, set `profile_version = 4` (integer, matching `profile_min_version` and the §4.1 integer rule). Update the header to say `NONO_v0.4.3` is ratified and the file is still a draft pending nono-owner review. Change no other profile content, since §4 changes need re-ratification.
- **D-16:** Version labels move to 0.4.1: `Cargo.toml`, `liam.toml` `[agent] version`, run record `agent_version`, and the `src/evidence.rs` test that hardcodes 0.4.0. Add a line-ending policy for `specs/**` in `.gitattributes`.
- **D-17:** RESULTS.md gets one row per TC ID. Every locally runnable v0.4.1 TC gets its own row with n of n, tag, Liam commit, Windows build and limits. Existing grouped rows are split. Real-nono/E2E TCs keep `not measured` rows, and TC-START-07 is marked `deferred` (DEF-01).
- **D-18:** Measurements run on a clean tree at a code commit. Each row records that short hash, the Windows build (OS version) and the true tag, `local development` (not `ci-elevated` unless it really ran there). The RESULTS.md update lands in a separate docs-only commit afterwards. A CI run is not required for Phase 1.

### Claude's Discretion
- Refactoring `src/main.rs` as needed for D16, for example a request-context struct, a single `TaskError` → (outcome, code, reason) mapping, and retrying only `hometown.answer` rather than all of `task::run`. Behaviour must stay exactly as the spec says.
- Shape of the extended `HometownError`/`OliveError` enums carrying a code string.
- Where in the flow the gateway attempt fires, provided the run still completes through hometown.
- Updating the stub S-HT-1/S-OL-1 servers, the `contracts/*.json` schemas and README for the v0.4.1 shapes: four-key proposal, new error codes, olive v1 202/200 responses, and A1/A2/A5 status (CONF-05).
- Test layout and naming, following `<area>_contract.rs`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Governing specs (the specs win over planning docs; a disagreement is a stop condition)
- `specs/LIAM_DESIGN_SPEC_v0.4.1.md`: §3.3 (task flow, step 1 question length), §5.2–§5.4 (S-HT-1 request/response/errors), §7.2–§7.4 (S-OL-1, Liam's checks, olive responses), §8.1–§8.3 (evidence, join keys), §10.3 (stub mode), exit codes table, §12 REQ-LIAM-19..21, §13 assumptions A1/A2/A5
- `specs/LIAM_TEST_SPEC_v0.4.1.md`: §1 tags, §2 fixtures (`gateway-trap`), TC-NET-03/-04, TC-ANS-05..07, TC-ACT-03/-07/-08/-09, TC-SEC-03, TC-LVOC-01/02, TC-FAIL-03/-04/-08, exit criteria
- `specs/LIAM_AMENDMENT_v0.4.1_proxy-transport.md`: D14, revised TC-NET-03, TC-NET-05..08

### Project rules
- `AGENTS.md`: no secrets, loopback proxy only, stop on contract mismatch, content-free evidence, one reason line
- `.planning/REQUIREMENTS.md`: CONF-01..CONF-11
- `.planning/codebase/CONCERNS.md`: known v0.4.1 gaps and fragile areas (stub harness duplication, vocab parsing, evidence `event()` escaping)

### Contracts and fixtures
- `contracts/hometown-answer.S-HT-1.json`, `contracts/olive-proposal.S-OL-1.json`: stub wire schemas to update
- `contracts/catalog-0.4.json`: stub catalog, gains `parameters_schema` (D-03)
- `contracts/vendored/vocabulary.yaml`, `contracts/vendored/VENDORED.md`: vocabulary and hash provenance (D-14)
- `profiles/liam.nono.toml`: profile draft (D-15)
- `RESULTS.md`: measurement table format (D-17, D-18)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/clients/hometown.rs` / `olive.rs`: `HometownClient`/`OliveClient` traits and error enums. Extend these for codes rather than adding new seams.
- `src/task.rs`: existing catalog lookup, required-parameter check and suppression paths (`schema`, `not_in_catalog`), with in-memory fake clients in tests.
- `src/stubs.rs`: Hometown/Olive stubs, capture proxy (three near-identical proxy variants; change them together), stub record pair. The home for the gateway attempt and the trap.
- `src/evidence.rs`: run record and run log writer. Note that `event()` uses an unescaped `format!`, so new fields (`http_status`, `investigate`, `reason`) should use serde rather than string interpolation.
- Vendored crates available without a refresh: `serde_json` (`raw_value` feature), `indexmap`, `ryu`, `itoa`, `sha2`.

### Established Patterns
- Library functions return `Result<_, &'static str>`/typed enums. Only `main.rs` exits, via `fail(code, reason)` and `finish_failure`.
- Stub-only behaviour is gated by `cfg(feature = "stubs")` plus a `LIAM_STUB_*` env var.
- `.cargo/config.toml` sets `net.offline = true` with crates-io replaced by `vendor/`, so no new crates can be added without a bundle refresh.

### Integration Points
- `src/main.rs`: argument parsing (add the question-length check before any network use), error-to-exit mapping, retry, evidence finish.
- `src/clients/mod.rs`: the single proxy client constructor that TC-NET-03 asserts against.
- `tests/process_contract.rs`, `tests/proxy_contract.rs`: process and proxy harnesses to extend, plus the new `tests/static_contract.rs`.

</code_context>

<specifics>
## Specific Ideas

- The forwarded proposal should be byte-identical to hometown's, not merely JCS-equal. TC-ACT-07 is then checked with JCS as the spec states.
- The code-guard regex is exactly `^[a-z][a-z0-9_]{0,63}$`.
- The gateway stand-in host is `gateway.invalid`.

</specifics>

<deferred>
## Deferred Ideas

- Refusing non-`https` `base_url` values at config load (CONCERNS.md security note). This is hardening beyond the v0.4.1 conformance scope, and is a candidate for a later revision or Phase 3/4 when real URLs are configured.
- Replacing the hand-rolled vocabulary YAML line-scanner (CONCERNS.md tech debt). Not required by CONF-11 unless the canonical file's format breaks it, in which case it becomes in-scope.
- Executable-directory-relative config paths and the missing `%LOCALAPPDATA%` fallback (CONCERNS.md). Relevant to Phase 2's real launch.

</deferred>

---

*Phase: 01-v0-4-1-conformance*
*Context gathered: 2026-10-21*
