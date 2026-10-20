# Nono v0.4.3 — Agent Session Record

Version: 0.4.3
Status: **Ratified 2026-10-19**, jointly with the v0.4.3 package (see Ratification Record at the end).
Repository: nono (fork `OscarMackJr/nono`; place beside `NONO_v0.4.2.md`)
Amends: `NONO_v0.4.1.md` §5 (receipt additions) and `NONO_v0.4.2.md` §4 (receipt field). **Supersedes both receipt sections in full.** All other parts of v0.4.1 and v0.4.2 stand.
Audience: nono maintainer; popeye owner; Liam implementer.
Primary post-read action: implement §3–§9.

---

## 1. Why

`NONO_v0.4.1` §5 and `NONO_v0.4.2` §4 added an `agent` object and an `agent_key` layer to the per-session receipt. The Liam implementer's receipt-contract comparison against the fork at `29557a93` (`liam/contracts/receipt-fields.md`) showed that this collides with three invariants of the fork's `EnforcementReceipt` (`crates/nono/src/receipt.rs`):

- **D-20, startup-only.** The receipt is written once, before `ResumeThread`, and never carries a mid-session or end-of-session field. `agent.key_revoked`, set at session end, cannot live there.
- **D-05 / D-14, content-free.** `receipt_content_free_scan.rs` allows exactly one owned `String` field (`session_id`). `agent_id`, `purpose`, `on_behalf_of`, `key_id` and `injected_on` FQDNs as strings would fail it.
- **D-12, policy-free.** The receipt reports layer *status*; which layers an arm *requires* is CLI-side policy. The amendments assumed a `required` flag the fork deliberately does not carry.

These invariants are correct and are kept. This amendment moves the agent facts into two new supervisor-written records that obey the same rules, and adds one census row.

---

## 2. What Does Not Change

- `EnforcementReceipt` keeps every existing field, type and invariant (D-01, D-02, D-05, D-12, D-14, D-19, D-20).
- Minting, injection scope, `direct_gateway`, no fallback to the user key, no silent re-mint and revocation at session end are unchanged from v0.4.1 and v0.4.2. Only **where their facts are recorded** changes.
- nono stays semantics-blind.

---

## 3. One New Census Row: `LayerId::AgentKeyInjection`

Add a fourteenth `LayerId` variant, `AgentKeyInjection`, appended after `InterpreterCoverageGate`, mirrored in the CLI-side `layer_registry.rs` per the D-12 promotion rule (same name, same position in `ALL`).

| Session type | Status |
|---|---|
| Agent profile, exchange returned a valid agent key matching the profile (`NONO_v0.4.1` §4.4 checks passed) | `Confirmed` — the "confirmed-by-report-from-the-enforcing-component" sub-case already defined in `attestation.rs` |
| Agent profile, exchange failed or returned a non-conforming key | `Unconfirmed`, with `outcome = Refused` (fail secure, no child resumed) |
| Non-agent profile | `NotApplicable` |

`EnforcementReceipt.schema_version` becomes **2**. The census-completeness test (`layer_id_all_has_thirteen_entries_in_declaration_order`) is updated to fourteen. This is the only change to the receipt.

---

## 4. `KeyId` Type

```rust
/// Opaque agent-key identifier issued by popeye (POPEYE_v0.4.3: a UUID).
/// Content-free by construction: it can only hold 16 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyId(pub [u8; 16]);
```

Serialized as a lowercase canonical UUID string (`8-4-4-4-12`), deserialized only from that form; any other input is a parse error. Serialization as a JSON string, not a number, avoids precision loss in consumers that parse numbers as IEEE-754 doubles.

---

## 5. `AgentSessionRecord` (new, startup-only)

Written by the supervisor at the same attestation gate as the receipt, before `ResumeThread`, **only for agent profiles whose session is resumed** (`outcome = Ran`). D-19 applies: every field comes from the supervisor's own observation or from popeye's exchange response, never from the confined process.

```rust
pub struct AgentSessionRecord {
    pub schema_version: u16,          // 1
    pub session_id: String,           // equals the receipt's session_id (existing D-05 exception)
    pub agent_id: AgentId,            // enum of registered agents
    pub purpose: PurposeId,           // enum of the seven vocabulary purposes
    pub key_id: KeyId,                // from popeye's exchange response
    pub key_expires_at_unix: u64,     // popeye's granted expiry, seconds since epoch (UTC)
    pub injected_on: Vec<ServiceId>,  // resolved inject_on set, no FQDNs
    pub direct_gateway: bool,         // value sent at mint (NONO_v0.4.2 §3.1)
    pub profile_version: u32,         // the agent profile's version
    pub proxy_port: u16,              // nono-proxy's loopback port for this session
    pub written_at_unix: u64,         // supervisor clock at write time
}

pub enum AgentId { Liam }
pub enum PurposeId { Servicing, Origination, Collections, Analytics, Marketing, ComplianceReview, InternalSupport }
pub enum ServiceId { Gateway, Hometown, Olive }
```

Serialization: enums as lowercase kebab-case strings matching the vocabulary (`compliance-review`, `internal-support`); integers as JSON numbers (all values here are below 2⁵³).

**Deliberately absent:**

| Field | Why |
|---|---|
| `on_behalf_of` | Identity-shaped (a user principal name), not content-free. The ledger already carries it on every agent-key row (`POPEYE_v0.4` §2.3), joined by `key_id` |
| FQDNs | Host names are content-shaped. `ServiceId` carries the meaning; the FQDN mapping is profile policy |
| `required` flags, egress mode | Policy, not attestation (D-12). Egress enforcement is already attested by the `WfpEgressFilters` census row |
| `key_revoked` | End-of-session fact; see §6 |

**Enum drift guards.** A test asserts `PurposeId` matches the `purposes` list in nono's vendored `vocabulary.yaml` (`NONO_v0.4.1` §4.1 already vendors it). `AgentId` grows only by a nono release, which matches the agent-ID registry being a program decision (`POPEYE_v0.4.1` §5 O3).

---

## 6. `SessionEndRecord` (new, teardown)

Written by the supervisor when the session ends (normal or abnormal), after the revocation call. A separate file, so the receipt remains startup-only (D-20).

```rust
pub struct SessionEndRecord {
    pub schema_version: u16,   // 1
    pub session_id: String,    // existing D-05 exception
    pub key_id: KeyId,
    pub key_revoked: bool,     // false if revocation failed; the key still expires at key_expires_at_unix
    pub ended_at_unix: u64,
}
```

Agents never read it. It exists for the evidence chain (Liam TC-E2E-05) and operator review. A supervisor crash may leave no end record; that absence is itself the signal, and the key's expiry still bounds exposure (`NONO_v0.4.1` §4.3).

---

## 7. Content-Free Scan Extension

`receipt_content_free_scan.rs` is extended to scan all three structs, each against its own allowlist. The single owned-`String` exception (`session_id`) is unchanged for all three; **no new `String` exception is added.**

| Struct | Allowlisted types (beyond the `session_id: String` exception) |
|---|---|
| `EnforcementReceipt` | unchanged: `u16`, `u32`, `EntryPath`, `Option<TokenArm>`, `SessionOutcome`, `Vec<LayerReceiptRow>` |
| `AgentSessionRecord` | `u16`, `u32`, `u64`, `bool`, `AgentId`, `PurposeId`, `KeyId`, `Vec<ServiceId>` |
| `SessionEndRecord` | `u16`, `u64`, `bool`, `KeyId` |

The existing perturbation proof (a `PathBuf` field and a misnamed `String` field must fail) is repeated per struct.

---

## 8. Storage And Delivery

- All three records live in the directory the receipt already uses, named `<session_id>.receipt.json`, `<session_id>.agent.json`, `<session_id>.end.json`. Exact directory is the fork's existing choice.
- ACLs: writable by the supervisor only; readable by the confined child for `.receipt.json` and `.agent.json` (so the agent can self-check); `.end.json` need not be readable by the child.
- The confined child receives both paths in its environment: `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`. Agent-specific variable names (such as `LIAM_RECEIPT_PATH`) are retired; nono stays agent-neutral.

---

## 9. Integrity

The fork already has keyed DSSE signing for audit attestation bundles (`audit.rs`: `sign_audit_attestation_bundle`, `verify_audit_attestation_bundle`). **POC position:** records are protected by ACL only. If the supervisor has an audit signing key configured, it MAY additionally write `<session_id>.agent.dsse.json`, a DSSE envelope over the receipt and agent record together, using the existing signing path. Whether that becomes required is the open receipt-integrity question (`DECISION_desktop-agent-nono-integration.md` §7 Q2), not decided here.

---

## 10. Requirements

Supersedes REQ-END-11 (`NONO_v0.4.1`) and REQ-END-16 (`NONO_v0.4.2`).

| REQ | Requirement | TC | Test criteria | Method |
|---|---|---|---|---|
| REQ-END-11 (superseded) | — | — | Replaced by REQ-END-18 to -22 | — |
| REQ-END-16 (superseded) | — | — | Replaced by REQ-END-19 | — |
| REQ-END-18 | The receipt census includes `AgentKeyInjection` with the §3 status rules; `schema_version = 2` | TC-END-18 | Agent session: `Confirmed`; mint failure: `Unconfirmed` with `Refused`; non-agent session: `NotApplicable` | CI |
| REQ-END-19 | An `AgentSessionRecord` is written at the attestation gate for every resumed agent session, with the §5 fields, `session_id` equal to the receipt's | TC-END-19 | Record present and schema-valid; values match the exchange response and profile | CI (stub) |
| REQ-END-20 | No `AgentSessionRecord` is written for a refused session | TC-END-20 | Mint-failure run leaves a `Refused` receipt and no agent record | CI |
| REQ-END-21 | A `SessionEndRecord` is written at teardown with the revocation result | TC-END-21 | Normal end: `key_revoked=true`; forced revocation failure: `false` | CI (stub) |
| REQ-END-22 | All three structs pass the extended content-free scan with only the `session_id` exception | TC-END-22 | Scan passes; per-struct perturbation proofs fail as intended | CI |
| REQ-END-23 | `KeyId` serializes only as a lowercase canonical UUID string and rejects any other input | TC-END-23 | Round-trip; uppercase, braces, numeric and short inputs rejected | CI |
| REQ-END-24 | `PurposeId` matches the vendored vocabulary's `purposes` | TC-END-24 | Drift test against `vocabulary.yaml` | CI |
| REQ-END-25 | The child receives `NONO_RECEIPT_PATH` and `NONO_AGENT_RECORD_PATH`, and cannot write either file | TC-END-25 | Environment capture; write attempt from the child denied | dev-standard-user |

---

## 11. Open Items

- O1. Which `token_arm` value the fork records for an AppContainer launch on `EntryPath::DirectCli` (the `TokenArm` enum has no AppContainer variant). Liam v0.4 rejects `LowIlPrimary`, `BrokerLaunch` and `BrokerLaunchNoPty` and needs the accepted value confirmed (Liam O7).
- O2. Whether DSSE signing of the records becomes required (§9).
- O3. Exact receipt directory and its ACL, recorded in the fork's own docs.

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**: agent facts move from the receipt into supervisor-written records that keep the fork's invariants |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Ratified together with | `NONO_v0.4.3.md`, `POPEYE_v0.4.3.md`, `LIAM_DESIGN_SPEC_v0.4.md`, `LIAM_TEST_SPEC_v0.4.md` (one package) |
| Scope ratified | §2 (unchanged parts), §3 (`AgentKeyInjection` census row, receipt schema 2), §4 (`KeyId`), §5 (`AgentSessionRecord`), §6 (`SessionEndRecord`), §7 (content-free scan extension, no second `String` exception), §8 (storage, ACLs, agent-neutral environment variables), §9 (POC integrity position), §10 (REQ-END-18 to -25; REQ-END-11 and -16 superseded) |
| Not ratified here | Liam O7 / `NONO_v0.4.3` O1 (AppContainer `token_arm` value and the nono owner's review of Liam's expectancy table); `NONO_v0.4.3` O2 (whether DSSE becomes required) and O3 (receipt directory and ACL) |
| Effect | Supersedes the receipt sections of `NONO_v0.4.1` (§5) and `NONO_v0.4.2` (§4), which are annotated accordingly. Implementation may start |
| Change control | Any change to the three structs, the census row or the scan allowlists requires matching changes to Liam v0.4 §4.2–§4.4 and re-ratification of the package |
