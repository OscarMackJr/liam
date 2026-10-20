# Liam — Implementation Handoff Prompt

Paste this into the implementing agent's first message, with the two specs and the referenced program documents attached or available in the repository.

---

You are creating a NEW repository, `liam`, a Windows desktop agent in the Fiskroad program. Nothing exists yet.

## Read first, in this order
1. `DECISION_desktop-agent-nono-integration.md` (ratified 2026-10-19) — the boundary you are implementing. Liam runs **under** nono and integrates **only** through contracts.
2. `LIAM_DESIGN_SPEC_v0.4.md` and ratified `LIAM_AMENDMENT_v0.4.1_proxy-transport.md` — what to build. Section 2 holds the design decisions; section 12 the requirements; section 13 the assumptions and open items.
3. `LIAM_TEST_SPEC_v0.4.md` — how it is verified. Every REQ maps to TCs there.
4. `PROGRAM_SHARED_VOCABULARY_v0.4.md` and `vocabulary.yaml` — vendor the yaml unchanged into `contracts/vendored/` and record its SHA-256 in `contracts/vendored/VENDORED.md`.
5. The components Liam depends on: `NONO_FISKROAD_INTEGRATION_v0.3.md`, `NONO_v0.4.md`, `NONO_v0.4.1.md` (agent-key injection; supersedes v0.3 §2.1 for agent profiles), `POPEYE_CORRECTIONS_v0.3.md` §3 (endpoint detection contract), `POPEYE_v0.4.md` §2 and §4, `POPEYE_v0.4.1.md` (agent key exchange), `NONO_v0.4.2.md` and `POPEYE_v0.4.2.md` (injection scope, introspection, hometown relay; **ratified 2026-10-19**), `DECISION_agent-key-service-scope.md`, `NONO_v0.4.3.md` and `POPEYE_v0.4.3.md` (agent-session record, `KeyId`; **ratified 2026-10-19**), `HOMETOWN_v0.4.md`, `OLIVE_CHARTER_v0.4.md`, and the fork's `receipt.rs`, `attestation.rs` and `receipt_content_free_scan.rs` at `29557a93`.
6. `SPEC_ALIGNMENT_LIAM_v0.1.md` — the seam-by-seam check that these documents agree. If a seam it marks aligned turns out not to be, stop and report it.

## Hard rules (stop and say so rather than work around any of them)
- **No secrets anywhere.** No gateway key, provider key or token in code, config, environment handling, logs or tests. nono-proxy mints and injects a session agent key; Liam never reads one, and never sends an `Authorization` header to hometown or olive. Liam never calls the gateway; hometown makes the model call (design v0.3 D5, D12).
- **One socket destination, no DNS.** Connect only to nono-proxy at the validated `http://127.0.0.1:<port>`; build one HTTP client with `no_proxy()`, an explicit `Proxy::all` and a deny-all resolver; no socket or resolver calls in Liam's own source. Do not add an allowlist of your own.
- **No execution.** Liam proposes actions to olive. It never calls a target system and never runs model output.
- **No child processes.** Liam spawns nothing.
- **No nono coupling.** Liam reads the receipt and the agent-session record and nothing else of nono's: no IPC, no service calls, no profile edits, never the session-end record.
- **Content-free evidence.** Prompt text, responses, context values and party identifiers never appear in the run record, run log, stderr or any test artifact other than the sentinel checks that prove this.
- **Fail closed.** Every check failure exits with the documented code and one stderr line, before any network request.
- **Contract mismatch is a stop condition.** If the real receipt or agent-session record (`crates/nono/src/receipt.rs`; `NONO_v0.4.3` §5), hometown read API, olive proposal API or gateway metadata handling differ from the spec's assumptions (§4.2, §13), update the relevant table in the spec, record it in `README.md`, and stop for review. Do not silently adapt.

## Stack and layout
Rust, single static binary, per spec D1 and §11. Stubs behind a `stubs` cargo feature that release builds exclude. CI on `windows-latest`, tagged `ci-elevated`.

## Sequence
1. **Scaffold** the repository per §11: `README.md` (status: POC scaffold, assumptions A1–A4 listed), `AGENTS.md` (these hard rules), `specs/`, `contracts/`, `profiles/liam.nono.toml`, `.gitignore` covering `*.toml` overrides, `runs/`, `.env`.
2. **Write `contracts/receipt-fields.md`** from spec §4.2, then compare it to `receipt.rs` in the nono fork. Stop here and report differences before continuing.
3. **Implement startup**: config load, vocabulary hash check, proxy env check, receipt and agent-record self-check (design v0.4 §4.2–§4.4, including the §4.3 expectancy table), exit codes. Write the TC-START (01–16), TC-LVOC and TC-BUILD tests and make them pass.
4. **Implement the stubs** and the capture mechanism (§2.1 of the test spec), including the receipt fixture generator.
5. **Implement the two clients and the task** (§3.3, §5, §7) behind traits, against stub contracts S-HT-1 and S-OL-1 unless confirmed contracts were supplied at the checkpoint. Write TC-NET-01/03/04, TC-KEY, TC-META, TC-ANS, TC-ACT, TC-FAIL, TC-SEC-03.
6. **Implement evidence** (§8). Write TC-EVD, including the sentinel test.
7. **Write `scripts/launch-under-nono.ps1`** and document the `dev-standard-user` and `manual-hardware` cases as a runbook. Do not mark them verified; they are run by a person.
8. **Produce the results table** in the format of test spec §4, with every unmeasured case marked `not measured`.

## Checkpoints where you stop for review
- After step 2 (receipt contract comparison).
- Before step 5: report which of A1–A4 you could confirm. The agent-key scope and hometown relay are ratified, so the only checkpoint items left are A1/A2: a confirmed hometown/olive contract, or approval to build to S-HT-1/S-OL-1 behind traits. Do not build the hometown client to any other shape.
- Before adding any filesystem grant to `liam.nono.toml` beyond the three in spec §4.1.

## Summary you deliver at the end
What was built, which TCs pass in `ci-elevated`, which cases are runbook-only, every assumption you had to make, and every place the specs needed changing. Measured results only; no claims about environments you did not run in.

## Restart note (2026-10-19, after the receipt checkpoint)

Steps 1–2 are done: the scaffold exists and `contracts/receipt-fields.md` records the mismatch that led to `NONO_v0.4.3` and Liam v0.4. They were ratified on 2026-10-19 together with Liam v0.4. **Restart at step 3 now** against Liam v0.4 §4.2–§4.4:
- parse the fork's `EnforcementReceipt` (schema 2) and the `AgentSessionRecord` exactly as shown in §4.2, using your own Rust types mirroring the fork's (do not depend on the nono crate);
- implement the §4.3 expectancy table in `src/expectancy.rs` as a reviewed constant;
- leave `expected_token_arm` empty in shipped config (O7); stub fixtures supply a placeholder value;
- update `contracts/receipt-fields.md` with a short "resolved by NONO_v0.4.3 / Liam v0.4" section rather than rewriting the checkpoint findings.

## Transport note (2026-10-20, before Gate 3)

`LIAM_AMENDMENT_v0.4.1_proxy-transport.md` records how Liam reconciles the "no sockets / no DNS" rule with the HTTP library's Winsock and `getaddrinfo` imports (D14: one loopback socket destination, deny-all resolver, system proxy ignored). Once ratified, it replaces the "No direct network" hard rule above with the wording in its §4.5, and adds TC-NET-05 to -08. Until ratified, do not start Gate 3 work; Gate 2 work may continue.
