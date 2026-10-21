---
phase: 01-v0-4-1-conformance
plan: 09
subsystem: contracts-evidence
tags: [conformance, contracts, run-record, evidence, readme]
requires: [01-06]
provides:
  - "S-HT-1 / S-OL-1 contract JSON at v0.4.1 shapes"
  - "contracts/run-record.schema.json (additionalProperties false, proposal.reason, optional-null answer)"
  - "README v0.4.1 conformance notes (assumptions, deviations, harness limits)"
  - "tests/record_contract.rs with tc_evd_01..05"
affects: [01-11]
tech-stack:
  added: []
  patterns: ["Hand-rolled JSON-schema subset checker in the test (type, pattern, required, additionalProperties, items, oneOf)"]
key-files:
  created: [contracts/run-record.schema.json, tests/record_contract.rs]
  modified: [contracts/hometown-answer.S-HT-1.json, contracts/olive-proposal.S-OL-1.json, contracts/catalog-0.4.json, README.md]
key-decisions:
  - "run-record schema added; it does not touch spec section 4, so no change-control conflict and TC-EVD-01 is measured, not partial"
  - "README labels the plan's A3/A4/A5/A7 as 'conformance A#' because design spec section 13 already uses A3/A4/A5 for different assumptions"
requirements-completed: [CONF-05]
duration: 25min
completed: 2026-10-21
---

# Phase 1 Plan 09: Contracts, Run-Record Schema and Evidence TCs Summary

Both wire-contract JSON files, the stub catalog, a new additionalProperties-false run-record schema and the README now state the v0.4.1 shapes and every assumption, and 9 tests in tests/record_contract.rs measure TC-EVD-01..05 against that schema.

## Tasks

1. Contracts, schema, README: commit 456d973.
2. Evidence TCs: commit 704389b. Tests: tc_evd_01 (completed record valid, session id, log_sha256 equals SHA-256 of .jsonl, ends run.completed; refused_policy reason string; checker rejects extra/missing keys), tc_evd_02 (canary only on stdout), tc_evd_03 (party_ref_hash only), tc_evd_04 (hometown 5xx twice -> exit 21, outcome hometown_unavailable, log ends run.failed; olive failure keeps answer ids), tc_evd_05 (bad receipt path -> exit 10, no files, no network), plus a hostile-string event_with test. Exit 30 stays covered in process_contract.

## Deviations from Plan

- [Rule 1 - Doc accuracy] While writing README the plan's "A4: 403 without usable body -> hometown:403" was checked against code: any 403 is a purpose refusal (exit 20, reason falls back to hometown:403), not exit 21. README states the actual behaviour.
- Test-side fix: in the olive-failure test the answer text is legitimately printed on stdout, so the canary check allows stdout there (record, log and stderr stay canary-free).

## Findings for owner review (not silently adapted)

- Contract mismatch (flagged in README): `nono.layers_outside_expectancy_scope` is serialized as `[id, status]` pairs (Rust tuples); design spec 8.1 sketches `{id, status}` objects. The schema describes the actual output. Changing it needs a src/evidence.rs change, which this plan does not cover.
- Spec A5 (proxy URL form) stays unverified until Phase 2.

## Known Stubs

None.

## Verification

- All contract JSON parses; vocabulary hash text untouched (README hash count unchanged at 2).
- clippy `--all-targets --all-features -D warnings`, `cargo fmt --check`, `cargo test --locked --offline --features stubs` all green; vendor/ untouched.

## Self-Check: PASSED
