# Phase 2: Real nono Session - Pattern Map

**Mapped:** 2026-10-21
**Files analyzed:** 15 (new or modified)
**Analogs found:** 11 / 15 (4 have no code analog; RESEARCH.md snippets apply)

Phase 2 is measurement and evidence work. There is almost no Rust change. Analogs are PowerShell scripts, RESULTS.md rows, contract docs and the Phase 1 process-test fixtures.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `scripts/nono-diff-probe.ps1` | measurement script (read-only) | batch / transform | `scripts/measure-build.ps1` | role-match |
| `scripts/start06-acl-harness.ps1` | measurement script, ACL grant/restore | file-I/O | `scripts/measure-build.ps1` (PASS/FAIL), `scripts/make-stub-records.ps1` (record writing, read-only attr) | role-match |
| `scripts/e2e-direct-launch.ps1` | measurement script | request-response | `scripts/launch-under-nono.ps1` + `scripts/measure-build.ps1` | role-match |
| `scripts/e2e-wfp-stop.ps1` | elevated script, restore in `finally` | event-driven (service state) | none; RESEARCH.md "elevated script with restore" | no analog |
| `scripts/trace-procmon.ps1` | elevated script, capture start/stop | streaming / file-I/O | none; RESEARCH.md Procmon CLI | no analog |
| `scripts/trace-dns-etw.ps1` | elevated script, ETW start/stop | streaming | none; RESEARCH.md ETW example | no analog |
| `scripts/analyze-trace.ps1` | non-elevated analysis, allowlist diff, SHA-256 | transform | `scripts/measure-build.ps1` | role-match |
| `scripts/launch-under-nono.ps1` (possibly modified) | launch wrapper | request-response | itself | exact |
| `.planning/phases/02-real-nono-session/02-SIGNOFF.md` | gate doc | n/a | `.planning/phases/01-v0-4-1-conformance/01-HUMAN-UAT.md` | partial |
| `RESULTS.md` (modify rows) | evidence table | n/a | existing rows in `RESULTS.md` | exact |
| `contracts/receipt-fields.md` (modify on mismatch) | contract table | n/a | itself | exact |
| `README.md` (modify "Known gaps" / contract) | docs | n/a | `README.md` lines 134-150 | exact |
| `.gitignore` (add `evidence-local/`) | config | n/a | existing `.gitignore` | exact |
| `liam.toml` (`expected_token_arm`, only after O7 approval) | config | n/a | itself | exact |
| `.planning/ROADMAP.md`, `REQUIREMENTS.md` (move TC-E2E-06 to Phase 3) | planning docs | n/a | existing entries | exact |

No new Rust test files are expected. If a process-level test is added, copy from `tests/startup_contract.rs`.

## Pattern Assignments

### Shared script header and PASS/FAIL style (all `scripts/*.ps1` measurement scripts)

**Analog:** `scripts/measure-build.ps1`

**Header and root pattern** (lines 1-4):
```powershell
# TC-BUILD-01 / TC-BUILD-02 release-build checks. No network, no secrets.
# Run from the repository root: powershell -NoProfile -File scripts/measure-build.ps1
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)
```

**PASS/FAIL output and exit codes** (lines 7-10, 20-24, 39-42):
```powershell
if ($LASTEXITCODE -ne 0) { Write-Output 'FAIL build: cargo build --release failed'; exit 1 }
...
    foreach ($needle in $present) { Write-Output "FAIL TC-BUILD-01: release binary contains '$needle'" }
    exit 1
...
Write-Output ('PASS TC-BUILD-01: none of {0} present in release liam.exe' -f ($forbidden -join ', '))
...
    Write-Output 'NOTE TC-BUILD-02: dumpbin unavailable; import list not recorded'
    exit 0
```
Rules to copy: one `PASS|FAIL|NOTE <TC-ID>: <reason>` line per check, `exit 1` on first failure, content-free text only (counts and hashes, never paths with SIDs, party values or question text).

**Tool discovery fallback** (lines 27-38): `Get-Command` first, then search known roots with `-ErrorAction SilentlyContinue`. Reuse for `procmon64` (`C:\tools\bin\procmon64`) and `logman`.

---

### `scripts/start06-acl-harness.ps1` (measurement, file-I/O)

**Analog:** `scripts/make-stub-records.ps1` for the record shape and the read-only attribute that WR-05 is about; `scripts/measure-build.ps1` for output style.

**Parameter block pattern** (make-stub-records.ps1 lines 1-7):
```powershell
param(
    [Parameter(Mandatory = $true)] [string] $OutputDirectory,
    [Parameter(Mandatory = $true)] [int] $LiamPid,
    ...
)
$ErrorActionPreference = "Stop"
```

**Read-only attribute use (the WR-05 behaviour to contrast)** (lines 54-55):
```powershell
(Get-Item -LiteralPath (Join-Path $OutputDirectory "receipt.json")).IsReadOnly = $true
(Get-Item -LiteralPath (Join-Path $OutputDirectory "agent.json")).IsReadOnly = $true
```
The harness must do the opposite: clear the attribute, grant write by ACL (`icacls /save` first, `/restore` in `finally`), and expect `liam: 10 receipt:writable` / `agent_record:writable`. The reason codes come from `src/receipt.rs` `ensure_not_writable` (lines 103-119):
```rust
pub fn ensure_not_writable(path: &Path, field: &str) -> Result<(), StartupError> {
    match OpenOptions::new().write(true).open(path) {
        Ok(_) => Err(if field == "receipt" { receipt_error("writable") } else { agent_error("writable") }),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        ...
```
For the real-record check (D-06/D-07), record only "deny-write ACE present: yes/no" and "read-only attr: yes/no". Keep SDDL, SIDs and paths local.

**Restore pattern:** take backup before change, restore in `finally` (see the elevated example below; the same shape works without elevation on files the user owns).

**Process-level reference:** `tests/startup_contract.rs` mutates a record pair and asserts the one-line refusal. Imports at lines 30-33, assertion helper `assert_refused(code, reason)` at line 119. Copy the expected-output convention, not the code (the harness is PowerShell against the real release exe).

---

### `scripts/e2e-direct-launch.ps1` (TC-E2E-03)

**Analog:** `scripts/launch-under-nono.ps1` (plus measure-build output style).

**Parameters, guards and invocation** (lines 1-21):
```powershell
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Party,
    [Parameter(Mandatory)][string]$Question,
    [Parameter(Mandatory)][string]$Purpose,
    [string]$Profile = "profiles/liam.nono.toml",
    [string]$Config = "liam.toml"
)

$ErrorActionPreference = 'Stop'
if (-not (Get-Command nono -ErrorAction SilentlyContinue)) {
    throw 'nono executable was not found on PATH.'
}
if (-not (Test-Path -LiteralPath $Profile)) { throw "Profile not found: $Profile" }
...
nono run --profile $Profile --purpose $Purpose -- `
    .\target\release\liam.exe --purpose $Purpose --party $Party --question $Question
```
For E2E-03, run `.\target\release\liam.exe --purpose ... --party ... --question ...` directly (no nono), capture the exit code and the single stderr line, and assert exit is 10 or 11 and stderr matches `^liam: (10|11) `. Use fixed dummy party and question values. Run from repo root (the script uses a relative `.\target\release\liam.exe`). Note Pitfall 1: `nono run` in the installed 0.66.1 has no `--purpose`; if the pinned build differs, this wrapper is itself a contract mismatch to record, not to rewrite silently.

---

### `scripts/e2e-wfp-stop.ps1`, `scripts/trace-procmon.ps1`, `scripts/trace-dns-etw.ps1` (elevated, user-run)

**No codebase analog.** No existing script elevates, stops services or runs traces. Use the RESEARCH.md patterns:

- Elevated try/finally with before/after state compare (RESEARCH.md "Pattern: elevated script with restore", lines 133-142). Add D-13: on any difference print exact restore commands, `exit 1`, no auto retry. Never stop BFE.
- Procmon CLI (RESEARCH.md lines 215-218): `/AcceptEula /Quiet /Minimized /BackingFile`, `/Terminate`, `/OpenLog ... /SaveAs`. Verify with `procmon64 /?` (A3).
- ETW (RESEARCH.md lines 206-211): `logman create trace ... -ets`, `logman stop ... -ets`. NET-08 needs a positive control (`nslookup` in a normal shell must show up in the trace). Verify provider with `logman query providers "Microsoft-Windows-DNS-Client"` (A1).

Adopt from `measure-build.ps1`: `$ErrorActionPreference = 'Stop'`, `Set-Location` to repo root, PASS/FAIL/NOTE lines. Write raw output only under `evidence-local/`.

---

### `scripts/analyze-trace.ps1` and `scripts/nono-diff-probe.ps1` (non-elevated, transform)

**Analog:** `scripts/measure-build.ps1` (byte read and hash logic, PASS/FAIL output).

**Core transform pattern** (lines 9-24): read a file, compute a decision from a fixed list, print one line per finding, `exit 1` on failure. For `analyze-trace.ps1`, the fixed list is the D-10 allowlist and the output is counts only: child processes, out-of-allowlist paths, DNS queries from Liam's PID, plus `Get-FileHash -Algorithm SHA256` of each raw capture. Pitfall 3: list residual `ACCESS_DENIED` events in the output rather than excluding them. Pitfall 2: filter post-hoc by process name `liam.exe`, then by its PID; the nono parent spawning Liam is not counted.

For `nono-diff-probe.ps1`: print versions and SHA-256 of installed CLI, service and driver, and field names only from the records. No field values.

**Record field shapes to diff against** (make-stub-records.ps1 lines 29-50):
```powershell
$receipt = [ordered]@{
    schema_version = 2; session_id = ...; pid = ...; entry_path = "DirectCli"
    token_arm = $TokenArm; outcome = "Ran"; layers = $layers }
$agent = [ordered]@{
    schema_version = 1; session_id; agent_id = "liam"; purpose; key_id
    key_expires_at_unix; injected_on = @("hometown","olive"); direct_gateway = $false
    profile_version = 4; proxy_port; written_at_unix }
```
The 14 layer ids are in `make-stub-records.ps1` lines 12-27 and in `src/expectancy.rs` (the table the O7 census is compared against).

---

### `.planning/phases/02-real-nono-session/02-SIGNOFF.md` (gate doc)

**Analog:** `.planning/phases/01-v0-4-1-conformance/01-HUMAN-UAT.md` (frontmatter + numbered sections with `expected:` and `result: [pending]`).

**Pattern** (lines 1-17):
```markdown
---
status: partial
phase: 01-v0-4-1-conformance
...
---
## Tests
### 1. TC-ACT-03: ...
expected: ...
result: [pending]
```
Adapt to two sections: `## 1. Contract diff (NONO-01)` and `## 2. O7 token_arm and census (NONO-02)`. Each holds: measured values, nono commit, receipt version, `ver`+UBR, date, `approval: [pending]`. No plan proceeds past a gate until its section says approved. Content-free: field names, counts and layer statuses only.

---

### `RESULTS.md` rows (modify)

**Analog:** existing rows. Columns: `Claim | TC | n of n | Tag | Windows build | Liam commit | nono commit | Limits / not measured | Status`.

**Current rows to replace** (RESULTS.md lines 61-62, 76-77, and the TC-START-06 / TC-NET-08 rows):
```
| Zero child processes (real nono process trace) | TC-SEC-01 | 0 of 0 | not run | n/a | n/a | not available | Requires real nono session and process trace. ... | not measured |
| End-to-end check | TC-E2E-03 | 0 of 0 | not run | n/a | n/a | not available | Needs real nono launch (direct-launch exit 10/11). Manual-hardware tag. | not measured |
```
**Filled-row convention** (from measured rows):
```
| Valid pair runs, run record written | TC-START-01 | 1 of 1 | local development | 10.0.26200.9711 (cmd /c ver; UBR 9711) | d573907 | n/a (stubs) | <limits> | verified |
```
For Phase 2: replace `n/a (stubs)` with the pinned nono commit, add receipt version in Limits, re-probe `ver` and UBR at measurement, use the real Liam commit, put counts and capture SHA-256 in Limits. TC-E2E-06 row stays `not measured` with a note that it moved to Phase 3. Commit RESULTS as a docs-only commit after measurement. TC-START-06 row's WR-05 limits text should be kept and updated, not dropped.

---

### `contracts/receipt-fields.md` and `README.md` (change control on mismatch)

**Analog:** the files themselves.

`contracts/receipt-fields.md` structure: header with comparison source commit and `Status: **STOP FOR REVIEW.**`, a "Resolved by NONO v0.4.3 / Liam v0.4" section, then a field table `| Current field | Liam use | Comparison |`. On a NONO-01 mismatch, add a dated section for the pinned commit in the same table form (field, Liam use, comparison) and set the status line back to stop for review. Update the README in the same change, then STOP (D-05); do not touch `src/`.

`README.md` "Known gaps (deferred to §4 re-ratification)" (lines 134-150): WR-05 and WR-06 entries are bullets with the code location and the `NONO_v0.4.3` O-number. Add any D-07 finding (records rely on the read-only attribute only) as a measured note to WR-05, keeping the "code deliberately unchanged until re-ratification" wording.

---

### `.gitignore` (modify)

**Analog** (current contents):
```
runs/*
!runs/.gitkeep
stubs/records/generated/
/target/
```
Add `evidence-local/` (own line, with a comment in the style of line 1: `# Local configuration and evidence are never committed.`) before the first capture.

---

### `liam.toml` (modify, only after Gate 2 approval)

Single edit:
```toml
[nono]
profile_min_version = 4
record_max_age_seconds = 120
record_public_key = ""
expected_token_arm = ""
```
Set `expected_token_arm` only to the value approved in 02-SIGNOFF.md section 2. Keep `record_public_key = ""` (non-empty is refused as `receipt:signature`). Base URLs remain `*.invalid`, so after the self-check a run may exit 21; that still counts as passing the startup self-check (Pitfall 9).

## Shared Patterns

### One reason line, documented exit code
**Source:** `src/main.rs` `fail(code, reason)`; documented in CLAUDE.md. Expected stderr format `liam: {code} {reason}`; codes 10 receipt/agent record, 11 config/args/vocab/proxy, 21 Hometown, 22 Olive, 30 evidence. Every harness assertion should match this exact shape and nothing else on stderr.

### Content-free evidence
**Apply to:** all scripts, SIGNOFF, RESULTS. Record counts, statuses, field names, versions, commits and SHA-256 only. Use fixed dummy `--party` and `--question`. No SIDs, SDDL, machine paths, `netsh wfp` XML, proxy env values (port number only if needed), or Procmon rows in committed files. Raw captures live in gitignored `evidence-local/`.

### Run from repo root, release build, no stubs
**Source:** `scripts/launch-under-nono.ps1` line 20-21 and `measure-build.ps1` line 4. Build with `cargo build --locked --offline --release`; the stubs feature and `LIAM_STUBS`/`LIAM_STUB_PID` must not be in play. `scripts/measure-build.ps1` TC-BUILD-01 already proves the release binary is stub-free; rerun it at the recorded Liam commit.

### No credentials, no Claude self-elevation
Scripts accept no secret parameters (`launch-under-nono.ps1` comment lines 12-13). Admin steps are user-run `! powershell -File scripts/<name>.ps1` (D-12).

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `scripts/e2e-wfp-stop.ps1` | elevated service-state script | event-driven | No existing script stops services or elevates. Use RESEARCH.md try/finally pattern plus D-13 state compare |
| `scripts/trace-procmon.ps1` | elevated capture | streaming | No existing tracing. Use RESEARCH.md Procmon CLI; verify flags with `procmon64 /?` |
| `scripts/trace-dns-etw.ps1` | elevated capture | streaming | No existing ETW. Use RESEARCH.md `logman` example; verify provider; add positive control |
| `nono-diff` source comparison | read-only diff of `~/Nono` source | transform | External repo, read-only; no Liam-side analog (Pitfall 1: grep for `AgentSessionRecord`, `NONO_RECEIPT_PATH`, `AgentKeyInjection`, check `agent_daemon/launch.rs`) |

## Metadata

**Analog search scope:** `scripts/`, `tests/`, `src/receipt.rs`, `contracts/receipt-fields.md`, `README.md`, `RESULTS.md`, `.gitignore`, `liam.toml`, Phase 1 planning docs.
**Files scanned:** about 14.
**Pattern extraction date:** 2026-10-21
**Planner notes:** Gate 1 (NONO-01) may stop the phase (RESEARCH.md Pitfall 1); write downstream plans conditional on the SIGNOFF section being approved. NET-08 uses ETW, not Procmon (RESEARCH.md Summary). Moving TC-E2E-06 to Phase 3 in ROADMAP.md and REQUIREMENTS.md should be a small first task.
