
## From 02.1-04
- tc_key_03_scan_finds_no_credential_shaped_values (tests/hygiene_contract.rs) fails pre-existing: contracts/receipt-fields.md line 14 contains the 40-hex nono commit SHA cf3ebc56... (added in 02-03 commit 2fca0bb), flagged as hex-run:40. Fix options: add a named exception for the nono commit reference in the scanner, or abbreviate the SHA in the doc. Not caused by the service-URL change.
- RESOLVED (orchestrator, 2026-10-22): shortened the nono SHA in contracts/receipt-fields.md to 12 hex; full hash kept in 02-SIGNOFF.md. hygiene_contract: 10 passed.
