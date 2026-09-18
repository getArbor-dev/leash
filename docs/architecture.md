# Architecture

## v0

```text
[harness]
   │  skill: SKILL.md
   │  hook:  leash hook
   ▼
[leash hook]
   │  reads .leash/session.json
   │  canonicalize(path)
   │  radius check
   │  ruleset on hunk
   ▼
allow | deny
```

```text
[leash session]
   │  git diff --name-only HEAD
   │  optional --path seeds
   ▼
.leash/session.json
```

No long-running process. The crate is Rust ([ADR 0006](adr/0006-rust-hook.md)).
Engine runs at session start and on expand. Hook is a short-lived
binary the harness already knows how to spawn.

## v1

Same session file. Same hook. The engine is the swap.

```text
[leash session]
   │  git diff --name-only HEAD          (required)
   │  arbor diff . --json                (when arbor is on PATH)
   │  arbor callees / callers --json     (one hop)
   ▼
.leash/session.json
   engine: arbor | diff
```

`arbor map` is not the working set. See [ADR 0007](adr/0007-arbor-subprocess.md).
Missing Arbor, timeout, or junk JSON fall back to git diff and an
audit line. The PreToolUse hook does not spawn Arbor.

## v2

The engine’s deny/pack library is called from a GitHub Action.
Same rule ids. Different door.

## Language

Rust. See [ADR 0006](adr/0006-rust-hook.md).

Arbor integration remains a subprocess in v1, not a rewrite of
arbor-core into this repo.

## Data

`.leash/` is local, gitignored by default in the skill’s install
notes. Do not commit session files. They can contain path
rankings of private code.

## Telemetry

Off. No network in v0 hook or engine except what the user already
runs (`arbor`, `git`). A phone-home is a security bug.
