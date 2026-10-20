# Stub records

`make-stub-records.ps1` creates a schema-2 nono receipt and schema-1 agent
record pair for CI-only stub-mode tests. The fixtures are supervisor-shaped:
they contain no key material, only the opaque UUID key identifier required for
ledger joins.

The production executable never creates these records. It only reads paths
provided by nono.

