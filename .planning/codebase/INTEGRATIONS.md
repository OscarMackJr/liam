# External Integrations

**Analysis Date:** 2026-10-21

## APIs & External Services

**Hometown (answer service):**
- Used to obtain an answer (and optional proposal) for a party/question/purpose
  - Client: trait `HometownClient` in `src/clients/hometown.rs` (reqwest blocking)
  - Base URL: `[hometown].base_url` in `liam.toml` (placeholder `https://hometown.invalid`), timeout 90s
  - Auth: none held by Liam; nono-proxy injects the agent key at the proxy boundary (`HometownError::NoInjectedKey` / `KeyInactive` on failure)
  - Wire contract: stub S-HT-1, `contracts/hometown-answer.S-HT-1.json` (pending owner confirmation, README A1)

**Olive (proposal service):**
- Receives validated proposals only
  - Client: `src/clients/olive.rs`; base URL `[olive].base_url` in `liam.toml` (placeholder `https://olive.invalid`), timeout 30s
  - Catalog: `contracts/catalog-0.4.json`, version "0.4"
  - Wire contract: stub S-OL-1, `contracts/olive-proposal.S-OL-1.json` (README A2)

**nono (sandbox/supervisor):**
- Liam only reads two files: session receipt (`NONO_RECEIPT_PATH`, schema 2) and agent-session record (`NONO_AGENT_RECORD_PATH`, schema `contracts/agent-record.schema.json`); field mapping in `contracts/receipt-fields.md`
- Validated in `src/receipt.rs`; no IPC or other nono state. Optional DSSE signature (`record_public_key`) fails closed if set.
- Profile draft: `profiles/liam.nono.toml` (agent_id liam, inject_on hometown/olive)

**Model gateway / Popeye:** never called by Liam (constraint in `AGENTS.md`); Popeye controls are assumed (README A3).

## Network Transport

- All HTTP goes through the validated nono-proxy IPv4 loopback endpoint taken from `HTTPS_PROXY`/`HTTP_PROXY` (`validated_proxy` in `src/config.rs`, called from `src/main.rs`), with `NO_PROXY` checked
- Single client from `build_proxy_client` in `src/clients/mod.rs`: `no_proxy()`, `Proxy::all`, `DenyAllResolver` (any DNS lookup fails)
- Spec: `specs/LIAM_AMENDMENT_v0.4.1_proxy-transport.md`

## Data Storage

**Databases:**
- None

**File Storage:**
- Local filesystem only: content-free evidence written under `[evidence].runs_dir` (`%LOCALAPPDATA%\Fiskroad\liam\runs`) by `src/evidence.rs`; repo `runs/` is a placeholder
- Reads vocabulary `contracts/vendored/vocabulary.yaml` (SHA-256 pinned; supplied placeholder hash differs from canonical, see `contracts/vendored/VENDORED.md`)

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- nono / Popeye agent-key (external); Liam never sees credentials. Identity checked via receipt/agent-record agent_id, purpose, token arm (`expected_token_arm`), profile version, record age (`record_max_age_seconds` = 120)

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Single-line reason to stderr with documented exit codes (e.g. 10 receipt/record, 11 config/proxy/vocabulary); content-free evidence only

## CI/CD & Deployment

**Hosting:**
- Local Windows workstation binary; no server deployment

**CI Pipeline:**
- GitHub Actions `.github/workflows/ci.yml` (windows-latest): fmt check, tests (incl. `--ignored`, `--features stubs`), clippy `-D warnings`, release build; all `--locked --offline`

## Environment Configuration

**Required env vars:**
- `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH`, `HTTPS_PROXY` (or `HTTP_PROXY`), `NO_PROXY`, `LOCALAPPDATA`

**Secrets location:**
- None used by Liam; `.env` present at repo root (not read, not documented)

## Webhooks & Callbacks

**Incoming:**
- None (stub-mode local `tiny_http` servers in `src/stubs.rs` exist for tests only)

**Outgoing:**
- None

---

*Integration audit: 2026-10-21*
