# Requirements: Liam

**Defined:** 2026-10-21
**Core Value:** One Liam run, started only inside an enforced nono session, leaves content-free records that join end to end. It holds no credential, reaches nothing but the loopback nono-proxy and executes nothing.

Source of truth: `specs/LIAM_DESIGN_SPEC_v0.4.1.md` §12 (REQ-LIAM-*) and `specs/LIAM_TEST_SPEC_v0.4.1.md` (TC-*). Each requirement below cites the REQ-LIAM and TC IDs it closes. If this file and the specs disagree, the specs win, and the disagreement is a stop condition.

## v1 Requirements

### Spec Conformance (v0.4.1, local against stubs)

- [x] **CONF-01**: Liam forwards hometown's proposal to olive unchanged. The JCS SHA-256 of the four proposal fields olive receives equals the hash of what hometown returned, including nested objects, non-ASCII text, fractional numbers and unordered keys (D15; REQ-LIAM-19; TC-ACT-07, TC-ACT-03)
- [x] **CONF-02**: A hometown error exits 21 with `hometown:<code>` taken from the `{"error": "<code>"}` body, falling back to `hometown:<status>`. 429 `rate_limited` is not retried; a 5xx with an empty body is retried once, then exits `hometown:unavailable` (D16; REQ-LIAM-20; TC-ANS-05, TC-ANS-07)
- [x] **CONF-03**: olive errors exit 22 with `olive:<code>` per §7.4, with no retry. A 5xx or timeout gives `olive:unavailable`. A 200 `refused_policy` (with its reason) and a 200 idempotent replay are recorded in the run record and exit 0 (D16; REQ-LIAM-08/20; TC-ACT-08, TC-ACT-09)
- [ ] **CONF-04**: A `--question` outside 1..4000 Unicode scalar values exits 11 `input:question_length`. No request is sent and no run record is written (D17; REQ-LIAM-21; TC-ANS-06)
- [ ] **CONF-05**: The S-HT-1 and S-OL-1 stubs, the `contracts/` schemas and README match v0.4.1 §5 and §7: the four-key proposal, the new hometown error codes, olive's v1 responses, and A1/A2/A5 status recorded
- [ ] **CONF-06**: Version labels read 0.4.1 in `Cargo.toml`, `liam.toml` and the run record's `agent_version`. The profile version in `profiles/liam.nono.toml` matches the type and value Liam validates. Spec files have a line-ending policy in `.gitattributes`
- [ ] **CONF-07**: An automated TC-NET-03 static check finds zero `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, Winsock calls, second client constructors or gateway host/path references in Liam's own source outside the single client and the feature-gated stubs (REQ-LIAM-03)
- [ ] **CONF-08**: A local gateway-trap harness covers TC-NET-04. A normal run makes zero hits on the trap. A run with the stub-only gateway-attempt flag is refused by the test proxy, and its failure is recorded content-free (REQ-LIAM-17)
- [x] **CONF-09**: An answer carrying command text, extra keys or shell strings is parsed as data against S-HT-1. Extra keys are ignored and nothing is executed (REQ-LIAM-12; TC-SEC-03)
- [ ] **CONF-10**: RESULTS.md records a measurement (n of n, tag, Liam commit, limits) for every v0.4.1 TC that can run locally. TC-START-07 (DSSE) is marked deferred
- [ ] **CONF-11**: `liam.toml` pins the canonical vocabulary SHA-256 (`60a44f00…`) of the vendored `vocabulary.yaml` instead of the placeholder, so TC-LVOC-01/02 pass against the canonical file (REQ-LIAM-06)

### Real nono Session (this dev machine)

- [ ] **NONO-01**: A contract-diff checkpoint compares the nono fork's current `EnforcementReceipt`, `AgentSessionRecord` and proxy environment with `contracts/receipt-fields.md`, §4.2 and A5. Any mismatch updates the contract table and README, then stops for review
- [ ] **NONO-02**: The `token_arm` the fork records for an AppContainer launch on `DirectCli`, and the census against the §4.3 expectancy table, are measured and recorded (O7). Work stops for nono-owner review, and `expected_token_arm` is set only after sign-off
- [ ] **NONO-03**: Liam launched with `scripts/launch-under-nono.ps1` passes the startup self-check under the real nono fork. A receipt or agent record the user can write is refused (REQ-LIAM-01; TC-START-06)
- [ ] **NONO-04**: Each refusal case is measured: direct launch outside nono exits 10 or 11 with no network activity (TC-E2E-03); with the WFP service stopped, the session is refused or Liam exits 10 on the census (TC-E2E-04); with the key exchange unreachable, nono refuses the session (TC-E2E-06)
- [ ] **NONO-05**: Under a real session, Liam spawns zero child processes (TC-SEC-01), touches only its file grants (TC-SEC-02) and makes zero DNS queries from its own PID (TC-NET-08) (REQ-LIAM-03/11)

### Live Service Integration

- [ ] **LIVE-01**: A contract-diff checkpoint compares the running hometown §2.6 server with S-HT-1 as specified in v0.4.1 §5. Any mismatch updates the contract table and README, then stops for review
- [ ] **LIVE-02**: Under real nono, Liam gets an answer from live hometown through nono-proxy. Liam sends no `Authorization` header; the proxy injects the agent key; the request carries `run_id` and `nono_session_id` (REQ-LIAM-04/05/07; TC-KEY-01, TC-META-01)
- [ ] **LIVE-03**: A contract-diff checkpoint compares the published olive v1 proposal API with S-OL-1 and §7.2–§7.4. Any mismatch updates the contract table and README, then stops for review
- [ ] **LIVE-04**: Under real nono, Liam submits a catalog-valid proposal to live olive with `trace_ref` set to hometown's `trace_id`, and records olive's status (REQ-LIAM-05/08)
- [ ] **LIVE-05**: popeye's ledger rows for the run's ATE `primary_request_id` show `principal_type=agent`, `agent_id=liam`, `on_behalf_of` = launching user, the launch purpose, and the agent record's `key_id` (REQ-LIAM-16; TC-E2E-05)

### End-to-End Evidence (POC-2)

- [ ] **E2E-01**: One real run exits 0, and its nono receipt, hometown ATE, popeye ledger row, olive proposal and Liam run record join through the §8.3 keys (REQ-LIAM-18; TC-E2E-01)
- [ ] **E2E-02**: A deliberate direct-connect attempt is blocked and produces one `endpoint-detection/0.1` event that reaches popeye (REQ-LIAM-14; TC-E2E-02)
- [ ] **E2E-03**: A deliberate gateway attempt is blocked by nono and detected with the gateway host as `destination`, and the run still completes through hometown (REQ-LIAM-17; TC-NET-04 real, TC-E2E-07)
- [ ] **E2E-04**: RESULTS.md records every E2E measurement with tag, Liam, nono and service commits, and limits. It states that the runs were on a dev machine, not the managed workstation the test spec names

## v2 Requirements

### Deferred

- **DEF-01**: DSSE record-integrity verification with `record_public_key` (TC-START-07). Waiting on `NONO_v0.4.3` O2
- **DEF-02**: A repeat of the TC-E2E suite on a managed workstation (`manual-hardware`) as the test spec describes
- **DEF-03**: Profile distribution through the launcher or Intune (O3)
- **DEF-04**: A retention policy for `runs/` (O4)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Any UI beyond the console | Spec §1.4 non-goal |
| Calling the gateway, choosing models, building prompts | hometown owns these (D12, v0.3 G5) |
| Executing actions; holding any write credential | DECISION §5.3; Liam only proposes |
| Multi-step autonomous loops | §1.4: one answer request, at most one proposal per run |
| Liam-side adaptation to a contract mismatch | AGENTS.md: a mismatch is a stop condition |
| Implementing or patching nono, hometown, olive or popeye | Liam is a consumer only |
| Any nono IPC, service call or profile edit | Contract-only integration decision |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONF-01 | Phase 1 | Complete |
| CONF-02 | Phase 1 | Complete |
| CONF-03 | Phase 1 | Complete |
| CONF-04 | Phase 1 | Pending |
| CONF-05 | Phase 1 | Pending |
| CONF-06 | Phase 1 | Pending |
| CONF-07 | Phase 1 | Pending |
| CONF-08 | Phase 1 | Pending |
| CONF-09 | Phase 1 | Complete |
| CONF-10 | Phase 1 | Pending |
| CONF-11 | Phase 1 | Pending |
| NONO-01 | Phase 2 | Pending |
| NONO-02 | Phase 2 | Pending |
| NONO-03 | Phase 2 | Pending |
| NONO-04 | Phase 2 | Pending |
| NONO-05 | Phase 2 | Pending |
| LIVE-01 | Phase 3 | Pending |
| LIVE-02 | Phase 3 | Pending |
| LIVE-03 | Phase 4 | Pending |
| LIVE-04 | Phase 4 | Pending |
| LIVE-05 | Phase 3 | Pending |
| E2E-01 | Phase 5 | Pending |
| E2E-02 | Phase 5 | Pending |
| E2E-03 | Phase 5 | Pending |
| E2E-04 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0 ✓

---
*Requirements defined: 2026-10-21*
*Last updated: 2026-10-21 after roadmap creation*
