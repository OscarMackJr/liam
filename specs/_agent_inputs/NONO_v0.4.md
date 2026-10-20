# Nono v0.4 — Local Agent Coverage

Version: 0.4
Status: Draft — integration note, native conventions preserved
Repository: nono (place per this repo's own layout)
Primary post-read action: §2 is a genuinely new coverage gap that POC-2 creates. It is worth reading even if v0.3's detection contract is still outstanding.

**Carried from v0.3 and still outstanding:** the `twg-gateway-only` profile and detection emission in the `endpoint-detection/0.1` schema. Those remain the priority; §2 below is additive.

---

## 1. Why POC-2 Touches nono At All

POC-2 introduces agents. Agents increasingly run **on workstations** — in editors, in desktop assistants, in local tool runners. That creates a coverage question the program has not previously had to answer:

> An agent running on a laptop makes model calls **and tool calls**. The gateway sees the model calls if the agent is routed correctly. Nobody sees the tool calls at all.

A local agent reading files, calling internal APIs, or invoking a local tool server is doing governed-relevant work entirely outside every control the program has built. This is the shadow-AI problem's second act, and it arrives with agent adoption whether or not we are ready.

---

## 2. Scope For v0.4

**2.1 Extend the profile to agent egress.** The `twg-gateway-only` profile adds: local tool-server endpoints (Model Context Protocol servers and equivalents) are **not** permitted to reach non-approved external destinations. The proxy already does destination allowlisting; this is a policy addition, not new capability.

**2.2 Emit an agent-activity detection.** Reuse the existing detection schema with `action: "blocked"` and a `profile` value identifying the agent policy. No new schema, no new transport — the same JSONL file the v0.3 contract defines.

**2.3 Explicitly out of scope.** nono does not inspect tool-call arguments, does not understand agent semantics, and does not enforce which tools an agent may use — that is olive's tool policy (`OLIVE_CHARTER_v0.4` §5). nono remains semantics-blind, the same posture as the gateway.

| REQ | Requirement | TC | Test criteria | Method |
|---|---|---|---|---|
| REQ-END-06 | The profile blocks local agent egress to non-approved destinations | TC-END-06 | A local tool server attempting a blocked destination is stopped | DEMO |
| REQ-END-07 | Agent-related detections use the existing schema, content-free | TC-END-07 | Schema validation; no arguments or payloads present | CI |

---

## 3. The Honest Position

This is a **partial** control and should be described as one. nono sees network egress. An agent operating entirely on local files, or against a tool server on the same machine, is invisible to it — and that is a real gap with no cheap answer.

Full local-agent governance requires either an agent runtime the organisation controls or endpoint instrumentation well beyond nono's current scope. Both are Stage 3+ questions.

**Recorded now so that a POC-2 demonstration of endpoint coverage is not overstated.** Claiming complete agent coverage on the strength of network-level blocking would be exactly the kind of overclaim the program's review culture exists to prevent.
