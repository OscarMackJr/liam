# Cross-Component Execution Specification

Status: execution plan for the ratified v0.4.1–v0.4.3 package.

## 1. Reader and outcome

The reader is the owner or implementing agent for Hometown, Nono, Olive, or
Popeye. After completing the applicable work order, the reader must be able to
hand a named, versioned contract and test evidence to the next component
without exposing a key, weakening endpoint containment, or silently widening
an API.

The program outcome is one real Liam run that starts under Nono, receives an
answer from Hometown, optionally submits one proposal to Olive, and leaves a
joinable evidence chain. It is not achieved by stubs alone.

## 2. Governing decisions

The contract-only desktop-agent decision and the agent-key service-scope
decision are ratified. They establish these invariants:

- Liam is a confined consumer. It owns no credential, makes no model call,
  opens no direct network path, and never executes an action.
- Nono mints no credential itself. Its proxy obtains a short-lived session
  agent key through Popeye, injects it only on profile-named program services,
  and writes content-free supervisor records.
- Hometown authenticates the injected key through Popeye, applies
  purpose-based masking before context reaches a model, relays the model call
  under the agent key, and produces the answer trace.
- Olive authenticates the injected key through Popeye, authorizes and queues a
  proposal, and is the only component that may hold the target-system write
  credential.
- Popeye alone issues, introspects, revokes, and attributes agent keys. It is
  semantics-blind.

No component may replace these rules with a client-held key, direct gateway
access from Liam, an unreviewed service, a runtime Liam-to-Nono control channel,
or a token forwarded to a target system.

## 3. Required seams

| Seam | Producer | Consumer | Exit evidence |
|---|---|---|---|
| Agent-key exchange, introspection, relay and revocation | Popeye | Nono, Hometown, Olive | Versioned interface, negative tests, key-safe audit/ledger records |
| Profile, proxy injection and session records | Nono | Liam | Refused-session and resumed-session tests; schema-2 receipt plus schema-1 agent record |
| Answer API and ATE linkage | Hometown | Liam, Popeye, Olive | Owner-approved wire contract and trace proving the model call is relayed |
| Proposal API and review lifecycle | Olive | Liam, Popeye, Hometown | Owner-approved wire contract and policy-before-queue evidence |
| Real endpoint run | Liam with all owners | Program | Tagged results table with only measured claims |

## 4. Delivery order and gates

### Gate 0 — freeze the contracts

Hometown and Olive must each publish an owner-approved wire contract. The
current S-HT-1 and S-OL-1 shapes are explicitly local stubs, not production
contracts. The contracts must define request and response schemas, error
taxonomy, authentication expectations, idempotency/retry behavior, and
correlation fields. A mismatch with Liam's client is a stop condition.

At the same gate, Nono's owner must confirm the observed `token_arm` value for
an AppContainer `DirectCli` session and review Liam's layer-expectancy table.
Liam's shipped expected arm remains unset until then.

### Gate 1 — Popeye and Nono implementation

Implement the ratified v0.4.1–v0.4.3 seams together. Nono cannot report an
agent session as resumed unless key minting, profile validation, injection, and
the `AgentKeyInjection` census row succeed. Popeye cannot expose identity or
relay access outside the registered callers and relay.

Exit only when Nono can produce a schema-2 receipt and schema-1 agent-session
record for a real confined process, and Popeye can show the corresponding
content-free issuance, introspection, relay, and revocation evidence.

### Gate 2 — Hometown and Olive implementation

Implement the approved contracts behind their normal service authentication.
Both services receive the session agent key from Nono's proxy, never from Liam.
Hometown relays model calls only as the registered relay and writes an ATE that
joins `run_id` to its model request. Olive never forwards the session agent key
to a target system and performs policy validation before it queues a proposal.

The ATE endpoint-link gap remains open until the joint Hometown/Olive review
adopts the field shape. Do not claim a receipt-to-ATE lookup until that occurs.

### Gate 3 — Liam standard-user validation

With Gates 0–2 complete, run Liam under Nono as a standard user. Verify the
receipt and agent-record ACLs, startup failures before any network request,
proxy-only egress, injected-key behavior, answer flow, optional proposal flow,
revocation/end record, and content-free evidence. Record every case in the
results table with environment and commit identifiers.

### Gate 4 — end-to-end evidence review

Join the Nono receipt and agent record, Popeye ledger, Hometown ATE, Olive
proposal, Liam run record, and session-end record using their sanctioned keys.
Any missing or contradictory record is a detection/review event, not a
successful run.

## 5. Shared test expectations

- Test positive and negative paths. A happy-path response alone is not seam
  evidence.
- Use sentinel values to prove that secrets and protected content do not reach
  logs, evidence, or responses where prohibited.
- Do not mark Liam TC-E2E-01, TC-E2E-05, or TC-E2E-07 measured while either
  Hometown or Olive remains on a stub contract.
- Keep results scoped: `CI`, `ci-elevated`, `dev-standard-user`, and
  `manual-hardware` are different environments.
- Version every artifact that crosses a component boundary and capture the
  component commits used in an integration run.

## 6. Stop conditions

Stop and request joint review when any of these occurs:

1. The approved wire contract differs from Liam's existing stub shape.
2. A service requires Liam to hold, send, log, or refresh a credential.
3. Nono cannot establish a required layer, cannot protect its startup records,
   or reports an unreviewed AppContainer arm.
4. Popeye cannot bind `direct_gateway=false` to a key or cannot distinguish a
   registered Hometown relay from direct use.
5. Hometown would assemble context before purpose masking or would make Liam
   call the model gateway.
6. Olive would queue a proposal before policy refusal, execute without the
   required approval, or forward the session key.

## 7. Authoritative inputs

Use the ratified desktop-agent integration decision, agent-key scope decision,
Nono v0.4.3, Popeye v0.4.1–v0.4.3, Liam design and test specifications v0.4,
Hometown v0.4, Olive Charter v0.4, and the shared vocabulary. Later amendments
supersede earlier receipt sections; in particular, agent facts belong in the
agent-session and session-end records, not in the startup-only receipt.
