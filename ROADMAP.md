# Roadmap

Dates are not promises. Order is.

## Now — contract

- This repository: skill contract, hook contract, threat model, ADRs
- README one-liner and 30-second install UX frozen unless an ADR says
  otherwise

## v0 — enforce

- `leash.yml` schema
- PreToolUse hook that can deny Edit / Write / mutating Bash
- Diff-only working set fallback when Arbor is missing
- Fixture pack: out-of-radius write, secret in hunk, symlink path
- Fail-visible when the hook is not installed

## v1 — graph radius

- Arbor (or a vendored equivalent) as the working-set engine
- Token budget packing with reasons on every path
- Expand-set API with an audit line (who/why/tokens)

## v2 — same binary, more doors

- GitHub Action: PR comment is the working set + denials, not a
  novel
- CI gate on the same ruleset as the hook

## v3 — fleet (optional)

- Per-worktree working set and deny hook
- Cost cap per worker
- Not a new agent

## Explicitly never on this roadmap

- A coding agent
- A 200-skill marketplace
- Kernel isolation
- “Stops CVEs” as a headline
