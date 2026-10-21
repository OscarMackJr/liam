# Liam

Version 0.4.1 (design and test specs v0.4.1; labels below read 0.4.1).

Status: **POC implementation in progress; offline build and unit-contract
checks pass.**

Liam is a Windows command-line desktop agent in the Fiskroad program. It runs
under nono, obtains an answer only from Hometown, and submits only validated
proposals to Olive. It never receives a credential, calls the model gateway,
or executes an action.

## Receipt-contract checkpoint

The supplied nono receipt source is pinned to `OscarMackJr/nono` commit
`29557a93`. Its receipt contract differed materially from Liam v0.3 §4.2; see
`contracts/receipt-fields.md`. The ratified NONO v0.4.3 / Liam v0.4 package
resolves that mismatch through a schema-2 receipt and a separate supervisor
agent-session record. Liam mirrors these contracts without depending on nono.

## Current implementation boundary

Liam now performs the startup record checks, validates the pinned vocabulary,
uses proxy-forced Hometown and Olive clients behind traits, validates proposals
against the local stub catalog, and writes content-free local evidence. The
wire contracts remain owner-authorized stubs: S-HT-1 and S-OL-1.

The feature-gated `stubs` module provides scripted local Hometown/Olive stubs
(a sequence of canned replies per server, so retry and multi-step paths can be
exercised) and a forwarding capture proxy. It simulates nono-proxy injection at
the proxy boundary only; default release builds exclude it.
Their machine-readable repository contracts are
`contracts/hometown-answer.S-HT-1.json` and
`contracts/olive-proposal.S-OL-1.json`; the local stub catalog is
`contracts/catalog-0.4.json`.

The optional DSSE record integrity mode is deliberately fail-closed: a
non-empty `record_public_key` causes startup refusal until the nono audit key
format is supplied and its verifier can be added. Windows effective-ACL checks
and the capturing proxy integration harness remain outstanding; see
`RESULTS.md` for measured scope.

## Running under nono

Build with `cargo build --locked --offline --release`, then use
`scripts/launch-under-nono.ps1`. Run only from a standard-user `nono` session
with the reviewed profile. The managed-workstation / hardware procedure is not
yet verified, and TC-E2E-01, -05 and -07 remain not measured while A3/A4 are
unconfirmed.

## Assumptions

- A1: Hometown's agent answer wire contract is currently the trait-backed
  stub contract S-HT-1, pending owner confirmation.
- A2: Olive's proposal wire contract is currently the trait-backed stub
  contract S-OL-1, pending owner confirmation.
- A3: Popeye's ratified agent-key exchange, introspection, hometown relay,
  and `direct_gateway` controls are not yet confirmed as implemented.
- A4: Nono's ratified agent-key minting, service injection, and detection
  behavior are not yet confirmed as implemented.

## v0.4.1 conformance notes

Assumption numbering note: A1-A4 above follow design spec section 13. The
items below that the conformance phase calls A3/A4/A5/A7 are implementation
assumptions of this repository, labelled "conformance A#" to avoid the clash;
each is recorded here for user review and none is a confirmed upstream fact.

- A1/A2 status: S-HT-1 and S-OL-1 are still owner-authorized stub contracts.
  The v0.4.1 shapes (four-key proposal, `{"error": code}` bodies, guard
  pattern `^[a-z][a-z0-9_]{0,63}$`, HTTP status decides retry and exit) are
  stated in the two contract JSON files. Spec A5 (nono-proxy puts the IPv4
  literal `http://127.0.0.1:<port>` in `HTTPS_PROXY`/`HTTP_PROXY`) is
  **unverified until Phase 2** (nono contract diff).
- D-03: `contracts/catalog-0.4.json` is a stub catalog shape carrying
  `parameters_schema` only (no `required_parameters`). The real olive catalog
  shape is checked at the Phase 4 contract diff.
- D-04: deviation beyond design spec 7.3: a hometown proposal whose
  `party_ref` does not match the requested party is suppressed with reason
  `schema`. It is suppress-only (Liam never rewrites the proposal), so D15
  still holds.
- D-07 (additive): the run record's `proposal` carries `reason` (a guarded
  code for `refused_policy`, otherwise `null`). Conformance A7 (additive): a
  run that failed at olive also records `answer {trace_id, context_complete}`.
  Both are in `contracts/run-record.schema.json`.
- Conformance A3: a non-conforming hometown response carrying two proposals is
  modelled as a proposal array; the first is used, the rest are ignored and
  logged as `proposal.suppressed` reason `limit`. Pending confirmation.
- Conformance A4 (resolved): only a hometown 403 whose body error code is
  `purpose_refused` is a purpose refusal (exit 20). Any other 403, including
  one without a usable error body, follows the "Other 4xx" row of spec 5.4:
  no retry, exit 21 with `hometown:<code>` or `hometown:403`.
- Conformance A5: an unparseable hometown 200 is `hometown:unavailable`
  without retry; an olive 2xx that is not a valid proposal response is
  `olive:unavailable`. "Valid" includes the status: it must be one of the
  vocabulary's `action_outcomes` (design spec 7.2), and a 202 must be
  `awaiting_approval`. A 200 idempotent replay is recorded as the returned
  status (TC-ACT-09). The run record schema pins `proposal.status` to the
  same list.
- Harness limits: the local capture proxy and stubs use plain http with
  absolute-form requests; HTTPS `CONNECT` tunnelling through nono-proxy is
  not exercised locally and is left to the live phases.
- Evidence note: `layers_outside_expectancy_scope` is written as a list of
  `{"id", "status"}` objects, as in design spec 8.1.

## Vocabulary

`contracts/vendored/vocabulary.yaml` is the canonical Fiskroad vocabulary (fiskroad
`contracts/vocabulary.yaml`, v0.4, LF), vendored byte-for-byte and verified to
SHA-256 `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`,
which `liam.toml` pins. `.gitattributes` marks `contracts/vendored/**` as
`-text` so checkouts do not alter the bytes.

## Offline dependencies

`vendor/` and `Cargo.lock` were refreshed from the supplied
`liam-vendor-bundle-2.tar.gz`; `.cargo/config.toml` enforces the vendored,
offline source. The bundle supplies reqwest 0.12.28 with Rustls and Windows
ACL bindings. No dependency download is needed for the recorded build checks.

