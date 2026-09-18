---
name: leash
description: >
  Budget the coding agent's working set from the call graph, refuse
  out-of-radius Edit/Write/Bash, and keep the session inside a token
  budget. Use when starting a coding task, before applying a patch,
  when the agent is about to read the whole repo, or when a change
  might touch auth, billing, webhooks, or other high-radius code.
license: MIT
compatibility: >
  Claude Code, Codex, Cursor, OpenCode, Gemini CLI. Requires a
  PreToolUse hook that can deny Edit, Write, and mutating Bash.
metadata:
  author: getArbor-dev
  version: 0.2.0
  status: v1
---

# Leash

You are running under Leash. Leash is not a suggestion. It is a
budget and a deny list.

## Hard rules

1. Do not read files outside the current working set unless Leash
   expands the set and records why.
2. Do not Edit, Write, or run mutating Bash on paths outside the
   working set. If a hook is present, it will deny you. If a hook is
   missing, stop and tell the user Leash is not enforcing.
3. Do not dump the repository into context. If you need more files,
   ask Leash to expand the budget with a reason tied to a symbol or
   path already in the set.
4. If a proposed hunk matches a blocked agent-failure pattern
   (secrets, `eval`/`exec` on untrusted input, SQL/XSS in the changed
   hunk, new dependency with no lockfile change), do not apply it.
   Report the rule id and the file:line.
5. Never claim you “stopped a CVE.” Report rule ids, paths, and
   whether the deny was radius or ruleset.

## Working set

The working set is a ranked list of paths plus a token budget.

```text
leash.working_set:
  task: <one sentence>
  budget_tokens: <int>
  used_tokens: <int>
  paths:
    - path: <posix path>
      reason: <changed | caller | callee | test | config | expanded>
      symbols: [<name>]
  denials: []
```

`leash session` always reads `git diff --name-only HEAD`. If `arbor`
is on `PATH`, it also runs `arbor diff --json` plus one-hop
callers/callees and records `engine: arbor`. If Arbor is missing or
fails, it records `engine: diff` and an audit tag. Do not pretend
you walked a graph.

```bash
leash session --task "<one sentence>"
leash status
```

## When this skill activates

- Session start on a repo that has `leash.yml` or this skill installed
- User mentions blast radius, tokens, or “don’t touch that”
- Before the first Edit/Write of a session
- After a deny, to replan inside the set

## Commands

- `leash session --task TEXT [--path PATH]...`
- `leash hook` — PreToolUse stdin JSON, stdout Claude Code JSON
- `leash expand --path PATH --reason TEXT`
- `leash expand --symbol SYM --because TEXT`
- `leash status` — `enforcing` or `NOT ENFORCING`
- `leash install` — write `.claude/settings.json` if missing

## Output

Lead with the answer. Then the working set summary (paths, tokens
used / budget). Then denials, if any, as a table: rule, path, action.

Do not bury the patch behind a lecture.
