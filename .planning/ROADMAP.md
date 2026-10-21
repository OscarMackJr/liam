# Roadmap: Liam

## Overview

Liam already has a working v0.4.0 core that has been verified against stubs. This roadmap first brings the code into line with the ratified v0.4.1 specs, still against stubs. Next, it starts Liam inside a real nono session on this Windows dev machine, connects it to live hometown (with popeye attribution) and then to live olive, and finishes with one joined end-to-end evidence run. Each phase that relies on a real external contract starts with a contract-diff checkpoint. A mismatch updates the contract table and README, and then work stops for human review. Liam never adapts to a mismatch.

## Phases

**Phase Numbering:**

- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: v0.4.1 Conformance** - Code, stubs, contracts and RESULTS.md brought into line with the ratified v0.4.1 specs, verified locally
- [ ] **Phase 2: Real nono Session** - nono contract-diff and O7 gates, then Liam passes the self-check and the refusal cases under the real nono fork
- [ ] **Phase 3: Live hometown and popeye** - hometown contract-diff gate, then a live answer through nono-proxy that popeye attributes to the agent
- [ ] **Phase 4: Live olive** - olive contract-diff gate, then a live proposal submission linked to hometown's trace
- [ ] **Phase 5: End-to-End Evidence** - One real run whose five records join, plus the blocked direct-connect and gateway traps, all recorded in RESULTS.md

## Phase Details

### Phase 1: v0.4.1 Conformance

**Goal**: Against stubs, Liam behaves exactly as the ratified v0.4.1 design and test specs require, and every TC that can run locally has a measured result
**Depends on**: Nothing (first phase)
**Requirements**: CONF-01, CONF-02, CONF-03, CONF-04, CONF-05, CONF-06, CONF-07, CONF-08, CONF-09, CONF-10, CONF-11
**Success Criteria** (what must be TRUE):

  1. In a stub run, the JCS SHA-256 of the four-key proposal olive receives equals the hash of hometown's proposal, including nested objects, non-ASCII text, fractional numbers and unordered keys. Command-like text and extra keys in an answer are treated as data and never executed.
  2. A hometown error exits 21 and an olive error exits 22. The reason (`hometown:<code>` / `olive:<code>`) comes from the `{"error": "<code>"}` body and falls back to the status. 429 is not retried, and a 5xx is retried once. A 200 `refused_policy` or idempotent replay is recorded and exits 0.
  3. A `--question` that is empty or longer than 4000 scalar values exits 11 `input:question_length`, with no request sent and no run record written.
  4. The stubs, the `contracts/` schemas, README, `Cargo.toml`, `liam.toml`, `agent_version` and the profile version all agree with v0.4.1 (with A1/A2/A5 status recorded). The automated TC-NET-03 static check and the local TC-NET-04 gateway-trap harness pass in `cargo test`. `liam.toml` pins the canonical vocabulary hash (`60a44f00…`) and TC-LVOC passes against it.
  5. RESULTS.md has a measured row (n of n, tag, Liam commit, limits) for every v0.4.1 TC that can run locally. TC-START-07 (DSSE) is marked deferred.

**Plans**: 12 plansPlans:
**Wave 1**

- [x] 01-01-PLAN.md — 0.4.1 labels, profile type, gitattributes, shared test helper
- [x] 01-02-PLAN.md — TC-NET-03 automated static scan
- [x] 01-03-PLAN.md — scripted stubs, routing proxy, gateway trap
- [x] 01-04-PLAN.md — catalog parameters_schema validator

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 01-05-PLAN.md — RawValue pass-through, guarded error codes, task validation

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 01-06-PLAN.md — evidence events/record, main.rs question check, retry, mapping, gateway hook

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 01-07-PLAN.md — ANS/FAIL/input process tests
- [x] 01-08-PLAN.md — JCS canonicaliser, ACT/SEC/NET-04 process tests
- [x] 01-09-PLAN.md — contracts, run-record schema, README, EVD tests
- [x] 01-10-PLAN.md — START/KEY/META/LVOC/BUILD coverage

**Wave 5** *(blocked on Wave 4 completion)*

- [ ] 01-11-PLAN.md — clean-tree measurement and per-TC RESULTS.md

**Wave 6** *(blocked on Wave 5 completion)*

- [ ] 01-12-PLAN.md — canonical vocabulary pin (blocked on user file)

### Phase 2: Real nono Session

**Goal**: Liam starts, self-checks and refuses correctly inside a real nono fork session on this dev machine, using a `token_arm` that the nono owner has signed off
**Depends on**: Phase 1
**Requirements**: NONO-01, NONO-02, NONO-03, NONO-04, NONO-05
**Success Criteria** (what must be TRUE):

  1. STOP GATE (start of phase): the nono fork's receipt, agent-session record and proxy environment are diffed against `contracts/receipt-fields.md`, §4.2 and A5. If they match, that is recorded. If any mismatch is found, the contract table and README are updated and work stops for review before anything below starts.
  2. STOP GATE (O7): the AppContainer `token_arm` and the §4.3 census are measured on `DirectCli` and recorded, and work stops for nono-owner review. `expected_token_arm` is set only after sign-off. Liam is never changed to accept an unreviewed value.
  3. Liam launched through `scripts/launch-under-nono.ps1` passes the startup self-check under the real fork. A receipt or agent record the user can write is refused (TC-START-06).
  4. Each refusal case is measured: a direct launch outside nono exits 10/11 with no network activity, a stopped WFP service gives a refused session or exit 10, and an unreachable key exchange makes nono refuse the session.
  5. Under a real session, Liam spawns zero child processes, touches only its file grants and makes zero DNS queries from its own PID.

**Plans**: TBD

### Phase 3: Live hometown and popeye

**Goal**: Under real nono, Liam gets a governed answer from live hometown through nono-proxy, and popeye's ledger attributes the call to Liam as an agent
**Depends on**: Phase 2
**Requirements**: LIVE-01, LIVE-02, LIVE-05
**Success Criteria** (what must be TRUE):

  1. STOP GATE (start of phase): the running hometown §2.6 server is diffed against S-HT-1 as specified in v0.4.1 §5. If it matches, that is recorded. If any mismatch is found, the contract table and README are updated and work stops for review before any live answer request is made.
  2. A real-nono run gets an answer from live hometown through nono-proxy. The captured request has no `Authorization` header from Liam, has the proxy-injected agent key, and carries `run_id` and `nono_session_id`.
  3. popeye's ledger rows for the run's ATE `primary_request_id` show `principal_type=agent`, `agent_id=liam`, the launching user as `on_behalf_of`, the launch purpose, and the agent record's `key_id`.

**Plans**: TBD

### Phase 4: Live olive

**Goal**: Under real nono, Liam submits hometown's catalog-valid proposal unchanged to live olive and records olive's verdict
**Depends on**: Phase 3
**Requirements**: LIVE-03, LIVE-04
**Success Criteria** (what must be TRUE):

  1. STOP GATE (start of phase): the published olive v1 proposal API is diffed against S-OL-1 and §7.2–§7.4. If it matches, that is recorded. If any mismatch is found, the contract table and README are updated and work stops for review before any live proposal is submitted.
  2. A real-nono run submits a catalog-valid proposal to live olive with `trace_ref` equal to hometown's `trace_id`. olive's returned status is recorded in the Liam run record.

**Plans**: TBD

### Phase 5: End-to-End Evidence

**Goal**: One real POC-2 run on this dev machine leaves content-free records that join end to end, and the deliberate bypass attempts are blocked and detected
**Depends on**: Phase 4
**Requirements**: E2E-01, E2E-02, E2E-03, E2E-04
**Success Criteria** (what must be TRUE):

  1. One real run exits 0, and its nono receipt, hometown ATE, popeye ledger row, olive proposal and Liam run record join through the §8.3 keys.
  2. A deliberate direct-connect attempt is blocked and produces exactly one `endpoint-detection/0.1` event that reaches popeye.
  3. A deliberate gateway attempt is blocked by nono and detected with the gateway host as `destination`, and the run still completes through hometown.
  4. RESULTS.md records every TC-E2E measurement with tag, Liam, nono and service commits, and limits. It states that the runs were on a dev machine, not the managed workstation the test spec names.

**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. v0.4.1 Conformance | 10/12 | In Progress|  |
| 2. Real nono Session | 0/TBD | Not started | - |
| 3. Live hometown and popeye | 0/TBD | Not started | - |
| 4. Live olive | 0/TBD | Not started | - |
| 5. End-to-End Evidence | 0/TBD | Not started | - |
