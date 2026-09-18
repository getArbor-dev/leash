# Vuln gate

## Allowed claim

> Leash can deny a patch when the path is outside the working set,
> or when the hunk matches a small, named agent-failure ruleset.

## Forbidden claims

- Blocks CVEs
- Stops prompt injection
- Replaces SAST, DAST, or a sandbox
- Verifies the rest of the program is safe

## Default ruleset (v0)

Stable ids. Each id is a test fixture.

| Id | Hunk / path pattern | Why agents hit it |
| --- | --- | --- |
| `LEASH-SEC-001` | Secret-shaped assignments in added lines | Agents paste keys into config |
| `LEASH-SEC-002` | `eval` / `exec` / `Function(` on non-literal input in added lines | “quick dynamic” patches |
| `LEASH-SEC-003` | SQL string concat in added lines | Naive query patches |
| `LEASH-SEC-004` | HTML/markdown interpolated into DOM sinks in added lines | XSS in “just a component” |
| `LEASH-SEC-005` | Manifest add without lockfile change in the same apply | Silent dependency drift |

These are cheap, deterministic, and embarrassingly specific. That
is the point. Cloudflare’s skill can do a multi-phase audit. We
run before apply, on the hunk, in milliseconds.

## What a deny looks like

```text
deny RULE LEASH-SEC-001 path=src/config.ts
secret-shaped assignment in added lines; rotate if this was real
```

Do not echo the secret. Point at the line number.

## What happens after deny

The working set does not expand as a reward. The agent must change
the hunk or ask the user. “Retry until it applies” is a bug if the
rule still matches.

## Independent verification

v0 fixtures are the verification. A rule without a fixture does not
ship. We will not use an LLM-as-judge as the deny oracle.
