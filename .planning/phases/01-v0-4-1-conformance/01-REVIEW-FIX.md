---
phase: 01-v0-4-1-conformance
fixed_at: 2026-10-21T00:00:00Z
review_path: .planning/phases/01-v0-4-1-conformance/01-REVIEW.md
iteration: 2
findings_in_scope: 12
fixed: 10
skipped: 0
deferred_by_user: 2
status: partial
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-10-21
**Source review:** .planning/phases/01-v0-4-1-conformance/01-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 12 (CR-01, CR-02, WR-01..WR-10)
- Fixed: 10 (iteration 1: CR-01, WR-01, WR-02, WR-03, WR-07, WR-09 and part of CR-02; iteration 2: the rest of CR-02, WR-04, WR-08, WR-10)
- Deferred by user: 2 (WR-05, WR-06), held for §4 re-ratification with `NONO_v0.4.3`; documented, no code change
- Skipped: 0
- Status `partial` only because of the two user-deferred items. Every finding the user scoped for fixing is fixed.
- Checks after the last code commit (d573907), on master: `cargo fmt --check` clean, `cargo clippy --locked --offline --all-targets --all-features -- -D warnings` clean (also clean with default features), `cargo test --locked --offline --features stubs` 166 passed, 0 failed, 2 ignored. After iteration 1 it was 153 passed, 2 ignored. The final commit 5f3b0c3 changes docs only.

## Final status per finding

| Finding | Status | Commit(s) |
|---|---|---|
| CR-01 | fixed | fd0a740 |
| CR-02 | fixed (status in iteration 1, ids in iteration 2) | dea3a78, 5d77728 |
| WR-01 | fixed | c5b9a8b |
| WR-02 | fixed | 880794b |
| WR-03 | fixed: requires human verification | c6aef77 |
| WR-04 | fixed (user decision: exit 11) | 090b03f |
| WR-05 | deferred by user (§4 re-ratification, NONO_v0.4.3 O3) | 5f3b0c3 (docs only) |
| WR-06 | deferred by user (§4 re-ratification, §4.4/D10) | 5f3b0c3 (docs only) |
| WR-07 | fixed | 6027a58 |
| WR-08 | fixed (user decision: exit 11) | 090b03f |
| WR-09 | fixed: requires human verification | ad0e945 |
| WR-10 | fixed (user decision: U+FFFD) | d573907 |

## Fixed Issues

### CR-01: The proxy client follows HTTP redirects

**Files modified:** `src/clients/mod.rs`, `src/stubs.rs`, `tests/answer_contract.rs`, `tests/static_contract.rs`
**Commit:** fd0a740 (iteration 1)
**Applied fix:** The one `Client::builder` sets `redirect(Policy::none())`. A 3xx now reaches `classify`: hometown exits `21 hometown:<status>` and olive exits `22 olive:unavailable`. `TestProxy` passes a 3xx and its `Location` back to Liam without following it, so the new test can see the bug. TC-NET-03 also checks for the redirect policy.

### CR-02: Unchecked upstream strings (`status`, `proposal_id`, `trace_id`)

**Files modified:** iteration 1: `src/clients/olive.rs`, `contracts/run-record.schema.json`, `tests/action_contract.rs`, `tests/record_contract.rs`, `README.md`. Iteration 2: `src/clients/mod.rs`, `src/clients/hometown.rs`, `src/clients/olive.rs`, `contracts/run-record.schema.json`, `tests/action_contract.rs`, `tests/answer_contract.rs`, `tests/record_contract.rs`, `README.md`
**Commits:** dea3a78 (status), 5d77728 (ids)
**Applied fix:**
- Iteration 1: olive's `status` must be a vocabulary `action_outcome`, and a 202 must be `awaiting_approval`.
- Iteration 2 (user decision): a new `clients::id_ok` enforces `^[A-Za-z0-9._:-]{1,128}$` on hometown's `trace_id` (new `hometown::parse_answer`) and olive's `proposal_id` (in `olive::classify`). A failure goes through the existing `InvalidResponse` mappings: hometown exits `21 hometown:unavailable` with no retry (RetryOnce retries only `Unavailable`), olive exits `22 olive:unavailable`.
- `run-record.schema.json` pins `answer.trace_id` and `proposal.proposal_id` to the same pattern, and the record test's schema checker now supports that pattern.
- Tests: unit tests for the guard, hometown and olive. Process tests for empty, space, slash and ESC ids, checking the exit code, the single reason line, no retry and no leak into the record or log. Schema rejection cases.
- README note: "Liam-side guard pending confirmation in S-HT-1 / S-OL-1".

### WR-01: The catalog validator fails open

**Files modified:** `src/catalog.rs`, `src/task.rs`, `README.md`
**Commit:** c5b9a8b (iteration 1)
**Applied fix:** `deny_unknown_fields` on catalog types. A closed object schema with no `properties` accepts only `{}`. A catalog with a different `action_id_format` is refused. In iteration 2 these refusals moved to the startup prerequisite (see WR-04).

### WR-02: The run log is missing §3.3 events

**Files modified:** `src/task.rs`, `src/main.rs`, `tests/record_contract.rs`
**Commit:** 880794b (iteration 1)
**Applied fix:** Liam logs `answer.received`, `proposal.validated` and an enriched `proposal.submitted`. Event order is tested for five cases.

### WR-03: A suppressed proposal is not printed as text

**Files modified:** `src/task.rs`, `src/main.rs`, `tests/action_contract.rs`
**Commit:** c6aef77 (iteration 1)
**Status:** fixed: requires human verification (output format `Suggestion (not submitted): <action_type>: <justification>`)

### WR-04 + WR-08: Catalog and runs_dir are startup prerequisites

**Files modified:** `src/catalog.rs`, `src/config.rs`, `src/task.rs`, `src/main.rs`, `tests/startup_contract.rs`, `README.md`
**Commit:** 090b03f
**Applied fix (user decision):**
- New `catalog::load(text, expected_version)` parses the catalog, checks its version and checks `action_id_format`.
- New `config::resolve_runs_dir(path, local_app_data)` replaces `expand_local_app_data`. It fails if `%LOCALAPPDATA%` is present while the variable is unset or empty, if any `%` remains after expansion, or if the result is not absolute.
- Both run in `main` right after the vocabulary check, before the proxy, record and nono checks and before any network request. Each prints exactly `liam: 11 config:catalog` or `liam: 11 config:runs_dir` and writes no run record.
- `task::run` now takes a validated `&Catalog`. `TaskError::Catalog` and the `22 olive:catalog` path are removed, and the matching task unit test moved to `catalog::tests::load_refuses_unsupported_catalogs`.
- Process tests, each asserting exit 11, the exact reason line, zero hometown/olive hits and no run record:
  - catalog: missing file, corrupt file, version mismatch
  - runs_dir: `LOCALAPPDATA` unset or empty with a `%LOCALAPPDATA%` runs_dir (also checks that no literal `%LOCALAPPDATA%` folder is created), and a relative runs_dir
- README: updated the D-03 note, added an evidence-location note, and added the "Exit codes and reasons" table, which did not exist before.
- RESULTS.md: no row mentioned `olive:catalog`, so no row needed changing. The new tests have no measured RESULTS rows yet.

### WR-07: Contract-required fields are accepted when missing

**Files modified:** `src/clients/hometown.rs`, `src/clients/olive.rs`, `tests/answer_contract.rs`, `README.md`
**Commit:** 6027a58 (iteration 1)

### WR-09: `println!` panics on a stdout write error

**Files modified:** `src/main.rs`, `tests/common/mod.rs`, `tests/answer_contract.rs`
**Commit:** ad0e945 (iteration 1)
**Status:** fixed: requires human verification (a failed stdout write is silently dropped)

### WR-10: Model output is printed unsanitized

**Files modified:** `src/console.rs` (new), `src/lib.rs`, `src/main.rs`, `tests/answer_contract.rs`, `README.md`
**Commit:** d573907
**Applied fix (user decision):**
- New `console::printable` replaces every control character other than `\n` and `\t` with U+FFFD: C0 including ESC and `\r`, DEL, and C1 U+0080..U+009F.
- `main::say`, the only stdout writer, applies it to every line. That covers the answer, the notice, the `Proposal <id>: <status>` line and the suggestion line.
- Evidence and the stderr reason line (guarded codes only) are unchanged.
- Tests:
  - unit: CSI, OSC-8, the full C0/DEL/C1 set, and normal text left unchanged
  - process: an answer with `\x1b[2J`, `\r`, `\t` and `\n` prints with U+FFFD and no ESC or CR bytes
  - process: a normal answer prints unchanged
- README note.

## Deferred by user (§4 re-ratification with NONO_v0.4.3)

### WR-05: `ensure_not_writable` passes read-only-attribute files that the ACL lets Liam write

**File:** `src/receipt.rs:103-119`
**Decision:** deferred to §4 re-ratification (ties to NONO_v0.4.3 O3). No code change.
**Documented in:** README "Known gaps (deferred to §4 re-ratification)". RESULTS.md notes on TC-START-01 and TC-START-06. Commit 5f3b0c3.

### WR-06: An existing DSSE envelope is silently ignored

**File:** `src/main.rs`, `src/evidence.rs`
**Decision:** deferred to §4 re-ratification (spec §4.4/D10, NONO_v0.4.3 O2). No code change.
**Documented in:** README "Known gaps (deferred to §4 re-ratification)". RESULTS.md notes on TC-START-01 and TC-START-07. No pass/fail status changed. Commit 5f3b0c3.

## Notes

- Iteration 2 ran directly on the main working tree, on master, as instructed. No worktree or sentinel was used.
- No wire-contract or §4 change was needed beyond the decided items. The id pattern is a Liam-side guard, marked pending S-HT-1 / S-OL-1 confirmation.
- The vendor line-ending note from iteration 1 still applies. `vendor/` was not touched.

---

_Fixed: 2026-10-21_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
