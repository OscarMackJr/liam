# Liam Amendment v0.4.1 — Proxy Transport

Version: 0.4.1
Status: **Draft — for ratification.** Liam-only: amends `LIAM_DESIGN_SPEC_v0.4.md` §9, §10.2, §12 and `LIAM_TEST_SPEC_v0.4.md` §3.2. It does not touch design §4, so under the v0.4 change-control clause it needs no re-ratification of `NONO_v0.4.3`.
Repository: `liam/specs/`, beside the v0.4 specs.
Audience: Liam implementer; nono owner (assumption A5).
Primary post-read action: ratify before Gate 3 (real-session work).

---

## 1. Why

The release binary imports `ws2_32` and `getaddrinfo` through `reqwest` (via `hyper-util`'s default connector). That does not show Liam resolving service hosts at run time, but it contradicts the literal wording of design §9 ("no direct sockets … no local DNS resolution") and handoff hard rule 2. The wording has to match what the binary actually is.

No transport that reaches a network service avoids a socket: any HTTP client, including a hand-written one over `std::net::TcpStream`, links Winsock, and Rust's standard library itself links `getaddrinfo`. So the stricter rule cannot be met by swapping the transport. What can be guaranteed is **where** Liam connects and **that it resolves nothing**, and that is what this amendment records.

---

## 2. Decision (new D14)

**Liam opens TCP connections to exactly one endpoint: nono-proxy at `127.0.0.1:<proxy_port>`. It performs no name resolution at run time. Every request to hometown or olive is sent through that proxy, so the service host names travel only inside CONNECT requests and are resolved by nono-proxy, never by Liam.**

Enforced in Liam's code, not only by nono's fence:

1. **Proxy URL validation at startup.** `HTTPS_PROXY` and `HTTP_PROXY` must both be present, identical, and of the exact form `http://127.0.0.1:<port>`: `http` scheme, IPv4 loopback literal, no user information, no path. The port must equal the agent-session record's `proxy_port`. Anything else (`localhost`, `[::1]`, a host name, `https://`, credentials, mismatched values) exits 11 with `env:proxy_not_loopback`.
2. **`NO_PROXY` must be unset or empty.** Any other value exits 11 with `env:no_proxy_nonempty`, because a non-empty value could route a request around the proxy.
3. **One client, built once, explicitly.** The HTTP client is constructed in a single function with:
   - `ClientBuilder::no_proxy()` first, which disables environment and Windows system (WinINET/registry) proxy detection;
   - then `.proxy(Proxy::all(<validated loopback URL>))`, with no `NoProxy` list;
   - then `.dns_resolver(DenyAllResolver)`, a resolver that returns an error for every name.

   Because the proxy is an IP literal, connecting to it calls no resolver. Because every request is proxied, target hosts go into CONNECT unresolved. If any code path ever tried to resolve a name, the deny-all resolver makes it fail closed instead of issuing a query.
4. **No sockets or resolver calls in Liam's own source.** No `std::net`, `ToSocketAddrs`, `lookup_host`, raw Winsock or second HTTP client anywhere outside the single client constructor.

**What the imports now mean.** `ws2_32` and `getaddrinfo` remain linked as library code paths. The claim is behavioural and is evidenced by the tests in §5, not by the import table. TC-BUILD-02 is unchanged: `ws2_32.dll` is a Windows system DLL.

---

## 3. Alternatives Considered

| Option | Disposition |
|---|---|
| Permit only the loopback proxy connection, deny all resolution in code (**chosen**) | Matches what any HTTP client must do; adds two code-level guarantees on top of nono's fence |
| Replace HTTP with a named pipe or other IPC to nono | **Rejected.** A runtime channel from the agent to nono is what `DECISION_desktop-agent-nono-integration.md` §3.2 rejected, and it would make nono agent-specific |
| Hand-written HTTP/CONNECT client over `std::net::TcpStream` | **Rejected.** Still a socket, still links `getaddrinfo` through `std`, and replaces reviewed TLS and HTTP code with unreviewed code for no reduction in what is linked |
| Leave the wording as is and treat the imports as acceptable silently | **Rejected.** A security requirement that the binary visibly contradicts is a finding waiting to be raised; the wording has to be true |

---

## 4. Amended Text

### 4.1 Design §9, first bullet — replace with

> - **One socket destination, no name resolution (D14).** Liam connects only to nono-proxy at `127.0.0.1:<proxy_port>`; it refuses to start unless `HTTPS_PROXY`/`HTTP_PROXY` are that exact loopback URL and `NO_PROXY` is empty (`env:no_proxy`, `env:proxy_not_loopback`, `env:no_proxy_nonempty`). The single HTTP client disables system proxy detection, uses only that proxy, and carries a deny-all DNS resolver. Liam's own source contains no socket or resolver calls. Winsock and `getaddrinfo` are linked by the HTTP library; the guarantee is behavioural and tested (TC-NET-05 to -08).

### 4.2 Design §10.2 — append

> `HTTPS_PROXY` and `HTTP_PROXY` must be identical and of the form `http://127.0.0.1:<port>`, with the port equal to the agent-session record's `proxy_port`. `NO_PROXY` must be unset or empty. Liam does not read proxy settings from the Windows registry or WinINET.

### 4.3 Design §11 exit codes — code 11 meaning becomes

> Environment check failed (proxy missing, proxy not the loopback literal, `NO_PROXY` non-empty, vocabulary hash or purpose)

### 4.4 Design §12 — REQ-LIAM-03 becomes

| REQ | Requirement | Verified by |
|---|---|---|
| REQ-LIAM-03 | Liam opens network connections only to nono-proxy at `127.0.0.1:<proxy_port>`, performs no name resolution, ignores system proxy settings, and refuses to start on any non-conforming proxy environment | TC-NET-01..03, TC-NET-05..08 |

### 4.5 Handoff hard rule — replace "No direct network" with

> - **One socket destination, no DNS.** Connect only to nono-proxy at the validated `http://127.0.0.1:<port>`; build one HTTP client with `no_proxy()`, an explicit `Proxy::all` and a deny-all resolver; no socket or resolver calls in Liam's own source. Do not add an allowlist of your own.

---

## 5. Test Changes (`LIAM_TEST_SPEC_v0.4.md` §3.2)

| TC | REQ | Setup | Steps | Pass criteria | Tag |
|---|---|---|---|---|---|
| TC-NET-03 (revised) | 03, 17 | Static check | Grep Liam's own source for `std::net`, `ToSocketAddrs`, `lookup_host`, `TcpStream`, Winsock calls, any second client constructor, and any gateway host or path | Zero matches outside the single client constructor; the constructor calls `no_proxy()`, `proxy(Proxy::all(..))` and `dns_resolver(..)`; no gateway reference. Library imports are out of scope of this check (D14) | ci-elevated |
| TC-NET-05 | 03 | `valid` record pair; a listener on every alternative address used below | Run once per case: `HTTPS_PROXY` = `http://localhost:p`, `http://[::1]:p`, `http://proxy.lab:p`, `https://127.0.0.1:p`, `http://u:pw@127.0.0.1:p`; `HTTP_PROXY` differing from `HTTPS_PROXY`; `NO_PROXY=hometown.lab` | Exit 11 with `env:proxy_not_loopback` (first six) or `env:no_proxy_nonempty` (last); no listener receives a connection; no run record | ci-elevated |
| TC-NET-06 | 03 | Stub build with the resolver's invocation counter exposed; capturing test proxy on `127.0.0.1` | Full successful run with a valid proposal | Run completes; resolver invocation count is **0**; a unit test shows the resolver returns an error for any name | ci-elevated |
| TC-NET-07 | 03 | HKCU Internet Settings proxy (`ProxyEnable=1`, `ProxyServer`) pointed at a trap listener; environment points at the test proxy | Full run | Trap listener records zero connections; all requests reach the test proxy | ci-elevated |
| TC-NET-08 | 03 | Real nono session; ETW trace of the `Microsoft-Windows-DNS-Client` provider for the run | Full run | Zero DNS queries attributed to `liam.exe`'s PID; nono-proxy's own queries are expected and are not Liam's | dev-standard-user |

---

## 6. Assumption Added

- **A5.** nono-proxy places `http://127.0.0.1:<port>` (IPv4 literal, not `localhost`) in the confined child's `HTTPS_PROXY` and `HTTP_PROXY`, with the same port it records as `proxy_port`, and sets `NO_PROXY` empty. To be confirmed by the nono owner; if nono uses another form, the fix is on nono's side, because a host-name proxy URL would require Liam to resolve a name.

---

## 7. Requirements Summary

| Change | Item |
|---|---|
| New decision | D14 |
| Revised requirement | REQ-LIAM-03 |
| New exit reasons | `env:proxy_not_loopback`, `env:no_proxy_nonempty` |
| Revised test | TC-NET-03 |
| New tests | TC-NET-05, -06, -07 (`ci-elevated`), -08 (`dev-standard-user`) |
| New assumption | A5 (nono owner to confirm) |

---

## Ratification Record

| Field | Value |
|---|---|
| Decision | **Ratified**: D14 permits Liam's single validated loopback-proxy socket destination and prohibits runtime name resolution |
| Date | 2026-10-20 |
| Ratified by | Oscar Mack (architecture and risk decisions) |
| Scope ratified | D14; amended Liam design §§9, 10.2, 11 and 12; revised TC-NET-03; new TC-NET-05 through TC-NET-08; assumption A5 |
| Does not change | The contract-only Nono boundary, Nono v0.4.3 record contract, or the ratified agent-key seams |
| Implementation condition | Liam must implement the code-level controls and pass the local tests before a real-session claim; TC-NET-08 remains dev-standard-user evidence |
