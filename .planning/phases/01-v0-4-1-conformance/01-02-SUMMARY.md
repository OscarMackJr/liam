---
phase: 01-v0-4-1-conformance
plan: 02
subsystem: testing
tags: [static-scan, tc-net-03, egress]
requires: []
provides:
  - tests/static_contract.rs automated TC-NET-03, TC-ANS-04, TC-ACT-05, TC-SEC-01 static checks
key-files:
  created: [tests/static_contract.rs]
requirements-completed: [CONF-07]
metrics:
  completed: 2026-10-21
---

# Phase 1 Plan 02: Static contract scan Summary

Comment- and string-aware source scanner (`tests/static_contract.rs`, default features) that gates socket/resolver/process APIs, second client constructors and gateway/Authorization literals in non-stub `src/**/*.rs`, with negative controls.

## Tests
`tc_net_03_no_socket_or_resolver_apis`, `tc_net_03_single_proxy_client_constructor`, `tc_ans_04_no_gateway_reference`, `tc_act_05_no_process_spawn`, `tc_sec_01_no_authorization_literal`, and four `tc_net_03_negative_control_*` self-tests. All 9 pass; the scan found no real violations. `direct_gateway` is explicitly asserted as not flagged.

## Deviations
- [Rule 1] Initial substring match flagged `HttpHometownClient::new` as `Client::new`; token matching now requires an identifier boundary before the token.
- Build environment (not fixed, out of scope): committed `vendor/schannel/src/key_handle.rs` is LF but its `.cargo-checksum.json` expects CRLF bytes, so a fresh checkout fails `--locked --offline` with a checksum error. I built using `--config 'source.vendored-sources.directory="C:/Users/OMack/liam/vendor"'` (main checkout vendor, CRLF). Needs a follow-up to re-normalize vendor in git.

## Commits
- 41633e9: test(01-02): automate TC-NET-03 static scan with negative controls

## Self-Check: PASSED
