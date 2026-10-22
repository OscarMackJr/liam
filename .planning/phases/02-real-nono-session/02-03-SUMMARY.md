---
phase: 02-real-nono-session
plan: 03
subsystem: nono-contract
tags: [nono, contract, gate, stop]
requires: [02-02]
provides:
  - Gate 1 contract diff against nono cf3ebc56 in 02-SIGNOFF.md section 1
  - Dated cf3ebc56 section in contracts/receipt-fields.md (STOP FOR REVIEW)
affects: [02-04 through 02-08]
key-files:
  created: []
  modified:
    - .planning/phases/02-real-nono-session/02-SIGNOFF.md
    - contracts/receipt-fields.md
    - README.md
decisions:
  - Gate 1 verdict MISMATCH; nono owner chose stop. Plans 02-04..02-08 do not run until nono emits the contract or §4 is re-ratified with NONO_v0.4.3
metrics:
  duration: single session
  completed: 2026-10-22
requirements: [NONO-01]
---

# Phase 2 Plan 03: Gate 1 Contract Diff Summary

The pinned nono (cf3ebc56, v0.70.0) does not emit Liam's receipt/agent-record contract; the nono owner stopped the phase at gate 1.

## Tasks

| Task | Commit | Result |
|------|--------|--------|
| 1 | 2fca0bb | Read-only source, CLI-help diff; verdict MISMATCH; receipt-fields.md and README updated per change control |
| 2 | this commit | Gate 1 decision: stop (`approval: stopped` in SIGNOFF section 1) |

## Findings

| Field | Contract | nono cf3ebc56 | Verdict |
|---|---|---|---|
| schema_version | 2 | 1 | different |
| session_id | opaque id | present | match |
| token_arm | appcontainer-compatible arm | present, 5 variants | match (value not measured) |
| layers | 14 | 13 | different |
| AgentKeyInjection | 14th layer | absent | absent |
| AgentSessionRecord (schema 1) | separate record | absent | absent |
| NONO_RECEIPT_PATH | env var | absent (machine-wide sink, `<session_id>.jsonl`) | absent |
| NONO_AGENT_RECORD_PATH | env var | absent | absent |
| HTTPS_PROXY/HTTP_PROXY/NO_PROXY | loopback proxy | emitted by nono-proxy in proxy mode | unverified |
| --purpose | on launch | absent on `nono run` and `nono agent launch` | absent |

Both the `nono run` and `nono agent` paths were checked. scripts/launch-under-nono.ps1 (`nono run --profile --purpose`) matches neither; recorded, not rewritten. No runtime session was attempted because no launch path emits the contract records.

## Deviations

- Orchestrator independently re-verified the absent identifiers (0 hits in crates at cf3ebc56) and the missing `--purpose` on both commands.
- WFP driver: source shows a separate `nono-wfp-driver` kernel service that the build only stages when the .sys is present; a dev registration likely has no driver service. Not confirmed on the machine.

## Next

Phase 2 is halted. Resume requires one of: nono adds schema-2 receipt, AgentKeyInjection layer, AgentSessionRecord, NONO_RECEIPT_PATH / NONO_AGENT_RECORD_PATH and `--purpose` (then re-pin and re-run 02-03), or §4 is re-ratified with NONO_v0.4.3 and Liam's contract updated.

## Self-Check: PASSED
