# Technology Stack

**Analysis Date:** 2026-10-21

## Languages

**Primary:**
- Rust (edition 2021; local toolchain rustc 1.98.1) - all application code in `src/`, tests in `tests/`

**Secondary:**
- PowerShell - launch/helper scripts `scripts/launch-under-nono.ps1`, `scripts/make-stub-records.ps1`
- TOML - config (`liam.toml`) and nono profile draft (`profiles/liam.nono.toml`)
- JSON / JSON Schema / YAML - contracts in `contracts/` (`agent-record.schema.json`, `catalog-0.4.json`, `hometown-answer.S-HT-1.json`, `olive-proposal.S-OL-1.json`, `contracts/vendored/vocabulary.yaml`)

## Runtime

**Environment:**
- Native Windows CLI binary (`liam.exe`), Windows-only target (`x86_64-pc-windows-msvc`), statically linked CRT (`.cargo/config.toml`: `+crt-static`)
- Runs under the external `nono` sandbox; no toolchain pin file (CI uses `dtolnay/rust-toolchain@stable`)

**Package Manager:**
- Cargo
- Lockfile: present (`Cargo.lock`); all crates vendored in `vendor/` (~160 crates), `.cargo/config.toml` replaces crates-io with `vendor` and sets `net.offline = true`. Build with `--locked --offline`.

## Frameworks

**Core:**
- No application framework. Crate `liam` v0.4.0 (`Cargo.toml`) with lib `src/lib.rs` and bin `src/main.rs`.
- `reqwest` 0.12 (blocking, json, `rustls-tls-native-roots`, default features off) - HTTP client in `src/clients/`

**Testing:**
- Built-in `cargo test`; integration tests `tests/process_contract.rs`, `tests/proxy_contract.rs`; `--ignored` tests run in CI

**Build/Dev:**
- `cargo fmt`, `cargo clippy -D warnings`
- Cargo feature `stubs` (optional `tiny_http` 0.12) enables local Hometown/Olive fixtures and capture proxy in `src/stubs.rs`; excluded from release builds

## Key Dependencies

**Critical:**
- `reqwest` 0.12 - proxy-forced client built with `no_proxy()`, explicit `Proxy::all`, and `DenyAllResolver` (`src/clients/mod.rs`)
- `serde` 1 / `serde_json` 1 - config, receipt, agent-record, wire contracts
- `toml` 0.8 - parses `liam.toml` (`src/config.rs`)
- `sha2` 0.10 - vocabulary SHA-256 pinning (`src/config.rs`)
- `uuid` 1 (v7, serde) - run IDs

**Infrastructure:**
- `windows-sys` 0.59 (Windows only; Foundation, Security, Security_Authorization, Storage_FileSystem, System_Registry, System_Threading) - ACL/file-writability and process checks
- `tiny_http` 0.12 (optional, `stubs` feature)

## Configuration

**Environment:**
- `liam.toml` (agent id/version, nono record policy, Hometown/Olive base URLs and timeouts, vocabulary path + sha256, evidence `runs_dir` using `%LOCALAPPDATA%`)
- Env vars read in `src/main.rs`: `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`, `HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY` (supplied by nono); `LOCALAPPDATA` (`src/config.rs`); test-only `LIAM_STUBS`, `LIAM_STUB_PID`
- A `.env` file exists at repo root (contents not read)
- CLI args: `--purpose`, `--party`, `--question`

**Build:**
- `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, `.gitattributes` (preserves vendor checksums)
- Release: `cargo build --locked --offline --release`

## Platform Requirements

**Development:**
- Windows, Rust stable with MSVC target, vendored sources (no network needed), PowerShell

**Production:**
- Windows workstation running a standard-user `nono` session with the reviewed profile; launched via `scripts/launch-under-nono.ps1`

---

*Stack analysis: 2026-10-21*
