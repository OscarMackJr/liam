---
phase: 01-v0-4-1-conformance
fixed_at: 2026-10-21T00:00:00Z
review_path: .planning/phases/01-v0-4-1-conformance/01-REVIEW.md
iteration: 1
findings_in_scope: 12
fixed: 7
skipped: 5
status: partial
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-10-21
**Source review:** .planning/phases/01-v0-4-1-conformance/01-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 12 (CR-01, CR-02, WR-01..WR-10)
- Fixed: 7 (CR-02 is only partly fixed; see its entry)
- Skipped: 5 (all need a user decision)
- Checks after the last commit, on master: `cargo fmt --check` clean, clippy `-D warnings` clean, `cargo test --locked --offline --features stubs` 153 passed, 0 failed, 2 ignored (baseline was 138 passed, 2 ignored)

## Fixed Issues

### CR-01: The proxy client follows HTTP redirects

**Files modified:** `src/clients/mod.rs`, `src/stubs.rs`, `tests/answer_contract.rs`, `tests/static_contract.rs`
**Commit:** fd0a740
**Applied fix:** The one `Client::builder` now sets `redirect(Policy::none())` (§3.3, §7.4, D12). A 3xx now reaches `classify`: hometown exits `21 hometown:<status>` and olive exits `22 olive:unavailable`, using the mappings that already existed. To make this testable, `Reply::redirect` was added, and `TestProxy` now passes a 3xx and its `Location` header back to Liam without following it. Before this, the proxy's own client followed redirects and dropped `Location`, so the bug could not show in tests. The new test failed before the fix (`hometown:403`, because Liam followed the redirect to the gateway host and the proxy refused it) and passes after. TC-NET-03 now also checks for the redirect policy.

### CR-02: Unchecked upstream strings (`status`, `proposal_id`, `trace_id`)

**Files modified:** `src/clients/olive.rs`, `contracts/run-record.schema.json`, `tests/action_contract.rs`, `tests/record_contract.rs`, `README.md`
**Commit:** dea3a78
**Applied fix (partial):** olive's `status` must be one of the vocabulary's `action_outcomes` (§7.2), and a 202 must be `awaiting_approval`. Anything else is `InvalidResponse`, which exits `22 olive:unavailable` (S-OL-1). A unit test keeps the list in step with the pinned `vocabulary.yaml`. The run-record schema now limits `proposal.status` to the same list.
- **Deviation from the reviewer:** a 200 accepts any vocabulary outcome, not only `awaiting_approval`/`refused_policy`. TC-ACT-09 says a replay is "recorded as the returned status".
- **Not applied, needs your decision:** the identifier pattern for `proposal_id` and `trace_id` (and rejecting an empty `trace_id`). The spec does not define an ID format, so this would tighten what Liam accepts on the wire. Question: what format should these IDs have, e.g. `^[A-Za-z0-9._:-]{1,128}$`? And should hometown and olive confirm it first in S-HT-1 / S-OL-1?

### WR-01: The catalog validator fails open

**Files modified:** `src/catalog.rs`, `src/task.rs`, `README.md`
**Commit:** c5b9a8b
**Applied fix:** `Catalog`, `CatalogAction` and `Schema` reject unknown fields. The stub catalog's `note` is allowed explicitly. An unsupported catalog therefore becomes `TaskError::Catalog`, which takes the existing `22 olive:catalog` path. An object schema with `additionalProperties:false` and no `properties` now accepts only `{}`. `task::run` refuses a catalog whose `action_id_format` is not exactly `^[a-z0-9_.]+@[0-9]+$` (§7.3). Unit tests were added.

### WR-02: The run log is missing §3.3 events

**Files modified:** `src/task.rs`, `src/main.rs`, `tests/record_contract.rs`
**Commit:** 880794b
**Applied fix:** Liam now logs `answer.received` `{trace_id, context_complete, has_proposal, latency_ms}` and `proposal.validated`. `proposal.submitted` now carries `{proposal_id, status, reason, http_status}`, and `proposal.refused` is kept. Latency covers the hometown call including its one retry. When olive fails, the log still shows `answer.received` and `proposal.validated`. A test checks the event order for five cases: submitted, refused, no proposal, suppressed and olive failure.

### WR-03: A suppressed proposal is not printed as text

**Files modified:** `src/task.rs`, `src/main.rs`, `tests/action_contract.rs`
**Commit:** c6aef77
**Status:** fixed: requires human verification (the output format is my choice)
**Applied fix:** After the answer, Liam prints `Suggestion (not submitted): <action_type>: <justification>` (§7.3, TC-ACT-02). If the fields are not strings, it prints the justification alone, or failing that the proposal's raw JSON. The text is display only and is never passed to `Evidence`; the tests check that it stays out of the record and the log. Please confirm this format.

### WR-07: Contract-required fields are accepted when missing

**Files modified:** `src/clients/hometown.rs`, `src/clients/olive.rs`, `tests/answer_contract.rs`, `README.md`
**Commit:** 6027a58
**Applied fix:** hometown's `proposal` field now uses `deserialize_with` with no default. JSON `null` is still None, but a missing key is now `InvalidResponse`, which exits `21 hometown:unavailable` with no retry. Removing `#[serde(default)]` alone would not have been enough, because serde already treats a missing `Option` field as None. olive's `refused_policy` without a guarded reason is now `InvalidResponse` (`22 olive:unavailable`), so the log and the record can no longer disagree. The tc_ans_02 fixture now includes `"proposal":null`.

### WR-09: `println!` panics on a stdout write error

**Files modified:** `src/main.rs`, `tests/common/mod.rs`, `tests/answer_contract.rs`
**Commit:** ad0e945
**Status:** fixed: requires human verification (the error policy is my choice)
**Applied fix:** A new `say()` writes to stdout with `writeln!` and drops any error. `fail()` writes to stderr with `writeln!`. The print order is unchanged, because TC-FAIL-05 expects the answer to be printed before an exit-30 evidence failure. On a broken stdout the run still writes its record and exits with its documented code. The new test closes the read end of stdout. It failed with `println!` (panic) and passes now. Please confirm that silently ignoring a stdout failure is the policy you want; the spec gives no exit code for it.

## Skipped Issues

### WR-04: Catalog failures are handled two different ways

**File:** `src/main.rs:182-183`, `src/task.rs:76-79`
**Reason:** skipped: needs user decision. The spec gives no exit code or startup-order position for a catalog failure. Exit 11 does not list it, and exit 22 means "olive submission failed".
**Question:** Should loading and checking the catalog be a prerequisite, run next to the vocabulary check, with no run record? If so, which line: `11 config:catalog`, or keep `22 olive:catalog` and amend the exit-code table?
**Original issue:** A missing catalog exits `22 olive:catalog` with no record. A bad or wrong-version catalog writes a full record with `outcome: olive_failed`, even though olive was never called.

### WR-05: `ensure_not_writable` passes read-only-attribute files that the ACL lets Liam write

**File:** `src/receipt.rs:103-119`
**Reason:** skipped: needs user decision. This is §4.4 integrity behaviour and is tied to `NONO_v0.4.3` O3 (receipt directory and ACL), which is not ratified. The fix also means rewriting the stub fixtures (`src/stubs.rs`, `scripts/make-stub-records.ps1`) to use a restrictive DACL instead of `set_readonly`.
**Question:** Does "writable by its own token" (§4.4) include a token that can clear the read-only attribute or change the DACL (`FILE_WRITE_ATTRIBUTES`, `WRITE_DAC`, `WRITE_OWNER`)? If yes, may the fixtures move to deny-write DACLs?
**Original issue:** Opening with `write(true)` fails on `FILE_ATTRIBUTE_READONLY` even when the DACL grants full control, so a file the user can rewrite passes the check.

### WR-06: An existing DSSE envelope is silently ignored

**File:** `src/main.rs:130-140`, `src/evidence.rs:153`
**Reason:** skipped: needs user decision. This is §4.4, and changes to §4 need re-ratification with `NONO_v0.4.3`. The suggested stopgap (fail on any envelope that is present) would refuse a valid envelope, but §4.4 says a valid one gives `receipt_integrity: "dsse"`.
**Question:** Until a verifier exists, should Liam fail closed with `10 receipt:signature` whenever `<session_id>.agent.dsse.json` is present, as a documented stop condition? Or should the DSSE verifier from D10 be built now?
**Original issue:** Liam never looks for the envelope and always records `acl-only`.

### WR-08: An unset `LOCALAPPDATA` puts evidence in a relative folder

**File:** `src/config.rs:90-94`, `src/main.rs:165-169`
**Reason:** skipped: needs user decision. The spec defines no reason or exit code for this case.
**Question:** Should Liam fail before any network request when the placeholder cannot be expanded or the result is not absolute, with `11 config:runs_dir`? This would follow the existing `config:*` reasons and the "environment check" meaning of exit 11.
**Original issue:** The literal `%LOCALAPPDATA%\...` folder is created relative to the working directory, or the run fails late with exit 30 after the network calls.

### WR-10: Model output is printed unsanitized

**File:** `src/main.rs` (answer, suggestion and proposal output)
**Reason:** skipped: needs user decision. §3.3 step 5 and §9 say "print the answer", and the spec does not say whether Liam may change the displayed text.
**Question:** May Liam replace C0/C1 control characters other than `\n` and `\t` with U+FFFD in everything it prints to the console (answer, suggestion, proposal id/status)? Should this become a spec line under §9?
**Original issue:** ANSI/OSC sequences in the answer can rewrite the notice line or fake a `Proposal ...` line.

## Notes

- **Line endings in `vendor/` (already present before these fixes, not changed):** 14 files are LF in git but CRLF in the main checkout, for example `vendor/schannel/src/key_handle.rs` and `vendor/generic-array/**`. `.cargo-checksum.json` matches the CRLF bytes, so a fresh checkout fails `--all-features` builds with a checksum error. RESULTS.md already mentions this. In the throwaway worktree only, I copied the main checkout's bytes over those files to build; they were never staged or committed.
- RESULTS.md was not updated: none of its rows describe the changed behaviour. The new tests have no measured RESULTS rows yet.

---

_Fixed: 2026-10-21_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
