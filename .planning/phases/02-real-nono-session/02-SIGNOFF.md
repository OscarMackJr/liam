---
status: pending
phase: 02-real-nono-session
---

# Phase 2 Sign-off Gates

No later plan proceeds past a gate until its section contains the exact line `approval: approved`.
Entries hold field names, counts and layer statuses only. No values, SIDs or paths.

## 1. Contract diff (NONO-01)

nono commit: cf3ebc56e1074980236f89100145c86aa3f04928
receipt version:
ver + UBR: Microsoft Windows [Version 10.0.26200.9711] UBR 9711
date: 2026-10-22
measured values: probe exit 0, no FAIL. nono --version 0.70.0 matches pin. sha256 nono.exe 900f3492f97d41ada2e70a46d16572f8ec9801503d23c1e7af9003af7e95aa53; nono-wfp-service 4b320d0e9448e62dcfebb8f1f0ded95723411ef5a99b54c33bf036f9108f7763 (Running/Automatic, POC-signed, restarted after write); nono-agentd e4608b1f8c9ed987537bff75bd67db780a67c4df7a2b24ea669f9cef6f8953cd (Running/Automatic, NotSigned NOTE, restarted after write). Driver: NOTE, not located via registration (open item).
notes: Deviation (Rule 3 / plan assumption, user decision): nono services are a dev registration running from the nono build tree (target\release), not an MSI install under Program Files. No MSI is used; the user stops services, rebuilds and signs in place with the POC self-signed cert, then restarts them (scripts/nono-reinstall.ps1 -Stop / -Start). Pin evidence per binary: LastWriteTime after pin commit, Authenticode Valid or POC self-signed, service process started after binary write, nono --version equals pinned Cargo version. Program Files copy is unused. Re-pin (user decision): 4df47f0f was superseded because it captured a phase-126 WR-05 test perturbation in crates/nono/src/path.rs; pin is cf3ebc56 (phase-126 round-4 complete: WR-05 60bf200c, WR-03, IN-01..03). Known pre-existing Linux test failure (capability_ext removes_symlinked_file_grant) fails identically at a12c6f58 and is documented in the nono round-4 addendum.
approval: [pending]

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
