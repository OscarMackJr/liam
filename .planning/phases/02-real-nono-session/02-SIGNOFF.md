---
status: pending
phase: 02-real-nono-session
---

# Phase 2 Sign-off Gates

No later plan proceeds past a gate until its section contains the exact line `approval: approved`.
Entries hold field names, counts and layer statuses only. No values, SIDs or paths.

## 1. Contract diff (NONO-01)

nono commit: 4df47f0f4f6c4e1e3062d2d9b09ff633b5428770
receipt version:
ver + UBR: Microsoft Windows [Version 10.0.26200.9711] UBR 9711
date:
measured values:
notes: Deviation (Rule 3 / plan assumption, user decision): nono services are a dev registration running from the nono build tree (targetelease), not an MSI install under Program Files. No MSI is used; the user stops services, rebuilds and signs in place with the POC self-signed cert, then restarts them (scripts/nono-reinstall.ps1 -Stop / -Start). Pin evidence per binary: LastWriteTime after pin commit, Authenticode Valid or POC self-signed, service process started after binary write, nono --version equals pinned Cargo version. Program Files copy is unused.
approval: [pending]

## 2. O7 token_arm and census (NONO-02)

nono commit: 4df47f0f4f6c4e1e3062d2d9b09ff633b5428770
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
