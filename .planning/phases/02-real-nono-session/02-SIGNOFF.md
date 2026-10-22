---
status: pending
phase: 02-real-nono-session
---

# Phase 2 Sign-off Gates

No later plan proceeds past a gate until its section contains the exact line `approval: approved`.
Entries hold field names, counts and layer statuses only. No values, SIDs or paths.

## 1. Contract diff (NONO-01)

nono commit: cf3ebc56e1074980236f89100145c86aa3f04928
receipt version: schema_version 1 (contract expects 2)
ver + UBR: Microsoft Windows [Version 10.0.26200.9711] UBR 9711
date: 2026-10-22
verdict: MISMATCH
nono agent path checked: yes (agent_daemon launch source, `nono agent --help`, `nono agent launch --help`); `nono run --help` also checked.

| field | contract expects | fork shows | verdict |
|---|---|---|---|
| schema_version | 2 | present, value 1 | different |
| session_id | opaque id | present | match |
| token_arm | appcontainer-compatible arm | present, 5 variants (Null, WriteRestricted, LowIlPrimary, BrokerLaunch, BrokerLaunchNoPty) | match (O7 deferred to plan 05) |
| layers (count 14) | 14 | 13 LayerId variants | different |
| AgentKeyInjection | 14th layer row | absent | absent |
| AgentSessionRecord (schema 1) | separate record | absent from all crates | absent |
| NONO_RECEIPT_PATH | env var set for child | absent; receipt goes to machine-wide sink as <session_id>.jsonl | absent |
| NONO_AGENT_RECORD_PATH | env var set for child | absent | absent |
| HTTPS_PROXY/HTTP_PROXY/NO_PROXY | set to loopback proxy | emitted by nono-proxy in proxy mode (also NONO_NO_PROXY, NONO_PROXY_TOKEN); not runtime-verified | unverified |
| --purpose | on launch command | absent on `nono run` and on `nono agent launch` (agent launch has only --profile) | absent |

Wrapper note: scripts/launch-under-nono.ps1 uses `nono run --profile --purpose`; the fork exposes no --purpose, so the wrapper is itself a mismatch (recorded, not rewritten). Receipt is emitted on both run and daemon paths.
runtime session: not attempted. The fork source emits no schema-2 receipt or agent record, so a session could not show the contract fields; a run would write to the machine-wide sink and need the WFP service.
driver note: WFP filters are installed by nono-wfp-service; a kernel service nono-wfp-driver (data/windows/nono-wfp-driver.sys, copied by build.rs, removed by an MSI cleanup command) is referenced in source but the .sys is optional at build. Dev registration likely has no driver service, which explains the failed lookup.
change control: contracts/receipt-fields.md gained a dated section for this commit (status STOP FOR REVIEW); README.md note added; src, liam.toml, profiles, launch script untouched.
measured values (02-02): probe exit 0, no FAIL. nono --version 0.70.0 matches pin. sha256 nono.exe 900f3492f97d41ada2e70a46d16572f8ec9801503d23c1e7af9003af7e95aa53; nono-wfp-service 4b320d0e9448e62dcfebb8f1f0ded95723411ef5a99b54c33bf036f9108f7763 (Running/Automatic, POC-signed, restarted after write); nono-agentd e4608b1f8c9ed987537bff75bd67db780a67c4df7a2b24ea669f9cef6f8953cd (Running/Automatic, NotSigned NOTE, restarted after write). Driver: NOTE, not located via registration (open item).
notes: Deviation (Rule 3 / plan assumption, user decision): nono services are a dev registration running from the nono build tree (target\release), not an MSI install under Program Files. No MSI is used; the user stops services, rebuilds and signs in place with the POC self-signed cert, then restarts them (scripts/nono-reinstall.ps1 -Stop / -Start). Pin evidence per binary: LastWriteTime after pin commit, Authenticode Valid or POC self-signed, service process started after binary write, nono --version equals pinned Cargo version. Program Files copy is unused. Re-pin (user decision): 4df47f0f was superseded because it captured a phase-126 WR-05 test perturbation in crates/nono/src/path.rs; pin is cf3ebc56 (phase-126 round-4 complete: WR-05 60bf200c, WR-03, IN-01..03). Known pre-existing Linux test failure (capability_ext removes_symlinked_file_grant) fails identically at a12c6f58 and is documented in the nono round-4 addendum.
approval: stopped (2026-10-22, nono owner): MISMATCH against nono cf3ebc56 handed back to nono work or §4 re-ratification with NONO_v0.4.3; plans 02-04..02-08 not run

## 2. O7 token_arm and census (NONO-02)

nono commit: cf3ebc56e1074980236f89100145c86aa3f04928
receipt version:
ver + UBR: Microsoft Windows [Version 10.0.26200.9711] UBR 9711
date:
measured values:
approval: [pending]

## 3. Real-record writability (D-07, only if triggered)

nono commit:
receipt version:
ver + UBR:
date:
measured values:
approval: [pending]

## 4. Trace policy decisions (TC-SEC-02 loader denials)

nono commit:
receipt version:
ver + UBR:
date:
measured values:
approval: [pending]
