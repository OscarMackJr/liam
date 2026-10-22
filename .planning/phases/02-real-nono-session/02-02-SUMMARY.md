---
phase: 02-real-nono-session
plan: 02
subsystem: nono-pin
tags: [nono, pin, probe, reinstall, dev-registration]
requires: [02-01]
provides:
  - scripts/nono-diff-probe.ps1
  - scripts/nono-reinstall.ps1
  - nono pin cf3ebc56 recorded in 02-SIGNOFF.md
affects: [02-03 through 02-08]
key-files:
  created:
    - scripts/nono-diff-probe.ps1
    - scripts/nono-reinstall.ps1
  modified:
    - .planning/phases/02-real-nono-session/02-SIGNOFF.md
decisions:
  - Nono services are a dev registration running from the nono build tree, not an MSI install; reinstall is stop, build and sign, start
  - Pin re-pinned from 4df47f0f to cf3ebc56e1074980236f89100145c86aa3f04928 (phase-126 round 4 complete)
metrics:
  duration: multi-session
  completed: 2026-10-22
requirements: [NONO-01]
---

# Phase 2 Plan 02: Pin and Reinstall Summary

The nono build is pinned at cf3ebc56 (v0.70.0), the CLI and both services were rebuilt, POC-signed and restarted from it, and the non-elevated probe passes with no FAIL line.

## Tasks

| Task | Commit | Result |
|------|--------|--------|
| 1 | n/a (user decision) | Pin first 4df47f0f, then re-pinned to cf3ebc56 because 4df47f0f carried a phase-126 WR-05 test perturbation in crates/nono/src/path.rs; cf3ebc56 completes phase-126 round 4 |
| 2 | e74a3f1, a7f546b, 286a4ff, b5bbdb6, e2275a8 | Probe and reinstall scripts; rework to dev registration; SIGNOFF re-pin; path fix; agentd signature demoted to NOTE |
| 3 | user-run | reinstall -Stop, build and sign, reinstall -Start all PASS; final probe exit 0 |

## Probe evidence (final run)

- PASS pinned commit equals HEAD, tracked tree clean
- nono.exe on PATH resolves to the build tree release output; `nono --version` 0.70.0 matches the pinned Cargo version
- nono.exe sha256 900f3492f97d41ada2e70a46d16572f8ec9801503d23c1e7af9003af7e95aa53, built after the pin, POC-signed
- nono-wfp-service Running/Automatic, sha256 4b320d0e9448e62dcfebb8f1f0ded95723411ef5a99b54c33bf036f9108f7763, built after the pin, POC-signed, restarted after write
- nono-agentd_1838b8 Running/Automatic, sha256 e4608b1f8c9ed987537bff75bd67db780a67c4df7a2b24ea669f9cef6f8953cd, built after the pin, NotSigned (NOTE), restarted after write
- Driver: NOTE, not located via registration
- Windows 10.0.26200.9711

## Deviations from Plan

1. [Rule 3 - Plan assumption] The plan assumed an MSI install under Program Files. The real install is a dev registration running from the nono build tree. Per user decision there is no MSI; the flow is reinstall -Stop, cargo build and sign-poc-local, reinstall -Start. Scripts were reworked (a7f546b).
2. [User decision] Re-pin from 4df47f0f to cf3ebc56 (see Tasks).
3. nono-agentd is not signed by nono's own sign-poc-local.ps1, so the probe reports its signature as a NOTE, not a FAIL (e2275a8).
4. The driver could not be located via Win32_SystemDriver registration; recorded as NOTE and remains an open item.
5. sign-poc-local's MSI step fails with an absolute -OutputDir (nono script bug, irrelevant here; no MSI is used).
6. The user put the nono release directory ahead of Program Files on the machine PATH so the pinned CLI is the one resolved.

A pre-existing Linux test failure in nono (capability_ext removes_symlinked_file_grant) fails identically at a12c6f58 and is noted in 02-SIGNOFF.md.

## Open items

- Driver not located by registration; NONO-01 stays open until 02-03 measures it.
- Section 1 `approval:` is left pending.

## Known Stubs

None.

## Self-Check: PASSED
