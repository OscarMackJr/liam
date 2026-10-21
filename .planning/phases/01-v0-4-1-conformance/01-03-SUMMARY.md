---
phase: 01-v0-4-1-conformance
plan: 03
subsystem: testing
tags: [stubs, tiny_http, test-proxy, gateway-trap]
requires: []
provides:
  - ScriptedStub, Reply, CapturedRequest, TestProxy, ProxyStats, GatewayTrap, gateway_attempt, gateway_attempt_requested
affects: [later conformance test plans, plan 06 main.rs wiring]
key-files:
  modified: [src/stubs.rs]
requirements-completed: [CONF-05, CONF-08]
completed: 2026-10-21
---

# Phase 1 Plan 03: Scripted stubs, routing proxy, gateway trap Summary

Data-driven ScriptedStub (ordered replies, delay, request capture incl. Authorization presence), one routing TestProxy (sentinel injection only on hometown/olive, 403 refusal of gateway.invalid, per-route stats), and a GatewayTrap with a negative-control mode, all inside the `stubs` feature in `src/stubs.rs`.

## Tasks
1. ScriptedStub and routing TestProxy - commit c65b337
2. GatewayTrap, refusal, gateway_attempt - commit c65b337 (both tasks touched only src/stubs.rs and were committed together)

## Verification
- `cargo test --features stubs --lib stubs::` : 9 new tests pass (ordering, delay, error key, routing, sentinel injection, inbound auth count, drop, refusal, negative control, env unset); lib suite 22 passed.
- clippy `-D warnings` and `cargo fmt --check` clean.
- `LIAM_STUB_GATEWAY_ATTEMPT` and `gateway.invalid` appear only in src/stubs.rs.
- Absolute-form target is exposed by tiny_http `request.url()` (assumption A6 holds); Host header fallback is implemented anyway.

## Deviations from Plan
None to the code. Proxy handles each request on its own thread; stub threads poll a stop flag via `recv_timeout` and join on Drop. No unwrap/expect in spawned threads (proxy forward failures reply 502 and land in `errors()`).

## Issues / Environment (out of scope, not fixed)
- The worktree checkout holds LF-normalized copies of vendored files (`vendor/schannel/src/key_handle.rs`) and `contracts/vendored/vocabulary.yaml`, so cargo's vendor checksum check fails and the vocabulary SHA in liam.toml mismatches. I built with `--config source.vendored-sources.directory="C:/Users/OMack/liam/vendor"` (read-only use of the main checkout vendor dir).
- Consequently `tests/process_contract.rs` (8 tests) fails in this worktree with exit 11 (vocabulary hash mismatch). Unrelated to this plan (only src/stubs.rs was modified); should pass in a checkout with original bytes. Static scan test (plan 02) not present yet.

## Self-Check: PASSED
