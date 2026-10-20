# Nono v0.4.2 — Agent Key Injection Scope

Version: 0.4.2
Status: **Ratified 2026-10-19**, jointly with `POPEYE_v0.4.2.md` (one seam), via `DECISION_agent-key-service-scope.md`. See Ratification Record at the end.
Repository: nono (place beside `NONO_v0.4.1.md`)
Amends: `NONO_v0.4.1.md` §3, §4.1, §4.2, §5, §7 (REQ-END-08).
**Revised 2026-10-19 (pre-ratification)** for gap G5: hometown makes agent model calls; Liam's profile excludes the gateway; the exchange request now declares whether the session may reach the gateway directly (§3.1).
Audience: nono maintainer; popeye, hometown and olive owners; Liam implementer.
Primary post-read action: implement §2–§4 before Liam's end-to-end tests.

---

## 1. Why

`NONO_v0.4.1` §4.2 injects the session agent key on the gateway FQDN only. An agent's calls to hometown and olive therefore arrive with no principal, yet both services authorize by principal: hometown takes purpose "from the virtual key (authoritative)" and entitlement from directory groups (`HOMETOWN_v0.4` §4.2), and olive authorizes proposals against the requester (`OLIVE_CHARTER_v0.4` §2). This amendment extends injection to the program services an agent profile declares, and nothing else. Recorded as `SPEC_ALIGNMENT_LIAM_v0.1.md` gap G1.

---

## 2. The Rule (replaces `NONO_v0.4.1` §4.2)

`nono-proxy` injects the session agent key on requests to the **program services declared in the agent profile** (§3), and on no other destination. The user key is still used for one thing only: the exchange call (`NONO_v0.4.1` §4.1). It is never injected into child traffic.

Permitted services for POC-2: `gateway`, `hometown`, `olive`. No other service name is accepted.

---

## 3. Agent Profile (amends `NONO_v0.4.1` §3)

The `[agent]` block gains `inject_on`, naming services, each of which must map to an FQDN already on the profile's network allowlist:

```toml
[agent]
agent_id = "liam"
purpose_source = "launch_arg"
key_ttl_seconds = 3600
inject_on = ["hometown", "olive"]      # Liam's profile; an agent that calls the model itself may add "gateway"

[agent.services]
hometown = "<hometown-api-fqdn>"
olive    = "<olive-api-fqdn>"
```

**Fail secure.** The session refuses to start if:

- `inject_on` names a service outside the POC-2 set in §2;
- a named service has no entry in `[agent.services]`;
- a service FQDN is not on the profile's network allowlist;
- `inject_on` is absent or empty (an agent profile with nowhere to send its key is a misconfiguration, not a session without a key).

Matching is on the exact host the child connects to through the proxy (CONNECT host, case-insensitive, no wildcard). A host that merely resolves to the same address does not match.

---

### 3.1 Declaring direct gateway use at mint (amends `NONO_v0.4.1` §4.1)

The exchange request gains one field, `direct_gateway`, set to `true` only when `inject_on` contains `gateway`. popeye binds it to the key (`POPEYE_v0.4.2` §2.6). For a key minted with `direct_gateway=false`, popeye refuses any gateway call that does not arrive through a registered relay (hometown). This makes "the agent never calls the model directly" hold at popeye as well as at the endpoint, and closes the replay path for such keys from any service other than the registered relay.

## 4. Receipt (amends `NONO_v0.4.1` §5)

> **Superseded 2026-10-19 by `NONO_v0.4.3.md`** (ratified). The fields below were never implemented in this form: the fork's receipt is startup-only and content-free, so agent facts live in `AgentSessionRecord` and `SessionEndRecord`, and the `agent_key` layer is the `AgentKeyInjection` census row. Do not implement this section; it is kept for history.


The `agent` object gains one field:

| Field | Type | Source |
|---|---|---|
| `agent.injected_on` | array of `{service, fqdn}` | The resolved `inject_on` set for this session |
| `agent.direct_gateway` | bool | The value sent at mint (§3.1) |

The receipt still never carries the key.

---

## 5. Exposure Statement

The session agent key now reaches up to three services instead of one. Accepted for POC-2 because:

- all three are program services inside the same trust boundary, each behind the profile allowlist and the WFP fence;
- the key is short-lived (`NONO_v0.4.1` §3; popeye cap per `POPEYE_v0.4.1` §2.2), is revoked at session end, and carries no permission beyond the user's (intersection rule);
- a service that receives the key can only present it back to popeye for introspection (`POPEYE_v0.4.2` §2); it cannot mint, extend or onward-delegate with it.

**Sanctioned relay.** hometown presents the session key to the gateway for the session's model calls (`POPEYE_v0.4.2` §2.6). That is intended use, recorded on the ledger as `relayed_by=hometown`.

Residual risk, stated: for a key minted with `direct_gateway=false` (Liam's case), popeye refuses gateway calls that do not come through the registered relay, so olive or any other holder cannot replay it against the gateway. A compromised **hometown** could still misuse a live key within its TTL, as it can misuse any request it relays. For keys minted with `direct_gateway=true`, replay by a service remains possible until expiry or revocation. Both mitigated by TTL and revocation, not eliminated. Origin-bound keys are a Stage 3 question.

---

## 6. What Does Not Change

Minting, fail-secure rules on mint failure, no fallback to the user key, no silent re-mint, revocation at session end, and the detection contract are all unchanged from `NONO_v0.4.1`. nono stays semantics-blind: it injects a key on named hosts and does not know what any service does with it.

---

## 7. Requirements

| REQ | Requirement | TC | Test criteria | Method |
|---|---|---|---|---|
| REQ-END-08 (revised) | An agent-profile session injects the agent key, never the user key, on every service in `inject_on` | TC-END-08 | Gateway, hometown and olive stub captures each show the agent `key_id`; none shows the user key | CI (stub) + DEMO |
| REQ-END-14 | The agent key is injected on no destination outside `inject_on` | TC-END-14 | A request from the child to an allowlisted host not in `inject_on` arrives with no credential | CI (stub) |
| REQ-END-15 | A profile whose `inject_on` is empty, names an unknown service, or maps to a non-allowlisted FQDN does not start | TC-END-15 | Each misconfiguration refuses the session; the child never starts | CI |
| REQ-END-16 | The receipt records `agent.injected_on` and `agent.direct_gateway` | TC-END-16 | Receipt schema validation for an agent session | CI |
| REQ-END-17 | The exchange request carries `direct_gateway`, `true` only when `inject_on` contains `gateway` | TC-END-17 | Stub exchange capture for Liam's profile shows `direct_gateway=false` | CI |

---

## 8. Olive Requirement (one line, owned by olive)

**Olive accepts the session agent key on its proposal endpoint, validates it through popeye introspection (`POPEYE_v0.4.2` §2), authorizes against `on_behalf_of`'s entitlements intersected with the agent's, and never forwards the key to any target system.**

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**, jointly with `POPEYE_v0.4.2.md` as one seam (agent key scope across program services, as revised for gap G5) |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions), via `DECISION_agent-key-service-scope.md` |
| Scope ratified | §2 (injection rule), §3 (agent profile, `inject_on`, fail-secure checks), §3.1 (`direct_gateway` at mint), §4 (receipt fields), §5 (exposure statement), §6, §7 (REQ-END-08 revised, -14 to -17), §8 (olive one-line requirement, owned by olive) |
| Not ratified here | Olive's conformance to §8 (olive owner to confirm). `NONO_v0.4.1` §8 O1/O2 remain open |
| Effect | Amends the v0.4.1 predecessor as stated in the header. Implementation may start; Liam's end-to-end tests (TC-E2E-01, -05, -07) depend on it and stay `not measured` until it is implemented |
| Change control | Any change requires a matching change to `POPEYE_v0.4.2.md` and re-ratification of `DECISION_agent-key-service-scope.md` |
