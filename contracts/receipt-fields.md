# Receipt-contract comparison checkpoint

Comparison source: `OscarMackJr/nono` commit `29557a93`, supplied as
`specs/_agent_inputs/receipt.rs`, `attestation.rs`, and
`receipt_content_free_scan.rs`.

Status: **STOP FOR REVIEW.** The current nono receipt cannot satisfy Liam
v0.3 §4.2. The expected resolution is a NONO v0.4.3 amendment plus a matching
Liam §4.2 rewrite; Liam must not infer or invent an alternate record.

## Phase 2 gate 1 diff: pinned commit `cf3ebc56` (2026-10-22)

Status: **STOP FOR REVIEW.** Verdict MISMATCH. Comparison source: read-only
reading of `OscarMackJr/nono` at `cf3ebc56e107` (full hash in `.planning/phases/02-real-nono-session/02-SIGNOFF.md`)
(`nono --version` 0.70.0), including the `nono agent` / `nono-agentd` launch
path. Liam source is not adapted and nono is not patched.

| Current field | Liam use | Comparison |
|---|---|---|
| `schema_version: u16` (value 1) | version handling | Contract expects 2. Different. |
| `session_id`, `pid`, `entry_path`, `outcome` | join / none | Present, as before. |
| `token_arm: Option<TokenArm>` | confinement check | Present (`Null`, `WriteRestricted`, `LowIlPrimary`, `BrokerLaunch`, `BrokerLaunchNoPty`); same shape as `29557a93`. |
| `layers` (13 rows) | confinement check | 13 `LayerId` variants; `AgentKeyInjection` absent, so Liam's 14-ID table cannot be satisfied. |
| `AgentSessionRecord` (schema 1) | agent facts | Absent from all crates. |
| `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH` | record discovery | Not set anywhere. Receipt is written to a machine-wide sink as `<session_id>.jsonl`. |
| `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY` | proxy env | Emitted by nono-proxy (plus `NONO_NO_PROXY`, `NONO_PROXY_TOKEN`) only for proxy-mode runs; not verified at runtime. |
| `--purpose` | launch wrapper | Absent on `nono run` and `nono agent launch` (`--profile` only). |

## Resolved by NONO v0.4.3 / Liam v0.4

The v0.4.3 package is ratified. It retains the receipt's startup-only,
content-free, policy-free invariants; changes its schema to `2`; adds the
fourteenth `AgentKeyInjection` census row; and introduces a separate,
supervisor-written `AgentSessionRecord` for the agent facts. Liam v0.4 now
parses both records and owns the reviewed DirectCli/AppContainer expectancy
table. The original comparison below remains the reason for this design.

## Current receipt shape

`EnforcementReceipt` contains only:

| Current field | Liam use | Comparison |
|---|---|---|
| `schema_version: u16` | version handling | Name/type differ from `receipt_version: string`. |
| `session_id: String` | run-record join | Compatible opaque session identifier. |
| `pid: u32` | none | Not required by Liam. |
| `entry_path: EntryPath` | none | Not the profile identity required by Liam. |
| `token_arm: Option<TokenArm>` | confinement check | Not `arm=appcontainer`; AppContainer is a layer. |
| `outcome: SessionOutcome` | start-state check | Liam v0.3 does not specify this required check. |
| `layers: Vec<LayerReceiptRow>` | confinement check | Rows are `{ id, status }`, not `{ name, required, status }`. |

Layer status values are `Confirmed`, `EstablishedNotIndependentlyObservable`,
`Unconfirmed`, and `NotApplicable`. The receipt carries the exhaustive
13-entry `LayerId` census. It has no `required` flag; per-arm expectancy is
CLI policy, not receipt data.

## Differences requiring nono/Liam contract work

| Liam v0.3 §4.2 requirement | Current receipt result | Required disposition |
|---|---|---|
| `profile.name`, `profile.version` | Absent | Define an attested, content-free profile reference or revise Liam's contract. |
| `arm=appcontainer` | Absent; `token_arm` has `Null`, `WriteRestricted`, `LowIlPrimary`, `BrokerLaunch`, `BrokerLaunchNoPty` | Rewrite Liam's check around `AppContainerProfile=Confirmed` and permitted token-arm policy. |
| every required layer active | No `required`; four-state status vocabulary | Define Liam's arm-specific expectancy table and the acceptance mapping. |
| `agent_key` required layer | Absent from exhaustive `LayerId` census | Deliberate nono schema change required. |
| `agent.agent_id`, `purpose`, `key_id`, `key_expires_at`, `injected_on` | Entire object absent | Add a separate content-free agent-session record, rather than adding content-bearing fields to the startup receipt. |
| `direct_gateway` | Absent | Keep with the agent-session injection policy/record, not the startup receipt. |
| `egress.mode`, `egress.proxy_port` | Absent | Define separate attested egress/proxy facts or revise Liam's contract. |
| `started_at` | Absent | Define a content-free freshness mechanism or revise Liam's contract. |
| `integrity` | Absent | Define receipt-integrity/protected-path contract or revise Liam's contract. |
| end-of-session `agent.key_revoked` | Impossible by design | D-20 makes receipts immutable startup-only; record revocation separately at session end. |

## Invariants preserved by the current nono source

The receipt is a startup-only supervisor attestation written before the child
resumes. Its content-free scan permits only named structural types and one
owned-string exception: opaque `session_id`. Thus `agent_id`, `purpose`,
`on_behalf_of`, FQDNs, or arbitrary `key_id` text cannot be silently added to
this receipt. `on_behalf_of` is not needed by Liam and remains available from
the ledger.

## Ratified successor contract

The ratified successor is a separately written supervisor agent-session record
beside the receipt: purpose and service names are represented as enums,
`key_id` is a narrowly scoped opaque-ID exception, `on_behalf_of` is absent,
and a separate end-of-session record carries revocation. Liam v0.4 implements
the consumer-side schema-2/schema-1 validation. This report does not claim
that the pinned nono fork at `29557a93` implements the successor.

