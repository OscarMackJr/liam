<!-- GSD:project-start source:PROJECT.md -->
## Project

**Liam**

Liam is a Windows desktop agent in the Fiskroad program. It is a single-shot Rust CLI that runs confined under nono and performs one bounded task for a banker: it asks hometown for a governed answer about a party and, if the answer carries a catalog-valid proposal, submits that proposal unchanged to olive. Its POC value is the evidence chain it exercises from the endpoint side (nono receipt, hometown ATE, popeye ledger row, olive proposal, Liam run record), not the task itself.

**Core Value:** One Liam run, started only inside an enforced nono session, leaves content-free records that join end to end. It holds no credential, reaches nothing but the loopback nono-proxy and executes nothing.

### Constraints

- **Security**: No secrets, keys or credential-shaped values anywhere (code, config, env handling, logs, tests, fixtures). Why: nono-proxy injects the key and Liam never sees it (AGENTS.md)
- **Network**: Only the validated IPv4 loopback nono-proxy endpoint; one explicit client, deny-all resolver, no Liam-side allowlist. Why: D14
- **Execution**: Never calls the gateway, never executes target-system actions or model output, never spawns child processes. Why: DECISION §5.3, REQ-LIAM-11/12
- **nono coupling**: Reads only the receipt and agent-session record. No IPC, services, profile edits or other state. Why: contract-only integration decision
- **Evidence**: Content-free. No prompts, answers, context values, party refs or query-bearing URLs in logs, records or stderr. Why: REQ-LIAM-10
- **Fail closed**: A failed prerequisite prints exactly one reason line and exits with its documented code before any network request. Why: REQ-LIAM-13
- **Change control**: A contract mismatch stops work. Update the contract table and README, then stop for review. Changes to §4 need re-ratification with `NONO_v0.4.3`
- **Tech stack**: Rust, single static binary, stubs only behind the `stubs` feature and excluded from release; CI on windows-latest
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust (edition 2021; local toolchain rustc 1.98.1) - all application code in `src/`, tests in `tests/`
- PowerShell - launch/helper scripts `scripts/launch-under-nono.ps1`, `scripts/make-stub-records.ps1`
- TOML - config (`liam.toml`) and nono profile draft (`profiles/liam.nono.toml`)
- JSON / JSON Schema / YAML - contracts in `contracts/` (`agent-record.schema.json`, `catalog-0.4.json`, `hometown-answer.S-HT-1.json`, `olive-proposal.S-OL-1.json`, `contracts/vendored/vocabulary.yaml`)
## Runtime
- Native Windows CLI binary (`liam.exe`), Windows-only target (`x86_64-pc-windows-msvc`), statically linked CRT (`.cargo/config.toml`: `+crt-static`)
- Runs under the external `nono` sandbox; no toolchain pin file (CI uses `dtolnay/rust-toolchain@stable`)
- Cargo
- Lockfile: present (`Cargo.lock`); all crates vendored in `vendor/` (~160 crates), `.cargo/config.toml` replaces crates-io with `vendor` and sets `net.offline = true`. Build with `--locked --offline`.
## Frameworks
- No application framework. Crate `liam` v0.4.0 (`Cargo.toml`) with lib `src/lib.rs` and bin `src/main.rs`.
- `reqwest` 0.12 (blocking, json, `rustls-tls-native-roots`, default features off) - HTTP client in `src/clients/`
- Built-in `cargo test`; integration tests `tests/process_contract.rs`, `tests/proxy_contract.rs`; `--ignored` tests run in CI
- `cargo fmt`, `cargo clippy -D warnings`
- Cargo feature `stubs` (optional `tiny_http` 0.12) enables local Hometown/Olive fixtures and capture proxy in `src/stubs.rs`; excluded from release builds
## Key Dependencies
- `reqwest` 0.12 - proxy-forced client built with `no_proxy()`, explicit `Proxy::all`, and `DenyAllResolver` (`src/clients/mod.rs`)
- `serde` 1 / `serde_json` 1 - config, receipt, agent-record, wire contracts
- `toml` 0.8 - parses `liam.toml` (`src/config.rs`)
- `sha2` 0.10 - vocabulary SHA-256 pinning (`src/config.rs`)
- `uuid` 1 (v7, serde) - run IDs
- `windows-sys` 0.59 (Windows only; Foundation, Security, Security_Authorization, Storage_FileSystem, System_Registry, System_Threading) - ACL/file-writability and process checks
- `tiny_http` 0.12 (optional, `stubs` feature)
## Configuration
- `liam.toml` (agent id/version, nono record policy, Hometown/Olive base URLs and timeouts, vocabulary path + sha256, evidence `runs_dir` using `%LOCALAPPDATA%`)
- Env vars read in `src/main.rs`: `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`, `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY` (supplied by nono); `LOCALAPPDATA` (`src/config.rs`); test-only `LIAM_STUBS`, `LIAM_STUB_PID`
- A `.env` file exists at repo root (contents not read)
- CLI args: `--purpose`, `--party`, `--question`
- `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, `.gitattributes` (preserves vendor checksums)
- Release: `cargo build --locked --offline --release`
## Platform Requirements
- Windows, Rust stable with MSVC target, vendored sources (no network needed), PowerShell
- Windows workstation running a standard-user `nono` session with the reviewed profile; launched via `scripts/launch-under-nono.ps1`
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- snake_case single-word modules: `src/config.rs`, `src/evidence.rs`, `src/receipt.rs`, `src/task.rs`, `src/expectancy.rs`; client modules under `src/clients/` (`hometown.rs`, `olive.rs`).
- Integration tests named `<area>_contract.rs`: `tests/process_contract.rs`, `tests/proxy_contract.rs`.
## Code Style
## Import Organization
## Error Handling
- Library functions return `Result<_, &'static str>` (`config::load`, `config::validated_proxy`) or `Result<_, StartupError>`; the string is the exact one-line reason.
- Client errors are typed enums with classified variants (`Unavailable`, `Status(u16)`, `PurposeRefused`, `InvalidResponse`) — map HTTP outcomes in `src/clients/*.rs`.
- Only `src/main.rs` terminates the process, via `fn fail(code: i32, reason: &str) -> !` which prints exactly `liam: {code} {reason}` to stderr and exits. Use `.unwrap_or_else(|_| fail(CODE, "area:detail"))` for prerequisites; never `unwrap()`/`expect()` in non-test, non-stub code.
- Exit codes: 10 = receipt/agent-record failure, 11 = config/args/vocab/proxy environment, 21 = Hometown failure/unavailable, 22 = Olive failure, 30 = evidence write failure (`src/main.rs`). Prerequisites must fail before any network request with a single reason line (`AGENTS.md`).
- Post-run failures go through `finish_failure` (writes content-free evidence, then `fail`).
## Logging
## Comments
- Module-level `//!` doc comments state purpose and constraints (`src/config.rs`, `src/evidence.rs`, `src/receipt.rs`, `src/stubs.rs`).
- Sparse inline comments; no per-function rustdoc.
## Function Design
## Module Design
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## System Overview
```text
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
- One run = one question; no daemon, no async runtime in Liam code (reqwest `blocking`).
- Pure validation functions take inputs as arguments (`validate_startup(receipt, agent, policy, pid, now, port)`), so they are unit-testable without the filesystem.
- Network clients sit behind traits (`HometownClient`, `OliveClient`) so `task::run` is tested with in-memory fakes.
- Wire contracts are owner-authorized stubs (S-HT-1, S-OL-1); schemas in `contracts/`.
- Liam never holds credentials; nono-proxy injects the agent key on the loopback hop.
## Layers
- Purpose: sequence startup checks, run task, emit output and evidence
- Location: `src/main.rs`
- Depends on: all library modules
- Used by: nono launch (`scripts/launch-under-nono.ps1`)
- Purpose: refuse to proceed unless config, vocabulary, proxy env and nono records are valid
- Location: `src/config.rs`, `src/receipt.rs`, `src/expectancy.rs`
- Depends on: serde, sha2, uuid
- Used by: `src/main.rs`
- Purpose: apply local catalog check to any proposal from Hometown before submission to Olive
- Location: `src/task.rs`, catalog `contracts/catalog-0.4.json`
- Depends on: client traits
- Used by: `src/main.rs`
- Purpose: HTTP to Hometown and Olive through the loopback proxy only
- Location: `src/clients/`
- Depends on: reqwest
- Purpose: content-free audit output
- Location: `src/evidence.rs`
## Data Flow
### Primary Request Path
### Failure Flow
- No persistent state besides evidence files; all per-run state is local variables.
## Key Abstractions
- Purpose: expected values in; validated session identity out (session id, key id, token arm)
- Examples: `src/receipt.rs`
- Purpose: seam between task logic and HTTP
- Examples: `src/clients/hometown.rs`, `src/clients/olive.rs`
- Purpose: guarantee no DNS resolution and proxy-only egress; `resolver_calls()` exposes counter for tests
- Examples: `src/clients/mod.rs`
## Entry Points
- Location: `src/main.rs`
- Triggers: `nono run --profile profiles/liam.nono.toml --purpose <p> -- liam.exe --purpose --party --question` via `scripts/launch-under-nono.ps1`
- Responsibilities: everything above
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
### Writing content to evidence or stderr
### Silently adapting to contract drift
## Error Handling
- 10: nono record/receipt/agent-record failures; 11: config, args, vocabulary, proxy env; 20: purpose refused; 21: Hometown; 22: Olive/catalog; 30: evidence write.
- Library validators return `Result<_, &'static str>` or `StartupError(String)` with reason codes like `receipt:layer:AgentKeyInjection`, `agent_record:session_id`.
- Client errors are enums (`HometownError`, `OliveError`); HTTP 401 -> NoInjectedKey/KeyInactive, 403 -> PurposeRefused, 5xx/transport -> Unavailable (retryable once).
## Cross-Cutting Concerns
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
