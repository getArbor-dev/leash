# Install (frozen UX)

The README install block is a product decision. Changing it needs
an ADR.

```bash
npx skills add getArbor-dev/leash
cargo install --git https://github.com/getArbor-dev/leash --locked
leash install
```

`npx skills add` is the frozen 30-second path for the skill.
The binary is a second step until we ship a release artifact.
From a clone: `cargo install --path . --locked`.

If `leash` is not on `PATH`, the skill still loads and must print
`leash: NOT ENFORCING`. Dropping the deny to claim “works
everywhere” is a spec violation.

`leash install` writes `.claude/settings.json` only when that file
does not already exist. Merge by hand otherwise; the template lives
in `contrib/claude.settings.json`.

Expected result in ≤30 seconds on a machine that already has a
compatible agent:

- `SKILL.md` available as `/leash` or auto-invoked on session start
- Hook registered for PreToolUse on Edit, Write, and mutating Bash
- A one-line status: `leash: enforcing` or `leash: NOT ENFORCING`

## Compatibility target

| Harness | Skill | Hook |
| --- | --- | --- |
| Claude Code | required | required for v0 |
| Codex | required | required if the harness exposes a pre-tool deny |
| Cursor | required | required if hooks exist; otherwise fail-visible |
| OpenCode | required | same |
| Gemini CLI | best-effort | best-effort |

If a harness cannot deny, Leash may still pack context. It must
print `NOT ENFORCING`. Shipping “works everywhere” by dropping the
deny is a spec violation.

## Project file

Optional `leash.yml` at repo root (schema in
[skill-spec.md](skill-spec.md)). Missing file means defaults:

- budget 8,000 tokens of packed file bodies
- deny out-of-set writes
- default ruleset on
- expand allowed with a cited reason
- optional `include` / `exclude` posix globs

Arbor is optional. `cargo install arbor-graph-cli` puts `arbor` on
`PATH`. Without it, `leash session` still runs and labels
`engine: diff`.

## Uninstall

Remove the skill and the hook. Leave no daemon. v0 has no daemon.
