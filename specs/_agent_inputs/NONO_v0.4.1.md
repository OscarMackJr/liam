# Nono v0.4.1 — Agent Key Injection

Version: 0.4.1
Status: **Ratified 2026-10-19**, jointly with `POPEYE_v0.4.1.md` (one seam). See Ratification Record at the end.
Repository: nono (place per this repo's own layout, beside `NONO_v0.4.md`)
Amends: `NONO_FISKROAD_INTEGRATION_v0.3.md` §2.1 (credential injection) for **agent profiles only**; `NONO_v0.4.md` (adds §4).
Audience: nono maintainer; popeye team; Liam implementer.
Primary post-read action: implement §3–§5 before the Liam build reaches its end-to-end tests.

---

## 1. Why This Amendment Exists

`NONO_FISKROAD_INTEGRATION_v0.3` §2.1 has `nono-proxy` inject the **per-user** virtual key on gateway traffic. That was correct when the only workstation traffic was a person using a tool.

`POPEYE_v0.4` §2 then gave agents their own principal type. Agent traffic is expected on a **short-lived agent key** carrying `principal_type=agent`, `agent_id` and `on_behalf_of`. If nono keeps injecting the user's key for an agent session:

- every agent call lands in the ledger as the user, so REQ-AGT-03 (ledger rows carry agent identifiers) can never pass on agent traffic;
- the run cost alert (REQ-RUN-02) attributes a runaway agent to a person;
- the user's long-lived key is used for a workload the program says must run on a short-lived credential (`POPEYE_v0.4` §2.2).

The fix belongs on nono's side because nono is the component that already holds the key and decides what gets injected. The agent itself does not change: it never sees any key, before or after this amendment.

---

## 2. The Rule

For any session started with an **agent profile** (§3), `nono-proxy` injects a short-lived **agent key** minted for that session, with `on_behalf_of` set to the user. It never injects the user's key into an agent session.

Profiles that are not agent profiles (`twg-gateway-only` used by a person's own tools) are unchanged and keep per-user injection as specified in v0.3.

---

## 3. Agent Profiles

A profile is an agent profile when it declares an `[agent]` block. Minimum fields:

```toml
[agent]
agent_id = "liam"                 # must be registered with popeye as an agent_id
purpose_source = "launch_arg"     # purpose is supplied per session at launch, not fixed in the profile
key_ttl_seconds = 3600            # requested TTL; popeye may shorten it, never lengthen it
```

The rest of the profile (filesystem grants, gateway and service FQDNs, blocked provider domains) follows `NONO_v0.4` and the agent's own profile spec. Agent profiles inherit the v0.3 block list and v0.4 §2.1 egress restriction unchanged.

---

## 4. Session Key Lifecycle

### 4.1 Mint at session start

Before the confined child is started:

1. nono reads the session's declared purpose from the launch arguments and validates it against the vendored `vocabulary.yaml` (`purposes`). An unlisted or missing purpose refuses the session.
2. `nono-proxy` calls popeye's agent key exchange (`POPEYE_v0.4.1` §2) **authenticated with the user's own virtual key**, requesting `agent_id`, `purpose` and `ttl`.
3. popeye returns an agent key, its key identifier (not the secret), its effective `expires_at`, and the `on_behalf_of` value **popeye derived from the user key**. nono does not assert `on_behalf_of`; it records what popeye returned.
4. `nono-proxy` holds the agent key in memory only. It is never written to disk, the receipt, a log, the child's environment, or its command line.

### 4.2 Inject during the session

`nono-proxy` injects the agent key on requests to the gateway FQDN only, exactly as v0.3 §2.1 injects the user key today. The user key is used for one thing in an agent session: the exchange call in §4.1. It is never injected into child traffic.

### 4.3 Expiry and revocation

- If the agent key expires mid-session, `nono-proxy` does **not** re-mint silently. The next gateway request fails, and the failure is recorded as a session event. A session that needs longer than its TTL is a profile change, not a runtime behaviour.
- At session end (normal or abnormal), `nono-proxy` calls popeye's revocation (`POPEYE_v0.4.1` §2.4) for the session's key. A failed revocation is recorded in the receipt; the key still dies at `expires_at`.
- The WFP service's startup sweep (fork) is extended with no key logic: keys are memory-only, so a crashed proxy leaves nothing to sweep. Expiry bounds the exposure.

### 4.4 Fail secure

The session refuses to start, and the child never runs, if:

- the purpose is missing or not in the vocabulary;
- the exchange call fails, times out, or returns a key whose `principal_type` is not `agent` or whose `agent_id` differs from the profile;
- the returned `on_behalf_of` is empty or is itself an agent.

**There is no fallback to the user key.** A session that cannot get an agent key is not an agent session that can proceed on a weaker credential; it is a refused session. This is the same fail direction as a missing WFP layer.

---

## 5. Receipt Additions

> **Superseded 2026-10-19 by `NONO_v0.4.3.md`** (ratified). The fields below were never implemented in this form: the fork's receipt is startup-only and content-free, so agent facts live in `AgentSessionRecord` and `SessionEndRecord`, and the `agent_key` layer is the `AgentKeyInjection` census row. Do not implement this section; it is kept for history.


The per-session receipt gains an `agent` object, present only for agent profiles. It carries identifiers and times, never the key:

| Field | Type | Source |
|---|---|---|
| `agent.agent_id` | string | profile, confirmed by popeye's response |
| `agent.purpose` | string | launch argument, validated (§4.1) |
| `agent.on_behalf_of` | string | popeye's response (not asserted by nono) |
| `agent.key_id` | string | popeye's key identifier, for ledger joins; never the secret |
| `agent.key_expires_at` | timestamp | popeye's response |
| `agent.key_revoked` | bool | set at session end; `false` if revocation failed |
| `layers[]` entry `name="agent_key"` | `{required: true, status}` | `active` once §4.1 succeeds |

Adding `agent_key` as a required layer means an agent that self-checks its receipt (Liam §4.2) automatically refuses to run on a session that somehow started without one.

---

## 6. What Does Not Change

- nono remains semantics-blind. It validates `purpose` as a string against a list and passes it on; it does not know what a purpose permits.
- nono does not choose `on_behalf_of`, decide TTL policy, or register `agent_id` values. popeye owns all three.
- Detection emission (`endpoint-detection/0.1`) is unchanged. Exchange failures are session events in the receipt, not detections; a detection means an egress attempt was blocked.
- The honest position in `NONO_v0.4` §3 stands. Agent key injection improves attribution of network traffic; it adds no visibility into local file or local tool-server activity.

---

## 7. Requirements

| REQ | Requirement | TC | Test criteria | Method |
|---|---|---|---|---|
| REQ-END-08 | An agent-profile session injects an agent key, never the user key, on gateway traffic | TC-END-08 | Gateway capture for an agent session shows the agent `key_id`; ledger row has `principal_type=agent` | CI (stub) + DEMO |
| REQ-END-09 | A session whose exchange fails, or returns a non-agent or mismatched key, does not start | TC-END-09 | Stub exchange returns 500 / a user-type key / a different `agent_id`; child never starts; receipt records the refusal reason | CI |
| REQ-END-10 | No agent key reaches disk, logs, receipt, child environment or command line | TC-END-10 | Sentinel key value from the stub exchange is searched for in all of them after a session; zero matches | CI |
| REQ-END-11 | The receipt records the `agent` object and a required `agent_key` layer | TC-END-11 | Receipt schema validation for an agent session | CI |
| REQ-END-12 | The key is revoked at session end; revocation failure is recorded | TC-END-12 | Stub records the revocation call; forced revocation failure sets `agent.key_revoked=false` | CI |
| REQ-END-13 | No silent re-mint on expiry | TC-END-13 | Stub issues a 5 s key; a request after expiry fails; exchange call count stays at one | CI |

---

## 8. Open Items

- O1. How a user key is present on the workstation for the exchange call. v0.3 assumed `provision-desktop-keys.ps1` places it; confirm where it lives and that it is outside every agent session's filesystem grant.
- O2. Whether the user key is itself replaced by an Entra-token exchange later (no long-lived user key on the endpoint at all). Out of scope for POC-2; recorded because it would remove O1 entirely.

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**, jointly with `POPEYE_v0.4.1.md` as one seam (agent key minting and injection) |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Scope ratified | §2 (the rule), §3 (agent profiles), §4 (lifecycle, including no fallback to the user key), §5 (receipt additions), §6, §7 (REQ-END-08 to -13) |
| Not ratified here | §8 open items O1 and O2 remain open. The hometown and olive authentication question (`SPEC_ALIGNMENT_LIAM_v0.1.md` G1) is outside this amendment and remains open |
| Effect | `NONO_FISKROAD_INTEGRATION_v0.3.md` §2.1 is amended for agent profiles. Implementation may start; Liam's end-to-end tests depend on it |
| Change control | Any change to §4.1 request fields, §4.4 fail conditions or §5 receipt fields requires a matching change to `POPEYE_v0.4.1.md` and re-ratification of both |
