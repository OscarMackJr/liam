# Work Prompt — Nono Owner / Implementer

You own the endpoint-control implementation required for Liam's first real
session. Read the cross-component execution specification, the ratified Nono
v0.4.1–v0.4.3 amendments, the desktop-agent decision, and the Liam v0.4 design
and test specifications before changing code.

## Objective

Implement and demonstrate the Nono side of the agent-session contract while
preserving the fork's startup-only, content-free, policy-free receipt
invariants.

## Required work

1. Validate agent profiles fail-secure: `inject_on` must be non-empty, limited
   to the approved service names, mapped to exact allowlisted hosts, and use
   `direct_gateway=false` for Liam.
2. Obtain the session agent key through Popeye before the child is resumed.
   Never inject the user key into child traffic and never silently re-mint.
3. Inject the session agent key only on Liam's Hometown and Olive hosts. Block
   direct gateway use from Liam's profile and retain endpoint-detection events
   for blocked egress.
4. Add the `AgentKeyInjection` receipt census row and receipt schema version 2.
   Preserve the existing receipt shape and its content-free scan invariants.
5. Write the schema-1 agent-session record at the attestation gate only for a
   resumed agent session. Write the session-end record after attempted
   revocation. The child receives paths to the receipt and agent record only.
6. Enforce the specified ACL split: the supervisor writes all records; the
   confined child can read the receipt and agent record but cannot write either;
   the child need not read the end record.
7. Run an AppContainer `DirectCli` session and report the actual `token_arm`.
   Review Liam's layer-expectancy table against the Nono layer registry. This
   closes Liam O7 only with explicit owner sign-off.

## Evidence required

- CI coverage for the ratified Nono requirements, including mint failure,
  invalid profile forms, receipt census completeness, content-free scan
  perturbations, record serialization, and no record for a refused session.
- A standard-user proof that the child cannot modify the receipt or agent
  record and receives the two agent-neutral environment paths.
- Capture-proxy proof that only Hometown and Olive receive the agent key, and
  that the user key is absent from child traffic.
- A short compatibility report giving record samples with no real key or
  protected content, exact enum spellings, the observed AppContainer arm, and
  commits/test commands.

## Do not do

- Do not put agent fields, revocation state, FQDNs, or requirement flags in the
  enforcement receipt.
- Do not create a Liam-specific IPC channel, edit Liam's profile without owner
  review, or grant files beyond the reviewed profile surface.
- Do not claim that the existing Nono fork already implements the amendments.

## Stop for review if

The Popeye exchange or introspection behavior differs from the ratified seam,
the actual AppContainer arm is not accepted by Liam's policy, a required layer
cannot be confirmed, or the proposed record shape violates the fork's
content-free scan.
