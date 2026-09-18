# Governance

Leash is maintained by getArbor.

| Role | People |
| --- | --- |
| Maintainers | [@Anandb71](https://github.com/Anandb71), [@Akshay0047](https://github.com/Akshay0047) |
| Namespace | `getArbor-dev/leash` |
| License | MIT |

## Decisions

- Product shape (causal chain, non-goals, deny vs suggest) requires
  an ADR and both maintainers.
- Spec typos, comparison updates, and fixture docs: lazy consensus.
  Merge after one code-owner review if the other maintainer is silent
  for 48 hours on a working day.
- Security-sensitive hook/radius/ruleset changes: both maintainers,
  plus the process in [SECURITY.md](SECURITY.md).

## `main`

`main` is the only long-lived branch. It is protected by a repository
ruleset:

- pull request required
- 1 code-owner approval
- dismiss stale reviews
- last pusher cannot approve their own change
- conversation resolution required
- linear history
- no force push
- no branch deletion
- rules apply to administrators

There is no maintainer bypass actor. If GitHub is on fire, we
temporarily disable the ruleset in the UI, record why in the next PR,
and turn it back on.

## Releases

No runtime, no tags. When v0 ships, tags are `vMAJOR.MINOR.PATCH`,
signed if both maintainers have signing set up, changelog in
[CHANGELOG.md](CHANGELOG.md).

## Collaborators

Outside collaborators get **Maintain** on this repository, not org
owner. Org ownership stays with getArbor-dev admins.

## Conflict

If maintainers disagree on a non-goal, the existing ADR stands until
a replacement ADR is merged. Shipping a third product line in this
repo is not a compromise; it is a fork.
