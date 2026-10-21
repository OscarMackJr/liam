# Phase 1: v0.4.1 Conformance - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-21
**Phase:** 01-v0.4.1 Conformance
**Areas discussed:** Unchanged pass-through (D15), Error codes & run record, Gateway trap & static check, Vocab hash, profile, RESULTS

---

## Unchanged pass-through (D15)

| Option | Description | Selected |
|--------|-------------|----------|
| RawValue pass-through | Exact hometown bytes per field; validate a separate copy | ✓ |
| Value + preserve_order/arbitrary_precision | JCS-equal, not byte-equal | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-written, test-only JCS | Small RFC 8785 canonicaliser with ryu, no new crate | ✓ |
| Vendor a JCS crate | Needs a vendor bundle refresh | |

| Option | Description | Selected |
|--------|-------------|----------|
| JSON-Schema subset per action | `parameters_schema` + small validator; A2 stub change in README | ✓ |
| Simple type map | Simpler, further from olive v1 | |
| Ask the olive owner first | Stop CONF-01/TC-ACT-03 | |

| Option | Description | Selected |
|--------|-------------|----------|
| Keep party_ref check, record as deviation | Suppress-only, beyond §7.3 | ✓ |
| Drop it | Match §7.3 exactly | |
| Keep, raise spec issue | Propose adding it to §7.3 | |

**User's choice:** All recommended options.

---

## Error codes & run record

| Option | Description | Selected |
|--------|-------------|----------|
| Strict pattern, else status | `^[a-z][a-z0-9_]{0,63}$` | ✓ |
| Known codes only | Would lose `rate_limited` | |
| Any string as-is | Content injection risk | |

| Option | Description | Selected |
|--------|-------------|----------|
| Add `reason` to run record | Additive §8.1 field | ✓ |
| Log event only | Keep §8.1 exact | |

| Option | Description | Selected |
|--------|-------------|----------|
| Log flag only (`investigate: true`) | Exit 0, stderr empty | ✓ |
| Log flag + stderr notice | More visible | |

| Option | Description | Selected |
|--------|-------------|----------|
| HTTP status on log event | Run record identical; `http_status` 200/202 | ✓ |
| No marker | Indistinguishable from 202 | |

**User's choice:** All recommended options.

---

## Gateway trap & static check

| Option | Description | Selected |
|--------|-------------|----------|
| Env var under `stubs` | `LIAM_STUB_GATEWAY_ATTEMPT=1` | ✓ |
| CLI flag under `stubs` | Feature-dependent arg path | |

| Option | Description | Selected |
|--------|-------------|----------|
| In stubs.rs via Liam's one client | `gateway.invalid`, refused by test proxy | ✓ |
| Separate test binary | Weaker proof | |

| Option | Description | Selected |
|--------|-------------|----------|
| Rust test in cargo test | `tests/static_contract.rs` | ✓ |
| PowerShell script in CI | Not in cargo test | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hosts and paths only | Comments allowed | ✓ |
| Any "gateway" token | Noisier | |

**User's choice:** All recommended options.

---

## Vocab hash, profile, RESULTS

| Option | Description | Selected |
|--------|-------------|----------|
| I'll supply the canonical file | CONF-11 blocking task until it arrives | ✓ |
| Record stop, defer CONF-11 | Placeholder stays, labelled | |
| Placeholder is actually canonical | Spec correction | |

| Option | Description | Selected |
|--------|-------------|----------|
| Integer 4, still marked draft | Header updated, nothing else changed | ✓ |
| Only record the mismatch | File untouched | |

| Option | Description | Selected |
|--------|-------------|----------|
| One row per TC ID | Split grouped rows | ✓ |
| Keep grouped claims | Compact | |

| Option | Description | Selected |
|--------|-------------|----------|
| Code commit, then docs commit | Clean tree, short hash, Windows build, true tag | ✓ |
| Also require a CI run | CI pushes in Phase 1 | |

**User's choice:** User supplies the canonical vocabulary; all other recommended options.
**Notes:** Scouting found both vendored and handoff-source vocabulary files hash to `f3412d68…`.

---

## Claude's Discretion

- main.rs refactor for error mapping and retry scope
- Error enum shape
- Placement of the gateway attempt in the flow
- Stub/contract/README updates for CONF-05
- Test layout

## Deferred Ideas

- Reject non-https `base_url`
- Replace hand-rolled vocabulary parser
- Exe-relative config paths / `%LOCALAPPDATA%` fallback
