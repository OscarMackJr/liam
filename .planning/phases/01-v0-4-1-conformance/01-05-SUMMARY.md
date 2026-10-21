---
phase: 01-v0-4-1-conformance
plan: 05
subsystem: clients-task
tags: [rawvalue, passthrough, error-codes, CONF-01, CONF-02, CONF-03, CONF-09]
requires: [catalog::validate_parameters]
provides: [clients::error_code, clients::reason, HometownError::outcome, TaskError::Olive carrying answer]
affects: [plan 06 (main.rs retry/events/evidence)]
tech-stack:
  added: []
  patterns: [serde_json RawValue byte forwarding, pure classify(status, body) fns]
key-files:
  created: []
  modified: [src/clients/mod.rs, src/clients/hometown.rs, src/clients/olive.rs, src/task.rs, src/main.rs, contracts/catalog-0.4.json, tests/proxy_contract.rs]
decisions:
  - "Status alone decides class; body only supplies a guarded reason code"
  - "Added TaskError::Catalog for catalog parse/version failure (previously mis-mapped to Olive); main exits 22 olive:catalog"
metrics:
  tasks: 2
  files: 7
completed: 2026-10-21
---

# Phase 1 Plan 05: Pass-through clients and strict task validation Summary

Hometown's proposal is held as RawValue and its four fields are forwarded to olive byte for byte; validation runs on a separate strict four-key copy using catalog.rs; error reasons come only from a guarded `^[a-z][a-z0-9_]{0,63}$` code or the numeric status.

## Commits
- d751199: clients (error_code/reason, hometown/olive classify, RawValue ProposalRequest, 200/202)
- a727bb0: task.rs raw validation, TaskError::Olive with answer, main.rs compile adaptation, catalog `required_parameters` removed, proxy_contract migrated

## Notes
- Verification: `cargo test --locked --offline --features stubs` passes (38 lib, 8 process, 5 proxy, 9 static); `cargo clippy --features stubs -- -D warnings` clean for lib/bin.
- Manual check: no `to_value`/`Value::from` builds the olive request; olive body is built from `RawFour` borrowed RawValues.
- main.rs: only compile-level mapping (PurposeRefused -> 20, Rejected/Unavailable/Invalid -> 21, olive failure prints answer then exit 22 with Rejected reason or `olive:unavailable`). Retry still only on `Unavailable` (so unparseable 200 is not retried, A5). Retry restructure, events and evidence are plan 06.
- Assumptions for review: A3 (array = first used, rest counted in `limit_suppressed`; not yet surfaced in main), A4 (403 without usable body -> `hometown:403`), A5.

## Deviations from Plan
- [Rule 3] Task 1 alone cannot compile (task.rs depended on removed `Proposal`), so the two commits are sequential and only the second builds. Added `TaskError::Catalog` variant (not in plan) to avoid mis-reporting catalog failures as olive errors.
- Out of scope, not fixed: `cargo clippy --all-targets` reports 14 needless_borrow errors in pre-existing tests/process_contract.rs.

## Known Stubs
None.

## Self-Check: PASSED
Commits d751199 and a727bb0 exist; all modified files present.
