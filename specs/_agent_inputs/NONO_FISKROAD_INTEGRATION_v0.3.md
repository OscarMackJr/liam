# Nono × Fiskroad Integration Note v0.3

Version: 0.3
Status: Draft — nono's half of cross-repo finding X5
Repository: nono (suggested location: `docs/integrations/fiskroad.md` or the `.planning/` convention, per maintainer preference — nono keeps its native layout).
Audience: nono maintainer; popeye team.
Primary post-read action: encode the `twg-gateway-only` profile in nono configuration and wire detection emission; both are configuration-and-glue against capabilities nono already has.

## 1. Context

Popeye's `POPEYE_CORRECTIONS_v0.3` section 3 defines the endpoint detection contract: the policy content, the event schema, and the POC-grade transport. Popeye owns the policy (which domains, which key); nono owns the enforcement and emission (how the proxy applies it, how events leave the workstation). This note is the nono side, kept deliberately thin because the capabilities already exist — `nono-proxy` does domain/endpoint allowlisting with credential injection, which is precisely the mechanism required.

## 2. Nono's Two Deliverables

**2.1 The `twg-gateway-only` profile, encoded.**

Translate the popeye-owned policy into nono's native configuration:

- Allow the gateway FQDN (initially the Container App internal FQDN; `gateway-azure.ai.twg.internal` once the private zone lands) with credential injection of the per-user virtual key, so gateway traffic works with zero client configuration.
- Block the direct provider domains listed in the popeye contract (`api.openai.com`, `api.anthropic.com`, `*.openai.azure.com`, `bedrock*.amazonaws.com`, `generativelanguage.googleapis.com`, extensible by policy update).
- The profile ships as a named, versioned configuration artifact so Intune (or the eventual fleet mechanism) can distribute it and the version appears in detection events.

**2.2 Detection emission in the contract schema.**

On each blocked attempt, append one JSON object to a local JSONL file per the `endpoint-detection/0.1` schema in the popeye contract: timestamp, workstation, OS user, process, blocked destination, action, profile name. Two properties are load-bearing:

- **Content-free.** No request bodies, no headers beyond destination host, no prompt material. The event says an attempt happened, not what it said. This mirrors the program's no-content guardrail at the endpoint.
- **Append-only local file for the POC.** No new transport is built; the demo forwards the file. Fleet-scale shipping (Log Analytics agent or equivalent) is a Stage 3 concern and stays out of this note.

## 3. What Nono Does Not Take On

- Deciding the allowlist/blocklist content — popeye owns the policy; nono enforces the artifact it is handed.
- Any knowledge of data classes, tenants, budgets, or attribution semantics — the injected key carries all of that; nono stays semantics-blind, matching the gateway's own posture.
- Production event transport, alert rules, or ingestion — popeye-side, per the contract's staging.

## 4. Acceptance (shared with popeye's TC-END-02a/b/c)

The joint demo criterion: on a managed workstation running the profile, a process attempts `api.openai.com`; nono blocks it and emits a schema-valid, content-free event; the same workstation reaches the gateway successfully with the injected key; the event file reaches the popeye side and raises the "nono detections" notify alert. One detection, end to end — that is TC-END-02 done.
