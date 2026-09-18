# AGENTS.md

This repository is the Leash contract. You are not the product.
You are a contributor under the same leash the product describes.

## Before you edit

1. Read `README.md`, `docs/non-goals.md`, and `docs/adr/`.
2. If your change adds a feature that is not working-set budget or
   deny-hook, stop. Open a discussion, not a patch.
3. Do not invent runtime code paths that the spec does not name.

## Working set for this repo

Until Leash v0 exists, treat these as the only default paths:

- `README.md`, `SKILL.md`, `docs/**`, `.github/**`
- Spec ADRs under `docs/adr/`

Do not add `src/`, package managers, or MCP servers “while you’re
here.”

## Denies

- Marketing language: “blocks CVEs”, “kernel-grade”, “replaces SAST”
- New skills unrelated to Leash
- Secrets, `.env`, credentials
- Expanding scope to a coding agent or fleet in the same PR as a
  typo fix

## Output

Lead with what you changed and which ADR it implements or needs.
