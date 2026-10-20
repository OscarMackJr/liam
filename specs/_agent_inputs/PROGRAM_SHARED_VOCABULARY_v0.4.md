# Program Shared Vocabulary v0.4

Version: 0.4
Status: **Reconstructed 2026-10-19 from the August 2026 design session.** The original v0.4 document was delivered as a download in that session (inside `olive-agent-inputs.zip`). This reconstruction is faithful to `vocabulary.yaml` v0.4 and to the surviving prose, but its REQ/TC tables are rebuilt, not recovered. Replace with the original if found.
Repository: fiskroad, `specs/`. Machine-readable form: `contracts/vocabulary.yaml` (v0.4, SHA-256 `60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7`, LF line endings).
Audience: all project leads. hometown, popeye, olive and bluto implement against this; nono and Liam validate against it.
Primary post-read action: confirm the v0.4 additions (§2–§4) are ratified, then re-vendor `vocabulary.yaml` in every consuming repository and update each `VENDORED.md` hash.

Supersedes v0.3 (SHA-256 `dbe74c63d1f5df8680685159e9c0d12e6073a79a363315cc6ea5168d5e8b7f2d`). Everything in v0.3 is carried forward unchanged (§1); v0.4 adds purposes, principal types and action outcomes.

---

## 1. Carried Forward From v0.3 (unchanged)

### 1.1 Canonical tenant identifier (closes X2)

`tenant_id` is a UUID in lowercase canonical form (8-4-4-4-12), identical in every store, header, event, trace and seed record. Human-readable tenant names are display labels and never travel as identifiers.

Three POC tenants were minted once on 2026-08-02 and are permanent: `TENANT_ALPHA`, `TENANT_BETA`, `TENANT_GAMMA`. Their UUIDs live in `vocabulary.yaml` and nowhere else as a source of truth.

### 1.2 Canonical data-class taxonomy (closes X1)

Five classes, exact strings, ordered least to most restricted:

`public` < `internal` < `confidential-personal` < `regulated-financial` < `regulated-identifier`

- Class names are exact. No synonyms, no abbreviations. An unlisted name is a validation error.
- `regulated-identifier` is structurally unapprovable: no model may declare it as `approved_max_data_class`.

---

## 2. Purposes (new at v0.4; gap G3)

### 2.1 Definition

A **purpose** states why data is being accessed. It is orthogonal to data class: data class limits **which models** may see data; purpose limits **which data** may be seen at all.

Seven purposes, exact strings:

`servicing`, `origination`, `collections`, `analytics`, `marketing`, `compliance-review`, `internal-support`

### 2.2 No default

There is no default purpose. An undeclared purpose is a validation error, because the only safe fallback would be "no access", and a silent fallback to that is indistinguishable from an outage.

### 2.3 Where purpose is enforced

- **hometown** owns purpose semantics: which fields a purpose permits, and masking of the rest (HOMETOWN_v0.4 §4).
- **popeye** does not know what a purpose means. It validates the string against this list at key issuance, records it on the ledger, and enforces the one pairing rule in §2.4 (POPEYE_v0.4 §3).
- **olive** checks purpose against an action's requirements at authorization.
- **Agents** (Liam and successors) never choose a purpose on the user's behalf. On the endpoint the purpose is declared at session launch, bound to the session's agent key at mint (`NONO_v0.4.1` §4.1, `POPEYE_v0.4.1` §2.2), and carried by the key rather than per request.

### 2.4 Purpose/class prohibitions

The one hard-coded pairing rule; everything else is configuration:

| Purpose | May never receive unmasked individual-level records of |
|---|---|
| `analytics` | `regulated-financial`, `regulated-identifier` |
| `marketing` | `regulated-financial`, `regulated-identifier` |

popeye enforces this as a label relationship at request time, with no content inspection: an `analytics` or `marketing` key requesting a model approved for `regulated-financial` or above is refused with a policy error.

---

## 3. Principal Types (new at v0.4; gap G2)

### 3.1 Definition

Three principal types: `user`, `application`, `agent`. An agent is neither its user nor its application; it is a third kind of actor with its own identity in every record.

### 3.2 Agent delegation

- `on_behalf_of` is **mandatory** for an agent principal and may not itself be an agent.
- Onward agent-to-agent delegation is **refused** this revision (`parentRunId` in the ATE is reserved, not used).
- An agent's effective permissions are the **intersection** of its own and its delegator's, never the union. Delegation narrows.

### 3.3 Operational consequence

Agent credentials are short-lived. A long-lived agent credential on a workstation is the shadow-AI problem returning in costume. On the endpoint, the credential is injected by nono-proxy per session and the agent never holds it.

---

## 4. Action Outcomes (new at v0.4; gap G1)

### 4.1 Lifecycle

Eight outcomes, exact strings, in lifecycle order:

`proposed` → `refused_policy` | `awaiting_approval` → `approved` | `rejected` → `executed` | `execution_failed`; `expired` from any waiting state.

### 4.2 Two outcomes that must not be collapsed

- `refused_policy`: the system declined before any human saw the proposal.
- `rejected`: a human declined it.

Policy refusal precedes human review. A proposal the system would refuse never reaches the queue; otherwise approvers learn to rubber-stamp. Collapsing the two would make the traced-refusal demonstration (POC-2 beat 4) undemonstrable.

### 4.3 Action identifiers

`action_id_format: ^[a-z0-9_.]+@[0-9]+$` — a catalog action type followed by `@` and its version number. Olive owns the catalog.

---

## 5. Conformance

`contracts/vocabulary.yaml` gains `purposes`, `principal_types`, `action_outcomes` and the `purpose_class_prohibitions` pairing rule from §2.4. Consuming repositories vendor it with a recorded hash, as at v0.3. popeye's registry conformance check extends to purposes and principal types.

| REQ | Requirement | Verified by | Method |
|---|---|---|---|
| REQ-VOC-01 | Every stored `tenant_id` parses as a lowercase canonical UUID (v0.3, unchanged) | TC-VOC-01: hometown live attribution verifier; bluto and Intrepid column types | CI (live) |
| REQ-VOC-02 | Seed data uses the three recorded POC tenant UUIDs (v0.3, unchanged) | TC-VOC-02: seed script review; ledger rows join to Intrepid rows on tenant | DEMO |
| REQ-VOC-03 | popeye's model-approval registry uses only the program class names (v0.3, unchanged) | TC-VOC-03: CI comparison of registry `data_classes` against `vocabulary.yaml` (popeye REQ-DCE-05) | CI |
| REQ-VOC-04 | No registry entry approves `regulated-identifier` (v0.3, unchanged) | TC-VOC-04: registry schema validation rejects a deliberate bad entry (popeye REQ-DCE-06) | CI |
| REQ-VOC-05 | Each consuming repository vendors `vocabulary.yaml` unchanged and records its SHA-256 in `VENDORED.md` | Hash comparison against `fiskroad/contracts/vocabulary.yaml` | CI |
| REQ-VOC-06 | Purpose values validate against §2.1 at issuance and on every request; an undeclared or unlisted purpose is refused | popeye REQ-PUR-06, REQ-AGT-07; nono REQ-END-09; hometown and olive request validation | CI |
| REQ-VOC-07 | The §2.4 prohibition is enforced at request time without content inspection | popeye REQ-PUR-07 | CI |
| REQ-VOC-08 | Agent principals carry `on_behalf_of`, which may not itself be an agent | popeye REQ-AGT-01, REQ-AGT-06 | CI |
| REQ-VOC-09 | Action outcome values validate against §4.1; `refused_policy` and `rejected` are distinct in every store and trace | olive schema check; ATE validation | CI |
| REQ-VOC-10 | Onward agent-to-agent delegation is refused | popeye REQ-AGT-02, REQ-AGT-05 | CI |

**Numbering note.** REQ-VOC-01 to -04 (and TC-VOC-01 to -05, of which popeye uses -03 to -05) are recovered from v0.3. REQ-VOC-10 is fixed by external citation (`POPEYE_v0.4` §2.1 cites it for onward delegation). REQ-VOC-05 to -09 are assigned in this reconstruction; the original v0.4 numbering for them was not recoverable. Cite them by ID only after this document is ratified.

**Line endings.** Hashes are computed over LF. A Windows checkout with `core.autocrlf true` may present CRLF in the working tree while the committed blob stays LF; vendoring agents hash what they commit, not what they see.
