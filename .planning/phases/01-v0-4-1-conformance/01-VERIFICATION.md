---
phase: 01-v0-4-1-conformance
verified: 2026-10-21
status: human_needed
score: 5/5 roadmap criteria verified; 11/11 requirements satisfied in code
overrides_applied: 0
re_verification: false
gaps: []
deferred:
  - truth: "WR-05 effective-write-access (ACL-writable file with read-only attribute accepted); TC-START-06 not measured"
    addressed_in: "NONO_v0.4.3 re-ratification; Phase 2 criterion 3 (TC-START-06 under the real fork)"
    evidence: "User-accepted deferral. README 'Known gaps (deferred to §4 re-ratification)'; RESULTS TC-START-06 row 'not measured'."
  - truth: "WR-06 DSSE envelope not detected, integrity always recorded acl-only; TC-START-07 deferred"
    addressed_in: "NONO_v0.4.3 O2 re-ratification (DEF-01)"
    evidence: "README 'Known gaps'; RESULTS TC-START-01 and TC-START-07 rows."
human_verification:
  - test: "TC-ACT-03: confirm a malformed action id should report not_in_catalog"
    expected: "A decision on whether the catalog lookup should precede the id-format check. RESULTS flags this as an open spec question."
    why_human: "Spec interpretation, not something a test can settle."
  - test: "TC-ACT-06: confirm assumption A3 (multiple proposals arrive as an array)"
    expected: "The array shape is confirmed against S-HT-1, or corrected."
    why_human: "Contract assumption owned by the hometown owner."
  - test: "WR-03: review the printed suggestion line 'Suggestion (not submitted): <action_type>: <justification>'"
    expected: "The wording is acceptable and the suggestion is shown as text only."
    why_human: "Wording is a UX judgement."
  - test: "WR-09: review behaviour when stdout is closed"
    expected: "The run still writes its record and exits with its documented code (closed_stdout_still_writes_evidence_and_exit_code passes); silently dropping the answer is acceptable."
    why_human: "Policy decision; the spec gives no exit code for a broken stdout."
---

# Phase 1: v0.4.1 Conformance Verification Report

**Phase goal:** Against stubs, Liam behaves exactly as the ratified v0.4.1 design and test specs require, and every TC that can run locally has a measured result.
**Status:** human_needed (no blockers)

## What was run

- `cargo test --locked --offline --features stubs`: 166 passed, 0 failed, 2 ignored (lib 56, action 18, answer 30 with 1 ignored, hygiene 10, process 8, proxy 5 with 1 ignored, record 11, startup 19, static 9). The ignored tests are `tc_fail_04_timeout_100s_real_timeout` (long) and the HKCU system-proxy test (mutates the registry); both passed in the RESULTS re-measurement.
- `cargo fmt --check`: clean.
- `cargo clippy --locked --offline --all-targets --all-features -- -D warnings`: clean.
- Debt-marker grep (TBD, FIXME, XXX) over src, tests, scripts, README and liam.toml: none.

## Roadmap criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | JCS pass-through of the four-key proposal; command text and extra keys treated as data | VERIFIED | TC-ACT-07 and TC-SEC-03 pass (action and hygiene suites). |
| 2 | Hometown errors exit 21, olive errors exit 22; 429 not retried, 5xx retried once; refused_policy and replay exit 0 | VERIFIED | TC-ANS-05, TC-ANS-07, TC-FAIL-01..03, TC-FAIL-06, TC-FAIL-08, TC-ACT-08, TC-ACT-09 pass. Post-plan fix cdbfade (non-purpose_refused 403 exits 21) is covered by TC-ANS-03. |
| 3 | Empty or >4000-scalar question exits 11 `input:question_length`, no request, no record | VERIFIED | TC-ANS-06 (0, 1, 4000, 4001) passes. |
| 4 | Labels and contracts agree with 0.4.1; TC-NET-03 and TC-NET-04 pass in cargo test; canonical vocabulary hash pinned | VERIFIED | Cargo.toml and liam.toml read 0.4.1; profile_version 4; agent_version "0.4.1" in evidence tests; liam.toml pins 60a44f00db0daafc..., matching `sha256sum contracts/vendored/vocabulary.yaml`; README records A1/A2/A5 status; static suite and TC-NET-04 process tests pass. |
| 5 | RESULTS.md has a measured row for every locally runnable TC; TC-START-07 deferred | VERIFIED | Measured rows carry n of n, tag, Liam commit d573907 and limits; rows needing a real session or the ACL harness say "not run"/"not measured"; TC-START-07 is "deferred (DEF-01)". |

## Requirements coverage

All 11 IDs (CONF-01..CONF-11) appear in PLAN frontmatter and have implementation evidence. No orphaned requirements. Bookkeeping fix applied with this report: CONF-06 and CONF-07 were still unticked in REQUIREMENTS.md although their code and tests are complete.

## Deferrals and overclaiming

- WR-05 and WR-06 are documented under README "Known gaps"; RESULTS does not overclaim (TC-START-06 not measured, TC-START-07 deferred, TC-START-01 limits name both gaps).
- Other "not measured" TCs (TC-NET-02, TC-NET-08, TC-SEC-01, TC-SEC-02, TC-FAIL-05, TC-E2E-01..07) need a real nono session or the ACL harness.
- TC-NET-07 and TC-FAIL-04 are "verified (partial)" with stated limits; TC-FAIL-04's long variant is bounded by TestProxy's 30 s forward timeout, so Liam's own 90 s path is not exercised.

## Vendor LF/CRLF defect (WARNING)

Committed vendor/ blobs are LF while `.cargo-checksum.json` expects CRLF bytes, so a fresh clone (including CI on windows-latest) fails `--locked --offline`. Measurements were taken on the main checkout with a clean `git status`, so they stand, but the CI half of criterion 4 and the TC-BUILD claims are not repeatable until fixed. Scheduled as a separate quick task; re-measure on a fresh clone afterwards.

## Summary

Phase goal achieved against stubs. No goal-blocking gaps. Remaining: four human-verification items, and the scheduled vendor fix.
