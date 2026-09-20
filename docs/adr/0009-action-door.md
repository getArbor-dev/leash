# ADR 0009: GitHub Action is a door, not a new engine

## Status

Accepted

## Context

v1 packs a working set and denies on the hook. The same library
must decide in CI ([ADR 0001](0001-one-causal-product.md),
[ADR 0006](0006-rust-hook.md)). A PR checkout has a clean tree, so
`git diff HEAD` is empty. The Action cannot pretend Arbor walked
the graph: it does not spawn Arbor ([ADR 0007](0007-arbor-subprocess.md)).

## Decision

1. `leash ci --base REF` is the CI door. The range is
   `REF...HEAD` (merge-base). The pack is `engine::assemble` with
   that path list. The label is `engine: diff`. Audit includes
   `door=ci`.
2. Added lines come from `git diff -U0 REF...HEAD`. Each file is
   passed to `rules::scan_hunk` with every path in the range as
   `apply_paths` (so `LEASH-SEC-005` sees a sibling lockfile).
3. The CI gate is the ruleset, not RADIUS. A PR *is* the change.
   Radius stays a pre-apply hook concern.
4. Stdout is the working set plus denials, not a review essay.
   Exit `1` on any RULE deny, `2` on usage or git failure, `0`
   otherwise. The binary does not call GitHub.
5. `.github/workflows/leash.yml` builds this PR’s binary, runs
   `leash ci --base origin/<base>`, posts or updates one PR
   comment marked `<!-- leash -->`, then fails the job on exit 1.
6. Do not write `.leash/session.json` from `ci`.

## Consequences

- Other repos copy the workflow or wait for a release artifact.
  This ADR does not add a marketplace Action or a hosted service.
- CI without Arbor is labeled `diff`. That is honest.
- A fixture that adds a ruleset-shaped hunk to this repository
  fails the Action. Split trigger strings in tests.
