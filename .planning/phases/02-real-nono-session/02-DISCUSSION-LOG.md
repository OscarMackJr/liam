# Phase 2: Real nono Session - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log keeps the alternatives that were considered.

**Date:** 2026-10-21
**Phase:** 02-real-nono-session
**Areas discussed:** nono build and sign-off, TC-START-06 vs WR-05, Observation tooling, Disruptive refusal cases

---

## nono build and sign-off

| Option | Description | Selected |
|--------|-------------|----------|
| Build ~/Nono at a pinned commit | Clean build; commit recorded in RESULTS; diff reads the same source | ✓ |
| Installed 0.66.1 in Program Files | Already registered with WFP; source commit must be established | |
| Both, compared | Diff both, measure with whichever matches the specs | |

| Option | Description | Selected |
|--------|-------------|----------|
| You, recorded in-repo | User owns OscarMackJr/nono; phase pauses at each gate | ✓ |
| Someone else | Pause until another owner replies | |

| Option | Description | Selected |
|--------|-------------|----------|
| Reinstall from the pinned build | Whole stack matches one commit | ✓ |
| Record the mismatch, carry on | Measure with installed service, note as a limit | |
| You decide | Decide once versions are known | |

| Option | Description | Selected |
|--------|-------------|----------|
| You handle it, then give the commit | User resolves uncommitted nono changes; Liam never edits the nono repo | ✓ |
| Pin current HEAD, ignore the changes | Build clean 9945fed6 in a separate worktree | |
| Include them in the build | Measure dirty tree | |

| Option | Description | Selected |
|--------|-------------|----------|
| .planning phase dir | 02-SIGNOFF.md, one section per gate | ✓ |
| contracts/ + README | Ship sign-off with the code | |

## TC-START-06 vs WR-05

| Option | Description | Selected |
|--------|-------------|----------|
| dev-standard-user harness + real records | Spec harness plus check that real records pass | ✓ |
| Real records only | No separate harness | |

| Option | Description | Selected |
|--------|-------------|----------|
| Stop for review | WR-05 live, so record nono's behaviour and stop (feeds O3) | ✓ |
| Fix WR-05 in Liam now | §4 change ahead of re-ratification, needs sign-off | |

| Option | Description | Selected |
|--------|-------------|----------|
| Your normal non-admin session | The user nono launches Liam as | ✓ |
| A dedicated local test account | Separate standard user | |
| You decide | | |

## Observation tooling

| Option | Description | Selected |
|--------|-------------|----------|
| Procmon, PID-filtered | Scripted capture, CSV export, admin driver | ✓ |
| ETW via WPR/xperf | Built-in, heavier | |
| Sysmon | Persistent service, event IDs 1/11/22 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Allowlist diff against the profile | Profile grants plus known reads; out-of-list path fails | ✓ |
| Manual review of the capture | Judgement recorded in RESULTS | |

| Option | Description | Selected |
|--------|-------------|----------|
| Summary in RESULTS, raw files local only | Counts plus capture SHA-256; raw gitignored | ✓ |
| Commit filtered captures | Commit CSV after content-free scan | |

## Disruptive refusal cases

| Option | Description | Selected |
|--------|-------------|----------|
| Scripts you run elevated | Record state, change, restore in finally; user runs via `!` | ✓ |
| Manual steps, I just measure | User operates services by hand | |

| Option | Description | Selected |
|--------|-------------|----------|
| Not yet: defer TC-E2E-06 to Phase 3 | Measure once popeye runs | ✓ |
| Yes, it runs locally now | Stop popeye or closed port | |
| Simulate with a closed port | Only if fork config allows | |

| Option | Description | Selected |
|--------|-------------|----------|
| Stop and hand to you | Compare state, print restore commands, wait | ✓ |
| Auto-retry restore once | Retry then hand over | |

## Claude's Discretion

- Plan ordering after the stop gates; the Procmon filter and config details; script names
- The ACL grant and revoke mechanism for the TC-START-06 harness (must run as standard user and restore the ACL)

## Deferred Ideas

- TC-E2E-06 to Phase 3
- WR-05/WR-06 Liam fixes wait for NONO_v0.4.3 re-ratification
- Not discussed: release-build liam.toml under nono, the O7-rejected fallback, run IDs and party values for real runs
