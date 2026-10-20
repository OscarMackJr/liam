# Work Prompt — Hometown Owner / Implementer

You own Liam's answer contract and the trace around its model call. Read the
cross-component execution specification, Hometown v0.4, Popeye v0.4.2, the
agent-key scope decision, Liam design v0.4, and Liam test spec v0.4 before
implementation.

## Objective

Replace the local S-HT-1 stub with an owner-approved answer API that accepts a
Liam request through Nono's proxy, applies purpose-safe retrieval, relays the
model call as Hometown, and returns a traceable answer with an optional action
proposal.

## First deliverable: wire-contract decision

Publish a versioned contract for joint review before treating it as implemented.
It must settle, exactly:

- endpoint, transport, authentication behavior, versioning, and status/error
  taxonomy;
- request fields and validation for party reference, question, advisory
  purpose, Liam `run_id`, and Nono session identifier;
- response fields for trace identity, answer, context-completeness signal, and
  the optional proposal shape;
- timeout, retry, idempotency, and correlation behavior;
- whether and how the ATE gains the endpoint-link fields needed to join the
  session record. This is the open G6 joint-review item, not an assumption.

If any item differs from S-HT-1 or Liam's current client behavior, provide a
compatibility table and stop for Liam-owner review rather than silently
changing either side.

## Required implementation work after approval

1. Authenticate the injected session agent key by Popeye introspection.
   Treat the key's purpose as authoritative and the request purpose as
   advisory. Resolve `on_behalf_of` to entitlement groups.
2. Apply tenant isolation and purpose masking before context assembly. An
   undeclared field/purpose pair is absent, and no unmasked regulated value may
   enter a prompt.
3. Make the model request as the registered Hometown relay on the session
   agent key. Liam must never call the gateway directly.
4. Write an ATE containing the run relationship and policy decisions without
   protected record values. Preserve compatibility for single-turn answers.
5. Produce a proposal only as a suggestion in the answer. Do not execute it
   and do not acquire a target-system write credential.

## Evidence required

- Contract tests for malformed requests, purpose mismatch, authorization
  failure, timeout and retry semantics, and response-schema compatibility.
- Proof that masking happens before prompt assembly and purpose cannot widen a
  tenant boundary.
- A trace/ledger proof of a Hometown-relayed call attributed to the session
  agent key.
- A content scan proving protected values do not enter ATE policy detail or
  inappropriate logs.

## Stop for review if

Hometown would receive a key from Liam, would need Liam to call the gateway,
would make masking post-assembly, cannot provide the required answer/trace
contract, or cannot agree the ATE endpoint-link shape with Olive and the
program reviewers.
