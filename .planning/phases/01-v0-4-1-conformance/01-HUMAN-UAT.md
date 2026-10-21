---
status: partial
phase: 01-v0-4-1-conformance
source: [01-VERIFICATION.md]
started: 2026-10-21T00:00:00Z
updated: 2026-10-21T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. TC-ACT-03: a malformed action id reports not_in_catalog
expected: Decide whether the catalog lookup should precede the id-format check (current behaviour), or the id format should be checked first and report the schema reason.
result: [pending]

### 2. TC-ACT-06: assumption A3, multiple proposals arrive as an array
expected: Confirm the array shape against S-HT-1 (first element used, the rest logged as limit suppressions), or correct it.
result: [pending]

### 3. WR-03: printed suggestion line
expected: "Suggestion (not submitted): <action_type>: <justification>" is acceptable wording, shown as text only and never submitted.
result: [pending]

### 4. WR-09: stdout closed
expected: The run still writes its record and exits with its documented code; silently dropping the answer when stdout is broken is acceptable.
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
