---
quick_id: 261021-qrq
status: complete
date: 2026-10-21
key-files:
  modified:
    - vendor/generic-array/** (13 files)
    - vendor/schannel/src/key_handle.rs
    - RESULTS.md
---

# Quick 261021-qrq: Fix vendor line endings for fresh-clone build

**A fresh clone now builds and tests with `--locked --offline`.**

## What changed

- `735d925`: re-committed 14 vendored files with their checksummed CRLF bytes, using `git add --renormalize vendor`. The files are 13 under `generic-array` and `schannel/src/key_handle.rs`. They had been committed as LF before `vendor/** -text` (16c4425) took effect, so a fresh checkout failed cargo's vendor checksum check. No other vendor file changed. Afterwards `git ls-files --eol vendor` shows no index/worktree mismatch.
- RESULTS.md notes now record the fresh-clone evidence instead of the "main checkout only" caveat.

## Verification (fresh `git clone` into the scratchpad, core.autocrlf=true, no vendor copying)

- `cargo build --locked --offline`: pass
- `cargo test --locked --offline --features stubs`: 166 passed, 0 failed, 2 ignored
- `cargo clippy --locked --offline --all-targets --all-features -- -D warnings`: clean
- `contracts/vendored/vocabulary.yaml` SHA-256 in the clone is `60a44f00…`, which shows the `contracts/vendored/** -text` rule from plan 01-12 holds.

## Deviations

- The quick-task plan was written and executed inline by the orchestrator, with no planner or executor subagents, because the task was a single mechanical re-commit.
- A per-file checksum check through `git show` was too slow at about 8,600 files, so it was stopped. The fresh-clone `--locked --offline` build verifies every crate's checksums instead.
