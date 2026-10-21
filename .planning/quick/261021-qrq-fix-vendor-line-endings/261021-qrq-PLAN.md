---
quick_id: 261021-qrq
type: quick
description: Fix vendor line endings so a fresh clone builds with --locked --offline
files_modified: [vendor/generic-array/**, vendor/schannel/src/key_handle.rs, RESULTS.md, README.md]
---

<objective>
A fresh clone (including CI on windows-latest) must pass `cargo build --locked --offline` without copying vendor bytes from another checkout.
</objective>

<diagnosis>
`git ls-files --eol vendor` shows 14 files as `i/lf w/crlf attr/-text`: 13 under `vendor/generic-array/` and `vendor/schannel/src/key_handle.rs`. Their `.cargo-checksum.json` entries match the CRLF working-tree bytes (the published crate bytes), but the committed blobs are LF, because they were committed before `vendor/** -text` (16c4425) took effect. All other vendor files are already consistent.
</diagnosis>

<tasks>
<task type="auto">
  <name>Task 1: Re-commit the 14 vendor files with their checksummed bytes</name>
  <files>vendor/generic-array/**, vendor/schannel/src/key_handle.rs</files>
  <action>Run `git add --renormalize vendor` (with `-text`, this stores the working-tree bytes unchanged). Confirm that only the 14 files are staged, that `git ls-files --eol` shows `i/crlf w/crlf` for each, and that each staged blob's SHA-256 matches its `.cargo-checksum.json` entry. Commit.</action>
  <verify>No vendor file has an i/w EOL mismatch; for every vendored crate, each staged blob matches its checksum entry.</verify>
</task>
<task type="auto">
  <name>Task 2: Prove it on a fresh clone</name>
  <action>`git clone` the repo into the scratchpad. Run `cargo build --locked --offline`, then `cargo test --locked --offline --features stubs`, with no vendor copying. Use the same `.cargo/config.toml` as CI.</action>
  <verify>Both commands exit 0 in the fresh clone.</verify>
</task>
<task type="auto">
  <name>Task 3: Update the RESULTS.md and README notes</name>
  <action>Replace the "measured on the main checkout, not a fresh clone, because of the vendor line-ending defect" notes with the fresh-clone evidence and the fix commit. Do not change any TC status.</action>
</task>
</tasks>
