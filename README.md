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

**Phase 2 gate 1 (2026-10-22): MISMATCH, STOP FOR REVIEW.** The pinned fork
`cf3ebc56` (nono 0.70.0) does not implement the ratified successor contract: its
receipt is schema 1 with a 13-layer census (no `AgentKeyInjection`), there is no
agent-session record, no `NONO_RECEIPT_PATH` / `NONO_AGENT_RECORD_PATH`, and
neither `nono run` nor `nono agent launch` has `--purpose`. See
`contracts/receipt-fields.md` and `.planning/phases/02-real-nono-session/02-SIGNOFF.md`.
`scripts/launch-under-nono.ps1` (`nono run --purpose`) does not match the fork;
it is recorded, not rewritten. Liam source is unchanged pending nono-owner review.

**Announced nono deviation, now implemented (amendment v0.4.2, ratified 2026-10-22,
implemented in Phase 02.1).** nono-proxy does not terminate TLS, so it cannot inject the
agent key into a CONNECT tunnel as `NONO_v0.4.2` section 3 first assumed. nono instead
gives the child per-service reverse-proxy URLs (`NONO_SERVICE_HOMETOWN_URL`,
`NONO_SERVICE_OLIVE_URL`, and a gateway URL Liam never reads) and refuses CONNECT to any
`inject_on` FQDN in agent sessions. Liam now implements D14 as service-URL transport:
it connects only to the loopback routes `http://127.0.0.1:<proxy_port>/svc/hometown` and
`/svc/olive` taken from those two variables, with no forward proxy, no name resolution,
and `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY` and `NONO_SERVICE_GATEWAY_URL` ignored. This is
verified locally against stubs only; real-session evidence is Phase 2 plan 02-07, after
nono 126.1 is built. A record or URL port mismatch now exits 11 `env:service_url_port`
(it was 10 `agent_record:proxy_port`). Open item O-E: TC-E2E-03 needs redesign at its own
phase, because Liam never names a host. `profiles/liam.nono.toml` is unchanged (no
CONNECT-tied expectation found). Also announced, compatible with Liam: records under
`%PROGRAMDATA%\nono\receipts\<session_id>\` (resolves `NONO_v0.4.3` O3); mint failure
spawns no child and writes a Refused receipt with `pid=0`; purpose stays on
`nono run --profile <p> --purpose <p>`.

## Current implementation boundary

Liam now performs the startup record checks, validates the pinned vocabulary,
uses service-URL (loopback, no-proxy) Hometown and Olive clients behind traits, validates proposals
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
  **superseded by amendment v0.4.2**: Liam ignores the forward-proxy variables and
  reads `NONO_SERVICE_HOMETOWN_URL` / `NONO_SERVICE_OLIVE_URL` instead.
- D-03: `contracts/catalog-0.4.json` is a stub catalog shape carrying
  `parameters_schema` only (no `required_parameters`). The real olive catalog
  shape is checked at the Phase 4 contract diff. The validator supports
  only `type`, `properties`, `required`, `additionalProperties` and
  `maxLength`. Loading the catalog is a startup prerequisite, next to the
  vocabulary check: a missing or unparseable catalog, one with any other key,
  a version other than `olive.catalog_version`, or an `action_id_format`
  other than `^[a-z0-9_.]+@[0-9]+$` exits `11 config:catalog` before any
  network request and writes no run record (fail closed on contract drift).
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
  same list. A `refused_policy` without a guarded reason code is also not a
  valid proposal response. A hometown 200 without the required `proposal`
  key (S-HT-1; `null` when there is none) is `hometown:unavailable`.
- Identifier guard (Liam-side guard pending confirmation in S-HT-1 /
  S-OL-1): hometown's `trace_id` and olive's `proposal_id` must match
  `^[A-Za-z0-9._:-]{1,128}$`. An empty or out-of-pattern id makes the
  response invalid: hometown exits `21 hometown:unavailable` (no retry),
  olive exits `22 olive:unavailable`. The run record schema pins both
  fields to the same pattern, so no free text reaches the record, the log
  or stdout through them.
- Harness limits: the local capture listener and stubs use plain http with
  origin-form requests on the `/svc/hometown` and `/svc/olive` routes; no
  `CONNECT` is ever sent. Real nono-proxy route behaviour is left to the live phases.
- Evidence note: `layers_outside_expectancy_scope` is written as a list of
  `{"id", "status"}` objects, as in design spec 8.1.
- Console output: every line Liam prints to stdout (the answer, the
  `Suggestion (not submitted): ...` line and olive's `Proposal <id>: <status>`
  line) has control characters other than `\n` and `\t` replaced with
  U+FFFD: C0 controls including ESC and `\r`, DEL, and C1 (U+0080..U+009F).
  ANSI/OSC/CSI sequences therefore cannot move the cursor, erase the notice
  line or fake a proposal line. Other text is printed unchanged, and the
  evidence files are not affected (they are content-free). The stderr reason
  line only ever carries guarded codes. Proposed as a spec line under §9.
- Evidence location: `evidence.runs_dir` is resolved during the startup
  checks. If `%LOCALAPPDATA%` cannot be expanded (variable unset or empty),
  any other `%` placeholder remains, or the result is not an absolute path,
  Liam exits `11 config:runs_dir` before any network request and writes no
  run record.

## Known gaps (deferred to §4 re-ratification)

Both items below are design spec §4.4 (record integrity, D10). Changes to §4
need re-ratification with `NONO_v0.4.3`, so the code is deliberately
unchanged until then (user decision, code review 01 iteration 2).

- WR-05, effective write access (ties to `NONO_v0.4.3` O3, receipt directory
  and ACL): `ensure_not_writable` (`src/receipt.rs`) opens each record with
  `write(true)` and treats `PermissionDenied` as "not writable". On Windows
  that open is also denied when `FILE_ATTRIBUTE_READONLY` is set, even if the
  DACL grants Liam's token full control. A file Liam could rewrite (by
  clearing the attribute with `FILE_WRITE_ATTRIBUTES`, or through `WRITE_DAC`
  / `WRITE_OWNER`) therefore passes. The stub fixtures (`src/stubs.rs`,
  `scripts/make-stub-records.ps1`) protect the records with the read-only
  attribute, not a deny-write DACL, so the tests cannot tell a
  supervisor-owned ACL apart from a user-writable file with the read-only
  flag. Open question for O3: whether "writable by its own token" includes
  `FILE_WRITE_ATTRIBUTES`, `WRITE_DAC` and `WRITE_OWNER`, and whether the
  fixtures move to deny-write DACLs.
- WR-06, DSSE envelope not detected (ties to `NONO_v0.4.3` O2): Liam never
  looks for `<session_id>.agent.dsse.json` next to the agent record. An
  existing envelope, valid or not, is ignored and every run records
  `receipt_integrity: "acl-only"`; a present but invalid envelope does not
  fail with `receipt:signature` as §4.4 requires. Only a non-empty
  `nono.record_public_key` is refused (`10 receipt:signature`).

## Exit codes and reasons

Each failure prints exactly one line, `liam: <code> <reason>`, on stderr.
Codes follow design spec §11; the reasons added by this repository are
marked as such.

| Code | Meaning | Reasons |
|---:|---|---|
| 10 | Receipt or agent-record self-check failed | `receipt:*`, `agent_record:*` |
| 11 | Environment, configuration or input check failed (before any network request, no run record) | `args:required`, `input:question_length`, `config:missing`, `config:parse`, `config:catalog` (this repo), `config:runs_dir` (this repo), `vocab:missing`, `vocab:sha256`, `vocab:parse`, `vocab:purpose`, `env:service_url_missing`, `env:service_url_not_loopback`, `env:service_url_port`, `env:service_url_route` (the former `env:no_proxy`, `env:no_proxy_nonempty` and `env:proxy_not_loopback` are retired) |
| 20 | hometown refused the purpose | `hometown:purpose_refused` |
| 21 | hometown answer failed | `hometown:<code>`, `hometown:<status>`, `hometown:unavailable` |
| 22 | olive submission failed | `olive:<code>`, `olive:<status>`, `olive:unavailable` |
| 30 | Evidence write failed after the task ran | `evidence:write` |

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

