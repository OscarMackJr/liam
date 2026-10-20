# Liam

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

The feature-gated `stubs` module provides local Hometown/Olive one-request
fixtures and a forwarding capture proxy. It simulates nono-proxy injection at
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

## Vocabulary deviation

`contracts/vendored/vocabulary.yaml` is vendored unchanged from the supplied
placeholder input. Its SHA-256 is
`f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116`.
The canonical `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`
remains the POC exit criterion; this placeholder must not be treated as a
passing canonical-vocabulary check.

## Offline dependencies

`vendor/` and `Cargo.lock` were refreshed from the supplied
`liam-vendor-bundle-2.tar.gz`; `.cargo/config.toml` enforces the vendored,
offline source. The bundle supplies reqwest 0.12.28 with Rustls and Windows
ACL bindings. No dependency download is needed for the recorded build checks.

