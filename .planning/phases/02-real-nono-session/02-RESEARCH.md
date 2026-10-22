# Phase 2: Real nono Session - Research

**Researched:** 2026-10-21
**Domain:** Windows measurement and evidence work against an external nono fork (receipt/agent-record contract diff, AppContainer token_arm census, ACL harness, ETW/Procmon traces, disruptive WFP refusal cases)
**Confidence:** MEDIUM (local state verified by direct probing; Windows tracing specifics partly [ASSUMED])

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: "The fork" is a build of `~/Nono` (`OscarMackJr/nono`) at a commit the user pins. Every Phase 2 RESULTS row records that commit in its "nono commit" column, plus receipt version and Windows build (`ver` plus UBR) per test spec §2.3. The contract diff reads the same source tree.
- D-02: `~/Nono` has uncommitted changes (`crates/nono-cli/src/deny_model.rs`, `crates/nono/src/path.rs`; HEAD `v3.6-957-g9945fed6`). The user commits or discards these in the nono repo and names the commit to pin. Liam's plans never edit, commit to or stash in the nono repo; first plan step records the commit the user gives and checks the nono tree is clean at it.
- D-03: The nono WFP service and driver must come from the same pinned build. Installed copy is `C:\Program Files\nono\` (`nono 0.66.1`, binaries dated Jul 6); reinstall service and driver from the pinned build (admin step, D-11/D-12) and verify CLI, service and driver match before any measurement.
- D-04: The nono owner is the user. Each stop gate pauses the phase; approval is recorded in `.planning/phases/02-real-nono-session/02-SIGNOFF.md`, one section per gate (contract diff; O7 `token_arm` and census), each with measured values, nono commit, date, user approval. No plan proceeds past a gate until its section says approved. `expected_token_arm` is set in `liam.toml` only from the approved O7 value.
- D-05: A contract-diff mismatch follows change-control: update the contract table and README, then stop for review. Liam is not adapted to the fork.
- D-06: Measure TC-START-06 with the dev-standard-user harness. As a standard user, grant the test user write access on each record file via ACL, no read-only attribute, confirm `receipt:writable` / `agent_record:writable`. Also confirm the real records nono writes pass as non-writable. WR-05 stays a separately documented gap.
- D-07: If real nono records are "non-writable" only because of the read-only attribute (not a deny-write ACL), WR-05 is live: record exactly what nono does and STOP for nono-owner review (feeds NONO_v0.4.3 O3). Do not change Liam's §4 behaviour without re-ratification.
- D-08: Harness test user is the user's normal non-admin logon. Planner first verifies this is really a standard user (not an elevated admin token). If not, stop and ask.
- D-09: Use Sysinternals Process Monitor, scripted from the command line (`/BackingFile`, `/LoadConfig`, `/Quiet`, `/Minimized`, export to CSV). One capture covers process create, file/registry access and network events. Start/stop is an elevated step the user runs (D-11).
- D-10: TC-SEC-02 judged by allowlist diff (profile grants + receipt, agent record, `liam.toml`, vendored vocabulary, catalog, `runs_dir`, exe image and DLL loads). Out-of-allowlist path fails and is listed in RESULTS. TC-SEC-01 passes on zero process-create events from Liam's PID. TC-NET-08 passes on zero DNS queries from Liam's PID.
- D-11: RESULTS rows record counts plus SHA-256 of each raw capture. Raw captures stay in a gitignored local folder, never committed. Any committed summary must be content-free.
- D-12: Admin-only steps are small PowerShell scripts the user runs from an elevated prompt (e.g. `! powershell -File scripts/e2e-wfp-stop.ps1`). Claude never self-elevates. Each script records prior state, makes its change, restores in `finally`.
- D-13: After each disruptive script, compare system state with the pre-recorded state. On any difference: stop, print exact restore commands, wait for the user. No automatic retry.
- D-14: TC-E2E-03 and TC-E2E-04 are measured in Phase 2. TC-E2E-06 moves to Phase 3.

### Claude's Discretion
- Order of plans after the two stop gates, exact Procmon filter/config files, script names and layout under `scripts/`.
- How the TC-START-06 harness grants/revokes ACLs (`icacls` or Win32), provided it runs as the standard user and restores the original ACL.

### Deferred Ideas (OUT OF SCOPE)
- TC-E2E-06 (key exchange unreachable): Phase 3. Planner should update ROADMAP.md and REQUIREMENTS.md (NONO-04) to move it.
- WR-05 / WR-06 Liam-side fixes: wait for NONO_v0.4.3 re-ratification (O2, O3).
- Not discussed, left to planning: the release-build `liam.toml` used under nono, fallback if the O7 `token_arm` is rejected (stop, as with any gate), run IDs and party values for real-session runs.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NONO-01 | Contract diff of fork receipt/agent record/proxy env vs `contracts/receipt-fields.md`, §4.2, A5 | Source locations in fork; early finding that fork source appears NOT to implement v0.4.3 (see Summary, Pitfall 1) |
| NONO-02 | Measure AppContainer `token_arm` + §4.3 census on DirectCli; stop for sign-off | Needs a session that produces a record; measurement approach and SIGNOFF structure below |
| NONO-03 | `launch-under-nono.ps1` passes self-check; writable record refused | Launch script gaps, ACL harness design |
| NONO-04 | Direct launch (E2E-03) and WFP-stopped (E2E-04); E2E-06 moved to Phase 3 | Elevated script pattern, expected outcomes |
| NONO-05 | Zero child processes, grants only, zero DNS from Liam PID | Tooling split: Procmon for SEC-01/02, ETW DNS-Client for NET-08 |
</phase_requirements>

## Summary

This is a measurement-and-evidence phase with almost no Rust change. Its work products are scripts under `scripts/`, a `02-SIGNOFF.md`, RESULTS.md rows, a README/contract update if the diff mismatches, and one `liam.toml` edit (`expected_token_arm`) after O7 sign-off.

**Highest-risk finding (probed this session): the NONO-01 gate will very likely stop the phase on the first step.** In `C:\Users\OMack\Nono` (HEAD `9945fed6`, branch `milestone/v2.13-carryforward-closeout`), grep of `crates/` finds no `AgentSessionRecord`, no `NONO_RECEIPT_PATH`/`NONO_AGENT_RECORD_PATH`, and `LayerId` has 13 variants with no `AgentKeyInjection`; `receipt.rs` still describes the 13-entry census that `contracts/receipt-fields.md` records as the pre-v0.4.3 state. The installed `nono 0.66.1` `run --help` has no `--purpose` flag, and `launch-under-nono.ps1` passes `--purpose` to `nono run`. The only v0.4.3 material in the fork is `specs/NONO_v0.4.3.md` (untracked `specs/` in the nono repo). `nono agent` / `nono daemon` / `nono-agentd` exist (service `nono-agentd_1838b8` is running), so the agent-launch path may be where the v0.4.3 behaviour lives, rather than `nono run`; that must be checked before declaring mismatch. [VERIFIED: grep of ~/Nono, `nono run --help`, `Get-Service`]. The commit referenced in `contracts/receipt-fields.md` (`29557a93`) is an ancestor of the current branch (committed Sep 7 2026). [VERIFIED: git]

Consequence for planning: Plan 1 must be "pin commit, build/reinstall, diff" and Plan 2 must be the gate. Everything after (O7, TC-START-06, E2E, SEC/NET) cannot start until the fork actually emits a schema-2 receipt and schema-1 agent record. If the fork does not implement them, the correct outcome is: record the mismatch in the contract table and README, stop, and tell the user the phase is blocked on nono work (Liam is a consumer only and must not implement or patch nono). Plans past the gate should be written as conditional (`depends on 02-SIGNOFF.md section 1 approved`).

Tooling conclusion: Procmon (D-09) is right for process-create and file-access (TC-SEC-01, TC-SEC-02) but is NOT adequate for TC-NET-08. DNS resolution on Windows is performed by the DNS Client service (svchost/Dnscache), so Procmon's UDP/53 events are attributed to svchost, not `liam.exe`. The test spec itself says TC-NET-08 uses an ETW trace of the `Microsoft-Windows-DNS-Client` provider. Use ETW (built-in `logman`) for NET-08, and Procmon for SEC-01/02. This is a refinement of D-09/D-10 that stays within their intent; flag it to the user in the plan.

**Primary recommendation:** Plan 1 pins and verifies the nono build and runs a read-only source+runtime diff; the plan then halts at the SIGNOFF gate. Write all downstream measurement scripts as small, restore-in-`finally` PowerShell, with Procmon for file/process and `logman` ETW for DNS, all captures under a gitignored `evidence-local/` folder.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Receipt/agent-record production, ACLs, env vars | nono (external supervisor) | — | Liam only consumes (contract-only integration) |
| Startup self-check | Liam binary (`src/receipt.rs`, `main.rs`) | — | Already built in Phase 1; Phase 2 only measures |
| `expected_token_arm` value | `liam.toml` (config) | nono owner sign-off | Set only from approved O7 |
| Observation (process/file/DNS) | Windows tracing tools (Procmon, ETW) outside Liam | Elevated user-run scripts | Liam must not observe itself or spawn tools |
| Disruptive state changes (WFP stop, reinstall) | Elevated PowerShell run by user | — | D-12: Claude never self-elevates |
| Evidence storage | Gitignored local folder (raw) + RESULTS.md rows (counts, SHA-256) | `02-SIGNOFF.md` (committed, content-free) | D-11 |

## Standard Stack

No new crates. Everything is built-in Windows or Sysinternals.

| Tool | Version/Location | Purpose | Notes |
|------|------------------|---------|-------|
| Process Monitor | `C:\tools\bin\procmon64` (present) | SEC-01 (Process Create), SEC-02 (file/registry) | Needs admin to load driver. Version not probed. |
| `logman` | `C:\Windows\system32\logman.exe` (present) | ETW session for `Microsoft-Windows-DNS-Client` (NET-08) | Elevated start/stop; output `.etl` |
| `Get-WinEvent -Path x.etl -Oldest` / `tracerpt` | built-in | Parse ETL to CSV with ProcessId | Event header carries PID |
| `netsh wfp show filters` / `netsh wfp show state` | built-in | Pre/post state compare for WFP stop (D-13) | Elevated; outputs XML, may include machine paths |
| `Get-Service` / `sc.exe` | built-in | Service status/start type before/after | nono services: `nono-wfp-service`, `nono-agentd_1838b8`; BFE running |
| `icacls` / `Get-Acl` | built-in | ACL harness for TC-START-06 | Run as standard user on files the user owns |
| `Get-NetTCPConnection` | built-in | Optional cross-check of no connection from Liam PID in E2E-03 | Short-lived process, so Procmon TCP/UDP events are the primary evidence |
| `pktmon` | present | Optional: confirm no packets for E2E-03 | Elevated; not required |

Not recommended: Sysmon (not installed: `Get-Service` shows none; installing a driver/config is invasive and unnecessary), and any Liam-side instrumentation (would violate "no child processes / no extra state").

Alternative considered: Procmon-only for DNS. Rejected: wrong PID attribution (see Summary). [CITED: specs/LIAM_TEST_SPEC_v0.4.1.md line 133 specifies the ETW DNS-Client trace]

## Package Legitimacy Audit

No external packages are installed in this phase. Sysinternals Procmon is already present at `C:\tools\bin\procmon64`. Packages removed: none. Packages flagged: none.

## Architecture Patterns

### Measurement Flow
```
user pins nono commit ──> plan verifies tree clean at commit ──> build/reinstall (admin, user-run)
        │
        ▼
read-only diff: fork source + one real session's receipt/agent record/env  ──>  GATE 1 (SIGNOFF §1)
        │ match                                 │ mismatch -> update contract table + README -> STOP
        ▼
launch under nono, record receipt + agent record (copy, hash, field names only)
        ▼
census + token_arm vs src/expectancy.rs  ──>  GATE 2 / O7 (SIGNOFF §2)  ──> set liam.toml expected_token_arm
        ▼
 ┌── TC-START-06 harness (standard user, ACL grant/revoke, restore)
 ├── real-record writability check (D-06/D-07; may re-stop for O3 review)
 ├── TC-E2E-03 direct launch (Procmon network capture running, expect exit 10/11)
 ├── TC-E2E-04 WFP service stopped (elevated script, restore in finally, D-13 state compare)
 └── SEC-01/02 (Procmon) + NET-08 (ETW) around one real run
        ▼
 RESULTS.md rows (docs-only commit) with Liam commit, nono commit, receipt version, ver+UBR
```

### Recommended layout
```
scripts/
├── nono-diff-probe.ps1        # records versions/hashes of installed CLI/service/driver, field names only
├── start06-acl-harness.ps1    # standard user; records ACL (SDDL), grants write, runs liam, restores in finally
├── e2e-direct-launch.ps1      # E2E-03
├── e2e-wfp-stop.ps1           # E2E-04 (elevated, user-run)
├── trace-procmon.ps1          # elevated start/stop; backing file + CSV export
├── trace-dns-etw.ps1          # elevated logman start/stop + parse
└── analyze-trace.ps1          # non-elevated: allowlist diff, counts, SHA-256
evidence-local/                # gitignored (add to .gitignore): raw .pml/.csv/.etl, receipt copies
.planning/phases/02-real-nono-session/02-SIGNOFF.md
```
The existing precedent is `scripts/measure-build.ps1` (PASS/FAIL lines). `.gitignore` currently has `runs/*` but not `evidence-local/`; add it before the first capture.

### Pattern: elevated script with restore (D-12/D-13)
```powershell
# Source: standard PowerShell try/finally; names illustrative
$before = Get-Service nono-wfp-service | Select Name,Status,StartType
$filtersBefore = (netsh wfp show filters file=- 2>$null | Measure-Object -Line).Lines  # or hash of filter keys
try { Stop-Service nono-wfp-service -Force; <# launch under nono, record outcome #> }
finally { Start-Service nono-wfp-service
  $after = Get-Service nono-wfp-service | Select Name,Status,StartType
  if (Compare-Object $before $after -Property Status,StartType) {
     Write-Host 'STATE DIFFERS. Restore: sc.exe config nono-wfp-service start= auto; sc.exe start nono-wfp-service'; exit 1 } }
```

### Anti-Patterns
- Adapting Liam to the fork's shape on mismatch (violates D-05 / AGENTS.md).
- Editing `profiles/liam.nono.toml`-driven nono behaviour beyond the reviewed draft, or touching nono repo (D-02).
- Running Claude-side `Start-Process` of elevated tools; elevated steps are `! powershell -File ...` for the user.
- Setting `expected_token_arm` to whatever the first run shows.
- Committing raw captures, SDDL dumps with SIDs/machine paths, or `netsh wfp` XML.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process/file tracing | Custom hooks or Liam self-logging | Procmon CLI | Liam must stay free of tracing and child processes |
| DNS attribution | Parsing Procmon UDP/53 | ETW DNS-Client provider | Resolution happens in the DNS Client service |
| ACL manipulation | Raw Win32 in Rust tests | `icacls /save` / `/restore` or `Get-Acl`/`Set-Acl` with SDDL backup | Exact restore is the requirement |
| Record writability probe | New Liam code | Existing `ensure_not_writable` | §4 is frozen until re-ratification |

## Runtime State Inventory

Not a rename phase, but the phase mutates machine state, so the same discipline applies.

| Category | Items | Action |
|----------|-------|--------|
| Stored data | Real receipts in `%PROGRAMDATA%\nono\receipts` (fork `receipt_sink.rs`); agent records location per NONO_v0.4.3 §8 | Read-only inspection; copy to `evidence-local/` for hashing |
| Live service config | `nono-wfp-service` (running, auto), `nono-agentd_1838b8` (running, auto), BFE | Snapshot before/after every disruptive step |
| OS-registered state | WFP filters, kernel driver `nono-wfp-driver.sys`, Procmon driver (loaded on capture) | Reinstall from pinned build (user, elevated); unload Procmon after capture |
| Secrets/env vars | None for Liam; proxy env supplied by nono | None. Do not log env values (proxy port is fine as a number only) |
| Build artifacts | `C:\Program Files\nono` is 0.66.1, Jul 6; `Nono\target\release\*.exe` dated Aug 4; `liam\target\release\liam.exe` exists | Rebuild Liam release at a recorded commit; verify nono binaries match pinned commit |

## Common Pitfalls

### Pitfall 1: The fork does not emit the v0.4.3 records
**What goes wrong:** Every later step is blocked; Liam would exit `10` on a missing `NONO_RECEIPT_PATH`/agent record.
**Why:** Probed: no `AgentSessionRecord`/`AgentKeyInjection`/env-var code in `crates/`; installed `nono run` lacks `--purpose`.
**Avoid:** Treat as a legitimate stop at Gate 1. Check `nono agent` (daemon launch path, `crates/nono-cli/src/agent_daemon/launch.rs`) before concluding. If implemented there, `launch-under-nono.ps1` (which uses `nono run`) is itself a contract mismatch to record and surface, not to rewrite silently.

### Pitfall 2: Procmon cannot filter by Liam's PID in advance
PID is unknown until nono launches Liam, and Liam is a child of nono under an AppContainer token. Capture broadly (to a backing file), filter post-hoc by Process Name `liam.exe` and record its PID. Process-create check: no `Process Create` operation whose parent PID is Liam's PID. Note nono itself spawning Liam is expected and not counted.

### Pitfall 3: ACCESS_DENIED noise
TC-SEC-02 requires "no `ACCESS_DENIED` elsewhere". AppContainer loader and system DLL probes can produce benign denials; the allowlist must include image/DLL paths, and any residual denial must be listed in RESULTS rather than hidden. Decide with the user whether loader-origin denials count; do not silently exclude.

### Pitfall 4: WR-05 reality check
Stub fixtures use the read-only attribute. If real nono records rely on the same, D-07 triggers an O3 stop. Capture the actual DACL (SDDL), owner and attributes of both real files (kept local; commit only "deny-write ACE present: yes/no, read-only attr: yes/no").

### Pitfall 5: Standard-user verification
`IsInRole('Administrator')` returned `False` from this shell, but this must be checked with the token's elevation state (`whoami /groups` high-integrity label) from the actual PowerShell session used for the harness. Note Bash `whoami` does not accept `/groups`; use PowerShell. Stop and ask if elevated (D-08).

### Pitfall 6: WFP stop leaving filters or a changed start type
Follow D-13 exactly; BFE is a dependency of WFP, never stop it.

### Pitfall 7: Receipt version / Windows build recording
RESULTS needs receipt version, nono commit, and `ver` plus UBR. Current: build 10.0.26200, UBR 9711 (probed today; re-probe at measurement). Also record Liam commit and `liam.exe` SHA-256.

### Pitfall 8: Session lifetime
Records have a freshness window (`record_max_age_seconds = 120`) and `written_at` checks; harness runs must launch Liam promptly after session start, and `launch-under-nono.ps1` currently has no purpose/party validation beyond mandatory params. The script runs `.\target\release\liam.exe` relative to cwd, so run it from repo root.

### Pitfall 9: Release `liam.toml`
Base URLs are `*.invalid`. For Phase 2 the self-check passes before network, so a run may then exit 21 (Hometown unavailable) after the check. That is acceptable: "passes the startup self-check" means no exit 10/11. Plan must state this expected downstream exit and keep the proxy hop from being exercised for DNS (proxy is an IP literal; the deny-all resolver should keep `liam.exe` DNS count at 0, and the ETW run needs a positive control, e.g. a `nslookup` from a normal shell, to prove the trace captures DNS at all).

## Code Examples

### ETW DNS capture (elevated, user-run) [ASSUMED: provider GUID and flags; verify with `logman query providers "Microsoft-Windows-DNS-Client"`]
```powershell
logman create trace liam-dns -p "Microsoft-Windows-DNS-Client" 0xFFFFFFFFFFFFFFFF 0xFF -o evidence-local\dns.etl -ets
# ... run Liam under nono ...
logman stop liam-dns -ets
Get-WinEvent -Path evidence-local\dns.etl -Oldest | Where-Object ProcessId -eq $liamPid | Measure-Object
```
`Get-WinEvent` on a provider-less ETL may need `tracerpt dns.etl -of CSV`; pick whichever yields ProcessId on this build.

### Procmon CLI [CITED: Sysinternals Procmon command-line options; verify flags locally with `procmon64 /?`]
```powershell
procmon64 /AcceptEula /Quiet /Minimized /BackingFile evidence-local\liam.pml
procmon64 /Terminate
procmon64 /OpenLog evidence-local\liam.pml /SaveAs evidence-local\liam.csv
```

## State of the Art

| Old | Current | Impact |
|-----|---------|--------|
| 13-entry census, no agent record (receipt-fields.md pre-0.4.3) | 14 entries incl. `AgentKeyInjection`, separate agent-session record (NONO_v0.4.3) | Fork source probed today still shows 13; this is the gate |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Microsoft-Windows-DNS-Client` ETW session via `logman` records per-event PID and is parseable with `Get-WinEvent`/`tracerpt` on this build | Code Examples | NET-08 method must change (e.g. `wpr`/`pktmon`) |
| A2 | Procmon cannot attribute DNS queries to the app PID (Dnscache does the lookup) | Summary | If wrong, Procmon alone could cover NET-08; still harmless to use ETW |
| A3 | Procmon CLI flags as listed work with the installed `procmon64` version | Code Examples | Script tweak only |
| A4 | The v0.4.3 behaviour may live in `nono agent`/`nono-agentd` rather than `nono run` | Pitfall 1 | If absent everywhere, phase is blocked on nono work |
| A5 | Loader-origin ACCESS_DENIED events can occur in AppContainer traces | Pitfall 3 | Allowlist rules need a user decision |

## Open Questions

1. **Does the fork emit schema-2 receipts and agent records at any entry point?** Known: not in `crates/` grep and not in `nono run --help`. Unknown: the `nono agent` path. Recommendation: Plan 1 reads `agent_daemon/launch.rs` and `nono agent --help`, then Gate 1.
2. **Pinned commit and clean tree.** User must supply (D-02). Untracked `specs/` and `.planning/` in nono are not a build issue but the two modified `.rs` files are.
3. **Do ACCESS_DENIED loader events count against TC-SEC-02?** Ask the user at the gate with real data.
4. **ROADMAP.md / REQUIREMENTS.md edit moving TC-E2E-06 to Phase 3** should be a small first plan task.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| nono CLI | all | yes | 0.66.1 (installed, Jul 6) | rebuild from pin (user) |
| nono-wfp-service / driver | E2E-04, census | yes, running | installed copy | reinstall from pin (user, elevated) |
| nono-agentd | agent launch path | yes, running | service `nono-agentd_1838b8` | — |
| Procmon | SEC-01/02 | yes | `C:\tools\bin\procmon64`, version unprobed | — |
| logman, netsh, pktmon, wpr | NET-08, E2E-04 | yes | built-in | — |
| Sysmon | — | no | — | not needed |
| Windows build | RESULTS | yes | 10.0.26200, UBR 9711 | — |
| `liam.exe` release | all | yes | `target\release\liam.exe` (rebuild at recorded commit) | `cargo build --locked --offline --release` |
| Elevated shell | Procmon, logman, service stop, reinstall | no from this session (non-admin) | — | user runs `! powershell -File ...` |
| popeye key exchange | E2E-06 only | not here | — | deferred to Phase 3 |

**Blocking:** pinned commit from the user; an elevated prompt for admin steps; (possibly) a fork build that actually implements v0.4.3.

## Validation Architecture

Skipped: `workflow.nyquist_validation` is `false` in `.planning/config.json`. Note for planner: verification is by script PASS/FAIL lines and RESULTS rows, not `cargo test`; existing `cargo test` must stay green (no Rust changes expected except `liam.toml`).

## Security Domain

`security_enforcement` enabled, ASVS level 1.

| ASVS | Applies | Control |
|------|---------|---------|
| V2/V3 Auth/Session | no (Liam holds no credential) | nono-proxy injection |
| V4 Access Control | yes | TC-START-06, real-record ACL check, AppContainer grants |
| V5 Input Validation | limited | script params validated, no shell interpolation of `--question`/`--party` |
| V6 Cryptography | no (DSSE deferred) | — |
| V14 Config/Data protection | yes | raw captures local and gitignored; no SIDs/paths/party values/questions in committed files |

Threats: elevated scripts altering system state (mitigated by try/finally + D-13); evidence leaking machine paths or party/question text (use fixed dummy party and question, record only counts and hashes); a captured proxy env value leaking into RESULTS (record port number only if at all); Procmon capture including other processes' activity (filter and keep local).

## Sources

### Primary (HIGH)
- Local probing this session: `C:\Users\OMack\Nono` git state and grep, `nono run --help`, `Get-Service`, registry UBR, `command -v`
- `specs/LIAM_TEST_SPEC_v0.4.1.md` (§2.3, TC-START-06, TC-E2E-03/04/06, TC-SEC-01/02, TC-NET-08)
- `contracts/receipt-fields.md`, `src/expectancy.rs`, `liam.toml`, `profiles/liam.nono.toml`, `scripts/launch-under-nono.ps1`, README Known gaps, RESULTS.md rows

### Secondary / Tertiary
- Training knowledge for Procmon CLI flags, ETW/logman usage, DNS Client attribution (LOW-MEDIUM; flagged A1-A3, to verify with `procmon64 /?` and `logman query providers`)

## Metadata

**Confidence:** stack MEDIUM (tools present; flags unverified), architecture MEDIUM, pitfalls MEDIUM-HIGH (Pitfall 1 verified locally)
**Research date:** 2026-10-21; **Valid until:** until the user pins a commit (the fork changes daily)
