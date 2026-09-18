# Why Leash exists

Coding agents in 2026 do three expensive, related things:

1. They **read too much**. Grep, embeddings, and “just in case” file
   dumps burn tokens. Headroom compresses the dump. Caveman rewrites
   the voice. Neither stops the dump.
2. They **write too wide**. Orthogonal edits (Karpathy’s complaint,
   later a 200k-star `CLAUDE.md`) are a prompt problem until a hook
   denies the tool call.
3. They **ship agent-shaped bugs**. Secrets in diffs, `eval`, SQL/XSS
   in the hunk they just wrote, a dependency with no lockfile. Audit
   skills report this after. The patch already landed.

Those are not three products. They are one failure: **the agent has
no hard working set.**

Leash is that working set, plus a deny on anything outside it.

## Why a skill, not a new agent

GitHub star growth in 2026 is dominated by 30-second installs that
plug into Claude Code, Codex, Cursor, OpenCode, and Gemini CLI.
Harnesses (OpenClaw, OpenCode, ECC) are capital-intensive and
occupied. A skill plus a PreToolUse hook is the distribution path
that matches how developers actually add behavior.

## Why not Graphify

Graphify already won “deterministic graph skill, no vector store.”
Leash does not compete on visualization or `/graphify`. Leash
**consumes** a radius (Arbor when present, git diff when not) and
**enforces** it. Queryable graph ≠ denied write.

## Why docs first

A deny product that ships code before the threat model will fail
open, then market itself as a sandbox. The contract is the first
artifact so v0 has something to be wrong against.
