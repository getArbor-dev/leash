# Hook contract

The hook is the product. The skill is the briefing.

## Event

PreToolUse, before:

- `Edit`
- `Write`
- `Bash` when the command mutates (detected by a conservative
  prefix list: `rm`, `mv`, `git commit`, `git push`, `gh`,
  `npm publish`, `cargo publish`, redirects to files outside the
  set). Allowlisted read-only Bash does not hit the ruleset.

If the harness cannot distinguish Bash mutation, deny mutating
patterns and allow the rest only when every path argument is in
the set.

## Decision

```text
allow | deny
rule: RADIUS | RULE | HOOK_MISSING | PATH_UNCANONICAL
path: repo-relative POSIX after symlink resolve
message: one line, no secret values
```

Deny must be a real tool denial, not a chat message after the
write.

## Path canonicalization (non-negotiable)

Before set membership:

1. Reject NUL and control bytes
2. Resolve `.` and `..`
3. Resolve symlinks relative to repo root
4. Compare case-folded on case-insensitive volumes
5. Reject paths that escape the repo

A symlink in-set pointing out-of-set is a deny (`PATH_UNCANONICAL`
or `RADIUS`). Missing this is a bypass. See
[threat-model.md](threat-model.md).

## Hook missing

If Leash cannot register a hook, every skill response in that
session starts with `leash: NOT ENFORCING`. No quiet degradation.

## Idempotency

The same tool payload must get the same decision. No hidden
state except the working set file.

## Performance budget

Decision p50 < 50ms on a 2k-path working set without ruleset
scan; ruleset scan p50 < 150ms on a 500-line hunk. If we miss
this, we shrink the ruleset. We do not skip the radius check.
