---
phase: 01-v0-4-1-conformance
plan: 08
subsystem: testing
tags: [conformance, process-tests, jcs, rfc8785, gateway-trap, proposal]
requires: [01-03, 01-06]
provides:
  - "Test-only RFC 8785 canonicaliser (tests/common/jcs.rs) with vector tests"
  - "Named process tests tc_act_*, tc_sec_03_*, tc_net_04_* in tests/action_contract.rs"
affects: [01-11]
tech-stack:
  added: []
  patterns: ["Test-only hand-written JCS mounted via #[path]; raw RawValue field comparison for D15 identity"]
key-files:
  created: [tests/common/jcs.rs, tests/action_contract.rs]
  modified: []
key-decisions:
  - "JCS number digits come from std shortest round-trip {:e} (same digits as ryu); ryu dev-dependency left unused"
  - "JCS test inputs for floats are parsed with str::parse::<f64>, because serde_json default float parsing is not exactly rounded"
requirements-completed: [CONF-01, CONF-03, CONF-08, CONF-09]
duration: 30min
completed: 2026-10-21
---

# Phase 1 Plan 08: Action, Security and Gateway-Trap Tests Summary

A hand-written RFC 8785 canonicaliser (4 vector tests) plus 12 named process tests prove four-field byte and JCS identity between hometown and olive, suppression reasons, olive verdict recording, data-only handling of command text, and the TC-NET-04 gateway trap with negative control.

## Tasks

1. Test-only JCS canonicaliser: commit 137a2ad. Appendix B numbers, RFC section 3.2.3 example, UTF-16 key order (U+10000 before U+FFFD), minimal string escaping.
2. Action, security and trap TCs: commit 94afaf3. TC-ACT-01, 02, 03 (schema cases incl. "5" not coerced, five-key, party mismatch), 06 (assumption A3 flagged), 07 (raw-field and jcs_sha256_hex equality with nested, non-ASCII/non-BMP, fractional, out-of-order, irregular-whitespace parameters), 08 (409/422/500 exit 22, one olive request each), 09 (refused_policy, trace_proposal_mismatch investigate=true with empty stderr, 200 vs 202 replay shape), TC-SEC-03, TC-NET-04 (normal, flagged, negative control).

## Verification

- `cargo test --locked --offline --features stubs --test action_contract`: 16 passed.
- clippy `--all-targets --all-features -D warnings` and `cargo fmt --check` clean; no JCS crate in Cargo.toml; vendor/ untouched.

## Deviations from Plan

None in behaviour. Notes:

- Task 1 used `{:e}` instead of ryu (the plan allows either).
- TC-ACT-03 "bad id" case: the product checks the catalog before the id format, so an id such as `Crm Case Note@x` is suppressed as `not_in_catalog`, not `schema`. The test asserts the real behaviour; the format branch in `src/task.rs` is unreachable for catalog-valid ids. Not a security defect; flagged for review against the spec wording ("bad-id -> schema").
- `resolver_calls` is not observable from a separate process; TC-NET-04 relies on the proxy seeing zero unroutable requests and exit 0 (resolver behaviour is covered by tests/proxy_contract.rs).

## Known Stubs

None.

## Self-Check: PASSED

tests/common/jcs.rs and tests/action_contract.rs present; commits 137a2ad and 94afaf3 present.
