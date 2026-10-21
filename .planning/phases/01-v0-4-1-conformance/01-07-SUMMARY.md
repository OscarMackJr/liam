---
phase: 01-v0-4-1-conformance
plan: 07
subsystem: testing
tags: [conformance, process-tests, hometown, olive, retry, question-length]
requires: [01-03, 01-06]
provides:
  - "Named process tests tc_ans_* and tc_fail_* in tests/answer_contract.rs"
affects: [01-11]
tech-stack:
  added: []
  patterns: ["Case helper: ScriptedStub + TestProxy + run_liam, one-line stderr and content-free assertions"]
key-files:
  created: [tests/answer_contract.rs]
  modified: []
key-decisions:
  - "Long TC-FAIL-04 test asserts Liam output and two hometown hits only, because TestProxy records its own 30 s forward timeout as an error"
requirements-completed: [CONF-02, CONF-03, CONF-04]
duration: 25min
completed: 2026-10-21
---

# Phase 1 Plan 07: Answer and Failure Contract Tests Summary

24 named process tests (23 default, 1 ignored long) prove the hometown/olive error tables, retry-once rules, question-length boundaries and one-line stderr matrix against ScriptedStub and TestProxy.

## Tasks

1. Hometown, input and olive-failure TCs: commit 275a6b8. Covers TC-ANS-01..03, 05, 06, 07 and TC-FAIL-01..04, 06, 07, 08 (403 with and without code, 401 outcomes, 400/409/422 with and without codes, hostile bodies, 429 no retry, 5xx retry once, 503/504 coded retry, timeout retry with timeout_seconds = 2, olive 4xx/5xx/timeout with answer on stdout, question 0/1/4000/4001 including non-BMP).

## Verification

- `cargo test --locked --offline --features stubs --test answer_contract`: 23 passed, 1 ignored (about 10 s).
- `-- --ignored` (tc_fail_04_timeout_100s_real_timeout): passes, about 100 s wall time (not 190 s, see below).
- clippy `--all-targets --all-features -D warnings` and `cargo fmt --check` clean.

## Deviations from Plan

**1. [Rule 3 - Blocking] Long TC-FAIL-04 variant cannot use assert_failed**
- TestProxy forwards with reqwest's default 30 s timeout, so with a 100 s stub delay the proxy logs "forward send" errors and answers 502 before Liam's 90 s timeout fires. The test therefore asserts exit 21, exact stderr line, and two hometown hits, but not an empty proxy error list. Consequently it measures retry-once-then-21 within 190 s but does not exercise Liam's own 90 s timeout. Fixing would need a src/stubs.rs change (out of this plan's file list); recommended follow-up for plan 11 if the literal 90 s timeout evidence is required.

No product defects found.

## Known Stubs

None.

## Self-Check: PASSED

tests/answer_contract.rs present; commit 275a6b8 present.
