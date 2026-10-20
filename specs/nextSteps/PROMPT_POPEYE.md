# Work Prompt — Popeye Owner / Implementer

You own the agent-key authority needed by Nono, Hometown, and Olive. Read the
cross-component execution specification, Popeye v0.4.1–v0.4.3, the agent-key
scope decision, and the shared vocabulary before implementation.

## Objective

Implement a short-lived, attributable session agent key that Nono can obtain,
Hometown and Olive can inspect, and only registered Hometown can relay to the
gateway for Liam's `direct_gateway=false` sessions.

## Required work

1. Implement agent-key exchange authenticated by a user key. Derive
   `on_behalf_of` server-side, validate registered agent ID and purpose, cap
   TTL, inherit caller constraints, and reject non-user callers.
2. Return a lowercase canonical UUID `key_id`; record issuance and revocation
   events without storing the key secret in logs or response echoes.
3. Implement idempotent revocation limited to the minting user identity.
4. Implement read-only introspection for the registered Hometown and Olive
   service callers. Active responses contain only the ratified identity and
   constraint fields. Inactive, unknown, revoked, expired, and non-agent keys
   return only `active: false`.
5. Enforce the positive-introspection cache limit of 60 seconds and never past
   expiry.
6. Implement registered-relay validation. Hometown may make a model request
   using the agent key plus its own relay identity; the ledger records
   `relayed_by=hometown`. Direct calls and non-Hometown relays on a
   `direct_gateway=false` key receive a distinct refusal.
7. Maintain reviewed registries for agent IDs, introspection callers, and
   relays. Adding one is a program decision, not runtime configuration.

## Evidence required

- Tests for each `REQ-AGT-05` through `REQ-AGT-19`, including all negative
  identity, cache, revocation, and relay cases.
- A live or integration proof that one Hometown-relayed Liam call produces an
  agent-attributed ledger row with `key_id`, purpose, `on_behalf_of`, and
  `relayed_by=hometown`.
- A secret-sentinel scan proving an agent key never appears in introspection,
  ledger/event output, or test artifacts.
- Versioned interface documentation sufficient for Nono to implement exchange
  and for Hometown/Olive to implement introspection without guessing headers,
  authentication, or error codes.

## Do not do

- Do not expose a key secret through introspection, extend or refresh a key via
  introspection, or allow a service to mint/delegate a key.
- Do not make Popeye decide Hometown field masking or Olive action policy.
- Do not treat a direct gateway call as a valid substitute for the registered
  Hometown relay.

## Stop for review if

The requested identity fields or service-authentication mechanism differ from
the ratified contract, the registry owner/location is unsettled, or the relay
mechanism cannot reliably distinguish Hometown from an arbitrary key holder.
