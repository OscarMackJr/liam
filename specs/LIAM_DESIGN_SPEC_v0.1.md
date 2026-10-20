# Liam — Windows Desktop Agent: Design Specification v0.1

Version: 0.1
Status: **Draft, POC scope** — implements the contract-only decision in `DECISION_desktop-agent-nono-integration.md`
Repository: `liam` (new, under `oscarmackjr-twg`). Codename `liam` is a placeholder; renaming is a find-and-replace.
Program: Fiskroad. Liam is a **consumer** of hometown, popeye, olive and nono. It owns no governed store and holds no credential except its own per-user gateway key, which it never reads (§5.3).
Audience: the implementing LLM agent, the nono owner, the popeye owner, the olive owner.
Companion: `LIAM_TEST_SPEC_v0.1.md`. Every REQ in §12 has at least one TC there.

Read in this order: `DECISION_desktop-agent-nono-integration.md`, `PROGRAM_SHARED_VOCABULARY_v0.4.md`, `vocabulary.yaml` (vendor unchanged), `NONO_FISKROAD_INTEGRATION_v0.3.md`, `OLIVE_CHARTER_v0.4.md`, `POPEYE_v0.4.md` §4 (run metering), this document, then the test spec.

---

## 1. Purpose and Scope

### 1.1 What Liam is

Liam is a Windows desktop agent that performs one bounded task for a banker on request: it assembles governed context, calls an approved model through the program gateway, and either answers or **proposes** an action. It runs confined under nono. It is the first agent built to exercise every Fiskroad seam from the endpoint side, so its value in the POC is the evidence it produces, not the task it performs.

### 1.2 POC goal

Demonstrate, with records rather than assertions, that one run of a desktop agent:

1. started only inside an enforced nono session;
2. reached models only through popeye, by way of `nono-proxy`;
3. obtained context only through hometown's read path;
4. proposed an action to olive and executed nothing itself;
5. left a run record that the ATE and the nono receipt both reference.

### 1.3 In scope

- A command-line agent, invoked per task, exiting when the task ends.
- One task type for the POC: **"summarise and propose"** — given a party reference and a question, produce a short answer and, where the answer implies an action in olive's catalog, submit a proposal.
- The startup self-check against the nono receipt.
- A content-free local run log and a run record for the ATE.
- Stubs for hometown, popeye and olive so the agent can be tested without the live program.

### 1.4 Non-goals (POC)

- Any UI beyond the console. No tray, no window, no notifications.
- Executing actions. Liam never holds a write credential (DECISION §5.3).
- Multi-step autonomous loops. One task, bounded tool calls, one exit.
- Reading or writing the user's documents, mailbox or browser.
- Shipping logs off the endpoint. The POC forwards files by hand, as the nono detection contract does.
- Any dependency on nono's code, IPC or service. Liam reads one file (the receipt) and nothing else of nono's.
- Confinement of Liam by Liam. If nono is absent, Liam does nothing.

---

## 2. Design Decisions

| ID | Decision | Alternatives | Rationale |
|---|---|---|---|
| D1 | **Rust**, single static binary, no runtime dependency | .NET 9 (olive/bluto house style); Node | Field findings in the nono fork: .NET/PowerShell hosts fail to start the CLR under a `WRITE_RESTRICTED` token, and AppContainer requires explicit grants for every DLL sibling. One static binary minimises the grant list and keeps Liam runnable in every nono arm. nono itself is Rust, so one toolchain is reviewed. If the implementer has a strong reason to differ, stop and say so. |
| D2 | **Console process, one task per invocation** | Long-running service or tray agent | A per-invocation process maps one-to-one onto a nono session and a receipt, which is the evidence unit the ATE references. Long-running agents blur that boundary. |
| D3 | **HTTP proxy from environment, no direct sockets** | Direct egress with an allowlist in Liam | The fence is nono's. Liam has no network policy of its own; it uses the proxy nono-proxy exposes and treats any other route as a bug. |
| D4 | **Gateway key is injected by nono-proxy; Liam never reads it** | Liam holds the per-user virtual key | Matches `NONO_FISKROAD_INTEGRATION_v0.3` §2.1 (credential injection). A compromised Liam then has nothing to exfiltrate. |
| D5 | **Context only from hometown's read API** | Local retrieval; direct database access | Preserves hometown as the single governed read path and keeps purpose masking and definition provenance where they belong. |
| D6 | **Actions only as olive proposals** | Liam calls target systems | DECISION §5.3; olive charter §2. |
| D7 | **Receipt self-check is read-only, against a file, at startup only** | Runtime channel to nono | DECISION §3.2 rejected a channel. One check at start, fail closed. |
| D8 | **Logs are content-free** | Full prompt/response logging for debugging | Program no-content guardrail at the endpoint. Debugging uses stub mode (§10.3), never production logs. |
| D9 | **Identity is the Windows user plus the gateway's attribution; Liam adds `agent_id` and `run_id` only** | Liam performs its own authentication | popeye attributes by key; hometown and olive authorise by principal. Liam labels, it does not authenticate. |

---

## 3. Architecture

### 3.1 Components

```
+------------------------------------------------------------------+
|  nono session (AppContainer arm)                                  |
|                                                                   |
|   liam.exe                                                        |
|   +-- startup: config, receipt self-check, run_id mint            |
|   +-- context client  --------> HTTP(S) via proxy --> hometown    |
|   +-- model client    --------> HTTP(S) via proxy --> popeye      |
|   +-- action client   --------> HTTP(S) via proxy --> olive       |
|   +-- evidence writer --> run record (JSON), run log (JSONL)      |
|                                                                   |
+-------------------------------|-----------------------------------+
                                | only path out
                          nono-proxy (loopback, exact port)
                           - allowlist: gateway, hometown, olive FQDNs
                           - injects per-user virtual key on gateway calls
                           - emits endpoint-detection/0.1 on blocks
```

Everything outside the box is owned elsewhere. Liam's codebase contains the four clients, the evidence writer, and the stubs.

### 3.2 Process model

1. The endpoint launcher (POC: a PowerShell script owned by the endpoint recipe) starts `nono` with the Liam profile and the task arguments.
2. nono establishes its layers, writes the session receipt, and starts `liam.exe` inside the session with `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY=""` and `LIAM_RECEIPT_PATH` set.
3. Liam performs the startup sequence (§4). Any failure exits non-zero with a reason code and writes nothing else.
4. Liam runs the task (§3.3), writes the run record and log, and exits.
5. nono closes the session, finalises the receipt, and removes the per-session filters.

### 3.3 Task flow — "summarise and propose"

| Step | Action | Component called | Record |
|---|---|---|---|
| 1 | Parse `--party <ref> --question "<text>" --purpose <purpose>` | — | `run.started` |
| 2 | Fetch party context for the purpose | hometown read API (§6) | `context.fetched` (counts and provenance refs only) |
| 3 | Build prompt from a versioned template plus context | — | `prompt.built` (template version, token estimate) |
| 4 | Call model with `run_id`, `agent_id`, `purpose` in metadata | popeye via proxy (§5) | `model.called` (model, request id, token counts, latency) |
| 5 | Parse response: answer text plus zero or one proposed action in a fixed JSON shape | — | `response.parsed` (has_proposal) |
| 6 | If a proposal is present and its `action_type` is in olive's catalog, submit it | olive proposal API (§7) | `proposal.submitted` (proposal id, outcome) |
| 7 | Print the answer and the proposal id (if any) to the console | — | `run.completed` |
| 8 | Write the run record referencing the receipt | — | file |

Liam makes **at most one** model call and **at most one** proposal per run in the POC. Loops are a non-goal.

---

## 4. Runtime Contract with nono

### 4.1 nono profile (`profiles/liam.nono.toml`)

Versioned in this repository, reviewed by the nono owner (DECISION §7 Q4). Minimum content:

- **Executable:** `liam.exe` path; read/traverse grant for its directory only.
- **Filesystem:** read-write grant on `%LOCALAPPDATA%\Fiskroad\liam\runs\` (evidence); read on the config file; read on the receipt path; nothing else. No grant on user documents, profile root or temp.
- **Network:** proxy-only. Allow `gateway-azure.ai.twg.internal` (or the current Container App FQDN), hometown's API FQDN, olive's API FQDN. Block the direct provider domains listed in the popeye contract. Credential injection of the per-user virtual key on the gateway FQDN only.
- **Arm:** AppContainer. The Low-IL interactive arm is **not** permitted for Liam, because it lacks per-session SID-keyed WFP.
- **Profile version string**, recorded by nono in the receipt.

### 4.2 Receipt self-check

Liam reads the file at `LIAM_RECEIPT_PATH` once, before any other work. The fields Liam requires are a **contract to be confirmed against `crates/nono/src/receipt.rs`** before implementation; if the real receipt differs, the implementer updates this table and stops to say so.

| Field | Type | Liam's check |
|---|---|---|
| `receipt_version` | string | Known version |
| `session_id` | string | Non-empty; copied into the run record |
| `profile.name` / `profile.version` | string | Equals the profile Liam was built against (name exact, version ≥ minimum in config) |
| `arm` | enum | Equals `appcontainer` |
| `layers[]` | array of `{name, required, status}` | Every entry with `required=true` has `status=active`. Any `degraded`, `absent` or unknown status fails the check. |
| `egress.mode` | enum | Equals `proxy_only` |
| `egress.proxy_port` | int | Matches the port in `HTTPS_PROXY` |
| `started_at` | timestamp | Within the last 120 s (guards against a stale receipt from a previous session) |
| `integrity` | object or absent | See §4.3 |

Failure exits with code 10 and reason `receipt:<field>`. Liam writes no run record in that case, because there is no session to attribute it to. It prints one line to stderr.

### 4.3 Receipt integrity (open question, with a POC position)

DECISION §7 Q2 is unresolved. POC position: Liam checks that the receipt file is **not writable by the current token** (a receipt Liam could have written is not evidence). If `integrity.signature` is present, Liam verifies it against a public key in its config; if absent, Liam records `receipt_integrity: "acl-only"` in the run record. The choice of signing scheme belongs to nono and zt-infra, not Liam.

### 4.4 What Liam does not do with nono

No IPC, no named pipes, no calls to the WFP service, no profile edits, no reading of any nono state other than the receipt.

---

## 5. Model Calls (popeye)

### 5.1 Transport

OpenAI-compatible chat completions against the gateway FQDN, via the proxy from the environment. Liam sets no `Authorization` header; nono-proxy injects the per-user virtual key (D4). If a response indicates that no key was presented, Liam fails the run with reason `gateway:no_injected_key` and does not retry with any key of its own.

### 5.2 Request metadata

| Field | Value | Why |
|---|---|---|
| `metadata.run_id` | UUIDv7 minted at start | popeye records it on the ledger row (REQ-RUN-01) |
| `metadata.agent_id` | `liam` | Desktop-agent segment in the cost model |
| `metadata.purpose` | From `--purpose`, validated against `vocabulary.yaml` | Purpose/class prohibition is enforced by popeye, not Liam |
| `metadata.nono_session_id` | From the receipt | Lets the ledger row be joined to the receipt without Liam's own log |
| `model` | From config, must be in the model approval registry for the purpose | Liam does not choose models dynamically |

### 5.3 Secrets

Liam's process environment, config and logs contain no gateway key, no provider key and no token. The implementer adds a CI check that greps the binary's embedded strings and the config schema for key-shaped values.

### 5.4 Timeouts and retries

One attempt, 60 s timeout. No retry on 4xx. One retry on 5xx or timeout, then fail. Rationale: a retry loop under a runaway condition is the case popeye's run-cost alert exists to catch; Liam should not mask it.

---

## 6. Context (hometown)

- **Endpoint:** hometown's read API for party context, called with `party_ref`, `purpose` and the principal (D9).
- **Liam uses what it is given.** It does not request fields outside the purpose, does not merge context from other sources, and does not cache context across runs.
- **Provenance:** Liam records the `definition_provenance` and `context_complete` values hometown returns, and passes `context_complete` through to the run record. If `context_complete=false`, the answer printed to the console is prefixed with a fixed incompleteness notice.
- **Masking:** if hometown returns masked fields, they are passed to the model masked. Liam never attempts to resolve a party identifier itself (bluto owns that).

---

## 7. Actions (olive)

- **Proposal shape:** `{action_type, party_ref, parameters, justification, run_id, trace_ref}` where `action_type` must be in olive's action catalog version pinned in config.
- **Liam submits, then stops.** It records olive's response (`proposal_id`, `status` ∈ `{queued, rejected}`) and does not poll, approve, or execute. Execution is olive's simulated executor in the POC.
- **No catalog match → no submission.** The model's suggested action is recorded as `proposal.suppressed` with the reason `not_in_catalog`, and printed to the console as text only.
- **Parameters are validated against the catalog schema before submission.** Free text from the model never goes into a parameter field without a schema match.

---

## 8. Evidence

### 8.1 Run record (`runs/<run_id>.json`)

One JSON document per run, written once at completion (or at failure after the self-check passed). Fields:

```
run_id, agent_id, agent_version, started_at, ended_at, outcome,
nono: { session_id, profile_name, profile_version, receipt_integrity },
purpose, party_ref_hash,              # SHA-256 of party_ref, not the ref
context: { context_complete, provenance_refs[] },
model: { model, gateway_request_id, prompt_tokens, completion_tokens, latency_ms },
proposal: { proposal_id, status } | null,
log_sha256                             # hash of the run log file
```

`party_ref_hash` rather than `party_ref` because the run record may travel further than the run log.

### 8.2 Run log (`runs/<run_id>.jsonl`)

Append-only, one event per line, content-free (D8). Event names are those in §3.3. Allowed payloads: identifiers, counts, versions, durations, status codes, reason codes. Prohibited: prompt text, response text, context field values, party identifiers, URLs with query strings.

### 8.3 ATE linkage

Liam does **not** write the ATE; hometown does. Liam's contribution is `run_id` and `nono_session_id` on the gateway call and on the context request, so the ATE hometown assembles can reference both the ledger row and the receipt. This is seam S9 from the endpoint side (DECISION §5.4). Whether `nono_session_id` lands in the ATE as a top-level field or inside `retrieval` is a hometown decision; Liam only supplies it.

---

## 9. Security Requirements

- No direct sockets. All HTTP goes through the proxy from the environment. If the proxy variables are absent, Liam refuses to start (reason `env:no_proxy`). Liam does not resolve hostnames itself; it sends the hostname to the proxy via CONNECT, so DNS is not a channel Liam opens.
- No credentials in environment, config, binary or logs (§5.3).
- No write credential of any kind (D6).
- No file access outside the three grants in §4.1. Any `ERROR_ACCESS_DENIED` outside those paths is treated as a bug, not something to work around with a wider grant.
- No child processes. Liam spawns nothing.
- No execution of model output. Model output is parsed as data against a fixed JSON schema; it is never evaluated, templated into a command, or passed to a shell.
- Content never leaves through logs, stderr or the run record.
- Fail closed on every check above: exit non-zero, print one reason line.

---

## 10. Configuration

### 10.1 `liam.toml`

```toml
[agent]
version = "0.1.0"
agent_id = "liam"

[nono]
profile_name = "liam"
profile_min_version = "0.1.0"
receipt_max_age_seconds = 120
receipt_public_key = ""          # optional, PEM; empty = acl-only

[gateway]
base_url = "https://gateway-azure.ai.twg.internal"
model = "<from model approval registry>"
timeout_seconds = 60

[hometown]
base_url = "https://<hometown-api-fqdn>"

[olive]
base_url = "https://<olive-api-fqdn>"
catalog_version = "0.4"

[vocabulary]
path = "contracts/vendored/vocabulary.yaml"
sha256 = "60a44f00..."           # must match the vendored file; checked at startup

[evidence]
runs_dir = "%LOCALAPPDATA%\\Fiskroad\\liam\\runs"
```

### 10.2 Environment (set by nono, read by Liam)

`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`, `LIAM_RECEIPT_PATH`. Liam reads nothing else from the environment.

### 10.3 Stub mode

`LIAM_STUBS=1` plus a stub base URL set points all three clients at in-repo stub servers (§11). Stub mode **also requires** a stub receipt file and still runs the self-check against it, so the self-check code path is exercised in CI. Stub mode is a compile-time feature (`--features stubs`) that is absent from release builds.

---

## 11. Repository Layout

```
liam/
  README.md                       status: POC scaffold
  AGENTS.md                       constraints for implementing agents (no secrets, no direct egress,
                                  no child processes, stop on contract mismatch)
  specs/                          this document and the test spec
  profiles/liam.nono.toml         nono profile (reviewed by the nono owner)
  contracts/vendored/             vocabulary.yaml + VENDORED.md with SHA-256
  contracts/                      receipt-fields.md (the §4.2 table, confirmed against receipt.rs)
                                  olive-proposal.schema.json, model-output.schema.json
  src/
    main.rs                       startup sequence, exit codes
    receipt.rs                    receipt load + self-check
    clients/{hometown,gateway,olive}.rs
    evidence.rs                   run record + run log
    task/summarise_propose.rs
  stubs/                          hometown-stub, gateway-stub, olive-stub (small HTTP servers),
                                  receipt fixtures (valid, degraded, stale, writable, wrong-arm)
  scripts/
    launch-under-nono.ps1         POC launcher; the only supported launch path
    make-stub-receipt.ps1
  tests/                          see LIAM_TEST_SPEC_v0.1.md
  .github/workflows/ci.yml        windows-latest; tagged ci-elevated
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Run completed |
| 10 | Receipt self-check failed |
| 11 | Environment check failed (no proxy, bad vocabulary hash) |
| 20 | Context fetch failed |
| 21 | Gateway call failed |
| 22 | Proposal submission failed |
| 30 | Evidence write failed (after the task ran; the console still shows the answer) |

---

## 12. Requirements

| REQ | Requirement | Verified by |
|---|---|---|
| REQ-LIAM-01 | Liam refuses to start unless a receipt is present, fresh, non-writable, for the `appcontainer` arm, with every required layer `active` and `egress.mode=proxy_only` | TC-START-01..07 |
| REQ-LIAM-02 | Liam refuses to start if proxy environment variables are absent | TC-START-08 |
| REQ-LIAM-03 | Liam makes no network connection except through the proxy | TC-NET-01..03 |
| REQ-LIAM-04 | Liam sends no `Authorization` header and holds no key; a gateway response indicating no key fails the run without retry | TC-KEY-01..03 |
| REQ-LIAM-05 | Every gateway request carries `run_id`, `agent_id`, `purpose` and `nono_session_id` in metadata | TC-META-01 |
| REQ-LIAM-06 | `purpose` is validated against the vendored vocabulary before any call; the vendored file's hash is checked at startup | TC-VOC-01, TC-VOC-02 |
| REQ-LIAM-07 | Context comes only from hometown; `context_complete=false` is surfaced to the user and recorded | TC-CTX-01..03 |
| REQ-LIAM-08 | Actions are submitted only as olive proposals, only when `action_type` is in the pinned catalog, with schema-valid parameters; Liam never executes | TC-ACT-01..05 |
| REQ-LIAM-09 | At most one model call and one proposal per run | TC-ACT-06, TC-META-02 |
| REQ-LIAM-10 | The run record and run log are written per §8, are content-free, and reference the nono session | TC-EVD-01..05 |
| REQ-LIAM-11 | Liam spawns no child processes and accesses no file outside the three profile grants | TC-SEC-01, TC-SEC-02 |
| REQ-LIAM-12 | Model output is parsed as data against a fixed schema and never executed | TC-SEC-03 |
| REQ-LIAM-13 | All failures exit with the documented non-zero code and one reason line; no partial run proceeds | TC-FAIL-01..06 |
| REQ-LIAM-14 | A blocked egress attempt by Liam (deliberate, in test) produces an `endpoint-detection/0.1` event from nono-proxy, closing TC-END-02 from the agent side | TC-E2E-02 |
| REQ-LIAM-15 | Release builds contain no stub code and no key-shaped strings | TC-BUILD-01, TC-BUILD-02 |

---

## 13. Assumptions and Open Items

Assumptions the implementer should treat as true until told otherwise, and record in `README.md`:

- A1. hometown exposes a read endpoint that accepts `party_ref`, `purpose` and principal and returns context with `context_complete` and provenance. If the real shape differs, adapt the client and say so.
- A2. olive exposes a proposal endpoint matching §7 and a published catalog for version `0.4`. In the POC, olive's executor is simulated.
- A3. popeye's gateway is OpenAI-compatible and records `metadata.run_id` (REQ-RUN-01) and `metadata.agent_id`.
- A4. nono-proxy injects the per-user virtual key on the gateway FQDN only, and emits `endpoint-detection/0.1` on blocked attempts, per `NONO_FISKROAD_INTEGRATION_v0.3`.

Open items, owned outside Liam:

- O1. Receipt field names and the integrity mechanism (nono owner; DECISION Q2). §4.2 is Liam's requirement, not nono's current output.
- O2. Where `nono_session_id` lands in the ATE (hometown owner).
- O3. Whether the launcher script or Intune owns profile distribution for the POC (endpoint team).
- O4. Retention location for `runs/` once a pilot creates real records (DECISION Q1).
