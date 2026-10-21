---
phase: 01-v0-4-1-conformance
reviewed: 2026-10-21T21:19:45Z
depth: standard
files_reviewed: 28
files_reviewed_list:
  - src/catalog.rs
  - src/clients/hometown.rs
  - src/clients/mod.rs
  - src/clients/olive.rs
  - src/evidence.rs
  - src/lib.rs
  - src/main.rs
  - src/receipt.rs
  - src/stubs.rs
  - src/task.rs
  - tests/action_contract.rs
  - tests/answer_contract.rs
  - tests/common/jcs.rs
  - tests/common/mod.rs
  - tests/hygiene_contract.rs
  - tests/process_contract.rs
  - tests/proxy_contract.rs
  - tests/record_contract.rs
  - tests/startup_contract.rs
  - tests/static_contract.rs
  - scripts/measure-build.ps1
  - Cargo.toml
  - liam.toml
  - profiles/liam.nono.toml
  - contracts/catalog-0.4.json
  - contracts/hometown-answer.S-HT-1.json
  - contracts/olive-proposal.S-OL-1.json
  - contracts/run-record.schema.json
findings:
  critical: 2
  warning: 10
  info: 9
  total: 21
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-10-21T21:19:45Z
**Depth:** standard
**Files Reviewed:** 28
**Status:** issues_found

## Narrative Findings (AI reviewer)

## Summary

I reviewed the phase diff (`829e635..HEAD`) against `specs/LIAM_DESIGN_SPEC_v0.4.1.md` (§3.3, §4, §5, §7, §8, §10). Several things hold up:

- The four proposal fields are forwarded as `RawValue` bytes. Duplicate keys and extra keys are rejected through `deny_unknown_fields` and serde's duplicate-field errors.
- Prerequisite checks run before the client is built.
- The only `unwrap`/`expect` calls in non-test, non-stub code are in `#[cfg(test)]` modules.
- `stubs` is gated by `#[cfg(feature = "stubs")]` in `lib.rs`.
- Error-body reason codes are guarded by `^[a-z][a-z0-9_]{0,63}$`.

Problems:

- **Redirects:** the one HTTP client follows them (reqwest's default is `Policy::limited(10)`). One answer or proposal call can therefore become several requests and resend the question or party ref.
- **Unchecked upstream strings:** `status`, `proposal_id` and `trace_id` go into the run record and stdout without any check. Only `reason` is guarded.
- **Catalog validator fails open:** the schema subset silently ignores keywords it does not support.
- **Missing log events:** several §3.3 run-log events are never written.
- **Integrity checks:** the record-integrity checks (§4.4) are weaker than the spec describes.

## Critical Issues

### CR-01: The proxy client follows HTTP redirects, so one call can become several requests and resend content

**File:** `src/clients/mod.rs:47-55`
**Issue:** `Client::builder()` never sets `.redirect(...)`, and reqwest's default is `redirect::Policy::limited(10)` (`vendor/reqwest/src/redirect.rs:161-163`). If hometown, olive or anything on the path answers 307/308, reqwest re-POSTs the same body to the `Location` target. For hometown that body holds `question` and `party_ref`. Up to 10 hops are followed, each through nono-proxy, which injects the agent key for allowlisted hosts. This breaks:
- §3.3, "exactly one answer request and at most one proposal per run".
- The olive "no retry" rule (§7.4).
- D12/§9: a `Location: https://<gateway-fqdn>/...` would make Liam itself send a request to the gateway, leaving only the proxy to stop it.

A 301/302/303 is turned into a GET, which reaches a second endpoint. Separately, `classify` has no defined handling for 3xx: in olive.rs it turns any 3xx into `olive:unavailable`, and in hometown.rs it turns it into `Rejected`. In practice no 3xx ever reaches `classify` today, because reqwest follows the redirect first.
**Fix:**
```rust
let client = Client::builder()
    .no_proxy()
    .proxy(Proxy::all(proxy)?)
    .redirect(reqwest::redirect::Policy::none())
    .dns_resolver(Arc::clone(&resolver))
    .build()?;
```
Add a TC that has the stub return `307 Location: http://gateway.invalid/` and asserts one request, exit 21/22, and zero gateway attempts on the TestProxy. Also add the `.redirect(` needle to `tc_net_03_single_proxy_client_constructor`.

### CR-02: Unchecked upstream strings (`status`, `proposal_id`, `trace_id`) are written to the run record and stdout

**File:** `src/clients/olive.rs:19-27,50-58`; `src/clients/hometown.rs:18-24`; `src/main.rs:210-212,231,243-260`; `src/evidence.rs:127-162`
**Issue:** D-06/REQ-LIAM-10 ("content-free evidence") is why `reason` and error codes go through `guard_code`. The same discipline is missing for these fields:
- **olive `status`:** §7.2 says it is a closed vocabulary value (`action_outcomes`). It is copied verbatim into `runs/<run_id>.json` (`proposal.status`), printed, and used as the `refused_policy` branch key.
- **olive `proposal_id`:** copied verbatim into the record and printed.
- **hometown `trace_id`:** copied verbatim into the record (`answer.trace_id`) and sent as `trace_ref`.

A faulty or compromised upstream can therefore put free text into the evidence file, for example the party reference or answer text. That includes control characters and arbitrarily large strings. `run-record.schema.json` constrains none of these fields. An empty `trace_id` is also accepted, which breaks the ATE join silently. A `status` outside the vocabulary (for example `"approved"` or `"executed"`) is recorded as a success with exit 0.
**Fix:** Validate in `classify` and in the hometown response path:
- Reject any `status` outside the expected set with `OliveError::InvalidResponse`. Expected set: `awaiting_approval` for 202; `awaiting_approval` or `refused_policy` for 200.
- Require `proposal_id` and `trace_id` to be non-empty and to match an identifier pattern, for example `^[A-Za-z0-9._:-]{1,128}$`. Otherwise fail with `InvalidResponse`, which maps to `olive:unavailable` / `hometown:unavailable`.
- Tighten `run-record.schema.json` (`status` enum, id patterns) so the record tests enforce these rules.
```rust
const OLIVE_STATUSES: &[&str] = &["awaiting_approval", "refused_policy"];
if !OLIVE_STATUSES.contains(&parsed.status.as_str()) || !id_ok(&parsed.proposal_id) {
    return Err(OliveError::InvalidResponse);
}
```

## Warnings

### WR-01: The catalog validator fails open on schema keywords it does not support, and on `additionalProperties:false` with no `properties`

**File:** `src/catalog.rs:27-39,66-82`
**Issue:**
1. `Schema` does not use `deny_unknown_fields`. If a catalog update adds `enum`, `pattern`, `minimum`, `minLength`, `items`, `const` and so on, those constraints are dropped and the proposal is submitted anyway. That is silent adaptation to contract drift, which the "contract mismatch stops work" rule forbids.
2. For `"type":"object"` without `properties`, the code returns `Ok(())` at line 71-73 before it looks at `additional_properties`. A schema of `{"type":"object","additionalProperties":false}` should reject every key, but it accepts anything.
3. The catalog's own `action_id_format` value is parsed and then ignored. `action_id_format_ok` is hard-coded, so a changed format is never noticed.

**Fix:** Add `#[serde(deny_unknown_fields)]` to `Schema` and `Catalog` (or an explicit allowlist of supported keywords) so an unsupported catalog becomes `TaskError::Catalog`. In the object branch, return `Err("schema")` when `properties` is `None`, `additional_properties == false` and the object has any key. Check that `catalog.action_id_format` equals the hard-coded pattern string, else `TaskError::Catalog`.

### WR-02: The run log is missing events required by §3.3

**File:** `src/main.rs:204-242`; `src/task.rs:80-152`
**Issue:**
- §3.3 requires `answer.received` (trace_id, context_complete, has_proposal, latency) after step 2. It is never emitted.
- §3.3 requires `proposal.validated` when validation passes. It is never emitted.
- `proposal.submitted` should carry "proposal id, outcome, reason if refused". It only logs `http_status`; the outcome and reason go into a separate `proposal.refused` event, and only for `refused_policy`.

The `.jsonl` log is the content-free evidence chain the POC exists to exercise. Right now it cannot show that an answer was received when no proposal followed.
**Fix:** Have `task::run` return the validation result and the hometown latency in `TaskResult` and in `TaskError::Olive`. In `main.rs`, emit:
- `answer.received` with `{trace_id, context_complete, has_proposal, latency_ms}`
- `proposal.validated` before submission
- `proposal.submitted` with `{proposal_id, status, reason, http_status}`

Add assertions on these events to `record_contract.rs`.

### WR-03: A suppressed proposal is not printed as text

**File:** `src/main.rs:205-224`; `src/task.rs:124-130`
**Issue:** §7.3 says that on validation failure "the suggestion is printed as text only". When a proposal is suppressed, `TaskResult` drops it and `main` prints only the answer, so the banker never sees the suggestion.
**Fix:** Carry the decoded `justification`/`action_type` (display only, never evidence) in `TaskResult` when `suppressed.is_some()`, and print a fixed-prefix line such as `Suggestion (not submitted): ...`. Keep it out of `Evidence`.

### WR-04: Catalog failures are handled two different ways and are not treated as a prerequisite

**File:** `src/main.rs:182-183`; `src/task.rs:76-79`; `src/main.rs:320`
**Issue:**
- A missing catalog file exits `22 olive:catalog` with no run record. This happens after `Evidence::new`, so `run.started` was already recorded in memory and is lost.
- A catalog that fails to parse, or whose version does not match, goes through `TaskError::Catalog`. That writes a full run record with `outcome: "olive_failed"`, although no olive call happened and the error is local configuration.

Both happen before any network request, and they should behave the same way. Exit 22 is documented as "olive submission failed", which does not describe either case.
**Fix:** Load, parse and version-check the catalog in the prerequisite block, next to the vocabulary check, before the record/proxy checks. Fail with one reason line (`11 config:catalog`, or keep 22 if the exit-code table is amended) and write no record. `task::run` should take a `&Catalog`.

### WR-05: `ensure_not_writable` passes files that are writable by ACL but have the read-only attribute set

**File:** `src/receipt.rs:103-119`
**Issue:** The check opens the file with `write(true)` and treats `PermissionDenied` as "not writable". On Windows, `CreateFileW(GENERIC_WRITE)` also fails with ACCESS_DENIED when `FILE_ATTRIBUTE_READONLY` is set, even if the DACL gives the caller full control. A file Liam's token can rewrite (by clearing the attribute with `FILE_WRITE_ATTRIBUTES`) therefore passes. The stub fixtures (`src/stubs.rs:209-213`) depend on exactly this attribute, so the tests can never tell "supervisor-owned ACL" apart from "user-writable plus read-only flag". With `receipt_integrity: "acl-only"` this check is the whole D10 integrity guarantee.
**Fix:** Check effective access on the DACL instead of the attribute: `windows-sys` `Security_Authorization` is already a dependency. Call `GetEffectiveRightsFromAclW` / `AccessCheck` for `FILE_WRITE_DATA | FILE_APPEND_DATA | FILE_WRITE_ATTRIBUTES | WRITE_DAC | WRITE_OWNER`. Alternatively, open with `FILE_WRITE_ATTRIBUTES` and treat success as writable. Make the fixtures use a restrictive DACL rather than `set_readonly`.

### WR-06: An existing DSSE envelope is silently ignored, so integrity drops to `acl-only`

**File:** `src/main.rs:130-140`; `src/evidence.rs:153`
**Issue:** §4.4 says that if `<session_id>.agent.dsse.json` exists next to the agent record, Liam verifies it, and a present but invalid envelope fails with `receipt:signature`. The code only refuses a non-empty `record_public_key`. It never looks for the envelope, and it always records `receipt_integrity: "acl-only"`. A supervisor that has started signing therefore gets a run that claims acl-only and never fails.
**Fix:** Until a verifier exists, check whether `<agent_dir>/<session_id>.agent.dsse.json` exists after parsing the session id, and fail closed with `10 receipt:signature` if it does. Document this as a stop condition.

### WR-07: Contract-required fields are accepted when missing (`proposal`; `reason` for `refused_policy`)

**File:** `src/clients/hometown.rs:22-23`; `src/main.rs:231-240`
**Issue:**
- S-HT-1 lists `proposal` under `response.required` ("always present"), but `#[serde(default)]` accepts a response without it as "no proposal".
- S-OL-1 says `200 refused_policy` carries a reason code. A missing or unguarded reason becomes `""` in the `proposal.refused` log event (via `unwrap_or_default`) and `"invalid_reason"` in the run record (`evidence.rs:134-140`). The two evidence artifacts then disagree, and a contract violation is recorded as a normal exit-0 refusal.

Both silently adapt to contract drift.
**Fix:** Drop `#[serde(default)]` on `proposal` and keep the `Option`, so a JSON `null` still yields `None` but a missing key fails as `InvalidResponse`. In olive `classify`, return `InvalidResponse` for `status == "refused_policy"` without a guarded reason, so log and record always carry the same value.

### WR-08: An unset `LOCALAPPDATA` makes evidence land in a relative `%LOCALAPPDATA%` folder under the working directory

**File:** `src/config.rs:90-94`; `src/main.rs:165-169`
**Issue:** `expand_local_app_data` returns the path unchanged when the variable is missing. `fs::create_dir_all("%LOCALAPPDATA%\\Fiskroad\\liam\\runs")` then creates a literal folder relative to the CWD. Either the evidence goes outside the granted evidence directory, or it fails late as `30 evidence:write` after the hometown and olive calls have already happened.
**Fix:** Make expansion fallible and fail before any network request, e.g. `fail(11, "config:runs_dir")`, when the placeholder cannot be expanded or the result is not absolute.

### WR-09: `println!` panics on a stdout write error, which skips evidence and breaks the one-reason-line rule

**File:** `src/main.rs:207-212,280-282`
**Issue:** `println!` panics if stdout is closed or the pipe is broken. That gives exit 101 with a multi-line panic message on stderr. It also happens before `evidence.finish`, so the run that already reached hometown/olive leaves no record. This breaks the "exactly one reason line with its documented code" rule and the evidence guarantee.
**Fix:** Write with `writeln!(io::stdout().lock(), ...)`, ignore or record the error, and always call `evidence.finish` first or regardless. For example, collect the output, finish the evidence, then print and map any write failure to a documented code.

### WR-10: Model output is printed to the console unsanitized, allowing terminal escape injection

**File:** `src/main.rs:209,211,282`
**Issue:** `result.answer`, which is model-generated text from hometown, and the olive `proposal_id`/`status` are printed raw. ANSI/OSC/CSI sequences such as cursor movement, line erase or OSC-8 hyperlinks can rewrite the "Notice: answer may be incomplete." line or fake a `Proposal ...: awaiting_approval` line in front of the banker. §9 treats `answer` as display text only. It should not be able to control the terminal.
**Fix:** Before printing, replace C0/C1 control characters other than `\n` and `\t` (`c.is_control() && c != '\n' && c != '\t'`) with U+FFFD or escape them.

## Info

### IN-01: The `token_arm` condition is redundant and hard to review

**File:** `src/receipt.rs:149-159`
**Issue:** The `matches!` already lists `LowIlPrimary`/`BrokerLaunch`/`BrokerLaunchNoPty`, and the condition then excludes the same three values again. The effective rule is `token_arm == expected && expected ∉ {"", LowIlPrimary, BrokerLaunch, BrokerLaunchNoPty}`.
**Fix:** Write that rule directly, with an explicit allowlist of the accepted arms.

### IN-02: Stub-only env names in `main.rs` rely on constant folding to stay out of release builds

**File:** `src/main.rs:141`
**Issue:** `cfg!(feature = "stubs") && env::var("LIAM_STUBS")...` still compiles the `"LIAM_STUBS"` literal into release builds. Only the optimizer removes it, which TC-BUILD-01 checks after the fact. Separately, the gateway attempt (`main.rs:184-191`) is gated on `LIAM_STUB_GATEWAY_ATTEMPT` alone, not on `LIAM_STUBS=1` like the other stub overrides.
**Fix:** Use a `#[cfg(feature = "stubs")]` helper function, as for `launch_process_id`, and also gate the gateway attempt on `LIAM_STUBS=1`.

### IN-03: The hand-written vocabulary purpose parser is fragile

**File:** `src/main.rs:82-98`
**Issue:** A blank line or a column-0 comment inside `purposes:` ends the list. Quoted items and items with trailing comments never match, and `purposes:` is matched at any indentation. It fails closed and the file is SHA-pinned, so this is low risk.
**Fix:** Parse with a YAML crate, or document that the parser accepts only the pinned layout.

### IN-04: Duplicate entries in `injected_on` are accepted

**File:** `src/receipt.rs:219-222`
**Issue:** `["hometown","olive","olive"]` collapses to the required set and passes. The spec says "exactly the set".
**Fix:** Also require `agent.injected_on.len() == 2`.

### IN-05: Olive classification and serialization have small inaccuracies

**File:** `src/clients/olive.rs:60,70`
**Issue:** `status >= 500 || !(400..500).contains(&status)` is redundant. A request-serialization error is labelled `InvalidResponse`.
**Fix:** Simplify to `!(400..500).contains(&status)` and add a dedicated variant, or an `Unavailable` comment, for the serialization case.

### IN-06: `party_ref_hash` is an unsalted SHA-256 of a likely low-entropy identifier

**File:** `src/evidence.rs:158`
**Issue:** The spec requires this field, but party refs can be recovered by enumerating candidates and hashing them, so the "content-free" claim is weaker than it looks.
**Fix:** Record this as a spec open item, for example a keyed hash or a program-wide pepper decided by the program owner.

### IN-07: Older stub helpers panic inside spawned threads and leave threads detached

**File:** `src/stubs.rs:32-185,231-552`
**Issue:**
- `CaptureProxy`, `HometownStub` and `OliveStub` use `expect` inside worker threads. A panic there only shows up as a confusing downstream failure or a hang.
- `forward_two_to` spawns detached inner threads.
- `ScriptedStub::new` silently yields an empty `base_url` if the bind fails.

**Fix:** Move the remaining tests to `ScriptedStub`/`TestProxy` and delete the legacy helpers. Make a bind failure panic in the constructor, on the test thread.

### IN-08: Stub mode overrides `expected_token_arm`, so the release refusal path is never tested

**File:** `src/main.rs:141-145`; `tests/common/mod.rs:45`
**Issue:** Every process test writes `expected_token_arm = ""` and depends on the stub override to `WriteRestricted`. No process test exercises what production does with an empty or mismatched configured arm.
**Fix:** Add a process test that sets `LIAM_STUBS` off, or makes the override opt-in, and asserts `10 receipt:token_arm` for an empty configured arm.

### IN-09: Config, vocabulary and catalog paths are resolved against the working directory

**File:** `src/main.rs:108-112,182`
**Issue:** `liam.toml` and the configured relative paths depend on the launcher's CWD, not on the executable's directory, which is the granted read location.
**Fix:** Resolve them against `std::env::current_exe()?.parent()`, or document that the launcher must set the CWD.

---

_Reviewed: 2026-10-21T21:19:45Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
