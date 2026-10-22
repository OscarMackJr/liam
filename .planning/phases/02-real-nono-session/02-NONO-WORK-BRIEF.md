# nono work brief: what Liam Phase 2 needs from nono

Date: 2026-10-22
From: Liam Phase 2, gate 1 (02-03), stopped by the nono owner
Measured against: `OscarMackJr/nono` `cf3ebc56e1074980236f89100145c86aa3f04928` (`nono` 0.70.0)
Audience: nono maintainer / nono Claude sessions

## Summary

Liam's Phase 2 cannot run against nono `cf3ebc56`. The specs nono must implement are already
**ratified** (2026-10-19). No new design is needed, only implementation:

| Spec (in `liam/specs/_agent_inputs/`) | Sections to implement | State at cf3ebc56 |
|---|---|---|
| `NONO_v0.4.1.md` Agent Key Injection | §3 agent profiles (`purpose_source = "launch_arg"`), §4 mint / inject / no silent re-mint / revoke / fail secure | Not present. No purpose launch argument, no vendored `vocabulary.yaml`, no agent-key exchange |
| `NONO_v0.4.2.md` Agent Key Injection Scope | §2 injection rule, §3 profile `inject_on` + §3.1 `direct_gateway` at mint | Not present (`inject_on`, `direct_gateway`: 0 hits) |
| `NONO_v0.4.3.md` Agent Session Record | §3–§9: 14th `LayerId::AgentKeyInjection`, receipt `schema_version = 2`, `KeyId`, `AgentSessionRecord`, `SessionEndRecord`, scan extension, storage + env vars, POC integrity | Not present. Receipt is schema 1 with 13 layers |

`NONO_v0.4.3` supersedes the receipt sections of v0.4.1 §5 and v0.4.2 §4; do not implement those.

## Gate 1 findings (field by field)

| Liam contract expects | nono cf3ebc56 | Spec that fixes it |
|---|---|---|
| receipt `schema_version` = 2 | 1 | v0.4.3 §3 |
| 14 layers, last `AgentKeyInjection` | 13 `LayerId` variants | v0.4.3 §3 (+ CLI `layer_registry.rs` mirror) |
| `AgentSessionRecord` schema 1 | absent | v0.4.3 §5 |
| `SessionEndRecord` | absent | v0.4.3 §6 |
| `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH` in child env | absent; receipt goes to machine-wide sink as `<session_id>.jsonl` | v0.4.3 §8 (`<session_id>.receipt.json`, `.agent.json`, `.end.json`) |
| purpose supplied at launch, validated against vocabulary | no `--purpose` on `nono run` or `nono agent launch` | v0.4.1 §3, §4.1 |
| agent key minted via popeye, injected by nono-proxy | not implemented | v0.4.1 §4, v0.4.2 §2–§3 |
| `session_id`, `token_arm` | present | no change |
| `HTTPS_PROXY` / `HTTP_PROXY` / `NO_PROXY` → loopback nono-proxy | emitted in proxy mode; not runtime-verified | no change expected; Liam re-verifies |

## Liam-side consumer facts (what Liam will check)

- `src/receipt.rs` refuses unless receipt `schema_version == 2` and agent record `schema_version == 1`.
- Agent record shape: `contracts/agent-record.schema.json` (all 11 v0.4.3 §5 fields required; `key_id` a UUID string).
- Liam reads exactly two env vars for the records: `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`; and the proxy vars above.
- Liam's `--purpose` must equal the agent record's `purpose` (one source of truth per run).
- Liam's launcher `scripts/launch-under-nono.ps1` calls `nono run --profile <p> --purpose <p> -- liam.exe ...`. If nono implements purpose on a different command (for example `nono agent launch`), say so; Liam updates its wrapper under change control. Liam will not guess.
- Both record files must be non-writable by the child (Liam checks; REQ-END-25).

## Open items nono should settle

- **Competing contract.** `crates/nono/src/session_credential.rs` scaffolds a "Nono↔Popeye Session Credential Contract v0.1" (`fiskroad/contracts/...`), with a stub issuance call. Decide whether v0.4.1's agent-key exchange replaces, extends or is that contract, before implementing either.
- `NONO_v0.4.3` O1 / Liam O7: which `token_arm` value an AppContainer launch on `EntryPath::DirectCli` records (Liam rejects `LowIlPrimary`, `BrokerLaunch`, `BrokerLaunchNoPty`). Liam Phase 2 plan 02-04 measures this and stops for sign-off.
- `NONO_v0.4.3` O3: exact receipt directory and ACL, documented in the fork.
- Popeye dependency: minting needs popeye's agent key exchange (`POPEYE_v0.4.1` §2) reachable. A stub is fine for nono CI (REQ-END-19/21 say "CI (stub)").
- Dev install note: on the owner's machine nono services run from `~/Nono/target/release`. Any rebuild needs `liam/scripts/nono-reinstall.ps1 -Stop`, build + `sign-poc-local.ps1`, then `-Start`.

## Done when (Liam resumes Phase 2)

1. nono commit implementing the above, clean tracked tree, tests for REQ-END-18..25 passing.
2. Liam re-pins to that commit (02-02 stop / build + sign / start, probe passes).
3. Liam re-runs 02-03; verdict `match` (or a mismatch the nono owner resolves) and gate 1 `approved`.

## Owner decisions (2026-10-22, relayed by the nono session)

- Tracked as nono Phase 126.1 "Agent-Key Session Contract" (nono ROADMAP commit `87279a8b`), discuss → plan → execute with review. Not yet built.
- The v0.4.1 agent-key exchange **replaces** the `session_credential.rs` v0.1 scaffold.
- Purpose stays on `nono run --profile <p> --purpose <p>`, matching `scripts/launch-under-nono.ps1`. The nono session will warn before any change lands.
- `4df47f0f` must never be pinned (WR-05 perturbation). Clean heads are `cf3ebc56` and later.
