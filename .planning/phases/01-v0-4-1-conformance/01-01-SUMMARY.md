---
phase: 01-v0-4-1-conformance
plan: 01
subsystem: build-config-tests
tags: [version, profile, cargo, test-helper]
requires: []
provides: [0.4.1 labels, profile_version 4, raw_value + ryu, tests/common helper]
affects: [tests/process_contract.rs, Cargo.toml, liam.toml, profiles/liam.nono.toml]
key-files:
  created: [tests/common/mod.rs]
  modified: [Cargo.toml, Cargo.lock, liam.toml, profiles/liam.nono.toml, .gitattributes, tests/process_contract.rs]
decisions:
  - "run_liam takes the exe path as a parameter (CARGO_BIN_EXE_liam is unusable inside tests/common)"
  - "Added write_test_config_with_runs for the evidence-path-failure test, which needs a custom runs_dir"
metrics:
  tasks: 2
  completed: 2026-10-21
requirements: [CONF-06]
---

# Phase 1 Plan 01: Labels, profile and shared test helper Summary

Liam labels moved to 0.4.1, the nono profile carries integer `profile_version = 4`, serde_json gains `raw_value`, ryu is a dev-dependency, and one `tests/common/mod.rs` helper now owns the test liam.toml text, version and vocabulary hash.

## Commits
- 8faa5ff chore: labels, Cargo features, profile, gitattributes
- 920a734 test: shared helper and process_contract migration
- follow-up style commit: cargo fmt

## Deviations from Plan

**1. [Rule 3 - Blocking] Worktree vendor and vocabulary bytes had wrong line endings**
- Found: `cargo build --locked --offline` failed with a vendor checksum error (`vendor/schannel/src/key_handle.rs`), then the process tests failed with `vocab:sha256`.
- Cause: the committed blobs for these files are LF while the checksums and the pinned vocabulary hash correspond to the CRLF bytes present in the main checkout. This is a baseline defect, not caused by this plan.
- Fix: copied the main checkout's `vendor/` and `contracts/vendored/vocabulary.yaml` into the worktree working copy only. These are NOT committed, so no vendor/ diff exists in any commit. The orchestrator should be aware that fresh worktrees and CI checkouts hit this unless the vendor blobs or attributes are corrected (plan 12 / CONF-11 owns the vocabulary hash).
- Side effect: `.gitattributes` `specs/** -text` makes git show the three `specs/*.md` files as modified in checkouts where they have CRLF; they were not touched or committed.

## Verification
- `cargo build --locked --offline --features stubs` passes; no vendor changes committed.
- `cargo test --locked --offline --features stubs --test process_contract`: 8 passed.

## Known Stubs
None.

## Self-Check: PASSED
