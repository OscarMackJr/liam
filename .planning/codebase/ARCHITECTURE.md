<!-- refreshed: 2026-10-21 -->
# Architecture

**Analysis Date:** 2026-10-21

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│  nono supervisor (external; launches Liam in AppContainer)   │
│  writes receipt + agent-session record, sets proxy env vars  │
└───────────────┬─────────────────────────────────────────────┘
                │ NONO_RECEIPT_PATH, NONO_AGENT_RECORD_PATH,
                │ HTTPS_PROXY/HTTP_PROXY (127.0.0.1:port)
                ▼
┌─────────────────────────────────────────────────────────────┐
│  Binary `liam` — `src/main.rs` (CLI, exit codes, orchestration)
├───────────────┬───────────────┬─────────────────────────────┤
│ config.rs     │ receipt.rs    │ task.rs                      │
│ proxy/vocab   │ expectancy.rs │ answer -> catalog check ->   │
│ validation    │ startup check │ proposal                     │
└───────────────┴───────┬───────┴──────────────┬──────────────┘
                        │                      ▼
                        │      ┌──────────────────────────────┐
                        │      │ clients/{hometown,olive}.rs   │
                        │      │ trait + Http impl             │
                        │      │ clients/mod.rs: proxy client  │
                        │      └──────────────┬───────────────┘
                        ▼                     ▼ (only via loopback nono-proxy)
              `evidence.rs` writes      Hometown / Olive (remote)
              content-free run records  `%LOCALAPPDATA%\Fiskroad\liam\runs`
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| CLI orchestrator | Parse args, run prerequisites in order, map errors to exit codes, print answer | `src/main.rs` |
| Config | Parse `liam.toml`, SHA-256 helper, `validated_proxy`, `%LOCALAPPDATA%` expansion | `src/config.rs` |
| Receipt validation | Parse nono receipt (schema 2) and agent-session record (schema 1); `validate_startup`; `ensure_not_writable` | `src/receipt.rs` |
| Expectancy | Static per-layer acceptance table and the 14 layer IDs | `src/expectancy.rs` |
| Proxy client | Single reqwest blocking client with explicit proxy, `no_proxy()`, `DenyAllResolver` | `src/clients/mod.rs` |
| Hometown client | `HometownClient` trait, `HttpHometownClient`, error classification (S-HT-1) | `src/clients/hometown.rs` |
| Olive client | `OliveClient` trait, `HttpOliveClient` (S-OL-1) | `src/clients/olive.rs` |
| Task | Orchestrate answer -> catalog validation -> at most one proposal; returns `TaskResult` | `src/task.rs` |
| Evidence | Content-free `<run_id>.json` + `.jsonl` writer | `src/evidence.rs` |
| Stubs (feature `stubs`) | Local Hometown/Olive stubs, capture proxy, stub record pair for tests | `src/stubs.rs` |

## Pattern Overview

**Overall:** Single-shot, synchronous CLI pipeline (library + thin binary) with fail-closed startup gating and trait-abstracted network clients.

**Key Characteristics:**
- One run = one question; no daemon, no async runtime in Liam code (reqwest `blocking`).
- Pure validation functions take inputs as arguments (`validate_startup(receipt, agent, policy, pid, now, port)`), so they are unit-testable without the filesystem.
- Network clients sit behind traits (`HometownClient`, `OliveClient`) so `task::run` is tested with in-memory fakes.
- Wire contracts are owner-authorized stubs (S-HT-1, S-OL-1); schemas in `contracts/`.
- Liam never holds credentials; nono-proxy injects the agent key on the loopback hop.

## Layers

**Entry/orchestration:**
- Purpose: sequence startup checks, run task, emit output and evidence
- Location: `src/main.rs`
- Depends on: all library modules
- Used by: nono launch (`scripts/launch-under-nono.ps1`)

**Startup validation:**
- Purpose: refuse to proceed unless config, vocabulary, proxy env and nono records are valid
- Location: `src/config.rs`, `src/receipt.rs`, `src/expectancy.rs`
- Depends on: serde, sha2, uuid
- Used by: `src/main.rs`

**Task / domain:**
- Purpose: apply local catalog check to any proposal from Hometown before submission to Olive
- Location: `src/task.rs`, catalog `contracts/catalog-0.4.json`
- Depends on: client traits
- Used by: `src/main.rs`

**Transport/clients:**
- Purpose: HTTP to Hometown and Olive through the loopback proxy only
- Location: `src/clients/`
- Depends on: reqwest

**Evidence:**
- Purpose: content-free audit output
- Location: `src/evidence.rs`

## Data Flow

### Primary Request Path

1. `main` loads `liam.toml`, requires `--purpose --party --question` (`src/main.rs`; exit 11 on failure).
2. Vocabulary file hash checked against `liam.toml` and purpose looked up in the `purposes:` list (`src/main.rs`, `vocabulary_contains_purpose`).
3. `validated_proxy` requires `HTTPS_PROXY == HTTP_PROXY == http://127.0.0.1:<port>` and empty `NO_PROXY` (`src/config.rs`).
4. Receipt and agent record paths read from env, checked not writable, read, and passed to `validate_startup` (exit 10 on failure) (`src/receipt.rs`).
5. `build_proxy_client` constructs the single client; `HttpHometownClient` and `HttpOliveClient` share it (`src/clients/mod.rs`).
6. `task::run` loads the catalog, calls Hometown `answer`; a proposal is submitted to Olive only if `party_ref` matches, `action_type` is in catalog, and parameters exactly match required keys; otherwise it is suppressed (`src/task.rs`).
7. Hometown `Unavailable` is retried once (`src/main.rs`, `or_else`).
8. Answer printed to stdout; `Evidence::finish` writes run record (exit 30 on write failure).

### Failure Flow

1. `TaskError` variant maps to outcome label + exit code (20 purpose refused, 21 Hometown, 22 Olive) in `src/main.rs`.
2. `finish_failure` writes evidence, then `fail(code, reason)` prints `liam: <code> <reason>` to stderr and exits.

**State Management:**
- No persistent state besides evidence files; all per-run state is local variables.

## Key Abstractions

**`StartupPolicy` / `ValidatedSession`:**
- Purpose: expected values in; validated session identity out (session id, key id, token arm)
- Examples: `src/receipt.rs`

**`HometownClient` / `OliveClient` traits:**
- Purpose: seam between task logic and HTTP
- Examples: `src/clients/hometown.rs`, `src/clients/olive.rs`

**`ProxyClient` + `DenyAllResolver`:**
- Purpose: guarantee no DNS resolution and proxy-only egress; `resolver_calls()` exposes counter for tests
- Examples: `src/clients/mod.rs`

**`TaskResult`:** answer, `context_complete`, `trace_id`, optional `(proposal_id, status)`, optional suppression reason. `src/task.rs`.

## Entry Points

**`liam` binary:**
- Location: `src/main.rs`
- Triggers: `nono run --profile profiles/liam.nono.toml --purpose <p> -- liam.exe --purpose --party --question` via `scripts/launch-under-nono.ps1`
- Responsibilities: everything above

**Library crate `liam`:** `src/lib.rs` re-exports modules; contains startup-validation unit tests.

## Architectural Constraints

- **Threading:** Synchronous, single-threaded in Liam code (reqwest blocking spins its own internal runtime). `src/stubs.rs` uses threads for test servers.
- **Global state:** None at module level; `DenyAllResolver` uses an `AtomicUsize` counter owned by the client.
- **Circular imports:** None; `evidence` -> `receipt`, `config`; `task` -> `clients`; `receipt` -> `expectancy`.
- **Windows target:** `windows-sys` dependency under `cfg(windows)`; static CRT via `.cargo/config.toml`; offline vendored crates (`vendor/`).
- **AGENTS.md rules:** no secrets, loopback IPv4 proxy only, no Liam-side allowlist, no child processes, no model gateway calls, no nono IPC beyond reading the two records, content-free evidence, one reason line + non-zero code before any network request, contract mismatch is a stop condition.
- **Feature gating:** `stubs` module and `LIAM_STUBS`/`LIAM_STUB_PID` env overrides in `src/main.rs` exist only with `--features stubs`; release builds exclude them.
- **Signature mode:** non-empty `nono.record_public_key` is refused (`receipt:signature`) until a verifier exists.

## Anti-Patterns

### Reading ambient proxy or system settings

**What happens:** Default reqwest honors env/system proxies.
**Why it's wrong:** Violates the loopback-only transport (D14).
**Do this instead:** Build clients only via `build_proxy_client` (`.no_proxy()` + explicit proxy + deny-all resolver) in `src/clients/mod.rs`.

### Writing content to evidence or stderr

**What happens:** Logging prompts, answers, party refs or URLs.
**Why it's wrong:** Evidence must be content-free.
**Do this instead:** Pass only hashes and join keys (`party_ref_hash`, `trace_id`) as in `src/evidence.rs`.

### Silently adapting to contract drift

**Do this instead:** Update `contracts/` tables and `README.md`, then stop for review.

## Error Handling

**Strategy:** Fail closed with numeric exit codes and a single stderr line `liam: <code> <reason>`.

**Patterns:**
- 10: nono record/receipt/agent-record failures; 11: config, args, vocabulary, proxy env; 20: purpose refused; 21: Hometown; 22: Olive/catalog; 30: evidence write.
- Library validators return `Result<_, &'static str>` or `StartupError(String)` with reason codes like `receipt:layer:AgentKeyInjection`, `agent_record:session_id`.
- Client errors are enums (`HometownError`, `OliveError`); HTTP 401 -> NoInjectedKey/KeyInactive, 403 -> PurposeRefused, 5xx/transport -> Unavailable (retryable once).

## Cross-Cutting Concerns

**Logging:** No logging framework; stderr reason line only; evidence JSONL events (`run.started`, `run.completed`/`run.failed`, `proposal.suppressed`).
**Validation:** Strict, up-front; vocabulary SHA-256 pinned in `liam.toml`.
**Authentication:** None in Liam; key injection by nono-proxy, verified via receipt layer `AgentKeyInjection` and agent record.

---

*Architecture analysis: 2026-10-21*
