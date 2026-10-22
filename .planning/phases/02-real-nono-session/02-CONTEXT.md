# Phase 2: Real nono Session - Context

**Gathered:** 2026-10-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Liam starts, runs its self-check and refuses correctly inside a real nono fork session on this dev machine, using a `token_arm` the nono owner has signed off (NONO-01..05). The phase has two stop gates: a contract diff of the fork's receipt, agent-session record and proxy environment at the start, and O7 (`token_arm` plus the §4.3 census measured on `DirectCli`) before `expected_token_arm` is set. There is no hometown, olive or popeye integration here; those are Phases 3 and 4.

**Scope adjustment agreed in discussion:** TC-E2E-06 (key exchange unreachable makes nono refuse the session) moves to Phase 3, because popeye's key exchange is not running on this machine yet. ROADMAP Phase 2 criterion 4 and NONO-04 still list it. The planner should update ROADMAP.md and REQUIREMENTS.md to move that case to Phase 3, and must not treat it as a Phase 2 gap.

</domain>

<decisions>
## Implementation Decisions

### nono build and stop-gate sign-off (NONO-01, NONO-02)
- **D-01:** "The fork" is a build of `~/Nono` (`OscarMackJr/nono`) at a commit the user pins. Every Phase 2 RESULTS row records that commit in its "nono commit" column, plus the receipt version and the Windows build (`ver` plus UBR), as test spec §2.3 requires. The contract diff reads the same source tree.
- **D-02:** `~/Nono` currently has uncommitted changes (`crates/nono-cli/src/deny_model.rs`, `crates/nono/src/path.rs`; HEAD `v3.6-957-g9945fed6`). The user commits or discards these in the nono repo and then names the commit to pin. Liam's plans never edit, commit to or stash in the nono repo; the first plan step is to record the commit the user gives and check that the nono tree is clean at it.
- **D-03:** The nono WFP service and driver must come from the same pinned build. The installed copy is `C:\Program Files\nono\` (`nono 0.66.1`, binaries dated Jul 6), so reinstall the service and driver from the pinned build (admin step, see D-11) and verify that the CLI, service and driver match before any measurement.
- **D-04:** The nono owner is the user. Each stop gate pauses the phase, and approval is recorded in `.planning/phases/02-real-nono-session/02-SIGNOFF.md`, one section per gate (contract diff; O7 `token_arm` and census). Each section holds the measured values, the nono commit, the date and the user's approval. No plan proceeds past a gate until its section says approved. `expected_token_arm` is set in `liam.toml` only from the approved O7 value, never a guessed or unreviewed one.
- **D-05:** A contract-diff mismatch (receipt, agent record or proxy environment against `contracts/receipt-fields.md`, design §4.2 and A5) follows the project change-control rule: update the contract table and README, then stop for review. Liam is not adapted to the fork.

### TC-START-06 and WR-05 (NONO-03)
- **D-06:** Measure TC-START-06 with the test spec's dev-standard-user harness. As a standard user, grant the test user write access on each record file through its ACL, with no read-only attribute set, and confirm `receipt:writable` / `agent_record:writable`. Also confirm that the real records nono writes pass as non-writable. WR-05 stays a separately documented gap (README "Known gaps").
- **D-07:** If the real nono records are "non-writable" only because of the read-only attribute (not a deny-write ACL), WR-05 becomes live: Liam would accept a file its own token can make writable. Then record exactly what nono does and STOP for nono-owner review; this feeds `NONO_v0.4.3` O3 (receipt directory and ACL). Do not change Liam's §4 behaviour without a re-ratification sign-off.
- **D-08:** The harness test user is the user's normal non-admin logon, which is the user nono launches Liam as. The planner first verifies that this session really is a standard user (not an elevated admin token). If it is not, stop and ask.

### Observation tooling (NONO-05: TC-SEC-01, TC-SEC-02, TC-NET-08)
- **D-09:** Use Sysinternals Process Monitor, scripted from the command line (`/BackingFile`, `/LoadConfig` with a PID filter on Liam, `/Quiet`, `/Minimized`, then export to CSV). One capture covers process create, file and registry access, and network events. Procmon needs admin to load its driver, so starting and stopping the capture is an elevated step the user runs (D-11).
- **D-10:** TC-SEC-02 "touches only its file grants" is judged by an allowlist diff. Build the allowlist from the `profiles/liam.nono.toml` grants plus the paths Liam legitimately reads: receipt, agent record, `liam.toml`, vendored vocabulary, catalog, `runs_dir`, and the exe's own image and DLL loads. Any path in the capture outside the allowlist fails the test and is listed in RESULTS. TC-SEC-01 passes on zero process-create events from Liam's PID. TC-NET-08 passes on zero DNS queries from Liam's PID.
- **D-11:** Evidence: RESULTS rows record counts (child processes = 0, out-of-grant paths = 0, DNS queries from Liam's PID = 0) plus the SHA-256 of each raw capture. Raw captures stay in a gitignored local folder (they may contain machine paths) and are never committed. Any committed summary must pass the content-free rules.

### Disruptive refusal cases (NONO-04)
- **D-12:** Admin-only steps run as small PowerShell scripts the user runs from an elevated prompt (for example `! powershell -File scripts/e2e-wfp-stop.ps1`). Examples: stopping the WFP service for TC-E2E-04, reinstalling the service and driver, and starting or stopping Procmon. Claude never self-elevates. Each script records the prior state (service status, start type, filters present), does its change, and restores in a `finally` block.
- **D-13:** After each disruptive script, the script compares the system state with what it recorded beforehand. On any difference (service won't restart, filter left behind, start type changed) it stops, prints the exact restore commands, and the phase waits for the user. There is no automatic retry.
- **D-14:** TC-E2E-03 (direct launch outside nono exits 10 or 11 with no network activity) and TC-E2E-04 (WFP service stopped, so the session is refused or Liam exits 10 on the census) are measured in Phase 2. TC-E2E-06 moves to Phase 3 (see Phase Boundary).

### Claude's Discretion
- The order of plans after the two stop gates, the exact Procmon filter and config files, and the script names and layout under `scripts/`.
- How the TC-START-06 harness grants and revokes ACLs (`icacls` or Win32), provided it runs as the standard user and restores the original ACL afterwards.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Governing specs (a disagreement is a stop condition)
- `specs/LIAM_DESIGN_SPEC_v0.4.1.md`: §4 (nono coupling: receipt, agent-session record, §4.2 field mapping, §4.3 expectancy table and census, §4.4 non-writable check and DSSE/acl-only, §4.5 what Liam does not do), §8.1 run record
- `specs/LIAM_TEST_SPEC_v0.4.1.md`: §2.3 (real nono: pinned fork commit, record receipt version and Windows build), TC-START-06 (dev-standard-user), TC-E2E-03/-04/-06, TC-SEC-01/-02, TC-NET-08
- `specs/_agent_inputs/NONO_v0.4.3.md`: receipt schema 2, `AgentSessionRecord`, §8 storage/ACLs/env vars, Ratification Record; O1 = Liam O7 (`token_arm`), O2 (DSSE), O3 (receipt directory and ACL). Changes here need re-ratification

### Liam contracts and config
- `contracts/receipt-fields.md`: the field table the NONO-01 diff compares against
- `contracts/agent-record.schema.json`: agent-session record schema
- `profiles/liam.nono.toml`: the nono profile (grants feed the D-10 allowlist; `profile_version = 4`)
- `liam.toml`: `expected_token_arm` (empty until O7 sign-off), `record_max_age_seconds`, `runs_dir`
- `scripts/launch-under-nono.ps1`: launch path for NONO-03 (runs `target/release/liam.exe`)

### Project records
- `README.md`: "Known gaps (deferred to §4 re-ratification)" (WR-05, WR-06)
- `RESULTS.md`: rows for TC-START-06, TC-E2E-*, TC-SEC-01/02, TC-NET-08 (currently not measured)
- `.planning/phases/01-v0-4-1-conformance/01-REVIEW.md`, `01-REVIEW-FIX.md`: WR-05/WR-06 detail
- nono fork source: `~/Nono` (`C:\Users\OMack\Nono`). Receipt and agent record are in `crates/nono/src/receipt.rs`, `crates/nono-cli/src/receipt_sink.rs`, `crates/nono-cli/src/agent_daemon/launch.rs`, `crates/nono-cli/src/exec_strategy_windows/{attestation,launch,layer_registry}.rs`. Read-only for Liam

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/receipt.rs`: `validate_startup(receipt, agent, policy, pid, now, port)` (a pure function) and `ensure_not_writable`; reason codes like `receipt:writable`, `receipt:token_arm`, `receipt:layer:*`
- `src/expectancy.rs`: the static 14-layer §4.3 table the O7 census is compared against
- `scripts/make-stub-records.ps1`: stub record pair generator, a reference for real-record field shapes
- `tests/startup_contract.rs`, `tests/hygiene_contract.rs`: process-level startup tests with mutated record pairs (patterns for the TC-START-06 harness)
- `scripts/measure-build.ps1`: precedent for a PowerShell measurement script that writes PASS/FAIL lines

### Established Patterns
- Prerequisite failures print exactly one `liam: <code> <reason>` line and exit before any network request
- `stubs` feature and `LIAM_STUBS`/`LIAM_STUB_PID` overrides exist only in stub builds; Phase 2 must run the release build (no `stubs`) under real nono
- RESULTS rows: n of n, true tag (`local development` unless really run elsewhere), Liam commit, nono commit, Windows build, limits; docs-only RESULTS commit after measurement

### Integration Points
- `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`, `HTTPS_PROXY`/`HTTP_PROXY`, `NO_PROXY` are supplied by nono; Liam reads nothing else from nono
- `liam.toml` `expected_token_arm` is the single place the O7 value lands after sign-off

</code_context>

<specifics>
## Specific Ideas

- The phase starts by asking the user for the pinned nono commit (D-02), then reinstalls the service and driver from it (D-03).
- Sign-off file: `.planning/phases/02-real-nono-session/02-SIGNOFF.md`, one section per gate, checked before continuing.
- Elevated steps go through `! <command>` prompts so their output lands in the session.

</specifics>

<deferred>
## Deferred Ideas

- TC-E2E-06 (key exchange unreachable): Phase 3, once popeye's key exchange runs locally.
- WR-05 / WR-06 Liam-side fixes: wait for `NONO_v0.4.3` re-ratification (O2, O3). Phase 2 only measures and, under D-07, may trigger the review.
- Not discussed and left to planning: the release-build `liam.toml` used under nono, the fallback if the O7 `token_arm` is rejected at sign-off (stop, as with any gate), and run IDs and party values for real-session runs.

</deferred>

---

*Phase: 02-real-nono-session*
*Context gathered: 2026-10-21*
