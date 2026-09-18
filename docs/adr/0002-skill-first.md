# ADR 0002: Skill-first distribution

## Status

Accepted

## Context

Octoverse 2025 and 2026 star boards show agent skills and harness
plugins as the install path developers actually use. A CLI-only
tool that is not a skill will not be loaded at session start.

## Decision

v0 distribution is `npx skills add getArbor-dev/leash` plus a
PreToolUse hook. No daemon. No account.

The skill briefs the model. The hook enforces. Both ship together.
A skill without a hook is `NOT ENFORCING`.

## Consequences

- Compatibility matrix is harnesses with deny-capable hooks first
- We will not claim “works on every agent” by dropping deny
- Root `SKILL.md` is a contract file with semver implications
