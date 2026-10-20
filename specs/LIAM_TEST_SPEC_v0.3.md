# Liam — Windows Desktop Agent: Test Specification v0.3

Version: 0.3
Status: **Draft, POC scope** — companion to `LIAM_DESIGN_SPEC_v0.3.md`. Section references (§) point there unless stated.
Audience: the implementing LLM agent, reviewers of the POC evidence.

**Changes in 0.3 (gap G5):** the gateway stub is removed; hometown's answer endpoint absorbs the model-path behaviours. TC-CTX replaced by TC-ANS. TC-KEY, TC-META, TC-FAIL and TC-ACT rewritten against hometown and olive stubs. New TC-START-13 (injection set), TC-NET-04 (gateway blocked), TC-E2E-07 (gateway attempt detected). TC-E2E-01 and -05 revised for hometown-made model calls.

**Changes in 0.2:** agent-key receipt fixtures; TC-START-10..12; TC-E2E-05/06; TC-LVOC rename.

Principle: **a test passes when Liam refuses, fails closed, or produces the required record.** A run that proceeds with a check missing is a failure even if nothing bad happened. Results are measurements with their limits, never claims.

---

## 1. Environments and Tags

| Tag | Where | Token | Notes |
|---|---|---|---|
| `ci-elevated` | GitHub `windows-latest` | elevated | Logic, stubs, schema, build. Not sufficient for anything depending on token type or ACLs. |
| `dev-standard-user` | Developer workstation, standard account, nono fork installed | standard | ACL, receipt-writability, real-nono tests. |
| `manual-hardware` | Managed workstation with CrowdStrike Falcon, live program endpoints | standard | End-to-end and detection seam. Coordinate with the endpoint team. |

A test is never marked verified from an environment with a different tag.

---

## 2. Fixtures

### 2.1 Stubs (`stubs/`, feature `stubs`)

| Stub | Behaviours |
|---|---|
| `hometown-stub` (S-HT-1) | 200 answer without proposal; 200 with one valid proposal; 200 with an out-of-catalog proposal; 200 with schema-invalid parameters; 200 with two proposals (non-conforming); 200 with `context_complete=false`; 200 whose answer and justification contain command text and extra JSON keys; 401 no key; 401 key inactive; 403 purpose refused; 409; 429; 500; 100 s delay |
| `olive-stub` (S-OL-1) | 202 `awaiting_approval`; 200 `refused_policy` with reason; 400 schema error; 500; catalog endpoint serving `catalog-0.4.json` |
| `gateway-trap` | A listener on a host standing in for the gateway FQDN, used only to prove Liam never reaches it (TC-NET-04). It records any hit. |

Every stub writes each request (method, path, header names with an `Authorization` presence flag, body) to a capture file.

**Injection simulation.** In `ci-elevated` runs there is no nono-proxy. The capturing test proxy (TC-NET-01) adds a sentinel credential header on hometown and olive requests only, standing in for nono's injection, so stubs can assert the header arrived from the proxy and not from Liam.

### 2.2 Receipt fixtures (`stubs/receipts/`)

| Fixture | Content |
|---|---|
| `valid.json` | All required layers `active` incl. `agent_key`; arm `appcontainer`; `proxy_only`; `agent` object (`agent_id=liam`, `purpose=servicing`, `key_id`, `key_expires_at` +1 h, `injected_on=[{hometown,…},{olive,…}]`); fresh timestamp |
| `no-agent-key.json` | `agent_key` layer and `agent` object absent |
| `agent-mismatch.json` | `agent.agent_id=other` |
| `purpose-mismatch.json` | `agent.purpose=analytics` with `--purpose servicing` |
| `key-expiring.json` | `agent.key_expires_at` +60 s (below the 180 s floor from `2 × 90 s`) |
| `inject-gateway.json` | `injected_on` includes a `gateway` entry |
| `inject-missing-olive.json` | `injected_on` lacks `olive` |
| `degraded-layer.json`, `absent-layer.json`, `wrong-arm.json`, `stale.json`, `wrong-profile.json`, `old-profile-version.json`, `port-mismatch.json` | As named |
| `signed-valid.json` / `signed-bad.json` | Signature valid / invalid against the test public key |

Writability is set by the harness (TC-START-06).

### 2.3 Real nono

The nono fork at a pinned commit with `profiles/liam.nono.toml`. Every result records the fork commit, receipt version and Windows build (`ver` plus UBR).

---

## 3. Test Cases

Columns: TC | REQ | Setup | Steps | Pass criteria | Tag.

### 3.1 Startup and receipt self-check

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-START-01 | 01 | `valid.json`, proxy env, stubs | Run | Exit 0; run record written; `run.started` first log event | ci-elevated |
| TC-START-02 | 01 | `degraded-layer.json` | Run | Exit 10 `receipt:layers`; no record, no log, no stub request | ci-elevated |
| TC-START-03 | 01 | `absent-layer.json` | Run | Exit 10 `receipt:layers`; no requests | ci-elevated |
| TC-START-04 | 01 | `wrong-arm.json` | Run | Exit 10 `receipt:arm`; no requests | ci-elevated |
| TC-START-05 | 01 | `stale.json`; `wrong-profile.json`; `old-profile-version.json`; `port-mismatch.json` | Run per fixture | Exit 10 with matching reason; no requests | ci-elevated |
| TC-START-06 | 01 | `valid.json`, ACL grants write to test user | Run | Exit 10 `receipt:writable` | dev-standard-user |
| TC-START-07 | 01 | `signed-valid.json` with key; `signed-bad.json` | Run per fixture | Valid: exit 0, `receipt_integrity=signed`. Bad: exit 10 `receipt:signature` | ci-elevated |
| TC-START-08 | 02 | `valid.json`, proxy env unset | Run | Exit 11 `env:no_proxy`; no requests | ci-elevated |
| TC-START-09 | 01 | No receipt | Run | Exit 10 `receipt:missing`; no requests | ci-elevated |
| TC-START-10 | 01 | `no-agent-key.json` | Run | Exit 10 `receipt:layers`; no requests | ci-elevated |
| TC-START-11 | 01 | `agent-mismatch.json`; `purpose-mismatch.json` | Run per fixture | Exit 10 `receipt:agent_id` / `receipt:purpose`; no requests | ci-elevated |
| TC-START-12 | 01 | `key-expiring.json` | Run | Exit 10 `receipt:key_expiry`; no requests | ci-elevated |
| TC-START-13 | 01 | `inject-gateway.json`; `inject-missing-olive.json` | Run per fixture | Exit 10 `receipt:injected_on`; no requests | ci-elevated |

### 3.2 Egress path

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-NET-01 | 03 | Stubs reachable only through a capturing proxy; direct ports firewalled from the test process | Run | Both stubs receive requests via the proxy; direct listeners record zero hits | ci-elevated |
| TC-NET-02 | 03 | Real nono session; loopback listener on a non-proxy port; LAN lab listener | Run with the direct-connect test flag | Both refused (10013 or refused); task completes via proxy; one schema-valid `endpoint-detection/0.1` event per blocked attempt (popeye `config/endpoint-detection.schema.json`, else `POPEYE_CORRECTIONS_v0.3` §3.2) | dev-standard-user |
| TC-NET-03 | 03, 17 | Static check | Grep for socket construction outside the HTTP client, DNS resolution calls, and any reference to a gateway host or path | Zero matches outside `clients/`; no gateway reference anywhere in source or config schema | ci-elevated |
| TC-NET-04 | 17 | `gateway-trap` listening; test proxy configured to refuse the gateway host as nono's allowlist would | Full run, plus a run with a test flag that attempts one request to the gateway host | Normal run: zero hits on `gateway-trap`. Flagged run: the attempt is refused by the proxy, Liam's run still completes via hometown, and `gateway-trap` records zero hits | ci-elevated |

### 3.3 Credential handling

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-KEY-01 | 04 | Stub captures, injection simulation on | Run | No request **leaving Liam** carries `Authorization` (asserted at the proxy's inbound side); hometown and olive captures show only the proxy's sentinel credential | ci-elevated |
| TC-KEY-02 | 04 | `hometown-stub` 401 no key; separately 401 key inactive | Run per response | Exit 21 `hometown:no_injected_key` / `hometown:key_inactive`; exactly one hometown request each; no olive request | ci-elevated |
| TC-KEY-03 | 04, 15 | Release build | Scan binary strings, config schema, all test-run logs for key-shaped values | Zero matches | ci-elevated |

### 3.4 Run identity

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-META-01 | 05 | `valid.json`; hometown returns a valid proposal | Run | hometown capture: `run_id` (UUIDv7), `nono_session_id` = receipt `session_id`, `step_role=answer`. olive capture: same `run_id`, `trace_ref` = hometown's `trace_id`. Run record: same `run_id`, `answer.trace_id`, `nono.agent_key_id` = receipt `agent.key_id` | ci-elevated |
| TC-META-02 | 09 | Any successful run | Count hometown requests | Exactly one | ci-elevated |

### 3.5 Vocabulary

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-LVOC-01 | 06 | `--purpose invented` | Run | Exit 11 `vocab:purpose`; no requests | ci-elevated |
| TC-LVOC-02 | 06 | Vendored vocabulary modified by one byte | Run | Exit 11 `vocab:sha256`; no requests | ci-elevated |

### 3.6 Answers (hometown)

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-ANS-01 | 07 | 200 answer, no proposal | Run | Answer printed; exit 0; no olive request; run record `proposal=null`, `answer.trace_id` set | ci-elevated |
| TC-ANS-02 | 07 | 200 with `context_complete=false` | Run | Console begins with the incompleteness notice; run record `context_complete=false` | ci-elevated |
| TC-ANS-03 | 07 | 403 purpose refused | Run | Exit 20 `hometown:purpose_refused`; no retry; no olive request; run record outcome `purpose_refused` | ci-elevated |
| TC-ANS-04 | 07 | Static check | Grep for prompt templates, model names, completion parsing or context-masking code | None present in Liam (design §6) | ci-elevated |

### 3.7 Actions (olive)

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-ACT-01 | 08 | hometown returns a valid proposal | Run | One olive request per S-OL-1 with `run_id`, `trace_ref`, schema-valid parameters; run record `proposal.status=awaiting_approval`; proposal id printed | ci-elevated |
| TC-ACT-02 | 08 | Out-of-catalog proposal | Run | No olive request; `proposal.suppressed` `not_in_catalog`; suggestion printed as text | ci-elevated |
| TC-ACT-03 | 08 | Schema-invalid parameters | Run | No olive request; `proposal.suppressed` `schema`; exit 0 | ci-elevated |
| TC-ACT-04 | 08 | `olive-stub` returns `refused_policy` | Run | Run record `proposal.status=refused_policy` with reason; exit 0; no resubmission | ci-elevated |
| TC-ACT-05 | 08 | Static check | Grep for HTTP destinations other than the two base URLs and olive methods other than proposal submission and catalog read | Zero matches | ci-elevated |
| TC-ACT-06 | 09 | hometown returns two proposals | Run | One olive request (first); second logged `proposal.suppressed` `limit` | ci-elevated |

### 3.8 Evidence

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-EVD-01 | 10 | Completed run | Validate against `run-record.schema.json` | Valid; `nono.session_id` matches receipt; `log_sha256` matches log | ci-elevated |
| TC-EVD-02 | 10 | Sentinel `CANARY-7f3a` in party ref, question, answer, justification and parameters | Scan record, log, stdout, stderr | Sentinel only in stdout; never in record or log | ci-elevated |
| TC-EVD-03 | 10 | Completed run | Check party handling | Record holds `party_ref_hash` only | ci-elevated |
| TC-EVD-04 | 10 | hometown 500 twice (exit 21) | Inspect | Record outcome `hometown_unavailable`; log ends `run.failed` with reason | ci-elevated |
| TC-EVD-05 | 10 | Self-check failure (exit 10) | Inspect `runs/` | No new files | ci-elevated |

### 3.9 Security

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-SEC-01 | 11 | Real nono session, process trace | Run | Zero child processes | dev-standard-user |
| TC-SEC-02 | 11 | Real nono session, file trace | Run | Accesses only exe dir (read), config (read), receipt (read), `runs/` (write); no `ACCESS_DENIED` elsewhere | dev-standard-user |
| TC-SEC-03 | 12 | hometown answer and justification contain command text; response has extra keys; a parameter holds a shell string | Run | Response parsed against S-HT-1; extra keys ignored; shell string only inside a schema-valid string field or triggers `proposal.suppressed`; nothing executed | ci-elevated |

### 3.10 Failure modes

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-FAIL-01 | 13 | hometown 500 then 200 | Run | Two hometown requests; exit 0 | ci-elevated |
| TC-FAIL-02 | 13 | hometown 500 then 500 | Run | Two requests; exit 21 `hometown:unavailable` | ci-elevated |
| TC-FAIL-03 | 13 | hometown 429; separately 409 | Run per response | One request each; exit 21 (no retry on 4xx) | ci-elevated |
| TC-FAIL-04 | 13 | hometown delays 100 s | Run | Timeout at 90 s, one retry, exit 21 within 190 s | ci-elevated |
| TC-FAIL-05 | 13 | `runs/` not writable | Run | Answer printed; exit 30 | dev-standard-user |
| TC-FAIL-06 | 13 | olive 500 | Run | Answer printed; exit 22; one olive request | ci-elevated |
| TC-FAIL-07 | 13 | Any failure | Inspect stderr | Exactly one line `liam: <code> <reason>` | ci-elevated |

### 3.11 Build

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-BUILD-01 | 15 | Release build | Strings scan for stub paths, `LIAM_STUBS`, stub symbols | Zero matches | ci-elevated |
| TC-BUILD-02 | 15 | Release build | Static binary; DLL imports | Windows system DLLs only | ci-elevated |

### 3.12 End-to-end

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-E2E-01 | 18 | Managed workstation, nono fork, live hometown/popeye/olive (POC instances), standard user, v0.4.2 pair implemented | Launch via `launch-under-nono.ps1` with a POC tenant party and an approved purpose | Exit 0. Records exist and join per design §8.3: Liam run record → ATE (`trace_id`, `run.runId`); ATE → ledger (`primary_request_id`); ledger → receipt (`key_id`); olive proposal → ATE (`trace_ref`). ATE has `policy.principalType=agent` and `onBehalfOf` = launching user. An auditor can answer "was this produced inside an enforced sandbox, for whom, under which purpose?" from the records alone | manual-hardware |
| TC-E2E-02 | 14 | As above, direct-connect flag | Run once | One `endpoint-detection/0.1` event per blocked attempt reaches popeye and raises the "nono detections" alert | manual-hardware |
| TC-E2E-03 | 01 | As above, `liam.exe` launched directly | Run once | Exit 10 or 11; no network activity (capture) | manual-hardware |
| TC-E2E-04 | 01 | As above, nono WFP service stopped | Launch | nono refuses the session, or Liam exits 10 on a degraded receipt. Record which | manual-hardware |
| TC-E2E-05 | 16 | As TC-E2E-01 | Query the ledger for rows carrying the run's ATE `primary_request_id` | Each row: `principal_type=agent`, `agent_id=liam`, `on_behalf_of` = launching user, `purpose` = launch purpose, `key_id` = receipt `agent.key_id`. No row for the run is attributed to hometown's own application key. Receipt shows `agent.key_revoked=true` after the session | manual-hardware |
| TC-E2E-06 | 01 | As TC-E2E-01, exchange unreachable | Launch | nono refuses the session; Liam never starts; no traffic on the user key in the ledger | manual-hardware |
| TC-E2E-07 | 17 | As TC-E2E-01, gateway-attempt flag | Run once | Attempt blocked by nono (gateway not allowlisted); one `endpoint-detection/0.1` event with the gateway host as `destination`; run still completes via hometown | manual-hardware |

---

## 4. Reporting

| Claim | TC | n of n | Tag | Windows build (ver + UBR) | Liam commit | nono commit | Limits / not measured | Status |

Status: `verified`, `refuted`, `not measured`. Three runs minimum for `dev-standard-user` and `manual-hardware`. State what was not measured. No extrapolation across tags, builds or nono arms. Retain sentinel and capture files.

Record every stub contract in force (S-HT-1, S-OL-1, or the confirmed contract that replaced it) with each result set.

---

## 5. Exit Criteria for the POC

1. Every `ci-elevated` TC green on the pinned toolchain.
2. Every `dev-standard-user` TC `verified`, 3 of 3.
3. TC-E2E-01, -03, -04, -05, -06 and -07 `verified` on at least one managed workstation; TC-E2E-02 has carried one detection end to end.
4. S-HT-1 and S-OL-1 confirmed or replaced by owner-approved contracts.
5. `NONO_v0.4.2` / `POPEYE_v0.4.2` ratified and implemented; ATE gap G6 resolved or recorded as blocking.
6. The vendored vocabulary hashes to the spec value `60a44f00…e8e7` (any recorded placeholder deviation removed).
7. Results published in the `liam` repository and summarised for the Fiskroad coordination repository.

Not required and must not be claimed: fleet distribution, log shipping, execution of actions, CrowdStrike coexistence beyond the nono fork's own soak.
