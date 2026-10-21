---
phase: 01-v0-4-1-conformance
plan: 04
subsystem: catalog
tags: [catalog, validator, schema, CONF-01]
requires: []
provides: [catalog::validate_parameters, catalog::action_id_format_ok, catalog parameters_schema]
affects: [plan 05 (task.rs switch to catalog.rs)]
tech-stack:
  added: []
  patterns: [hand-written JSON-Schema subset validator, no coercion]
key-files:
  created: [src/catalog.rs]
  modified: [src/lib.rs, contracts/catalog-0.4.json]
decisions:
  - "Integer = is_i64/is_u64 (5.0 rejected), stricter than JSON Schema 2020-12"
metrics:
  tasks: 1
  files: 3
completed: 2026-10-21
---

# Phase 1 Plan 04: Catalog parameters_schema and validator Summary

Pure hand-written validator in `src/catalog.rs` (type/properties/required/additionalProperties/maxLength subset, no coercion, Err("schema") on any violation) plus a catalog JSON carrying per-action `parameters_schema` for crm.case_note@1, crm.follow_up@1 (integer days) and crm.case_update@1 (free-form details object).

## Commits
- cae78b7: feat(01-04): add catalog parameters_schema and read-only validator

## Notes
- `required_parameters` is deliberately kept on each action in `contracts/catalog-0.4.json` so the unchanged `src/task.rs` still parses; plan 05 must remove it when it switches task.rs onto `catalog.rs`.
- `action_id_format` key added to the catalog JSON.
- 5 unit tests in `catalog::tests` cover all five plan behaviours and deserialise the repo catalog file; full `--features stubs` suite passes.

## Deviations from Plan
None in code. Environment finding (out of scope, not fixed in git): at base 829e635 the committed vendor files are LF but cargo checksums expect the original bytes for some crates (e.g. vendor/schannel/src/key_handle.rs, generic-array), so `--locked --offline` builds fail in a fresh worktree. I verified locally by copying the main checkout's vendor/ into the worktree (uncommitted, unstaged) and by re-checking out `contracts/vendored/vocabulary.yaml` with `core.autocrlf=false` (CRLF working copy changed its SHA-256 and made process_contract tests exit 11). Neither change was committed.

## Known Stubs
None.

## Self-Check: PASSED
src/catalog.rs, contracts/catalog-0.4.json exist; commit cae78b7 exists.
