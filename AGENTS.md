# Liam implementation constraints

- Do not place secrets, keys, tokens, or credential-shaped values in code, configuration, environment handling, logs, tests, or fixtures.
- Use only the proxy supplied by the environment; do not create sockets, resolve names, or add a Liam-side network allowlist.
- Liam never calls the model gateway, executes a target-system action, executes model output, or spawns child processes.
- Liam reads only its session receipt and agent-session record from nono. It uses no nono IPC, services, profile edits, or other nono state.
- Evidence must be content-free: never write prompts, answers, context values, party references, or query-bearing URLs to logs, records, or stderr.
- Fail closed: before a network request, a failed prerequisite prints exactly one reason line and returns its documented non-zero exit code.
- A contract mismatch is a stop condition. Update the applicable contract table and README, then stop for review; do not silently adapt.

