# Architecture

## v0 (specified, not built)

```text
[harness]
   │  skill: SKILL.md
   │  hook:  leash-hook
   ▼
[leash-hook]
   │  reads .leash/session.json
   │  canonicalize(path)
   │  radius check
   │  ruleset on hunk
   ▼
allow | deny
```

```text
[leash-engine]
   │  git diff
   │  optional: arbor map --from-diff --budget-tokens N
   ▼
.leash/session.json
```

No long-running process. Engine runs at session start and on
expand. Hook is a short-lived binary or script the harness already
knows how to spawn.

## v1

Replace diff-only ranking with Arbor radius. Same session file
schema. Same hook. Engine is the only swap.

## v2

The engine’s deny/pack library is called from a GitHub Action.
Same rule ids. Different door.

## Language

Not decided in this ADR set on purpose. Constraints:

- Hook must start fast (see [hooks.md](hooks.md) budgets)
- Ship as a single static binary **or** a script with zero
  user-installed language runtime beyond what the harness has
- Arbor integration is a subprocess in v1, not a rewrite of arbor-core
  into this repo

Candidate later: Rust (matches Arbor, hook latency) or Go. Not a
Python daemon.

## Data

`.leash/` is local, gitignored by default in the skill’s install
notes. Do not commit session files. They can contain path
rankings of private code.

## Telemetry

Off. No network in v0 hook or engine except what the user already
runs (`arbor`, `git`). A phone-home is a security bug.
