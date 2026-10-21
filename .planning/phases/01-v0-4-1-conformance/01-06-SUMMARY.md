---
phase: 01-v0-4-1-conformance
plan: 06
subsystem: evidence-orchestration
tags: [rust, evidence, main, retry, events]
requires: ["01-02", "01-05"]
provides: [serde-built run events, proposal.reason in run record, question length check, hometown-only retry, gateway hook under stubs]
affects: [src/evidence.rs, src/main.rs, src/stubs.rs]
key-files:
  modified: [src/evidence.rs, src/main.rs, src/stubs.rs]
decisions:
  - "Retry implemented as a RetryOnce HometownClient adapter in main.rs so only hometown.answer is retried and task::run is called once"
  - "Gateway event name lives in stubs.rs (GATEWAY_EVENT) because the static scan forbids a gateway. literal in non-stub sources"
  - "TaskError::Catalog now goes through the common mapping and writes a failed record (exit 22 olive:catalog unchanged)"
metrics:
  completed: 2026-10-21
  tasks: 2
  files: 3
---

# Phase 01 Plan 06: Evidence events and main orchestration Summary

Serde-built content-free run events, `proposal.reason` in the run record, the 1..=4000 scalar question check before any I/O, a hometown-only single retry, one TaskError mapping, and a stubs-only gateway attempt hook.

## Tasks

1. Evidence (056f1fe): `event_with(name, fields)` built with serde_json; `ProposalOutcome {proposal_id, status, reason}` with reason null unless set and guarded via `guard_code`; test literal now 0.4.1; tests for hostile field JSON validity and reason null/non-null.
2. main.rs (0831a58): `args_os` with non-UTF-8 -> `liam: 11 args:required`; question check first (`input:question_length`); `RetryOnce` adapter; single `map_error`; answer printed before olive failure and `answer{trace_id, context_complete}` recorded; events `proposal.suppressed{reason}` (plus `limit` per extra), `proposal.submitted{http_status}`, `proposal.refused{reason, investigate}`; `#[cfg(feature = "stubs")]` gateway attempt logging `gateway.attempt{outcome}`.

## Verification

- `cargo test --locked --offline --features stubs`: all pass (40 lib, 8 process, 5 proxy, 9 static).
- `cargo clippy --locked --offline --all-targets --all-features -- -D warnings`, clippy without features, `cargo fmt --check`: clean.
- Release build succeeds. `liam.exe --purpose servicing --party p --question ""` printed `liam: 11 input:question_length`, exit 11.

## Deviations from Plan

**[Rule 3 - Blocking] Gateway event literal.** `tests/static_contract.rs` (tc_ans_04) rejects the `gateway.` string in non-stub sources, so the event name constant was added to `src/stubs.rs` (`GATEWAY_EVENT`) and referenced from main.rs. Files: src/stubs.rs, src/main.rs.

**Catalog failure now writes a failed run record** via the common mapping (previously exited immediately). Exit code and reason unchanged.

## Known Stubs

None.

## Self-Check: PASSED
