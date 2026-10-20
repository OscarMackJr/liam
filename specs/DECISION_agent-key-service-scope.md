# Decision Record: Agent Key Scope Across Program Services

| Field | Value |
|---|---|
| Status | **Ratified 2026-10-19** — ratifies `NONO_v0.4.2.md` and `POPEYE_v0.4.2.md` together (see Ratification Record) |
| Date | 2026-10-19 |
| Owner | Oscar Mack (architecture and risk decisions) |
| Components affected | nono (fork), popeye, hometown, olive, Liam (tests only) |
| Related | `LIAM_DESIGN_SPEC_v0.3.md`, `DECISION_desktop-agent-nono-integration.md`, `NONO_v0.4.1.md`, `POPEYE_v0.4.1.md` (ratified 2026-10-19), `HOMETOWN_v0.4.md` §4.2, `OLIVE_CHARTER_v0.4.md` §2, `SPEC_ALIGNMENT_LIAM_v0.1.md` gap G1 |
| Supersedes | — |

---

## 1. Context

The ratified v0.4.1 seam gives each agent session a short-lived agent key, minted through popeye and injected by nono-proxy on the gateway FQDN only. An agent's other calls, to hometown for context and to olive for proposals, therefore carry no principal.

Both services authorize by principal. hometown takes purpose from the virtual key and entitlement from directory groups (`HOMETOWN_v0.4` §4.2). olive checks purpose and entitlement against the requester (`OLIVE_CHARTER_v0.4` §2). Without a principal, neither can apply its controls, and the POC's end-to-end test (Liam TC-E2E-01) cannot pass.

The question is how hometown and olive learn who an agent session acts for.

---

## 2. Decision

**nono-proxy injects the same session agent key on the hometown and olive FQDNs declared in the agent profile, and on no other destination. hometown and olive validate it through a new read-only popeye introspection path.** Specified in `NONO_v0.4.2` (injection scope, fail-secure profile checks, receipt field) and `POPEYE_v0.4.2` (introspection), ratified together.

olive's obligation is one line, recorded in both amendments: accept the key, validate via introspection, authorize on the intersection with `on_behalf_of`, never forward the key.

**Revised before ratification (gap G5).** hometown makes the model call for an agent's answer and relays it **on the session agent key**, identified as a registered relay, so the ledger attributes the call to the agent and its user (`relayed_by=hometown`). The exchange request declares `direct_gateway`; for Liam it is `false`, so popeye refuses any gateway call on Liam's key that does not come through hometown. Liam's profile no longer allowlists the gateway at all (`LIAM_DESIGN_SPEC_v0.3` D12).

---

## 3. Alternatives Considered

### 3.1 Keep hometown and olive stub-only pending their owners' decision

- **For:** no change to ratified specs; no new popeye surface.
- **Against:** defers a decision hometown's own spec has already made (purpose "from the virtual key"); TC-E2E-01 stays unpassable; the agent's evidence chain stops at the gateway.
- **Rejected** as an end state. The Liam stub build proceeds this way anyway, so nothing is lost by deciding now.

### 3.2 Per-service tokens

A separate credential per service, minted and injected alongside the gateway key.

- **For:** each service sees only its own credential; a compromised service cannot replay against the gateway.
- **Against:** a second minting path, a second revocation path and a second set of receipt fields, multiplied by each service; three credentials per session to keep consistent on `on_behalf_of` and `purpose`, with drift becoming a new failure mode.
- **Rejected for POC-2.** Revisit if the replay risk in §5 becomes material (see §7).

### 3.3 Same session agent key plus introspection (chosen)

- **For:** one credential, one mint, one revocation, one `key_id` that joins the receipt, the ledger and both services' records; hometown §4.2 works as written; popeye stays semantics-blind (it reports identity; the services decide).
- **Against:** the key reaches three services instead of one; a new popeye path; revocation reaches services within a 60 s cache bound rather than instantly.
- **Accepted.**

### 3.4 Services validate by making a gateway request with the key

- **For:** no new popeye path.
- **Against:** spends tokens and ledger rows to answer an identity question; pollutes run metering; still does not return `on_behalf_of` or `purpose` in a usable form.
- **Rejected.**

---

## 4. Rationale

1. **One identity per session.** The program's join keys (`run_id`, `key_id`) work only if every service sees the same credential. Separate tokens would need reconciling, and reconciliation is where vocabularies fork.
2. **The spec already presumes it.** hometown §4.2 assumes a key arrives on its requests. Deciding otherwise would mean amending hometown, not just nono.
3. **Identity, not capability.** Introspection tells a service who the key acts for. It confers no ability to mint, extend or onward-delegate, so widening injection does not widen what a service can do.
4. **Least change to ratified work.** v0.4.1's minting, fail-secure and revocation rules are untouched; only the injection list and one read path change.

---

## 5. Consequences

**Positive**

- hometown and olive can apply purpose masking and entitlement checks to agent traffic, using `on_behalf_of` and the intersection rule.
- Liam TC-E2E-01 becomes passable once implemented; Liam's code is unchanged (it never handles the key).
- Every service record for a session carries the same `key_id`, which completes the receipt-to-trace-to-ledger chain alongside ATE gap G6.

**Negative and mitigations**

- **Wider key exposure.** Mitigated by short TTL, session-end revocation, the allowlist and WFP fence, introspection conferring no capability, and `direct_gateway=false` restricting gateway use to the registered relay. **Residual:** a compromised hometown could misuse a live key within its TTL; keys minted with `direct_gateway=true` remain replayable by a holder until expiry or revocation. Accepted for POC-2; stated in both amendments.
- **Revocation latency at services up to 60 s** (cache bound). Stated in `POPEYE_v0.4.2` §2.2 and tested (REQ-AGT-16).
- **New popeye surface.** Read-only, restricted to registered service callers, inactive keys reveal nothing.

---

## 6. Ratification Conditions

Accepting this decision ratifies `NONO_v0.4.2` and `POPEYE_v0.4.2` together, under the change-control clauses of their v0.4.1 predecessors. Not covered and remaining open:

- `OLIVE_CHARTER_v0.4` conformance to the one-line olive requirement (olive owner to confirm).
- hometown's agent answer endpoint contract (Liam stub contract S-HT-1) and ATE gap G6 (endpoint link in the trace). Gap G5's direction (hometown makes the model call) **is** covered by this decision.
- The registered-caller list location (`POPEYE_v0.4.1` §5 O3).

---

## 7. Revisit Triggers

- Evidence that replay from a program service is a realistic threat (an incident, a penetration-test finding, or a service moving outside the trust boundary) → reconsider §3.2 or origin-bound keys.
- A fourth service needing agent identity → extend `inject_on` by program decision, not configuration.
- Replacement of user keys with an Entra-token exchange (`NONO_v0.4.1` §8 O2) → re-examine whether services should validate Entra tokens directly.

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**: same session agent key injected on profile-named program services; read-only popeye introspection; hometown relays model calls on the session key; `direct_gateway` bound at mint (§2, as revised for gap G5) |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Ratifies | `NONO_v0.4.2.md` and `POPEYE_v0.4.2.md`, jointly, under the change-control clauses of `NONO_v0.4.1` and `POPEYE_v0.4.1` |
| Ratified together with | `DECISION_desktop-agent-nono-integration.md` (revision 2) |
| Not ratified here | §6 items: olive charter conformance to the one-line olive requirement; hometown's agent answer endpoint contract (S-HT-1); ATE gap G6; registered caller and relay list location (`POPEYE_v0.4.1` §5 O3) |
| Residual risk accepted | A compromised hometown could misuse a live session key within its TTL; keys minted with `direct_gateway=true` remain replayable by a holder until expiry or revocation (§5) |
| Change control | Any change to injection scope, introspection, relay rules or `direct_gateway` semantics requires matching changes to both amendments and re-ratification of this decision |
