---
gsd_state_version: 1.0
milestone: v0.4.1
milestone_name: Conformance
status: executing
last_updated: "2026-10-22T16:17:05.658Z"
last_activity: 2026-10-22
progress:
  total_phases: 6
  completed_phases: 1
  total_plans: 26
  completed_plans: 19
  percent: 17
stopped_at: Phase 01 complete (12/12) — ready to discuss Phase 2
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-10-21)

**Core value:** One Liam run, started only inside an enforced nono session, leaves content-free records that join end to end. It holds no credential, reaches nothing but the loopback nono-proxy and executes nothing.
**Current focus:** Phase 02.1 — liam-service-url-transport-amendment-v0-4-2

## Current Position

Phase: 02.1 (liam-service-url-transport-amendment-v0-4-2) — EXECUTING
Plan: 5 of 6
Status: Ready to execute
Last activity: 2026-10-22

Progress: [███████░░░] 73%

## Performance Metrics

**Velocity:**

- Total plans completed: 12
- Average duration: -
- Total execution time: 0.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 12 | - | - |

**Recent Trend:**

- Last 5 plans: -
- Trend: -

*Updated after each plan completion*
| Phase 01 P06 | 20min | 2 tasks | 3 files |
| Phase 01 P07 | 25min | 1 tasks | 1 files |
| Phase 01 P08 | 30min | 2 tasks | 2 files |
| Phase 01 P09 | 25min | 2 tasks | 6 files |
| Phase 01 P10 | 30min | 3 tasks | 3 files |
| Phase 01 P11 | 15min | 2 tasks | 1 files |
| Phase 01 P12 | 10min | 2 tasks | 8 files |
| Phase 02.1 P01 | 5min | 2 tasks | 1 files |

## Accumulated Context

### Roadmap Evolution

- Phase 2.1 inserted after Phase 2: Implement Liam amendment v0.4.2 service-URL transport (nono 126.1 replaces CONNECT injection with loopback /svc/<service_id> routes); execution gated on ratification (URGENT)

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 5 coarse phases: conformance (stubs) → real nono → live hometown + popeye → live olive → joined E2E
- [Roadmap]: Contract-diff stop gates open the phase that depends on them: NONO-01 and NONO-02/O7 (Phase 2), LIVE-01 (Phase 3), LIVE-03 (Phase 4). A mismatch updates the contract table + README and halts for human review
- [Roadmap]: DSSE (TC-START-07) remains deferred (DEF-01); Phase 1 records it as deferred in RESULTS.md
- [Phase 01]: 01-06: hometown retry via RetryOnce adapter; gateway event name const in stubs.rs
- [Phase 01]: 01-07: long TC-FAIL-04 asserts Liam output only; TestProxy 30s forward timeout limits real 90s coverage
- [Phase 01]: 01-08: JCS test digits via std {:e}; bad-id proposals report not_in_catalog since catalog lookup precedes format check
- [Phase 01]: 01-09: run-record schema added (no change-control conflict); layers_outside_expectancy_scope serialized as pairs vs spec 8.1 objects flagged for owner
- [Phase 01]: Only 403 purpose_refused maps to exit 20; other 403 exit 21. Layer entries serialized as {id,status} objects (spec 8.1)
- [Phase 01]: 01-10: numeric key_id reports agent_record:parse vs spec key_id (defect, test ignored); TC-KEY-03 scanner has named exceptions
- [Phase 01]: 01-11: RESULTS measured on main checkout at 05c83ef (clean status), not a fresh clone, due to known vendor CRLF defect
- [Phase 01]: Vendored canonical vocabulary with contracts/vendored/** -text; pinned 60a44f00
- [Phase 02]: 02-02: nono services are a dev registration from the build tree (no MSI); reinstall = stop, build+sign, start
- [Phase 02]: 02-02: nono pin re-pinned 4df47f0f to cf3ebc56e1074980236f89100145c86aa3f04928 (phase-126 round 4 complete, v0.70.0)
- [Phase 02.1]: [02.1-01] Amendment v0.4.2 ratified 2026-10-22; O-B no injected_on check

### Pending Todos

None yet.

### Blockers/Concerns

- Phase 2: `expected_token_arm` can only be set after nono-owner sign-off on O7. Until then, every non-stub run exits `10 receipt:token_arm`.
- Phases 2–4: these need builds of the nono fork, hometown §2.6, olive v1 and popeye key exchange on this machine. The user reports they are available or soon available.
- Resolved in Phase 1 (01-12): `liam.toml` pins the canonical vocabulary hash `60a44f00…`, sourced from `fiskroad/contracts/vocabulary.yaml`.
- Phase 2 STOPPED at gate 1 (02-03, 2026-10-22): nono cf3ebc56 does not emit Liam's contract (schema 1 receipt, 13 layers, no AgentKeyInjection, no AgentSessionRecord, no NONO_RECEIPT_PATH/NONO_AGENT_RECORD_PATH, no --purpose). Resume needs nono work + re-pin + re-run 02-03, or §4 re-ratification with NONO_v0.4.3. Plans 02-04..02-08 not run.
- Resume trigger for Phase 2: nono Phase 126.1 (Agent-Key Session Contract, nono ROADMAP 87279a8b) delivers a commit with REQ-END-18..25 passing; the nono session will message liam-ed the hash. See 02-NONO-WORK-BRIEF.md.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 261021-qrq | Fix vendor line endings so a fresh clone builds with `--locked --offline` | 2026-10-21 | 735d925 | [261021-qrq-fix-vendor-line-endings](./quick/261021-qrq-fix-vendor-line-endings/) |

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| Integrity | DSSE record verification (DEF-01, TC-START-07) | Waiting on NONO_v0.4.3 O2 | Init |
| Validation | Managed-workstation TC-E2E repeat (DEF-02) | v2 | Init |
| Integrity | WR-05 effective write access (read-only attribute accepted) and WR-06 DSSE envelope not detected | Waiting on NONO_v0.4.3 re-ratification (§4) | Phase 1 review |

## Session Continuity

Last session: 2026-10-22T16:17:05.642Z
Stopped at: Phase 2 context gathered
Resume file: None
