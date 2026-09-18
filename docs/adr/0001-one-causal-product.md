# ADR 0001: One causal product

## Status

Accepted

## Context

The 2026 GitHub meta rewards kitchen-sink skill packs (ECC, gstack,
superpowers) and single-pain skills (caveman, i-have-adhd). We also
considered shipping five separate repos (sandbox, Arbor skill,
Action, context governor, fleet).

## Decision

Leash is one mechanism: a budgeted working set from the call graph
(diff fallback in v0). Token savings and patch denial are
consequences, not sibling products.

The public one-liner is:

> Budgeted context for coding agents. Out-of-radius patches don't land.

## Consequences

- README install is one command
- PRs that add a second product line are closed
- v2 Action and v3 fleet reuse the same engine; they are doors, not
  new theses
