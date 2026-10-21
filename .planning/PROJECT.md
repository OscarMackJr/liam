# Liam

## What This Is

Liam is a Windows desktop agent in the Fiskroad program. It is a single-shot Rust CLI that runs confined under nono and performs one bounded task for a banker: it asks hometown for a governed answer about a party and, if the answer carries a catalog-valid proposal, submits that proposal unchanged to olive. Its POC value is the evidence chain it exercises from the endpoint side (nono receipt, hometown ATE, popeye ledger row, olive proposal, Liam run record), not the task itself.

## Core Value

One Liam run, started only inside an enforced nono session, leaves content-free records that join end to end. It holds no credential, reaches nothing but the loopback nono-proxy and executes nothing.

## Requirements

### Validated

<!-- Existing capabilities at commit 50b1659 (v0.4.0 + D14), verified locally against stubs. See RESULTS.md. -->

- ✓ Startup self-check of the nono receipt (schema 2) and agent-session record (schema 1), with Liam-owned 14-layer expectancy table (REQ-LIAM-01, partial: no ACL/real-nono harness) — existing
- ✓ Proxy environment required and validated: `http://127.0.0.1:<port>` in both vars, empty `NO_PROXY` (REQ-LIAM-02) — existing
- ✓ D14 transport: single reqwest client, `no_proxy()` + explicit proxy, `DenyAllResolver`, no Liam-side allowlist (REQ-LIAM-03, partial) — existing
- ✓ No `Authorization` header; proxy-side key injection; key failure fails without retry (REQ-LIAM-04, partial) — existing
- ✓ `run_id` / `nono_session_id` / `trace_ref` metadata on requests (REQ-LIAM-05) — existing
- ✓ Purpose validated against vendored vocabulary with hash check (REQ-LIAM-06) — existing
- ✓ One answer request, at most one proposal, catalog/schema check before olive (REQ-LIAM-08/09, partial) — existing
- ✓ Content-free run record + run log, failed startup leaves no evidence (REQ-LIAM-10, partial) — existing
- ✓ Retry exactly once on hometown 5xx/timeout, then exit 21 (TC-FAIL-01, partial) — existing
- ✓ Stubs behind the `stubs` cargo feature; static-CRT release build (REQ-LIAM-15, partial) — existing

### Active

<!-- Milestone A: v0.4.1 conformance (local, stubs) -->

- [ ] D15 / REQ-LIAM-19: proposal forwarded to olive exactly as hometown returned it, with no coercion, defaulting, stripping or normalising
- [ ] D16 / REQ-LIAM-20: hometown and olive exit reasons come from the `{"error": "<code>"}` body, falling back to status (exit 21/22 per §5.4/§7.4)
- [ ] D17 / REQ-LIAM-21: `--question` outside 1..4000 chars → exit 11 `input:question_length` before any request
- [ ] S-HT-1 and S-OL-1 stubs, contracts/ schemas and README updated to the v0.4.1 §5/§7 shapes (four-key proposal, new error codes, olive v1 responses)
- [ ] Version labels move to 0.4.1 (Cargo.toml, liam.toml, evidence); profile version type/value reconciled with what Liam validates
- [ ] TC-NET-03 static check automated (no `std::net`, `TcpStream`, `ToSocketAddrs`, second client, gateway refs outside stubs)
- [ ] Every v0.4.1 TC Liam can satisfy locally is measured and recorded in RESULTS.md

<!-- Milestone B: POC-2 end to end on this dev machine -->

- [ ] O7: measure the AppContainer `token_arm` under the real nono fork and diff the §4.3 expectancy table, then stop for nono-owner review
- [ ] Liam starts and passes the self-check under the real nono fork (receipt, agent record, proxy env, AppContainer), including the ACL non-writable check (TC-START-06)
- [ ] Contract-diff checkpoint against the real hometown §2.6 server, then live answer integration
- [ ] Contract-diff checkpoint against the published olive v1 proposal API, then live proposal integration
- [ ] popeye agent key exchange in the loop: ledger attributes the run as `principal_type=agent`, `agent_id=liam`, with the agent record's `key_id` (REQ-LIAM-16)
- [ ] Gateway trap: a deliberate gateway attempt is blocked by nono and detected (`endpoint-detection/0.1`) (REQ-LIAM-14/17, TC-NET-04/08, TC-E2E-02/07)
- [ ] One real run whose records join via §8.3 keys: receipt ↔ ATE ↔ ledger ↔ olive proposal ↔ run record (REQ-LIAM-18, TC-E2E-01..07 measured)

### Out of Scope

- DSSE record-integrity verification (TC-START-07): stays a documented fail-closed deferral. The integrity mechanism is still open (`NONO_v0.4.3` O2)
- Any UI beyond the console; multi-step autonomous loops (spec §1.4 non-goals)
- Calling the model gateway, choosing models or building prompts: those belong to hometown
- Executing actions or holding any write credential (DECISION §5.3)
- Liam-side workarounds for contract mismatches. A mismatch is a stop condition, never an adaptation
- Implementing nono, hometown, olive or popeye. Liam only consumes them
- Managed-endpoint / Intune distribution (O3) and `runs/` retention policy (O4)

## Context

- Brownfield: ~1.9k lines Rust in `src/`, ~760 in `tests/`; codebase map in `.planning/codebase/` (CONCERNS.md lists the v0.4.1 gaps)
- Governing specs (committed 408a3d0): `specs/LIAM_DESIGN_SPEC_v0.4.1.md`, `specs/LIAM_TEST_SPEC_v0.4.1.md`, `specs/LIAM_AMENDMENT_v0.4.1_proxy-transport.md`
- Wire contracts S-HT-1 / S-OL-1 are owner-authorized stubs; `HOMETOWN_v0.4` §2.6 is agreed but not ratified at the ATE v0.4 joint review (A1); olive v1 wire contract not yet reviewed (A2)
- A3/A4 (popeye, nono implementations) are ratified specs, and the user reports builds of nono fork, hometown §2.6, olive v1 and popeye key exchange are available or soon available
- POC-2 E2E runs on this Windows dev machine, with all services local
- `expected_token_arm` ships empty until O7 closes, so only stub-mode runs start today
- RESULTS.md records measurements only, never extrapolations

## Constraints

- **Security**: No secrets, keys or credential-shaped values anywhere (code, config, env handling, logs, tests, fixtures). Why: nono-proxy injects the key and Liam never sees it (AGENTS.md)
- **Network**: Only the validated IPv4 loopback nono-proxy endpoint; one explicit client, deny-all resolver, no Liam-side allowlist. Why: D14
- **Execution**: Never calls the gateway, never executes target-system actions or model output, never spawns child processes. Why: DECISION §5.3, REQ-LIAM-11/12
- **nono coupling**: Reads only the receipt and agent-session record. No IPC, services, profile edits or other state. Why: contract-only integration decision
- **Evidence**: Content-free. No prompts, answers, context values, party refs or query-bearing URLs in logs, records or stderr. Why: REQ-LIAM-10
- **Fail closed**: A failed prerequisite prints exactly one reason line and exits with its documented code before any network request. Why: REQ-LIAM-13
- **Change control**: A contract mismatch stops work. Update the contract table and README, then stop for review. Changes to §4 need re-ratification with `NONO_v0.4.3`
- **Tech stack**: Rust, single static binary, stubs only behind the `stubs` feature and excluded from release; CI on windows-latest

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Bring code to ratified v0.4.1 first, then POC-2 E2E | Local conformance against stubs de-risks real-service integration | — Pending |
| POC-2 "done" = one real E2E run with joining records (TC-E2E-01..07 measured) | That is the POC goal (§1.2); passing against stubs alone does not demonstrate it | — Pending |
| Keep DSSE deferred (fail closed on `record_public_key`) | Integrity mechanism still open upstream (`NONO_v0.4.3` O2) | — Pending |
| Commit v0.4.1 specs before code changes | Ratified text in git before code is changed against it | ✓ Done (408a3d0) |
| O7: measure token_arm under real nono, then stop for nono-owner review | AGENTS.md stop rule: Liam doesn't guess the value | — Pending |
| Contract-diff checkpoint phase per real service (hometown, olive, nono) | Mismatches are stop conditions and need explicit review gates | — Pending |
| E2E on this dev machine with all services local | Available environment; managed-endpoint run not required for POC-2 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check: still the right priority?
3. Audit Out of Scope: are the reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-10-21 after initialization*
