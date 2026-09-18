# ADR 0007: Arbor is a subprocess, not the working set

## Status

Accepted

## Context

v0 packs from `git diff --name-only HEAD` and labels `engine:
diff`. The product thesis is a budgeted working set from the call
graph. Arbor already is that graph. Rewriting arbor-core into this
repo would make Leash a second graph product.

Arbor v3 (`arbor-graph-cli` >= 3.0) does not ship
`arbor map --from-diff --budget-tokens`. `arbor map` is a PageRank
skeleton of the whole repository. Using it as the working set would
deny (or allow) based on “important files,” not the blast radius of
the change.

## Decision

1. Arbor is a PATH subprocess. Do not vendor arbor-core.
2. The radius commands are:
   - `arbor diff . --json` for changed files
   - `arbor file-graph <path> --json` when the diff JSON has no
     symbol list
   - `arbor callees <sym> --json` and `arbor callers <sym> --json`
     for one-hop neighbors
3. `arbor map` is not the working set.
4. `engine: arbor` only after a successful Arbor run. Missing
   binary, non-zero exit, timeout, or unparseable JSON fall back to
   the v0 diff engine and an audit line (`arbor_missing`,
   `arbor_failed`, `arbor_timeout`). Never claim a graph walk.
5. The PreToolUse hook does not spawn Arbor. Session start and
   expand may.

## Consequences

- Contributors may install Arbor; they do not have to. Diff-only
  remains a complete v0 deny.
- Arbor graph correctness stays Arbor’s bug tracker. Leash’s job is
  the label.
- A later GitHub Action (v2) reuses this engine, not a new map.
