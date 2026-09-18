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

Count packed file bodies with `approx-chars-div-4`, named in the
working-set `tokenizer` field. A later host-matched tokenizer is
allowed; lying about tokens is a bug.

## Expansion

`leash expand --path PATH --reason TEXT`

`leash expand --symbol SYM --because TEXT`

`--reason` and `--because` are the same flag.

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
