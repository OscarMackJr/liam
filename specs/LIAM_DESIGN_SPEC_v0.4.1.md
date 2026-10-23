# Liam — Windows Desktop Agent: Design Specification v0.4.1

Version: 0.4.1
Status: v0.4 **ratified 2026-10-19, POC scope**; implements the contract-only decision in `DECISION_desktop-agent-nono-integration.md`. In v0.4.1:
- **D14 (proxy transport) is ratified 2026-10-20** (`LIAM_AMENDMENT_v0.4.1_proxy-transport.md`) and is now folded into this text.
- **The contract-alignment changes (D15–D17, §3.3, §5, §7, §8.3, REQ-LIAM-19..21) are ratified 2026-10-21 on Liam's side.** They bind Liam only. The hometown-side wire changes they mirror (`HOMETOWN_v0.4` §2.6) are still decided at the ATE v0.4 joint review; if that review changes them, Liam follows in a new revision. See the ratification record at the end.

§4 is unchanged, so no re-ratification with `NONO_v0.4.3` is needed.
Repository: `liam` (new, under `oscarmackjr-twg`). Codename `liam` is a placeholder; renaming is a find-and-replace.
Program: Fiskroad. Liam is a **consumer** of hometown, olive and nono. It owns no governed store, holds **no credential of any kind**, and **never calls the model gateway**. Its requests to hometown and olive carry a short-lived session agent key that nono-proxy mints and injects; Liam never sees it.
Audience: the implementing LLM agent, the nono owner, the hometown owner, the olive owner, the popeye owner.
Companion: `LIAM_TEST_SPEC_v0.4.1.md`. Every REQ in §12 has at least one TC there.

**Changes in 0.4.1:**
- **Proxy transport (ratified 2026-10-20).** D14 folded in: D3 superseded, §3.2, §9, §10.2, exit code 11 and REQ-LIAM-03 revised, new assumption A5.
- **Contract alignment (ratified 2026-10-21, Liam's side):**
  - with `HOMETOWN_v0.4` §2.6, the agent answer endpoint. It is agreed on `contract/ate-v0.4` and confirms S-HT-1 with changes, but is not yet ratified at the ATE v0.4 joint review. Changes: error reasons from hometown's code (D16), the question bound, the new error codes, the four-key proposal;
  - with `DECISION_olive-poc2-phase1_v0.5.md` (v0.5.2, ratified 2026-10-20). Changes: Liam as submitter confirmed, closing O6; olive's v1 responses and errors (§7.4); proposal binding by hash, so Liam forwards the proposal unchanged (D15).

**Changes in 0.4 (receipt contract):** §4.2 rewritten against the fork's real `EnforcementReceipt` (`crates/nono/src/receipt.rs` at `29557a93`) plus the new supervisor-written `AgentSessionRecord` (`NONO_v0.4.3`), after the receipt-contract checkpoint (`contracts/receipt-fields.md`) showed the v0.3 table did not match the fork. Liam now carries its own layer-expectancy table (policy stays consumer-side, per the fork's D-12). `key_id` is a UUID (`POPEYE_v0.4.3`). Environment variables become agent-neutral (`NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`). New D13, O7.

**Changes in 0.3 (gap G5):** hometown makes the model call, not Liam. Liam asks hometown for an answer; hometown assembles masked context, calls the gateway on the session agent key and writes the ATE, which is the only way the trace can join the ledger through `primary_request_id` (`HOMETOWN_v0.4` §2.1). Consequences: the gateway is removed from Liam's allowlist and from `inject_on` (D12); the context client becomes an answer client (§5–§6); model selection, prompt templates and model-output parsing move to hometown; Liam keeps proposal validation and submission to olive (§7). Depends on the `NONO_v0.4.2` / `POPEYE_v0.4.2` pair (hometown relays model calls on the session key), ratified 2026-10-19 with `DECISION_agent-key-service-scope.md`.

**Changes in 0.2:** agent-key injection per `NONO_v0.4.1` / `POPEYE_v0.4.1`; receipt-integrity POC default (D10); partial-control statement (§9).

Read in this order: `DECISION_desktop-agent-nono-integration.md`, `DECISION_agent-key-service-scope.md`, `PROGRAM_SHARED_VOCABULARY_v0.4.md`, `vocabulary.yaml` (vendor unchanged), `NONO_FISKROAD_INTEGRATION_v0.3.md`, `NONO_v0.4.md`, `NONO_v0.4.1.md`, `NONO_v0.4.2.md`, `NONO_v0.4.3.md`, `POPEYE_CORRECTIONS_v0.3.md` §3, `POPEYE_v0.4.md`, `POPEYE_v0.4.1.md`, `POPEYE_v0.4.2.md`, `POPEYE_v0.4.3.md`, `HOMETOWN_v0.4.md` (§2.5–§2.6), `OLIVE_CHARTER_v0.4.md`, `DECISION_olive-poc2-phase1_v0.5.md`, `LIAM_AMENDMENT_v0.4.1_proxy-transport.md`, this document, then the test spec.

---

## 1. Purpose and Scope

### 1.1 What Liam is

Liam is a Windows desktop agent that performs one bounded task for a banker on request: it asks hometown for a governed answer about a party and, where the answer carries a proposed action, submits that proposal to olive. It runs confined under nono. Its value in the POC is the evidence chain it exercises from the endpoint side, not the task itself.

### 1.2 POC goal

Demonstrate, with records rather than assertions, that one run of a desktop agent:

1. started only inside an enforced nono session;
2. reached a model only through hometown, which called popeye on the session agent key — Liam itself never reached the gateway;
3. received only purpose-masked context, applied by hometown before the model saw it;
4. proposed an action to olive and executed nothing itself;
5. left records that join: nono receipt, hometown ATE, popeye ledger row, olive proposal, Liam run record.

### 1.3 In scope

- A command-line agent, invoked per task, exiting when the task ends.
- One task type: **"summarise and propose"** — given a party reference and a question, obtain hometown's answer and, if it carries a catalog-valid proposal, submit it to olive.
- The startup self-check against the nono receipt and agent-session record.
- A content-free local run log and run record.
- Stubs for hometown and olive.

### 1.4 Non-goals (POC)

- Any UI beyond the console.
- Executing actions. Liam never holds a write credential (DECISION §5.3).
- Calling the model gateway, choosing a model, or building prompts. Those are hometown's.
- Multi-step autonomous loops. One task, one answer request, at most one proposal, one exit.
- Reading or writing the user's documents, mailbox or browser.
- Shipping logs off the endpoint.
- Any dependency on nono's code, IPC or service beyond reading the receipt and agent-session record.

---

## 2. Design Decisions

| ID | Decision | Alternatives | Rationale |
|---|---|---|---|
| D1 | **Rust**, single static binary | .NET 9; Node | CLR fails under a `WRITE_RESTRICTED` token in the fork's field findings; AppContainer needs grants for every DLL sibling. One static binary minimises both. nono is Rust. |
| D2 | **Console process, one task per invocation** | Long-running agent | One process maps to one nono session and one receipt, the evidence unit. |
| D3 | *Superseded by D14 (v0.4.1).* Was: HTTP proxy from environment, no direct sockets | — | The literal wording could not be met by any HTTP client; D14 states what is actually guaranteed |
| D4 | **Session agent key minted and injected by nono-proxy on hometown and olive; Liam never reads it** | Liam holds a key; per-user key injection | `NONO_v0.4.1`, `NONO_v0.4.2`. Attribution is `principal_type=agent`, `agent_id=liam`, `on_behalf_of=<user>`. |
| D5 | **Answers come from hometown's answer API** | Liam fetches context and calls the model (v0.2) | hometown assembles purpose-masked context, makes the model call, and writes the ATE, so the trace joins the ledger through its own `primary_request_id`. Liam's v0.2 flow left hometown unable to trace the answer (G5). |
| D6 | **Actions only as olive proposals, submitted by Liam** | hometown submits the proposal | Liam is the agent principal olive authorizes. hometown "proposes" in the charter sense (it produces the suggestion inside a traced answer); Liam carries it to olive with hometown's `trace_ref`. **Confirmed by olive** (`DECISION_olive-poc2-phase1` §3, ratified 2026-10-20); O6 closed. |
| D7 | **Self-check is read-only, against two supervisor-written files, at startup only** | Runtime channel to nono | DECISION §3.2. |
| D8 | **Logs are content-free** | Prompt/response logging | Program no-content guardrail. |
| D9 | **Identity comes from the injected key; Liam adds `run_id` and `nono_session_id` only** | Liam authenticates | hometown and olive learn identity through popeye introspection (`POPEYE_v0.4.2`). |
| D10 | **Record integrity: ACL-only POC default; DSSE verification built and tested with a test key** | Wait; ship a production key | DECISION §7 Q2 unresolved. The fork already has keyed DSSE signing (`audit.rs`); if nono writes `<session_id>.agent.dsse.json`, Liam verifies it. Run record states `receipt_integrity: "acl-only"` or `"dsse"`. |
| D11 | **Purpose is bound to the session** | Per-request purpose | The key carries one purpose set at mint. Liam's `--purpose` must equal the agent-session record's `purpose`. Liam may send it to hometown as advisory; the key is authoritative (`HOMETOWN_v0.4` §4.2). |
| D13 | **Liam owns its layer expectancy; nono's records stay policy-free** | Ask nono to add `required` flags to the receipt | The fork deliberately keeps expectancy CLI-side (D-12). Liam carries a versioned expectancy table for its launch path (§4.3), reviewed by the nono owner against `layer_registry.rs`. |
| D12 | **The gateway is not on Liam's allowlist** | Allow the gateway for future use | Liam has no reason to reach it. Removing it turns "Liam never calls the model directly" from a code property into an enforced one: a direct attempt is blocked by nono and produces a detection. |
| D14 | **One socket destination, no name resolution** (ratified 2026-10-20) | IPC to nono; hand-written HTTP client; leave the wording silently untrue | Liam opens TCP connections only to nono-proxy at the validated `http://127.0.0.1:<proxy_port>` and resolves nothing at run time; service host names travel only inside CONNECT requests. Enforced in code: proxy URL validation, `NO_PROXY` empty, one client built with `no_proxy()`, `Proxy::all` and a deny-all resolver. Detail and alternatives: `LIAM_AMENDMENT_v0.4.1_proxy-transport.md` |
| D15 | **The proposal is forwarded unchanged** | Liam normalises or repairs proposals before submitting | hometown hashes the proposal it returns into the ATE (`answer.proposalHash`, RFC 8785 JCS over the four keys), and olive recomputes the hash over what Liam submits (`DECISION_olive-poc2-phase1` §17.2). Any change Liam makes, even a harmless-looking one, refuses an honest proposal with `trace_proposal_mismatch`. Liam's validation (§7.3) is therefore read-only: no coercion, no defaults, no stripping, no trimming, no key reordering beyond what JSON serialisation does |
| D16 | **Exit reasons come from the service's error code** | Reasons from the HTTP status | hometown returns `{"error": "<code>"}` (`HOMETOWN_v0.4` §2.6). With status-based reasons, its two different 422s look the same in Liam's record. Falls back to the status only when the body carries no code |
| D17 | **Liam checks the question bound before sending** | Let hometown reject it | hometown accepts 1..4000 characters. Checking locally costs nothing and turns a wasted round trip into a clear local reason |

---

## 3. Architecture

### 3.1 Components

```
+------------------------------------------------------------------+
|  nono session (AppContainer arm)                                  |
|                                                                   |
|   liam.exe                                                        |
|   +-- startup: config, receipt self-check, run_id mint            |
|   +-- answer client  ---------> HTTP(S) via proxy --> hometown    |
|   +-- action client  ---------> HTTP(S) via proxy --> olive       |
|   +-- evidence writer --> run record (JSON), run log (JSONL)      |
|                                                                   |
+-------------------------------|-----------------------------------+
                                | only path out
                          nono-proxy (loopback, exact port)
                           - allowlist: hometown, olive FQDNs (gateway NOT allowed)
                           - injects the session agent key on hometown and olive
                           - emits endpoint-detection/0.1 on blocks

      hometown --(introspect key)--> popeye
      hometown --(masked context; model call on the session agent key)--> popeye gateway
      hometown --> ATE (run.runId = Liam run_id; primary_request_id = gateway request)
      olive    --(introspect key)--> popeye
```

Liam's codebase contains two clients, the evidence writer and the stubs.

### 3.2 Process model

1. The endpoint launcher starts `nono` with the Liam profile, the session purpose and the task arguments, passing the same purpose to nono and to Liam's `--purpose`.
2. nono validates the purpose, mints the session agent key (`NONO_v0.4.1` §4.1), establishes its layers, writes the receipt and the agent-session record at the attestation gate (`NONO_v0.4.3`), and starts `liam.exe` with `HTTPS_PROXY` and `HTTP_PROXY` both set to `http://127.0.0.1:<proxy_port>`, `NO_PROXY=""`, `NONO_RECEIPT_PATH` and `NONO_AGENT_RECORD_PATH` (assumption A5). If the key cannot be minted, the session is refused (receipt `outcome = Refused`, no agent record) and Liam never starts.
3. Liam runs the startup sequence (§4). Any failure exits non-zero with one reason line and writes nothing else.
4. Liam runs the task (§3.3), writes the run record and log, and exits.
5. nono revokes the session key, writes the session-end record (`NONO_v0.4.3` §6), and removes the per-session filters. The receipt is not modified after launch.

### 3.3 Task flow — "summarise and propose"

| Step | Action | Component called | Record |
|---|---|---|---|
| 1 | Parse `--party <ref> --question "<text>" --purpose <purpose>`; the question must be 1..4000 characters (D17), else exit 11 `input:question_length` | — | `run.started` |
| 2 | Request an answer: `party_ref`, `question`, `purpose` (advisory), `run_id`, `nono_session_id` | hometown answer API (§5) | `answer.received` (trace_id, context_complete, has_proposal, latency) |
| 3 | Validate the returned proposal (if any) against the pinned olive catalog and parameter schema | — | `proposal.validated` or `proposal.suppressed` |
| 4 | If valid, submit it **unchanged** (D15) with hometown's `trace_id` as `trace_ref` | olive proposal API (§7) | `proposal.submitted` (proposal id, outcome, reason if refused) |
| 5 | Print the answer and the proposal id (if any) | — | `run.completed` |
| 6 | Write the run record | — | file |

Liam makes **exactly one** answer request and **at most one** proposal per run.

---

## 4. Runtime Contract with nono

### 4.1 nono profile (`profiles/liam.nono.toml`)

> **Editorial note (2026-10-22, Liam owner; not a contract change).** nono parses profiles
> only as JSON(C), so the profile is `profiles/liam.nono.json`. The TOML below is the spec
> form. nono 126.1's JSON translation keeps the same field names and meanings, and adds
> `agent.user_key` (a credential locator, never key material) and `agent.profile_version`.
> `agent.services` also carries `gateway` (nono derives the popeye exchange URL from it).
> gateway is not in `inject_on` and not on `network.allow_domain`, so D12 is unchanged.
>
> **Grant addition (2026-10-23, Liam owner).** The filesystem grants also include read-only
> access to the catalog (`contracts/catalog-0.4.json`) and vocabulary
> (`contracts/vendored/vocabulary.yaml`) files that `liam.toml` names, because Liam reads both
> at startup. Without them a real session exits 11.

Versioned in this repository, reviewed by the nono owner. Minimum content:

- **Executable:** `liam.exe`; read/traverse grant for its directory only.
- **Filesystem:** read-write on `%LOCALAPPDATA%\Fiskroad\liam\runs\`; read on the config file, the receipt and the agent-session record; nothing else. The user's virtual key location (`NONO_v0.4.1` §8 O1) is outside every grant.
- **Network:** proxy-only. Allow hometown's API FQDN and olive's API FQDN. **The gateway FQDN is not allowed** (D12). Block the direct provider domains in `POPEYE_CORRECTIONS_v0.3` §3.1. Inherits the `NONO_v0.4` §2.1 agent-egress restriction.
- **Agent block** (`NONO_v0.4.1` §3, `NONO_v0.4.2` §3):

  ```toml
  [agent]
  agent_id = "liam"
  purpose_source = "launch_arg"
  key_ttl_seconds = 3600
  inject_on = ["hometown", "olive"]

  [agent.services]
  hometown = "<hometown-api-fqdn>"
  olive    = "<olive-api-fqdn>"
  ```

- **Arm:** AppContainer. Low-IL is not permitted.
- **Profile version** (integer), recorded in the agent-session record as `profile_version`.

### 4.2 Startup self-check

Liam reads two files once, before any other work, and checks them against its configuration and its own launch arguments. Both shapes are fixed by the fork (`crates/nono/src/receipt.rs`, `NONO_v0.4.3`). Any parse error, unknown enum value, missing file or failed check exits 10 with `receipt:<field>` or `agent_record:<field>`; no run record is written.

#### 4.2.1 `EnforcementReceipt` (`NONO_RECEIPT_PATH`)

Example (agent session, abbreviated census):

```json
{
  "schema_version": 2,
  "session_id": "20261019-201500-7",
  "pid": 9132,
  "entry_path": "DirectCli",
  "token_arm": "<confirmed AppContainer-path value; O7>",
  "outcome": "Ran",
  "layers": [
    {"id": "AppContainerProfile", "status": "Confirmed"},
    {"id": "WfpEgressFilters", "status": "Confirmed"},
    {"id": "AgentKeyInjection", "status": "Confirmed"}
  ]
}
```

| Field | Liam's check |
|---|---|
| `schema_version` | `2` |
| `session_id` | Non-empty; equals the agent record's `session_id` |
| `pid` | Equals Liam's own process id (the receipt describes this launch, not another) |
| `entry_path` | `DirectCli` |
| `token_arm` | Not `LowIlPrimary`, `BrokerLaunch` or `BrokerLaunchNoPty`, and equal to the value configured in `nono.expected_token_arm` (O7) |
| `outcome` | `Ran` |
| `layers` | Exactly fourteen rows, one per `LayerId`, no duplicates; each row meets the expectancy table in §4.3 |

#### 4.2.2 `AgentSessionRecord` (`NONO_AGENT_RECORD_PATH`)

```json
{
  "schema_version": 1,
  "session_id": "20261019-201500-7",
  "agent_id": "liam",
  "purpose": "servicing",
  "key_id": "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
  "key_expires_at_unix": 1792445700,
  "injected_on": ["hometown", "olive"],
  "direct_gateway": false,
  "profile_version": 4,
  "proxy_port": 50413,
  "written_at_unix": 1792442100
}
```

| Field | Liam's check |
|---|---|
| `schema_version` | `1` |
| `session_id` | Equals the receipt's |
| `agent_id` | `liam` (config `agent.agent_id`) |
| `purpose` | Equals `--purpose` (D11) and is in the vendored vocabulary |
| `key_id` | Lowercase canonical UUID; copied into the run record for ledger joins |
| `key_expires_at_unix` | At least `2 × max(hometown.timeout_seconds, olive.timeout_seconds)` after Liam's current time |
| `injected_on` | Exactly the set `{hometown, olive}`; **no `gateway`** |
| `direct_gateway` | `false` |
| `profile_version` | ≥ `nono.profile_min_version` |
| `proxy_port` | Equals the port in `HTTPS_PROXY` |
| `written_at_unix` | Within `nono.record_max_age_seconds` of Liam's current time, and not more than 5 s in the future |

`on_behalf_of` is not in the record and Liam does not need it; attribution to the user is the ledger's job (`NONO_v0.4.3` §5).

### 4.3 Layer expectancy (Liam-side policy)

Liam's launch path is AppContainer on `DirectCli`. Each census row must meet the table below; a row outside its accepted set fails the check with `receipt:layer:<LayerId>`. The table is versioned in `src/expectancy.rs` and reviewed by the nono owner against the fork's `layer_registry.rs` expectancy for this arm (O7). If the review changes a row, this table changes; Liam does not infer expectancy at run time.

| `LayerId` | Accepted status |
|---|---|
| `AppContainerProfile` | `Confirmed` |
| `DaclPackageSidGrant` | `Confirmed` or `EstablishedNotIndependentlyObservable` |
| `DaclAncestorTraverse` | `Confirmed` or `EstablishedNotIndependentlyObservable` |
| `DaclAncestorReadAttrs` | `Confirmed` or `EstablishedNotIndependentlyObservable` |
| `WfpEgressFilters` | `Confirmed` |
| `JobObjectContainment` | `Confirmed` |
| `InterpreterCoverageGate` | `Confirmed` or `EstablishedNotIndependentlyObservable` |
| `AgentKeyInjection` | `Confirmed` |
| `MinifilterAbsence` | `NotApplicable` |
| `RestrictedToken`, `MandatoryIntegrityLabel`, `DaclSessionSidGrant`, `FirewallRulesEgress`, `BrokerAuthenticodeTrustGate` | Any status (not part of this arm's guarantee; recorded in the run record, not checked) |

### 4.4 Record integrity

Liam checks that neither file is **writable by its own token**. If `<session_id>.agent.dsse.json` exists next to the agent record, Liam verifies the DSSE envelope over both files against `nono.record_public_key` and records `receipt_integrity: "dsse"`; a present but invalid envelope fails with `receipt:signature`. Otherwise it records `receipt_integrity: "acl-only"` (D10).

### 4.5 What Liam does not do with nono

No IPC, no service calls, no profile edits, no nono state other than the receipt and agent-session record. Liam never reads the session-end record.

---

## 5. Answers (hometown)

### 5.1 Contract status

`HOMETOWN_v0.4` §2.6 (agreed on `contract/ate-v0.4` 2026-10-20, proposed for the ATE v0.4 joint review) confirms S-HT-1 with the changes listed in its "Changes to S-HT-1" table. Liam's stub contract below now carries those changes. Liam builds to it behind a client trait. A1 closes when the joint review ratifies §2.6; until then this remains a stub contract.

### 5.2 Request (stub contract S-HT-1)

`POST {hometown.base_url}/v1/agent/answer`

```json
{
  "party_ref": "<opaque party reference>",
  "question": "<user question>",
  "purpose": "<advisory; key is authoritative>",
  "run_id": "<uuidv7>",
  "nono_session_id": "<session_id from the receipt>",
  "step_role": "answer"
}
```

Exactly these six fields; hometown rejects missing or unknown fields with `400 invalid_request`. `question` is 1..4000 characters (D17). `run_id` is Liam's UUIDv7 in lowercase canonical form.

No `Authorization` header; nono-proxy injects the session agent key. hometown introspects it (`POPEYE_v0.4.2`), resolves `on_behalf_of` to directory groups, applies purpose masking before context assembly (`HOMETOWN_v0.4` §4.2), calls the gateway on the same key, and writes an ATE with `run.runId = run_id`, `run.stepRole = "answer"`, `policy.principalType = "agent"`, `policy.onBehalfOf` from introspection, and (proposed, gap G6) `endpoint.sessionId = nono_session_id`, `endpoint.keyId` from introspection.

### 5.3 Response (stub contract S-HT-1)

```json
{
  "trace_id": "<ATE id>",
  "answer": "<text>",
  "context_complete": true,
  "proposal": null | {
    "action_type": "<catalog id, action_id_format>",
    "party_ref": "<opaque>",
    "parameters": { },
    "justification": "<text>"
  }
}
```

`proposal` is always present, and `null` when there is none. When present it has exactly four keys, and hometown sends at most one (`HOMETOWN_v0.4` §2.6). Liam treats `answer` and `justification` as display text only, and `proposal` as data to validate (§7). The defensive checks stay: a proposal with other than four keys is suppressed (`schema`), and any additional proposals in a non-conforming response are ignored and logged as `proposal.suppressed` reason `limit`.

The trace binds the proposal (`answer.proposalHash`). Liam never modifies it (D15).

### 5.4 Errors, timeouts, retries

Error bodies are `{"error": "<code>"}` (`HOMETOWN_v0.4` §2.6). Liam's reason is `hometown:<code>` (D16). If the body is missing, unparseable or has no `error` string, the reason falls back to `hometown:<status>`.

| hometown response | Liam behaviour | Exit / reason |
|---|---|---|
| 401 `no_injected_key` / `key_inactive` | Fail; no retry (nono never re-mints) | 21 `hometown:no_injected_key` / `hometown:key_inactive` |
| 403 `purpose_refused` | Fail; no retry | 20 `hometown:purpose_refused` |
| 400 `invalid_request` | Fail; no retry | 21 `hometown:invalid_request` |
| 409 `run_conflict` | Fail; no retry | 21 `hometown:run_conflict` |
| 422 `agent_not_registered` / `party_not_found` | Fail; no retry | 21 `hometown:agent_not_registered` / `hometown:party_not_found` |
| Other 4xx | Fail; no retry | 21 `hometown:<code>`, or `hometown:<status>` without one |
| 503 `trace_not_saved` / `context_unavailable` / `unavailable`; 504 `deadline`; other 5xx; timeout | One retry, then fail | 21 `hometown:unavailable` |

Status handling (what retries, and the exit codes) is unchanged from v0.4. Only the reason strings change.

Timeout 90 s (the model call happens inside hometown). hometown keeps its own deadline at 80 s and returns `504 deadline` before Liam gives up, so Liam normally records a reason code rather than a bare timeout. A retry is a new step in the same run on hometown's side (next `stepIndex`). Rationale for a single retry is unchanged: a retry loop under a runaway condition is what popeye's run-cost alert exists to catch.

### 5.5 Incompleteness

If `context_complete=false`, the console answer is prefixed with a fixed incompleteness notice and the run record carries `context_complete=false`.

### 5.6 Secrets

No key, token or credential in Liam's environment, config, binary or logs. CI greps the binary strings and config schema for key-shaped values.

---

## 6. What Moved To hometown

For the avoidance of doubt, these v0.2 responsibilities are no longer Liam's and must not be implemented in Liam:

- model choice and the model-approval registry lookup;
- prompt templates and prompt assembly;
- context masking (it was always hometown's; now Liam never sees unmasked or masked context at all, only the answer);
- `gateway_request_id`, token counts and model latency (they live in the ATE and the ledger).

---

## 7. Actions (olive)

### 7.1 Contract status

`OLIVE_CHARTER_v0.4` fixes olive's boundary, and `DECISION_olive-poc2-phase1_v0.5.md` (ratified) fixes the v1 semantics: `POST /v1/proposals`, idempotency on `(agent_id, run_id)`, the pre-queue trace checks, and the refusal reasons. The v1 wire contract (JSON Schema plus OpenAPI 3.1) is not yet published. Its S-OL-1 → v1 compatibility table goes to joint review with the Liam owner and the independent reviewer, James O'Meara, before Liam's fixtures are regenerated (decision record §3, §12). Until then the shape below is a **stub contract** (A2). Liam adopts any optional v1 fields (for example an advisory `nono_session_id`, or `expires_in_seconds`) only after that review.

### 7.2 Request (stub contract S-OL-1)

`POST {olive.base_url}/v1/proposals`

```json
{
  "action_type": "<catalog id>",
  "party_ref": "<opaque>",
  "parameters": { },
  "justification": "<text>",
  "run_id": "<uuidv7>",
  "trace_ref": "<hometown trace_id>"
}
```

No `Authorization` header; the session agent key is injected. olive validates it through introspection, authorizes against `on_behalf_of` ∩ agent (one-line olive requirement, `NONO_v0.4.2` §8).

`action_type`, `party_ref`, `parameters` and `justification` are copied from hometown's `proposal` **unchanged** (D15). olive hashes those four fields and compares the hash with the trace's.

Response: `202 {proposal_id, status: "awaiting_approval"}` or `200 {proposal_id, status: "refused_policy", reason}`. Status values are the vocabulary's `action_outcomes`.

### 7.3 Liam's checks before submitting

- The proposal has exactly the four keys `action_type`, `party_ref`, `parameters`, `justification`.
- `action_type` is in the catalog version pinned in config and matches `action_id_format`.
- `parameters` validate against that action's schema from the catalog.
- **Validation is read-only (D15).** No type coercion (`"5"` against an integer schema fails; it is not converted), no injected defaults, no stripping of unknown keys, no trimming or Unicode normalisation. Liam submits the object it received, or nothing.
- On failure: no submission; `proposal.suppressed` with `not_in_catalog` or `schema`; the suggestion is printed as text only.

Liam submits once, records the response, and stops. It does not poll, approve, retry or execute.

### 7.4 olive responses and errors

From the ratified decision record. Error bodies carry a code, and Liam's reason is `olive:<code>`, falling back to `olive:<status>` (D16).

| olive response | Meaning | Liam behaviour | Exit / record |
|---|---|---|---|
| 202 `awaiting_approval` | Accepted and queued | Record `proposal.status` | 0 |
| 200 `refused_policy` with `reason` | Refused before queueing. Reasons include `not_in_catalog`, purpose and entitlement refusals, and the trace checks `trace_unresolved`, `trace_tenant_mismatch`, `trace_agent_mismatch`, `trace_proposal_mismatch` | Record status and reason; no resubmission | 0 |
| 200 with an existing `proposal_id` | An identical earlier submission for this run (idempotent replay) | Record as for 202 or 200 | 0 |
| 400 | Malformed body | Fail; no retry | 22 `olive:<code>` |
| 409 `idempotency_conflict` | A different proposal was already submitted for this `run_id` | Fail; no retry | 22 `olive:idempotency_conflict` |
| 422 `trace_not_found` (retryable on olive's side) | olive could not resolve the `trace_ref` | Fail; no retry. hometown saves the trace before answering, so this signals a fault, not a race | 22 `olive:trace_not_found` |
| 401 / 403 | Key or principal problem | Fail; no retry | 22 `olive:<code>` |
| 5xx or timeout | olive or a dependency unavailable | Fail; no retry | 22 `olive:unavailable` |

`trace_proposal_mismatch` from olive means Liam, or something between hometown and olive, changed the proposal. Treat it as a defect to investigate, not an expected refusal.

---

## 8. Evidence

### 8.1 Run record (`runs/<run_id>.json`)

```
run_id, agent_id, agent_version, started_at, ended_at, outcome,
nono: { session_id, profile_version, token_arm, receipt_integrity, agent_key_id,
        layers_outside_expectancy_scope: [ {id, status} ] },   # the "any status" rows, recorded not judged
purpose, party_ref_hash,
answer: { trace_id, context_complete },
proposal: { proposal_id, status } | null,
log_sha256
```

### 8.2 Run log (`runs/<run_id>.jsonl`)

Append-only, content-free. Events from §3.3. Allowed: identifiers, counts, versions, durations, status codes, reason codes. Prohibited: question, answer, justification, parameters values, party references, URLs with query strings.

### 8.3 Join keys

| From | To | Key |
|---|---|---|
| Liam run record | ATE | `answer.trace_id`; also `run_id` = ATE `run.runId` |
| ATE | popeye ledger | `primary_request_id` (hometown's gateway request) |
| popeye ledger | nono agent-session record | `key_id` |
| ATE | nono receipt and agent record | `endpoint.keyId` (authoritative) and `endpoint.sessionId` (asserted by Liam), proposed in `HOMETOWN_v0.4` §2.6, gap G6 |
| nono agent record | nono session-end record | `session_id`, `key_id` |
| olive proposal | ATE | `trace_ref` |
| olive proposal | ATE `answer.proposalHash` | SHA-256 of the RFC 8785 form of the four proposal fields (olive §17.2) |

---

## 9. Security Requirements

- **One socket destination, no name resolution (D14).** Liam connects only to nono-proxy at `127.0.0.1:<proxy_port>`; it refuses to start unless `HTTPS_PROXY`/`HTTP_PROXY` are that exact loopback URL and `NO_PROXY` is empty (`env:no_proxy`, `env:proxy_not_loopback`, `env:no_proxy_nonempty`). The single HTTP client disables system proxy detection, uses only that proxy, and carries a deny-all DNS resolver. Liam's own source contains no socket or resolver calls. Winsock and `getaddrinfo` are linked by the HTTP library; the guarantee is behavioural and tested (TC-NET-05 to -08).
- No credentials anywhere in Liam (§5.6).
- No write credential (D6).
- No file access outside the three grants.
- No child processes.
- No execution of anything hometown returns. `answer` is printed; `proposal` is validated as data against fixed schemas.
- No modification of the proposal between hometown and olive (D15).
- No request to the gateway FQDN; it is not allowlisted (D12).
- Fail closed: exit non-zero with one reason line.

**Coverage limit.** Per `NONO_v0.4` §3, nono governs network egress and attributes it via the agent key; Liam's local file reads are bounded by AppContainer grants and not observed by the detection pipeline. Demonstrations describe coverage as network-level plus filesystem confinement only.

---

## 10. Configuration

### 10.1 `liam.toml`

```toml
[agent]
version = "0.4.1"
agent_id = "liam"

[nono]
profile_name = "liam"
profile_min_version = 4          # integer, per NONO_v0.4.3 §5
record_max_age_seconds = 120
record_public_key = ""           # PEM; empty = acl-only (D10)
expected_token_arm = ""          # set once O7 is confirmed; empty = refuse to start

[hometown]
base_url = "https://<hometown-api-fqdn>"
timeout_seconds = 90

[olive]
base_url = "https://<olive-api-fqdn>"
catalog_version = "0.4"
timeout_seconds = 30

[vocabulary]
path = "contracts/vendored/vocabulary.yaml"
sha256 = "60a44f00..."           # spec value; see README for any recorded temporary deviation

[evidence]
runs_dir = "%LOCALAPPDATA%\\Fiskroad\\liam\\runs"
```

There is no `[gateway]` section.

### 10.2 Environment

`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`, `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`. Nothing else.

`HTTPS_PROXY` and `HTTP_PROXY` must be identical and of the form `http://127.0.0.1:<port>`, with the port equal to the agent-session record's `proxy_port`. `NO_PROXY` must be unset or empty. Liam does not read proxy settings from the Windows registry or WinINET (D14).

### 10.3 Stub mode

`--features stubs` and `LIAM_STUBS=1` point both clients at in-repo stubs. The self-check still runs against stub receipt and agent-record fixtures. Absent from release builds.

---

## 11. Repository Layout

```
liam/
  README.md, AGENTS.md
  specs/
  profiles/liam.nono.toml
  contracts/vendored/             vocabulary.yaml + VENDORED.md
  contracts/                      receipt-fields.md (checkpoint report), agent-record.schema.json,
                                  hometown-answer.S-HT-1.json,
                                  olive-proposal.S-OL-1.json, catalog-0.4.json (stub copy)
  src/
    main.rs, receipt.rs, agent_record.rs, expectancy.rs
    clients/{hometown,olive}.rs   each behind a trait so a confirmed contract replaces the stub shape
    evidence.rs
    task/summarise_propose.rs
  stubs/                          hometown-stub, olive-stub, receipt + agent-record fixture pairs
  scripts/launch-under-nono.ps1, make-stub-records.ps1
  tests/
  .github/workflows/ci.yml
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Run completed |
| 10 | Receipt or agent-record self-check failed |
| 11 | Environment or input check failed: proxy missing, proxy not the loopback literal, `NO_PROXY` non-empty, vocabulary hash or purpose, question length (`input:question_length`) |
| 20 | hometown refused the purpose |
| 21 | hometown answer failed (key, other 4xx, unavailable); reason from hometown's error code (D16) |
| 22 | olive submission failed (4xx other than a 200 refusal, 5xx, timeout); reason from olive's error code (§7.4) |
| 30 | Evidence write failed after the task ran |

---

## 12. Requirements

| REQ | Requirement | Verified by |
|---|---|---|
| REQ-LIAM-01 | Liam refuses to start unless the receipt and agent-session record are present, non-writable, mutually consistent and fresh; the receipt shows `Ran` on `DirectCli` with the configured token arm and a fourteen-row census meeting §4.3; and the agent record matches config, `--purpose`, `{hometown, olive}` and `direct_gateway=false` | TC-START-01..16, TC-E2E-03, -04, -06 |
| REQ-LIAM-02 | Liam refuses to start without proxy variables | TC-START-08 |
| REQ-LIAM-03 | Liam opens network connections only to nono-proxy at `127.0.0.1:<proxy_port>`, performs no name resolution, ignores system proxy settings, and refuses to start on any non-conforming proxy environment | TC-NET-01..03, TC-NET-05..08 |
| REQ-LIAM-04 | Liam sends no `Authorization` header to hometown or olive and holds no key; a key failure from hometown fails the run without retry | TC-KEY-01..03 |
| REQ-LIAM-05 | The answer request carries `run_id` and `nono_session_id`; the olive request carries `run_id` and hometown's `trace_id` as `trace_ref` | TC-META-01 |
| REQ-LIAM-06 | `purpose` is validated against the vendored vocabulary; the vendored file's hash is checked | TC-LVOC-01, -02 |
| REQ-LIAM-07 | Answers come only from hometown; `context_complete=false` is surfaced and recorded; a purpose refusal ends the run with no olive request | TC-ANS-01..04 |
| REQ-LIAM-08 | Proposals go only to olive, only when catalog-valid and schema-valid; Liam never executes | TC-ACT-01..05, TC-ACT-08, -09 |
| REQ-LIAM-09 | Exactly one answer request and at most one proposal per run | TC-ACT-06, TC-META-02 |
| REQ-LIAM-10 | Run record and log per §8, content-free, referencing the nono session and hometown trace | TC-EVD-01..05 |
| REQ-LIAM-11 | No child processes; no file access outside the three grants | TC-SEC-01, -02 |
| REQ-LIAM-12 | Nothing returned by hometown is executed; proposals are parsed as data | TC-SEC-03 |
| REQ-LIAM-13 | Failures exit with the documented code and one reason line | TC-FAIL-01..08 |
| REQ-LIAM-14 | A blocked egress attempt produces an `endpoint-detection/0.1` event | TC-E2E-02 |
| REQ-LIAM-15 | Release builds contain no stub code and no key-shaped strings | TC-BUILD-01, -02 |
| REQ-LIAM-16 | The model call made for Liam's run is attributed in the ledger as `principal_type=agent`, `agent_id=liam`, `on_behalf_of=<user>`, with the agent record's `key_id` | TC-E2E-05 |
| REQ-LIAM-17 | Liam makes no request to the gateway; a deliberate attempt is blocked by nono and detected | TC-NET-04, TC-E2E-07 |
| REQ-LIAM-18 | The ATE for the run joins to Liam's run record, the ledger and the olive proposal through the §8.3 keys | TC-E2E-01 |
| REQ-LIAM-19 | The proposal reaches olive exactly as hometown returned it; Liam's validation never coerces, defaults, strips or normalises (D15) | TC-ACT-07, TC-ACT-03 |
| REQ-LIAM-20 | Exit reasons for hometown and olive errors come from the service's error code, falling back to the status (D16) | TC-ANS-05, TC-ANS-07, TC-ACT-08 |
| REQ-LIAM-21 | A question outside 1..4000 characters is refused before any request (D17) | TC-ANS-06 |

---

## 13. Assumptions and Open Items

Assumptions (record in `README.md`):

- A1. hometown provides an agent answer endpoint with the semantics in §5.2. `HOMETOWN_v0.4` §2.6 confirms S-HT-1 with changes (agreed 2026-10-20); it remains a stub contract until the ATE v0.4 joint review ratifies §2.6. **Checkpoint item.** The endpoint needs hometown's HTTP server on the Postgres trace store with forced row-level security; the server refuses to start otherwise.
- A2. olive provides a proposal endpoint with the semantics in §7.2–§7.4 and a published `0.4` catalog. Semantics ratified (`DECISION_olive-poc2-phase1` v0.5.2); the v1 wire contract and its S-OL-1 compatibility table are not yet published or reviewed. **Checkpoint item.** olive Phase 1 is simulation-only, and checks traces against fixtures until hometown and popeye deliver.
- A3. popeye implements the agent key exchange (`POPEYE_v0.4.1`), introspection, relay of model calls by hometown and `direct_gateway` (`POPEYE_v0.4.2`). Both ratified; implementation status unconfirmed.
- A4. nono implements agent-key minting (`NONO_v0.4.1`), injection on `inject_on` services with `direct_gateway` at mint (`NONO_v0.4.2`), the `AgentKeyInjection` census row, `AgentSessionRecord` and `SessionEndRecord` (`NONO_v0.4.3`, ratified), and emits `endpoint-detection/0.1`. Implementation status unconfirmed.

- A5. nono-proxy places `http://127.0.0.1:<port>` (IPv4 literal, not `localhost`) in the confined child's `HTTPS_PROXY` and `HTTP_PROXY`, with the same port it records as `proxy_port`, and sets `NO_PROXY` empty. To be confirmed by the nono owner; if nono uses another form, the fix is on nono's side, because a host-name proxy URL would require Liam to resolve a name (D14).

A3 and A4 are ratified specifications, not yet confirmed implementations: Liam's stubs accept an injected credential header, and TC-E2E-01, -05 and -07 stay `not measured` until both are implemented.

Open items, owned outside Liam:

- O1. Receipt and agent-record shapes: **resolved** by the checkpoint report and `NONO_v0.4.3` (ratified 2026-10-19). Integrity mechanism remains open (DECISION Q2; `NONO_v0.4.3` O2).
- O2. ATE `endpoint` object for `nono_session_id` and `key_id`: **proposed** in `HOMETOWN_v0.4` §2.6 (`keyId` verified from introspection, `sessionId` asserted by Liam); decided at the ATE v0.4 joint review (gap G6).
- O3. Profile distribution: launcher script or Intune (endpoint team).
- O4. Retention for `runs/` (DECISION Q1).
- O5. hometown and olive authentication — **resolved at specification level** 2026-10-19 (`NONO_v0.4.2`, `POPEYE_v0.4.2`, `DECISION_agent-key-service-scope.md`, ratified). Implementation pending.
- O6. **Closed 2026-10-20.** olive confirms Liam, the agent principal, submits proposals; hometown generates them inside traced answers (`DECISION_olive-poc2-phase1` §3, olive charter §2 amendment).
- O7. The `token_arm` value the fork records for an AppContainer launch on `DirectCli`, and the nono owner's review of the §4.3 expectancy table against `layer_registry.rs` (`NONO_v0.4.3` O1). Until confirmed, `expected_token_arm` is empty and Liam refuses to start outside stub mode; stub fixtures use a placeholder value.

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**: Liam v0.4 design, including the receipt and agent-record self-check and Liam-owned layer expectancy |
| Date | 2026-10-19 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Ratified together with | `NONO_v0.4.3.md`, `POPEYE_v0.4.3.md`, `LIAM_DESIGN_SPEC_v0.4.md`, `LIAM_TEST_SPEC_v0.4.md` (one package) |
| Scope ratified | Whole document as of this date, including D1–D13, §4.2–§4.4 and §12 |
| Not ratified here | Liam O7 / `NONO_v0.4.3` O1 (AppContainer `token_arm` value and the nono owner's review of Liam's expectancy table); `NONO_v0.4.3` O2 (whether DSSE becomes required) and O3 (receipt directory and ACL); Liam A1/A2 (S-HT-1 and S-OL-1 remain stub contracts, authorized behind traits); O2, O3, O4, O6 |
| Effect | The implementing agent restarts at handoff step 3 against §4.2–§4.4. `expected_token_arm` ships empty until O7 closes, so only stub-mode runs start |
| Change control | Changes to §4 require re-ratification with `NONO_v0.4.3`; other changes require a new Liam revision |

### v0.4.1: proxy transport

| Field | Value |
|---|---|
| Decision | **Ratified**: D14 permits Liam's single validated loopback-proxy socket destination and prohibits runtime name resolution |
| Date | 2026-10-20 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Record | `LIAM_AMENDMENT_v0.4.1_proxy-transport.md` |
| Scope ratified | D14; §§9, 10.2, 11 (exit code 11) and REQ-LIAM-03 as folded in here; assumption A5 |

### v0.4.1: contract alignment

| Field | Value |
|---|---|
| Decision | **Ratified (Liam's side)**: D15–D17; §3.3 steps 1 and 4; §5.1–§5.4; §7.1–§7.4; §8.3 additions; exit codes 21 and 22 and the `input:question_length` reason; REQ-LIAM-19..21; A1, A2, O2 and O6 updates |
| Date | 2026-10-21 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Binds | Liam only. The mirrored hometown wire changes (`HOMETOWN_v0.4` §2.6) stay proposed until the ATE v0.4 joint review; A1 stays open until then. If the review changes them, Liam follows in a new revision |
| Depends on | `HOMETOWN_v0.4` §2.6 (agreed, not yet ratified at the ATE v0.4 joint review); `DECISION_olive-poc2-phase1_v0.5.md` (ratified) |
| Review | The S-HT-1 changes are part of the ATE v0.4 joint review, with the Liam owner; the S-OL-1 → v1 changes go to the olive compatibility review with James O'Meara as independent reviewer |
| Does not change | §4 (no re-ratification with `NONO_v0.4.3`); D1–D14 |
| Change control | As v0.4 |
