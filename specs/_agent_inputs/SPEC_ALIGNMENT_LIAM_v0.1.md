# Spec Alignment Check — Liam POC v0.1

Date: 2026-10-19
Status: Pre-build check. Read before handing the Liam specs to an implementing agent.
Scope: every document the Liam build reads, checked seam by seam for agreement. A seam is **aligned** when each side's text says the same thing about the same field, owner and failure behaviour.

## 1. Document Set

| Document | Version | Provenance | Role for Liam |
|---|---|---|---|
| `DECISION_desktop-agent-nono-integration.md` | Ratified (rev 2) | Written 2026-10-19; revised and **ratified** 2026-10-19 | Boundary: contract-only |
| `LIAM_DESIGN_SPEC_v0.4.md` | 0.2 | Revised to 0.3 on 2026-10-19 (G5: hometown makes the model call) | What to build |
| `LIAM_TEST_SPEC_v0.4.md` | 0.2 | Revised to 0.3 on 2026-10-19 | How it is verified |
| `LIAM_AGENT_HANDOFF_PROMPT.md` | — | Revised for v0.2 set | Agent instructions |
| `PROGRAM_SHARED_VOCABULARY_v0.4.md` | 0.4 | **Reconstructed**; REQ table renumbered | Purposes, principals, outcomes |
| `vocabulary.yaml` | 0.4 | **Reconstructed**; tenant UUIDs are placeholders. Restore with `Restore-Vocabulary.ps1` | Vendored by Liam; hash checked at startup |
| `NONO_FISKROAD_INTEGRATION_v0.3.md` | 0.3 | Recovered verbatim | Profile and detection, nono side |
| `NONO_v0.4.md` | 0.4 | Recovered verbatim | Agent egress; partial-control statement |
| `NONO_v0.4.1.md` | 0.4.1 | Written and **ratified** 2026-10-19 | Agent-key minting, injection, receipt `agent` object |
| `POPEYE_CORRECTIONS_v0.3.md` | 0.3 | Recovered verbatim | Endpoint detection contract (§3) |
| `POPEYE_v0.4.md` | 0.4 | Recovered verbatim (header now recovered) | Agent principals, purpose label, run metering |
| `POPEYE_v0.4.1.md` | 0.4.1 | Written and **ratified** 2026-10-19 | Agent key exchange |
| `NONO_v0.4.2.md` / `POPEYE_v0.4.2.md` | 0.4.2 | Drafted and **ratified** 2026-10-19 | Agent key injected on hometown and olive; popeye introspection (closes G1 when ratified) |
| `DECISION_agent-key-service-scope.md` | Ratified | Written and **ratified** 2026-10-19 | Decision behind the v0.4.2 pair |
| `NONO_v0.4.3.md` / `POPEYE_v0.4.3.md` | 0.4.3 | Drafted and **ratified** 2026-10-19 with Liam v0.4 | Agent-session record, session-end record, `AgentKeyInjection` census row, `KeyId` as UUID; supersedes the receipt sections of v0.4.1/v0.4.2 |
| fork `receipt.rs`, `attestation.rs`, `receipt_content_free_scan.rs` | `29557a93` | Read 2026-10-19; compared in `liam/contracts/receipt-fields.md` | Real receipt shape and invariants (D-05, D-12, D-14, D-19, D-20) |
| `OLIVE_CHARTER_v0.4.md` | 0.4 | **Not in this set** | Proposal API (Liam assumption A2) |
| `HOMETOWN_v0.4.md` | 0.4 | Recovered verbatim 2026-10-19 | ATE v0.4, purpose masking, read path (Liam A1, O2) |

## 2. Seam Matrix

| # | Seam | Sides | Status | Evidence |
|---|---|---|---|---|
| S1 | Launch: purpose supplied at session start, same value to nono and Liam | Liam §3.2 · NONO_v0.4.1 §3–§4.1 | Aligned | Launcher passes one value; receipt `agent.purpose` checked against `--purpose` (Liam D11, TC-START-11) |
| S2 | Key minting: user key calls the exchange; `on_behalf_of` set by popeye | NONO_v0.4.1 §4.1 · POPEYE_v0.4.1 §2.1–§2.2 | Aligned | Request `agent_id`, `purpose`, `ttl`; response fields match on both sides |
| S3 | Key injection: agent key on gateway FQDN only, never the user key | NONO_v0.4.1 §4.2 · POPEYE_v0.4.1 §3 · Liam D4, §5.1 | Aligned | POPEYE_v0.4.1 §3 amends POPEYE_CORRECTIONS_v0.3 §3.1; NONO_v0.4.1 amends NONO_FISKROAD_v0.3 §2.1 for agent profiles |
| S4 | TTL: requested 3600 s, cap 8 h, Liam needs ≥ 120 s remaining | Liam §4.1, §4.2 · NONO_v0.4.1 §3 · POPEYE_v0.4.1 §2.2 | Aligned | 3600 < 8 h; no silent re-mint (NONO §4.3 ↔ Liam §5.1 no retry) |
| S5 | Fail secure on mint failure: no fallback to user key | NONO_v0.4.1 §4.4 · Liam §3.2 step 2 · TC-E2E-06 | Aligned | Session refused; Liam never starts |
| S6 | Receipt census, agent-session record, session-end record | fork `receipt.rs` · NONO_v0.4.3 §3–§7 · Liam v0.4 §4.2–§4.4 · test fixtures | Aligned (ratified) | Liam reads `agent_id`, `purpose`, `key_id`, `key_expires_at`; ignores `on_behalf_of`, `key_revoked` (both set by nono, neither needed for Liam's start decision) |
| S7 | Run identity: `run_id`, `nono_session_id` to hometown; `run_id`, `trace_ref` to olive | Liam v0.3 §5.2, §7.2 · HOMETOWN_v0.4 §2.1 | Aligned (stub contracts) | Identity and purpose come from the key; TC-META-01 |
| S8 | Ledger attribution of the hometown-relayed model call | POPEYE_v0.4.2 §2.6 · Liam REQ-LIAM-16, TC-E2E-05 | Aligned (ratified) | `relayed_by=hometown`; `key_id` joins ledger ↔ receipt; `primary_request_id` joins ATE ↔ ledger |
| S9 | Revocation at session end | NONO_v0.4.1 §4.3 · POPEYE_v0.4.1 §2.4 · Liam §3.2 step 5 | Aligned | Idempotent; failure recorded as `agent.key_revoked=false` |
| S10 | Purpose vocabulary and validation | vocabulary §2 · NONO_v0.4.1 §4.1 · POPEYE_v0.4.1 §2.2 · Liam TC-LVOC-01 | Aligned | Validated at three points (launch, mint, Liam start); vocabulary §2.3 updated to say purpose is bound at session launch |
| S11 | Detection events | POPEYE_CORRECTIONS_v0.3 §3.2 · NONO_FISKROAD_v0.3 §2.2 · NONO_v0.4 §2.2 · Liam TC-NET-02, TC-E2E-02 | Aligned | Schema unchanged by the amendments; mint failures are receipt events, not detections (NONO_v0.4.1 §6) |
| S12 | Coverage limit statement | NONO_v0.4 §3 · NONO_v0.4.1 §6 · Liam §9 | Aligned | Liam demos describe network-level plus filesystem confinement only |
| S13 | Onward delegation refused | vocabulary §3.2, REQ-VOC-10 · POPEYE_v0.4 §2.1 · POPEYE_v0.4.1 §2.2, REQ-AGT-05 | Aligned | REQ-VOC-10 now exists in the vocabulary table |
| S14 | Hometown and olive authentication of Liam's calls | Liam §6, §7 · NONO_v0.4.2 §2 · POPEYE_v0.4.2 §2 · HOMETOWN_v0.4 §4.2 | **Resolved at spec level (G1)** | v0.4.2 pair ratified 2026-10-19; implementation pending |
| S15 | ATE linkage of `run_id` and `nono_session_id` | Liam §8.3 · HOMETOWN_v0.4 §2.1 | **Partial — gap G6** | `run.runId` carries Liam's `run_id`; ATE v0.4 has no slot for `nono_session_id` or the agent `key_id` |
| S16 | Proposal API shape | Liam §7 · OLIVE_CHARTER_v0.4 (absent) | **Open** (Liam A2) | Built against stub; confirm against olive before E2E |
| S17 | Receipt field names in the fork | Liam v0.4 §4.2 · NONO_v0.4.3 · `receipt.rs` @ `29557a93` | **Resolved in spec** (checkpoint report; v0.4.3 ratified) | Remaining: token arm value and expectancy review (Liam O7, NONO_v0.4.3 O1) |
| S18 | Vendored vocabulary hash | Liam §10.1 · vocabulary.yaml | **Open** | Placeholder UUIDs; hash `60a44f00…` holds only once real UUIDs are restored |

## 3. Gaps Found

**G1 — Liam's hometown and olive calls carry no principal (blocks E2E, not the stub build).**
nono-proxy injects the agent key only on the gateway FQDN, and Liam holds no credential. Liam §6 and §7 assume hometown and olive receive a principal, but nothing in the set supplies one. Two candidate fixes, both needing nono, hometown and olive owners:
- Inject the same session agent key on the hometown and olive FQDNs, with both services validating it against popeye. This keeps one credential per session and gives both services `agent_id`, `on_behalf_of` and `purpose` from an authoritative source.
- Issue per-service tokens. More moving parts; not recommended for POC-2.
Recorded as Liam O5. Does not block handing Liam to the agent, because clients are built against stubs first.

**G2 — Two recovered v0.3 documents still describe per-user injection without qualification.**
`NONO_FISKROAD_INTEGRATION_v0.3` §2.1 and `POPEYE_CORRECTIONS_v0.3` §3.1 were left verbatim. Both are amended by reference (`NONO_v0.4.1` header, `POPEYE_v0.4.1` §3). Recommendation: when committing, add a one-line "Amended for agent profiles by …" note under each heading, so a reader of the v0.3 file alone is not misled.

**G3 — Program TC-VOC IDs collided with Liam-local test IDs.**
Fixed: Liam's two vocabulary tests are now TC-LVOC-01/-02. Program TC-VOC-01..05 are untouched.

**G4 — Reconstructed vocabulary REQ numbering.**
Fixed for internal consistency: REQ-VOC-01..04 match v0.3, REQ-VOC-10 matches popeye's citation, REQ-VOC-05..09 are new assignments noted as such. Replace with the original numbering if the August document is found.

**G5 — Liam calls the gateway directly, but the ATE is assembled by hometown (found on recovering HOMETOWN_v0.4). Resolution drafted 2026-10-19:** Liam v0.3 asks hometown for the answer; hometown relays the model call on the session agent key (`POPEYE_v0.4.2` §2.6); Liam's profile drops the gateway; `direct_gateway=false` restricts the key at popeye. Direction ratified 2026-10-19 with the v0.4.2 pair; closes fully when S-HT-1 is confirmed by the hometown owner.
ATE traces join to the ledger through `primary_request_id`, the request hometown itself made. In Liam's design, Liam takes context from hometown and then calls the gateway itself, so hometown never sees the model request and cannot write a trace for the answer. Options: (a) Liam asks hometown to answer (hometown makes the model call and writes the trace; Liam only proposes), which matches hometown's existing role and the "hometown proposes, olive executes" boundary; (b) Liam reports its gateway request id back to hometown for trace assembly, which needs a new hometown write endpoint and contradicts hometown's read-only posture. **Recommended: (a).** It changes Liam's task flow (steps 3–4 move into hometown), not Liam's boundary. Blocks TC-E2E-01; does not block the stub build. Needs a decision before Liam's client step.

**G6 — ATE v0.4 has no field for the endpoint evidence link.**
`run.runId` covers the run, and `policy.principalType` / `onBehalfOf` cover the agent identity, but nothing in ATE v0.4 carries `nono_session_id` or the agent `key_id`, which is how a receipt is found from a trace (DECISION §5.4). Proposed: an optional `endpoint: { sessionId, keyId }` object, added in the ATE v0.4 joint review (HOMETOWN_v0.4 §2.4) rather than a later revision, consistent with that section's "batch it" rule.

**G1 update — direction confirmed by HOMETOWN_v0.4 §4.2.** hometown takes purpose "from the virtual key (authoritative)" and checks entitlement against directory groups. That presumes a key arrives on hometown requests, so the recommended fix stands: nono-proxy injects the session agent key on the hometown (and olive) FQDNs too, and hometown resolves `on_behalf_of` to the user's groups (intersection rule). This is now the expected reading of hometown's own spec, not only a proposal.

## 4. Before The Build Starts

0. **Done 2026-10-19:** `NONO_v0.4.3`, `POPEYE_v0.4.3` and Liam v0.4 ratified together; the agent restarts at handoff step 3 against Liam v0.4 §4.2. O7 can follow: until it closes, `expected_token_arm` stays empty and only stub-mode runs start.

1. **Open.** Restore the three POC tenant UUIDs and confirm SHA-256 `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`. The UUIDs could not be recovered from the August session (generated in shell variables; outputs not retained). Run `Restore-Vocabulary.ps1` with the values from any surviving copy; it writes the file only on a hash match. If no copy survives, re-minting is a recorded decision that changes the hash; update `liam.toml` and every `VENDORED.md`.
2. **Done 2026-10-19.** `NONO_v0.4.1` and `POPEYE_v0.4.1` ratified jointly; ratification records are appended to both.
3. Decide G1 (inject the session agent key on hometown and olive FQDNs; confirmed by HOMETOWN_v0.4 §4.2) and G5 (recommended: hometown makes the model call). Add G6's `endpoint` object to the ATE v0.4 joint review.
4. Attach `HOMETOWN_v0.4.md` (now recovered) and `OLIVE_CHARTER_v0.4.md` (recoverable on request) to the handoff.
5. Add the G2 amendment notes when committing the v0.3 files.

Items 1–2 should be done before the agent starts. Items 3–5 can follow; the handoff's checkpoints stop the agent before any of them matters.

**G7 — Receipt contract did not match the fork (found at the receipt checkpoint, 2026-10-19).**
The v0.4.1/v0.4.2 receipt additions collided with the fork's startup-only (D-20), content-free (D-05/D-14) and policy-free (D-12) invariants, and Liam v0.3 §4.2 assumed fields the fork does not have (profile, egress, freshness, integrity, `required`). **Resolution ratified 2026-10-19:** `NONO_v0.4.3` keeps the receipt unchanged except one new census row (`AgentKeyInjection`, schema 2), adds a supervisor-written `AgentSessionRecord` (enums, integers, `KeyId`, no FQDNs, no `on_behalf_of`) and a teardown `SessionEndRecord`, and extends the content-free scan without a second `String` exception; `POPEYE_v0.4.3` fixes `key_id` as a lowercase UUID; Liam v0.4 rewrites §4.2 and carries its own layer expectancy (§4.3). Open: the AppContainer token-arm value and the nono owner's review of Liam's expectancy table (Liam O7).

**G8 — Transport wording contradicted the binary (found 2026-10-20).** The release binary links Winsock and `getaddrinfo` through `reqwest`, contradicting design §9's "no sockets / no DNS". **Resolution drafted:** `LIAM_AMENDMENT_v0.4.1_proxy-transport.md` (D14): Liam connects only to `127.0.0.1:<proxy_port>`, validates the proxy environment, ignores system proxy settings, installs a deny-all resolver, and proves zero resolver calls and zero DNS queries by test. Adds assumption A5 for the nono owner (nono-proxy must hand Liam an IPv4-literal proxy URL). Must be ratified before Gate 3.
