# Decision Record: Desktop Agent and nono Integration

| Field | Value |
|---|---|
| Status | **Ratified 2026-10-19** (revision 2; see §9 and the Ratification Record) |
| Date | 2026-10-19 |
| Owner | Oscar Mack (architecture and risk decisions) |
| Components affected | nono (Windows fork), desktop agent, popeye, olive, hometown (ATE) |
| Related | `NONO_v0.4.md`, `NONO_v0.4.1.md` (ratified), `NONO_v0.4.2.md` (ratified), `POPEYE_v0.4.md`, `POPEYE_v0.4.1.md` (ratified), `POPEYE_v0.4.2.md` (ratified), `HOMETOWN_v0.4.md`, `OLIVE_CHARTER_v0.4.md`, `DECISION_agent-key-service-scope.md`, `LIAM_DESIGN_SPEC_v0.3.md`, `SPEC_ALIGNMENT_LIAM_v0.1.md`, seam S9 (nono → detection → alert), nolabs-ai/nono#2125 |
| Supersedes | — |

---

## 1. Context

Fiskroad will place an AI desktop agent on banker workstations in a corporate banking setting. nono is the program's local containment component. It confines an agent's process, filesystem and network on the endpoint, and on Windows it runs as the fork `OscarMackJr/nono` (AppContainer, an elevated per-session WFP service, a fail-direction contract and per-session enforcement receipts).

The question is how the agent and nono relate. They could be built as one product, shipped as a paired unit, or kept independent and joined only through the program's contracts.

Constraints that bear on the decision:

- **Independent control.** In a bank, model risk management, internal audit and the second line expect a control to be separate from the system it governs, with its own owner, change process and evidence.
- **Program boundary discipline.** Every Fiskroad component already refuses scope outside its invariant: olive holds the one write credential, popeye is semantics-blind, and hometown is read-only. In v0.4, maintaining these boundaries was the load-bearing work.
- **More than one agent.** The bank will run several agents (vendor copilots, coding agents, desk-adopted tools), not only ours.
- **Upstream and platform trajectory.** The fork's standing in NEP-0005 depends on it being nono with a Windows backend. Microsoft Execution Containers (MXC) is a likely future confinement backend.
- **Endpoint ownership.** The endpoint team owns CrowdStrike Falcon policy, servicing rings and approval of elevated services.

---

## 2. Decision

**The desktop agent runs under nono and is integrated with it only through Fiskroad contracts (contract-only).** The two are built, versioned, released and approved independently. Neither links the other's code or ships inside the other's package.

One narrow dependency is permitted: the agent may verify, at startup, that it is running inside a valid nono session, and refuse to start if it is not (§5.5). This dependency points from the agent to nono only. nono never depends on the agent.

---

## 3. Alternatives Considered

### 3.1 Bundle: agent and sandbox as one product

The agent and nono share a codebase or installer. The agent starts its own confinement and the two release together.

- **For:** simplest deployment, with one artifact and one install; tight integration; the agent can tune its own profile.
- **Against:**
  - The confined system controls its own confinement, so the control is not independent. An agent defect or compromise sits on the same trust boundary as the fence.
  - Every agent release becomes a change to a security control, which pulls the sandbox into every agent review and change-advisory cycle, and pulls the agent into every control review.
  - nono stops being agent-neutral and cannot confine other agents the bank adopts without carrying ours.
  - It breaks the program's one-invariant-per-component rule in the component whose job is containment.
  - It diverges the fork from upstream nono and complicates adopting MXC as a backend.
- **Rejected.**

### 3.2 Sidecar: separate binaries, paired deployment

The agent and nono are separate processes but ship and install as a unit. A local channel (IPC or a shared service) lets the agent query or steer nono at run time, for example to request a profile change or to read enforcement state.

- **For:** keeps separate codebases; gives the agent richer runtime awareness; one deployment package for the endpoint team.
- **Against:**
  - A runtime control channel from the confined process to its confiner is new attack surface that crosses the containment boundary. Even read-only channels need an authorization model and a threat review.
  - Paired versioning recreates most of the bundle's change-control coupling.
  - Other agents would not use the channel, so nono gains agent-specific surface that only our agent exercises.
  - The useful part (the agent knowing it is confined) is achievable without a live channel; see §5.5.
- **Rejected.** It can be revisited if a concrete need arises that receipts cannot meet (§8).

### 3.3 Contract-only (chosen)

The agent and nono are independent. The agent is started through a nono profile, all its egress passes through `nono-proxy`, and evidence is joined through receipts and the ATE.

- **For:**
  - nono stays an independent, agent-neutral control with its own owner and evidence.
  - It matches the program's existing boundary discipline.
  - Each component's reviews stay small and separate.
  - nono's backend can change (fork today, MXC later) without changing the agent.
  - It preserves the fork's position upstream.
- **Against:**
  - Two artifacts for the endpoint team to deploy and keep in step.
  - The agent learns about its confinement only from records (receipts), not from a live channel.
  - Integration correctness depends on contracts being versioned and enforced, which the program already requires.
- **Accepted.**

---

## 4. Rationale

1. **The confined system must not hold its own fence.** Independence is the property auditors and examiners test first, and contract-only is the only option that preserves it structurally rather than by policy.
2. **Least privilege applies to components as well as credentials.** Olive exists to be the smallest component that holds a write credential. By the same logic, nono should be the smallest component that holds endpoint containment, and the agent should hold neither.
3. **Agent neutrality is nono's platform value.** A control that confines any agent covers the bank's real portfolio. A control built around one agent does not.
4. **Separate change control lowers delivery cost.** Agent iteration is frequent and control changes are expensive. Coupling them would slow the agent down to the pace of control changes.
5. **Backend portability.** A contract boundary lets nono move to an MXC backend, with the WFP fence and receipts kept on top, without touching the agent.

---

## 5. Contract Touchpoints

Each touchpoint names its contract, owner and failure behavior. The fail-direction contract governs nono's side of each one.

### 5.1 Launch

- **Contract:** a nono profile for the agent, versioned with the agent and reviewed by the nono owner.
- **Mechanism:** the endpoint deployment starts the agent only through nono. No other launch path is supported.
- **Failure:** if nono cannot establish every required layer, the agent does not run (nono fails secure).

### 5.2 Egress and credentials

- **Contract:** all agent network traffic goes through `nono-proxy`, to program services the agent profile allowlists. The agent holds no credential. nono-proxy mints a short-lived **agent key** per session through popeye's exchange, using the user's key for that call only, and injects the agent key on the services named in the profile (`NONO_v0.4.1` and `NONO_v0.4.2`, both ratified). The user's key is never injected into agent traffic.
- **Model calls:** an agent obtains answers from hometown, which masks context by purpose and makes the model call on the session agent key as a registered relay (`POPEYE_v0.4.2` §2.6, ratified). An agent whose profile does not allowlist the gateway (Liam) cannot reach it directly, and popeye refuses its key on any unrelayed gateway call (`direct_gateway=false`).
- **Result:** every model call is attributed in the ledger to `principal_type=agent`, the agent's ID and the user it acts for, and is recorded independently by nono (what left the endpoint), popeye (what was billed and attributed) and hometown (what was answered and under which purpose).
- **Failure:** direct egress is blocked by the WFP fence; a session that cannot obtain an agent key does not start, with no fallback to the user's key. Disagreement between the records is a detection signal (§5.4).

### 5.3 Actions

- **Contract:** the agent proposes actions; olive authorizes and executes them, per `OLIVE_CHARTER_v0.4.md`.
- **Constraint:** the agent never holds a write credential. A compromised or misbehaving agent can only propose, even when its confinement holds.
- **Failure:** proposals olive rejects are recorded, never executed.

### 5.4 Evidence

- **Contract:** each nono per-session receipt carries the session ID and the agent key's `key_id`. The ledger carries the same `key_id`; the ATE carries the run ID and joins the ledger through hometown's `primary_request_id`. The ATE link to the receipt (`endpoint.sessionId`, `endpoint.keyId`) is proposed for the ATE v0.4 joint review and not yet adopted (gap G6).
- **Seam:** S9 (nono → detection → alert).
- **Result:** "Was this answer or action produced inside an enforced sandbox?" can be answered from the records, without relying on the agent's own account.
- **Failure:** an ATE with no matching valid receipt, or a receipt that records a degraded layer, raises an alert. It is not silently accepted.

### 5.5 Agent self-check (the one permitted dependency)

- **Contract:** at startup, the agent reads the receipt for its own session and confirms that every layer the profile requires is active, including the `agent_key` layer, and that the receipt's agent identity, purpose and injection set match the agent's own configuration and launch purpose.
- **Direction:** agent → nono, read-only, against a record. There is no runtime control channel.
- **Failure:** if the receipt is missing, invalid or degraded, the agent refuses to start. Launching the agent outside nono therefore does nothing.

---

## 6. Consequences

**Positive**

- nono can be reviewed, approved and evidenced as an endpoint control independently of any agent.
- Other agents the bank adopts can be confined with no change to nono.
- The agent ships on its own release cadence.
- An MXC backend can be adopted without changing the agent.

**Negative and mitigations**

- **Two deployable artifacts.** Mitigation: one endpoint deployment recipe that installs both, while keeping their versioning and approvals separate.
- **Profile drift between agent and nono.** Mitigation: the profile is versioned with the agent, reviewed by the nono owner, and its version is recorded in the receipt.
- **The agent has only record-based awareness of its confinement.** Accepted. The self-check in §5.5 covers the startup case, which is the case that matters.

**Not changed by this decision**

- The **key-person risk** of a single-maintainer fork running an elevated service across the fleet. It must be addressed separately (support model, second maintainer) before production approval.
- Coverage gaps: local file and local tool-server activity (`NONO_v0.4.md` §3), and service-mediated network paths (DNS, SMB and WebDAV redirectors, BITS, out-of-process COM) identified but not yet measured in nolabs-ai/nono#2125. In this setting they are exfiltration routes for material non-public information and regulated identifiers, and must be stated plainly in the control description.
- The residual key-misuse risk recorded in `DECISION_agent-key-service-scope.md` §5: a compromised hometown could misuse a live session key within its TTL.

---

## 7. Open Questions

1. **Receipt retention.** Once the ATE references receipts, they fall under the bank's retention and legal-hold rules. Where are they stored, for how long, and who owns them?
2. **Receipt integrity.** How does the agent's self-check (§5.5) know a receipt was written by nono and not forged? This needs a signing or protected-path decision. *POC position (Liam D10):* ACL-only (the receipt must not be writable by the agent's token), with the signature path built and tested against a test key; POC evidence states `receipt_integrity: "acl-only"`. The scheme itself remains open for nono and zt-infra.
3. **Endpoint team agreement.** Approval of the elevated service, Falcon coexistence (soak test pending), and the servicing level required for any MXC backend.
4. **Profile ownership.** Confirm that the nono owner, not the agent team, approves changes to the agent's profile. The profile now carries security-relevant agent fields (`agent_id`, `inject_on`, TTL), which strengthens the case.
5. **Proposal submitter.** Whether olive expects the agent principal or hometown to submit proposals (Liam O6). The reference agent submits in its v0.3 design.

---

## 8. Revisit Triggers

Reopen this decision if any of the following occurs:

- A concrete requirement emerges that receipts cannot meet and a runtime channel would (reconsider §3.2 with a threat review).
- nono's confinement backend changes in a way that alters the launch contract (for example, adopting MXC).
- The bank's control framework requires the agent and its containment to be certified as one system.
- Upstream nono defines an agent-integration interface that Fiskroad should adopt instead.

---

## 9. Revision History

| Rev | Date | Change |
|---|---|---|
| 1 | 2026-10-19 | Initial decision: contract-only integration. |
| 2 | 2026-10-19 | Decision unchanged. Touchpoints brought in line with later work: per-session agent key minted and injected by nono-proxy, never the user key (§5.2; `NONO_v0.4.1` / `POPEYE_v0.4.1`, ratified); injection on profile-named services and hometown-relayed model calls (§5.2; `NONO_v0.4.2` / `POPEYE_v0.4.2` and `DECISION_agent-key-service-scope.md`, draft at the time of this revision and ratified the same day); evidence joins through `key_id` and the pending ATE `endpoint` object (§5.4); self-check covers the agent identity and injection set (§5.5); POC position on receipt integrity (§7 Q2); proposal-submitter question added (§7 Q5); coverage gaps and residual key risk restated (§6). |

None of the revision-2 changes alters the decision in §2 or reopens the alternatives in §3. The documents those touchpoints depend on were ratified on 2026-10-19 through `DECISION_agent-key-service-scope.md`, together with this record (see Ratification Record).

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**: contract-only integration of the desktop agent with nono (§2), revision 2 |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Scope ratified | §2 (decision), §3 (alternatives and their dispositions), §4 (rationale), §5 (contract touchpoints as revised), §6 (consequences) |
| Not ratified here | §7 open questions 1–5 remain open (Q2 carries a POC position only). Gap G6 (ATE endpoint link) and the hometown and olive wire contracts (S-HT-1, S-OL-1) remain open |
| Ratified together with | `DECISION_agent-key-service-scope.md`, which in turn ratifies `NONO_v0.4.2.md` and `POPEYE_v0.4.2.md`. The touchpoints in §5.2 that depended on those drafts now rest on ratified documents |
| Effect | The Liam build (`LIAM_DESIGN_SPEC_v0.3.md`) proceeds on this boundary. Any agent or integration that bundles with nono or adds a runtime channel to it requires reopening this decision (§8) |
| Change control | Changes to §2 or §5 require a new revision and re-ratification. §8 revisit triggers are unchanged |
