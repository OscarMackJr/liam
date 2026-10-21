# Coding Conventions

**Analysis Date:** 2026-10-21

Rust 2021 crate `liam` (library `src/lib.rs` + binary `src/main.rs`). Project constraints live in `AGENTS.md` and take precedence over everything here.

## Naming Patterns

**Files:**
- snake_case single-word modules: `src/config.rs`, `src/evidence.rs`, `src/receipt.rs`, `src/task.rs`, `src/expectancy.rs`; client modules under `src/clients/` (`hometown.rs`, `olive.rs`).
- Integration tests named `<area>_contract.rs`: `tests/process_contract.rs`, `tests/proxy_contract.rs`.

**Functions:** snake_case verbs (`validated_proxy`, `build_proxy_client`, `ensure_not_writable`, `sha256_hex`).

**Types:** PascalCase; clients are `HttpXClient` implementing trait `XClient` (`HometownClient`/`HttpHometownClient`, `OliveClient`/`HttpOliveClient`). Errors are `XError` enums (`HometownError`, `OliveError`, `TaskError`) or a tuple struct (`StartupError(pub String)`).

**Failure reason strings:** `area:detail` lowercase, e.g. `receipt:missing`, `agent_record:written_at`, `vocab:sha256`, `env:proxy_not_loopback`, `hometown:unavailable`, `olive:unavailable`.

## Code Style

**Formatting:** `cargo fmt` default rustfmt (no `rustfmt.toml`); CI enforces `cargo fmt -- --check` (`.github/workflows/ci.yml`).

**Linting:** `cargo clippy --locked --offline --all-targets --all-features -- -D warnings`. Warnings are errors. Where unavoidable use a narrow `#[allow(clippy::too_many_arguments)]` (see `src/task.rs` `run`, `src/main.rs` `finish_failure`).

**Build:** offline vendored deps (`.cargo/config.toml`, `vendor/`); always pass `--locked --offline`. No secrets, keys or tokens in code, config, tests, fixtures (`AGENTS.md`).

## Import Organization

**Order:** crate-external and `crate::` imports grouped with `use` blocks; nested braces for same-root paths (`use liam::{clients::{...}, config::{...}}`, `use std::{env, fs, ...}`). No path aliases. Test modules use `use super::{...}` or `use super::*`.

## Error Handling

**Patterns:**
- Library functions return `Result<_, &'static str>` (`config::load`, `config::validated_proxy`) or `Result<_, StartupError>`; the string is the exact one-line reason.
- Client errors are typed enums with classified variants (`Unavailable`, `Status(u16)`, `PurposeRefused`, `InvalidResponse`) — map HTTP outcomes in `src/clients/*.rs`.
- Only `src/main.rs` terminates the process, via `fn fail(code: i32, reason: &str) -> !` which prints exactly `liam: {code} {reason}` to stderr and exits. Use `.unwrap_or_else(|_| fail(CODE, "area:detail"))` for prerequisites; never `unwrap()`/`expect()` in non-test, non-stub code.
- Exit codes: 10 = receipt/agent-record failure, 11 = config/args/vocab/proxy environment, 21 = Hometown failure/unavailable, 22 = Olive failure, 30 = evidence write failure (`src/main.rs`). Prerequisites must fail before any network request with a single reason line (`AGENTS.md`).
- Post-run failures go through `finish_failure` (writes content-free evidence, then `fail`).

## Logging

**Framework:** none (no `log`/`tracing`). stderr only for the one-line failure reason; stdout carries the answer. Evidence is written by `src/evidence.rs` as `<run_id>.json` plus `<run_id>.jsonl`. Never write prompts, answers, context, party references, or query-bearing URLs to evidence/stderr; only hashes (`party_ref_hash`, `log_sha256`), ids, status.

## Comments

- Module-level `//!` doc comments state purpose and constraints (`src/config.rs`, `src/evidence.rs`, `src/receipt.rs`, `src/stubs.rs`).
- Sparse inline comments; no per-function rustdoc.

## Function Design

**Parameters:** borrowed `&str` request structs with lifetimes (`AnswerRequest<'a>`); dependencies injected as `&dyn HometownClient` / `&dyn OliveClient` (`src/task.rs`).
**Return Values:** `Result` with typed errors; pure helpers where possible (`vocabulary_contains_purpose`, `validated_proxy`).
**Platform/feature gating:** `#[cfg(feature = "stubs")]` for CI-only code (`src/stubs.rs`, `launch_process_id` in `src/main.rs`); `#[cfg(windows)]` for Windows-only code. Stub code must never be in default release builds.

## Module Design

**Exports:** `pub mod` list in `src/lib.rs`; `main.rs` consumes the library. Serde structs use `#[derive(Debug, Deserialize)]` / `Serialize`; borrowed `Serialize` structs for evidence records.
**Barrel Files:** none; `src/clients/mod.rs` declares submodules and hosts `build_proxy_client`/`DenyAllResolver`.
**Network:** a single explicit `reqwest` blocking client with `.no_proxy()`, explicit loopback proxy, and `DenyAllResolver` (`src/clients/mod.rs`); no Liam-side allowlist; no child processes.

---

*Convention analysis: 2026-10-21*
