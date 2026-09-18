# Context budget

The working set is the only context Leash endorses.

## Inputs

- User task (one sentence)
- Git diff (required)
- Arbor radius when `arbor` is on `PATH` (v1)
- `leash.yml` budget and path globs

## Packing

Rank:

1. Changed paths
2. Direct callees of changed symbols
3. Direct callers
4. Tests that reference those symbols
5. Config/lockfiles if the diff touches manifests

Stop when `budget_tokens` would be exceeded. Remaining ranks are
listed as `omitted` with counts, not silently dropped.

Each included path has a `reason`. No reason, no include.

## Token accounting

Count packed file bodies with the same tokenizer the host agent
uses if we can detect it; otherwise a documented approximation
(e.g. cl100k) named in the working-set header. Lying about tokens
is a bug.

## Expansion

`leash expand <path-or-symbol> --because <reason>`

- Must cite a path or symbol already in the set, or the user
- Appends an audit line
- Fails if the new pack exceeds budget unless the user raises
  `budget_tokens`

## What we do not do

- Embeddings
- “Dump the repo, then compress” (that is headroom’s job)
- Caveman rewrites
- Silent 98% clipping of tool output (context-mode’s job)

We avoid the dump. We do not compress a dump we should not have
made.
