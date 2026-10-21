---
phase: 01-v0-4-1-conformance
plan: 11
subsystem: testing
tags: [conformance, results, measurement]
requires: [01-02, 01-05, 01-07, 01-08, 01-09, 01-10]
provides:
  - RESULTS.md with one row per v0.4.1 TC (CONF-10)
affects: [01-12 CONF-11]
key-files:
  modified: [RESULTS.md]
key-decisions:
  - "Measured on the main checkout with a clean git status at 05c83ef (code unchanged since 56922a0), not a fresh clone, because a fresh clone fails --locked --offline from a known vendor line-ending defect (out of scope)"
  - "TC-NET-07 and TC-FAIL-04 recorded as verified (partial); TC-LVOC-01/02 partial pending CONF-11"
requirements-completed: [CONF-10]
duration: 15min
completed: 2026-10-21
---

# Phase 1 Plan 11: Per-TC measurements Summary

RESULTS.md now has one row per TC (docs-only commit `docs(01): record v0.4.1 conformance measurements`), measured at code commit 05c83ef on Windows 10.0.26200.9711 (UBR 9711), tag `local development`.

## Measurements
- `cargo fmt --check`, clippy -D warnings (with and without `stubs`): clean.
- `cargo test --features stubs`: all pass (40 lib, 16 action, 24 answer, 9 hygiene, 8 process, 5 proxy, 10 record, 16 startup, 9 static; 1-2 ignored per suite).
- `-- --ignored`: TC-FAIL-04 long passed (100.78 s); TC-NET-07 HKCU test passed (HKCU write was allowed this session); `tc_lvoc_canonical_hash_pinned` FAILED (placeholder pin, plan 01-12 / CONF-11).
- `scripts/measure-build.ps1`: TC-BUILD-01 and TC-BUILD-02 PASS.

## Row status highlights
- not measured: TC-START-06, TC-NET-02, TC-NET-08, TC-SEC-01, TC-SEC-02, TC-FAIL-05, TC-E2E-01..07. Deferred: TC-START-07 (DEF-01).
- partial: TC-LVOC-01/02 (placeholder hash, blocked on 01-12), TC-NET-07 (ignored by default, mutates HKCU), TC-FAIL-04 (long variant bounded by TestProxy's 30 s forward timeout, so Liam's 90 s timeout path is not exercised).
- Post-plan fixes reflected: cdbfade (non-purpose 403 gives exit 21), e8a0ea3 ({id, status} layer entries), 56922a0 (TC-START-13 fully measured).
- Open spec question flagged in TC-ACT-03 row: bad action id reports `not_in_catalog`. TC-ACT-06 relies on assumption A3.
- TC-SEC-01 was not marked from `tc_sec_01_no_authorization_literal`/static checks: the spec requires a real nono process trace.

## Deviations from Plan
- Added `--no-fail-fast` second ignored run, because the failing CONF-11 test aborted cargo before later suites' ignored tests ran.
- n of n counts the named tests mapped to each TC (one execution each) rather than repeated runs.

## Self-Check: PASSED
RESULTS.md committed alone; no `uncommitted`/`not recorded` strings remain; TC-ACT rows: 9.

## Post-review re-measurement

Re-ran the measurements after the code review fixes (01-REVIEW-FIX.md) at the last code commit `d573907` (`git log --stat`: 5f3b0c3 and a0f6702 are docs only), on the main checkout with a clean `git status`, Windows 10.0.26200.9711 (UBR 9711). Not a fresh clone: the vendor LF/CRLF checksum defect is still open and `vendor/` was not touched.

- `cargo fmt --check`: clean.
- `cargo clippy --locked --offline --all-targets -- -D warnings`, with and without `--all-features`: clean.
- `cargo test --locked --offline --features stubs`: 166 passed, 0 failed, 2 ignored.
- `cargo test --locked --offline --features stubs --no-fail-fast -- --ignored`: 2 passed, 0 failed (TC-FAIL-04 long 100.72 s; TC-NET-07 HKCU test, settings restored on Drop).
- `cargo test --locked --offline` (default features): 56 passed, 0 failed.
- `scripts/measure-build.ps1`: TC-BUILD-01 PASS, TC-BUILD-02 PASS (same DLL set).

RESULTS.md (commit 7ae7135): all 58 measured rows now cite `d573907`. This includes TC-LVOC-01/02, which were at `f51c2fe`. Review-fix tests were added to the mappings: TC-NET-03 (redirect policy needle), TC-NET-04 4 of 4 (CR-01 redirect test), TC-ANS-05 6 of 6 (WR-07, CR-02 trace_id), TC-ACT-02 (WR-03 suggestion text), TC-ACT-03 (WR-01 note), TC-ACT-08 3 of 3 (CR-02 status and proposal_id), TC-EVD-01 4 of 4 (WR-02 events, schema enum and id pattern), TC-EVD-05 (WR-09 note) and TC-SEC-03 3 of 3 (WR-10). The WR-04/WR-08 startup tests pass but map to no TC row, which the Notes record. No status changed. The not-measured and deferred rows and the WR-05/WR-06 notes are unchanged.
