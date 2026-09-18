# AGENTS.md

This repository is Leash. You are a contributor under the same
leash the product describes.

## Before you edit

1. Read `README.md`, `docs/non-goals.md`, and `docs/adr/`.
2. If your change adds a feature that is not working-set budget or
   deny-hook, stop. Open a discussion, not a patch.
3. Do not invent runtime code paths that the spec does not name.

## Working set for this repo

Until a session is running, treat these as the default paths:

- `README.md`, `SKILL.md`, `docs/**`, `.github/**`
- `src/**`, `tests/**`, `fixtures/**`, `contrib/**`, `Cargo.toml`
- Spec ADRs under `docs/adr/`

Do not add an MCP server or a second product “while you’re here.”

## Denies

- Marketing language: “blocks CVEs”, “kernel-grade”, “replaces SAST”
- New skills unrelated to Leash
- Secrets, `.env`, credentials
- Expanding scope to a coding agent or fleet in the same PR as a
  typo fix

## Output

Lead with what you changed and which ADR it implements or needs.
