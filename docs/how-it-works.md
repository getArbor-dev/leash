# How it works

```text
task + diff
    │
    ▼
radius engine          Arbor v1, git-diff fallback v0
    │
    ▼
working set           paths + reasons + token budget
    │
    ├─► pack context     agent may only read these files
    │
    └─► deny hook        Edit/Write/mutating Bash
            │
            ├─ path outside set     → deny RADIUS
            ├─ hunk matches ruleset → deny RULE
            └─ else                 → allow
```

```text
[GitHub Action / leash ci]
    │  git diff BASE...HEAD
    ▼
working set + ruleset on added lines
    │
    ├─ RULE match  → comment + fail the job
    └─ else        → comment + pass
```

Tokens fall because packing is small. Vulns in the “agent just
wrote a footgun in the hunk” class are denied because the hook sees
the patch. Everything else is someone else’s job.

## Session

1. Skill loads. Agent is told the hard rules.
2. Engine builds the working set, writes it to `.leash/session.json`.
3. Hook on every Edit/Write/mutating Bash: canonicalize path, check
   set, optionally scan hunk, return allow or deny with rule id.
4. Expand is an explicit call with a reason. Silent growth is a bug.
5. On a pull request, `leash ci --base REF` packs the same set from
   `REF...HEAD` and runs the same ruleset. The comment is the set,
   omitted paths, and denials. RADIUS is not a CI failure.

## Failure modes we accept

- Model ignores the skill when the hook is missing. UX must scream.
- Ruleset will not catch novel vulns. We do not advertise that it
  will.
- Diff fallback misses callers. We say “diff-only, no graph” in the
  working-set header so nobody thinks Arbor ran.
