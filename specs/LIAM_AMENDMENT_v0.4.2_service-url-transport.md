# Liam Amendment v0.4.2 — Service-URL Transport

Version: 0.4.2
Status: **DRAFT — not ratified.** nono Phase 126.1 accepted C1–C4 in planning (2026-10-22; C3 by deterministic route derivation); awaiting nono operator sign-off, nono’s HTTPS_PROXY/NO_PROXY answer (O-A), and ratification. Liam code does not change until ratified.
Amends: `LIAM_DESIGN_SPEC_v0.4.1.md` D14, §9 first bullet, §10.2, §11 (code 11), §12 (REQ-LIAM-03); `LIAM_TEST_SPEC_v0.4.1.md` §3.2 (TC-NET-03, -05). Supersedes `LIAM_AMENDMENT_v0.4.1_proxy-transport.md` §2 items 1–3 and assumption A5.
Re-ratification required with: `NONO_v0.4.2` §2–§3 (injection mechanism) only. C3 is met without changing `AgentSessionRecord`, so `NONO_v0.4.3` §5 and Liam design §4 are not in scope.
Repository: `liam/specs/`.
Audience: Liam implementer; nono owner; popeye owner (no change expected).

---

## 1. Why

`NONO_v0.4.2` §3 and Liam D14 assume nono-proxy injects the agent key into requests that Liam sends through an HTTP CONNECT tunnel. nono-proxy does not terminate TLS, so inside a CONNECT tunnel it sees only ciphertext and cannot add a header. The ratified design cannot be built as written. nono Phase 126.1 (context commit `8a6b3446`) instead gives each `inject_on` service a local reverse-proxy route: the child sends plain HTTP to loopback, nono-proxy injects the key and opens TLS to the real service. CONNECT to an `inject_on` FQDN is refused in agent sessions.

The alternative, a TLS-intercepting proxy with a locally trusted CA, puts the same plaintext inside nono plus a machine-trusted interception CA, and is rejected (§4).

---

## 2. Decision (D14 revised)

**Liam opens TCP connections only to nono-proxy's loopback service routes at `127.0.0.1:<proxy_port>`, one per service it uses (hometown, olive). It performs no name resolution at run time and uses no forward proxy. Service host names never reach Liam.**

Enforced in Liam's code:

1. **Service URL validation at startup.** `NONO_SERVICE_HOMETOWN_URL` and `NONO_SERVICE_OLIVE_URL` must both be present and of the exact form `http://127.0.0.1:<port>/<route>`: `http` scheme, IPv4 loopback literal, no user information, no query or fragment. `<port>` must equal the agent-session record's `proxy_port`. `<route>` must equal `/svc/<service_id>` for that service (`/svc/hometown`, `/svc/olive`), the deterministic derivation nono uses (C3). Anything else exits 11 (reasons in §5). `NONO_SERVICE_GATEWAY_URL` is ignored; Liam never calls the gateway (REQ-LIAM-11).
2. **Base URLs leave `liam.toml`.** `[hometown].base_url` and `[olive].base_url` are removed; Liam's request paths are appended to the validated service URL. Config keeps timeouts and the catalog.
3. **One client, built once, no proxy.** The HTTP client is constructed in a single function with `ClientBuilder::no_proxy()` (disables environment and WinINET/registry proxy detection) and `.dns_resolver(DenyAllResolver)`. No `Proxy::all`. Because each URL is an IP literal, no resolver is called; any code path that tried to resolve a name fails closed.
4. **Forward-proxy variables are not used.** `HTTPS_PROXY`, `HTTP_PROXY` and `NO_PROXY` are not read for routing. (Whether Liam should refuse to start if they are set is open item O-A, pending nono's answer.)
5. **No sockets or resolver calls in Liam's own source** (unchanged from v0.4.1).
6. **Liam never sends `Authorization`** (unchanged; TC-KEY-01). nono-proxy injects the agent key on the loopback route.

**Trust note.** On the loopback hop Liam's request bodies (party reference, question) and Olive proposals are plaintext to nono-proxy. nono-proxy is already the trusted holder of the agent key; condition C4 requires it never to log bodies, query strings or paths beyond the route prefix.

---

## 3. What Does Not Change

- Liam's own evidence stays content-free (REQ-LIAM-10).
- Exit codes, prerequisite order (all before any network request), the deny-all resolver and its counter (TC-NET-06), WinINET immunity (TC-NET-07) and zero DNS from Liam's PID (TC-NET-08).
- Liam still reads only the two nono records via `NONO_RECEIPT_PATH` / `NONO_AGENT_RECORD_PATH`.

---

## 4. Alternatives Considered

| Option | Disposition |
|---|---|
| Per-service loopback reverse-proxy routes (**chosen**, nono 126.1) | Buildable without TLS interception; keeps one destination class (loopback literal) and no DNS in Liam |
| Keep CONNECT; nono-proxy terminates TLS with a local CA | **Rejected.** Same plaintext inside nono, plus a machine-trusted interception CA and certificate pinning problems |
| Liam attaches the agent key itself | **Rejected.** Violates the no-credential rule (Liam never holds a key) |
| Named pipe / IPC to nono | **Rejected** as in v0.4.1 (`DECISION_desktop-agent-nono-integration.md` §3.2) |

---

## 5. Amended Text

### 5.1 Design §9, first bullet — replace with

> - **Loopback service routes only, no name resolution (D14).** Liam connects only to nono-proxy's service routes `http://127.0.0.1:<proxy_port>/<route>` taken from `NONO_SERVICE_HOMETOWN_URL` and `NONO_SERVICE_OLIVE_URL`, validated against the agent-session record (port) and the `/svc/<service_id>` rule (route). The single HTTP client disables system proxy detection, uses no proxy, and carries a deny-all DNS resolver. Liam's own source contains no socket or resolver calls.

### 5.2 Design §10.2 — replace the v0.4.1 proxy paragraph with

> `NONO_SERVICE_HOMETOWN_URL` and `NONO_SERVICE_OLIVE_URL` must be of the form `http://127.0.0.1:<port>/<route>`, with the port equal to the agent-session record's `proxy_port` and the route equal to `/svc/hometown` or `/svc/olive` respectively. Liam does not use `HTTPS_PROXY`, `HTTP_PROXY` or `NO_PROXY` for routing and does not read proxy settings from the Windows registry or WinINET.

### 5.3 Design §11 — code 11 meaning becomes

> Environment check failed (service URL missing, not the loopback literal, port or route not matching the agent record; vocabulary hash or purpose)

### 5.4 Design §12 — REQ-LIAM-03 becomes

| REQ | Requirement | Verified by |
|---|---|---|
| REQ-LIAM-03 | Liam opens network connections only to nono-proxy's loopback service routes on `127.0.0.1:<proxy_port>`, performs no name resolution, uses no forward proxy, ignores system proxy settings, and refuses to start on any non-conforming service URL | TC-NET-01..03, TC-NET-05..09 |

### 5.5 Exit reasons (code 11)

| Reason | Cause |
|---|---|
| `env:service_url_missing` | `NONO_SERVICE_HOMETOWN_URL` or `NONO_SERVICE_OLIVE_URL` absent or empty |
| `env:service_url_not_loopback` | not `http`, not the IPv4 literal `127.0.0.1`, userinfo, query or fragment present |
| `env:service_url_port` | port differs from the agent record's `proxy_port` |
| `env:service_url_route` | route is not `/svc/hometown` / `/svc/olive` for the respective variable |

Retired: `env:proxy_not_loopback`, `env:no_proxy_nonempty` (unless O-A keeps a refusal).

### 5.6 Config

`liam.toml`: remove `[hometown].base_url`, `[olive].base_url`.

---

## 6. Conditions on nono (sent to nono Phase 126.1, 2026-10-22)

| ID | Condition | Status |
|---|---|---|
| C1 | Each `NONO_SERVICE_<SVC>_URL` is exactly `http://127.0.0.1:<port>/<route>`: IPv4 literal (never `localhost` or `::1`), no userinfo, no query | accepted; nono test asserts exact form |
| C2 | `<port>` equals `AgentSessionRecord.proxy_port` | accepted; nono test ties env port to `.agent.json` |
| C3 | Preferred: the agent record carries a content-free route per injected service (or the route is a documented deterministic function of `ServiceId`), so Liam validates supervisor-attested addresses | met by derivation: route = `/svc/<service_id>` from one nono function used by both route registration and env builder; no struct change |
| C4 | nono-proxy never logs request or response bodies, non-structural headers, query strings, or paths beyond the route prefix | accepted; tracing audit + sentinel test (path suffix, query, body) |

---

## 7. Test Changes (`LIAM_TEST_SPEC_v0.4.1.md` §3.2)

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-NET-03 (revised) | 03, 17 | Static check | Grep Liam's own source for `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, Winsock calls, `Proxy::`, any second client constructor, and any gateway host or path | Zero matches outside the single client constructor; the constructor calls `no_proxy()` and `dns_resolver(..)` and no `proxy(..)`; no gateway reference | ci-elevated |
| TC-NET-05 (revised) | 03 | `valid` record pair; a listener on every alternative address used below | Run once per case for each of the two variables: `http://localhost:p/r`, `http://[::1]:p/r`, `http://svc.lab:p/r`, `https://127.0.0.1:p/r`, `http://u:pw@127.0.0.1:p/r`, `http://127.0.0.1:p/r?x=1`, port ≠ `proxy_port`, variable unset | Exit 11 with the §5.5 reason; no listener receives a connection; no run record | ci-elevated |
| TC-NET-09 (new) | 03 | Stub build; capturing listener on `127.0.0.1:<proxy_port>` serving both routes | Full successful run | Every request arrives as plain HTTP on the expected route prefix; no CONNECT is ever sent; no `Authorization` header | ci-elevated |
| TC-NET-06, -07, -08 | 03 | unchanged | unchanged | unchanged (resolver count 0; WinINET ignored; zero DNS from Liam's PID) | as before |

---

## 8. Open Items

- **O-A.** Resolved by nono facts (126.1 plan head `ea68b3cf`, 2026-10-22): agent sessions always set `HTTPS_PROXY`/`HTTP_PROXY` (and lowercase forms) to `http://nono:<session token>@127.0.0.1:<proxy_port>`, plus `NODE_USE_ENV_PROXY=1`, `NONO_PROXY_TOKEN`, `NONO_NO_PROXY`, and `NO_PROXY=localhost,127.0.0.1[,bypass hosts]`. Liam therefore **ignores** all forward-proxy variables (refusing would refuse every real session). Because these values carry a session token, Liam must never read, parse, log or echo them; the TC-NET-03 static check should confirm no remaining `HTTPS_PROXY`/`HTTP_PROXY`/`NO_PROXY` reads in non-test source. (Under v0.4.1 D14, Liam's exact-form check would have refused these real values anyway.)
- **O-B.** Whether `injected_on` must contain both `hometown` and `olive` as a Liam startup check (currently implied by the record's content-free validation; make explicit at ratification).
- **O-C.** Resolved: C3 by derivation, no `NONO_v0.4.3` change.
- **O-D.** nono facts to record at ratification: CONNECT refusal is host-normalized for `inject_on` FQDNs only, other hosts follow the profile allowlist (a Liam profile allowing no other hosts gets no CONNECT); only `inject_on` services get `NONO_SERVICE_*`; inherited `NONO_SERVICE_*`, `NONO_RECEIPT_PATH`, `NONO_AGENT_RECORD_PATH` are scrubbed; every agent profile declares the gateway FQDN (supervisor derives the popeye exchange URL from it) but the child gets no gateway URL unless gateway is in `inject_on`.

---

## 9. Requirements Summary

| Change | Item |
|---|---|
| Revised decision | D14 |
| Revised requirement | REQ-LIAM-03 |
| New exit reasons | `env:service_url_missing`, `env:service_url_not_loopback`, `env:service_url_port`, `env:service_url_route` |
| Retired exit reasons | `env:proxy_not_loopback`, `env:no_proxy_nonempty` (subject to O-A) |
| Revised tests | TC-NET-03, TC-NET-05 |
| New test | TC-NET-09 |
| Superseded assumption | A5 (forward-proxy URL form) |
| Config | remove service `base_url`s |

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | *pending* |
| Date | *pending* |
| Ratified by | *pending* (Oscar Mack, architecture and risk decisions) |
| Ratified together with | `NONO_v0.4.2` §2–§3 revision (no `NONO_v0.4.3` change) |
| Implementation condition | No Liam code change before ratification; then implement and pass TC-NET-03, -05..-09 locally before any real-session claim |
