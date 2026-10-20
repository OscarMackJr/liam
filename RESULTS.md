# Liam POC Results

Results are measurements, not extrapolations. The owner-authorized stub
contracts in force are **S-HT-1** and **S-OL-1**. No live service, nono, or
managed-endpoint claim is made here.

| Claim | TC | n of n | Tag | Windows build | Liam commit | nono commit | Limits / not measured | Status |
|---|---|---:|---|---|---|---|---|---|
| Offline compilation, unit contracts and lint | TC-BUILD-01 (partial) | 1 of 1 | local development | not recorded | uncommitted | not available | `cargo fmt -- --check`, 20 offline tests, clippy with warnings denied, and release build; Windows path canonicalization warning remains environmental | verified |
| Static-CRT release import audit | TC-BUILD-01 (partial) | 1 of 1 | local development | not recorded | uncommitted | not applicable | Fresh `dumpbin /imports` found no `VCRUNTIME*.dll` or UCRT import; system socket/DNS imports remain a separate design checkpoint | verified |
| Receipt/agent-record validation logic | TC-START-01, 03–05, 09–16 (partial) | 7 unit tests | local development | not recorded | uncommitted | fixture only | No ACL, DSSE, real nono or process harness | verified |
| Content-free evidence sentinel, matching log hash, and evidence-write ordering | TC-EVD-01/-02/-03 (partial) | 3 unit/process tests | local development | not recorded | uncommitted | not applicable | Evidence-path failure prints the answer then exits `30 evidence:write`; does not cover external file ACL | verified |
| Proxy-only service requests and proxy-side credential injection | TC-NET-01, TC-KEY-01 (partial) | 2 integration tests | local development | not recorded | uncommitted | S-HT-1/S-OL-1 local stubs; not a real nono proxy or firewall test | verified |
| One answer / at-most-one proposal process flow | TC-META-02, TC-ACT-01/-06 (partial) | 2 process tests | local development | not recorded | uncommitted | S-HT-1/S-OL-1 local stubs only | verified |
| Retryable Hometown failures | TC-FAIL-01 (partial) | 4 integration/process tests | local development | not recorded | uncommitted | Confirms exactly one retry after a 500; repeated 500 and timeout exit `21 hometown:unavailable` after exactly two Hometown requests, with no Olive call | verified |
| Failed startup produces no evidence | TC-EVD-05, TC-FAIL-07 (partial) | 1 process test | local development | not recorded | uncommitted | Covers missing receipt only | verified |
| Proxy path and header injection | TC-NET-01, TC-KEY-01 | 0 of 0 | ci-elevated | — | — | — | Capturing proxy/stub integration harness not yet run | not measured |
| Non-writable records / real nono | TC-START-06 | 0 of 0 | dev-standard-user | — | — | — | Requires nono records and Windows ACL harness | not measured |
| Endpoint detection and sandbox controls | TC-NET-02, TC-E2E-02, TC-E2E-07 | 0 of 0 | dev-standard-user/manual-hardware | — | — | — | Requires implemented nono/popeye pair and managed endpoint | not measured |
| End-to-end ledger/ATE/proposal joins | TC-E2E-01, -05 | 0 of 0 | manual-hardware | — | — | — | A3/A4 implementation confirmation pending; stubs in force | not measured |
