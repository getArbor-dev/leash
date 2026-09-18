# ADR 0003: Block, do not suggest

## Status

Accepted

## Context

Cloudflare’s security-audit-skill produces findings. Superpowers
and Karpathy-style CLAUDE.md files produce discipline. Agents
ignore all of these under prompt injection or sloppiness.

## Decision

Leash’s security and radius behavior is a tool-level **deny**.
Chat text after a successful write is a bug, not a feature.

The default ruleset is small, deterministic, fixture-backed, and
honest about what it does not catch.

## Consequences

- Hook contract is a security surface
- “Blocks CVEs” is forbidden copy
- LLM-as-judge is not the deny oracle
