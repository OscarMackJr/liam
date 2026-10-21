---
phase: 01-v0-4-1-conformance
plan: 10
subsystem: testing
tags: [rust, conformance, startup, hygiene, build-scan]
requires: [01-01, 01-03, 01-06]
provides:
  - TC-START mutation tests, TC-KEY/META/LVOC/NET-05 tests, release-build scan script
affects: [01-11 RESULTS, 01-12 CONF-11]
tech-stack:
  added: []
  patterns: [on-disk mutation of StubRecordPair, named-allow-list credential-shape scanner]
key-files:
  created: [tests/startup_contract.rs, tests/hygiene_contract.rs, scripts/measure-build.ps1]
  modified: []
key-decisions:
  - "TC-KEY-03 scanner skips digit-only hex runs and rustc commit hashes in std paths; 64-hex values allowed only when pinned or in a sha256/party_ref_hash field"
  - "Numeric key_id case kept as an ignored test asserting the spec reason, because the product reports a different one"
requirements-completed: [CONF-10, CONF-11]
duration: 30min
completed: 2026-10-21
---

# Phase 1 Plan 10: Startup, hygiene and build measurement Summary

Named tests for TC-START-01..05/08..16, TC-KEY-01..03, TC-META-01/02, TC-LVOC-01/02, TC-NET-05 (all 7 cases) and a release-build scan script for TC-BUILD-01/02.

## Commits
- ac1ff28 tests/startup_contract.rs (15 tests + 1 ignored)
- 7e46cda tests/hygiene_contract.rs (9 tests + 1 ignored)
- scripts/measure-build.ps1 commit, then a clippy-fix commit (see git log)

## Coverage map for RESULTS
- TC-START-01..05, 08..16: tc_start_* in tests/startup_contract.rs (header map). TC-START-05 "empty expected arm outside stub mode" is measured by calling validate_startup with an empty policy, since stub mode forces the arm.
- TC-KEY-01..03, TC-META-01/02, TC-LVOC-01/02, TC-NET-05: tests/hygiene_contract.rs.
- TC-NET-01 -> process_contract successful run; TC-NET-06 -> proxy_contract resolver_calls()==0 plus client unit tests; TC-NET-07 -> proxy_contract HKCU test (#[ignore], row stays partial). No duplicate added.
- TC-BUILD-01/02: scripts/measure-build.ps1 run: PASS; imports are advapi32, api-ms-win-core-synch-l1-2-0, bcrypt, bcryptprimitives, crypt32, KERNEL32, ntdll, WS2_32 (no VCRUNTIME/UCRT).
- Not measured: TC-START-06, TC-FAIL-05, TC-NET-02, TC-NET-08, TC-SEC-01/02, TC-E2E-*. Deferred: TC-START-07 (DEF-01).

## Deviations from Plan
None to scope. Test-only lint allowances added (clippy permissions_set_readonly_false, type_complexity) to keep the CI gate green.

## Product defect found (reported, not fixed; src is out of scope)
**TC-START-13 `keyid-numeric`:** spec fixture (JSON number key_id) must give `agent_record:key_id`, but Liam exits `10 agent_record:parse` because the typed deserialize in src/receipt.rs fails first. Test `tc_start_13_numeric_key_id_spec_reason` asserts the spec reason and is `#[ignore]`d with that explanation. Fix (accept the field as a generic value and fail with `key_id`) belongs to a src change; the RESULTS row for TC-START-13 should be partial until then. The uppercase-UUID and key-expiry cases pass.

## Notes
- LVOC tests measure the working-tree vocabulary.yaml against liam.toml's pin (not a literal), so they pass before and after CONF-11. `tc_lvoc_canonical_hash_pinned` is `#[ignore = "blocked on CONF-11"]`; plan 01-12 must remove the ignore and update `common::VOCAB_SHA256`. The known missing `-text` attribute on vocabulary.yaml means a fresh checkout could change the hash; not fixed here per instructions.
- TC-KEY-03 does not open the repo-root `.env`.
- measure-build.ps1 does not depend on vendor CRLF state beyond what `cargo build --locked --offline` already does.

## Self-Check: PASSED
Gates run green: cargo fmt --check, clippy --all-targets --all-features -D warnings, cargo test --features stubs.

## Post-plan fixes

- `56922a0` fix(01-10): `key_id` in the agent-session record is now deserialized untyped, so a JSON-number `key_id` fails with `agent_record:key_id` (spec reason) instead of `agent_record:parse`. `tc_start_13_numeric_key_id_spec_reason` is un-ignored and passes; TC-START-13 is fully measured.
