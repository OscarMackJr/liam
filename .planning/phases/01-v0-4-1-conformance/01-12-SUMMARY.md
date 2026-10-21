---
phase: 01-v0-4-1-conformance
plan: 12
subsystem: contracts
tags: [vocabulary, sha256, pin]
requires: [01-01, 01-09, 01-10, 01-11]
provides:
  - canonical vocabulary vendored and pinned (CONF-11)
key-files:
  modified: [contracts/vendored/vocabulary.yaml, contracts/vendored/VENDORED.md, liam.toml, tests/common/mod.rs, tests/hygiene_contract.rs, README.md, RESULTS.md, .gitattributes]
key-decisions:
  - "Added contracts/vendored/** -text to .gitattributes so autocrlf cannot alter the canonical vocabulary bytes"
requirements-completed: [CONF-11]
duration: 10min
completed: 2026-10-21
---

# Phase 1 Plan 12: Canonical vocabulary pin Summary

The canonical fiskroad vocabulary.yaml (SHA-256 60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7) is vendored byte-for-byte and pinned in liam.toml and tests/common; the hash was verified on the working copy and on the committed blob.

## Tasks
1. Checkpoint: user supplied the file (hash verified by orchestrator).
2. Vendor, pin, document: commit f51c2fe (code/contracts); docs commits 7c6aa12 and cf3eea4 (RESULTS.md TC-LVOC-01/02 now `verified` at f51c2fe).

## Verification
fmt, clippy -D warnings (all targets/features) clean; `cargo test --features stubs` all pass, `tc_lvoc_canonical_hash_pinned` un-ignored and passing; TC-LVOC-01/02 pass. The src/config.rs scanner parsed the new file unchanged (only tenant_id lines differ), so no scanner change was needed.

## Deviations from Plan
None to scope. The plan's separate `servicing` manual process run was covered by the existing stub process tests rather than a standalone invocation. The vendor/ crate checksum line-ending defect was deliberately left alone per instruction.

## Self-Check: PASSED
