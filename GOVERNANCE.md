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
  plus the process in [SECURITY.md](SECURITY.md). GitHub does not
  enforce the second reviewer for organization admins
  ([ADR 0008](docs/adr/0008-org-admin-pr-bypass.md)). Maintain
  collaborators still cannot merge without a review.

## `main`

`main` is the only long-lived branch. It is protected by a repository
ruleset (id `23650014`, snapshot
[`.github/ruleset-main.json`](.github/ruleset-main.json)):

- pull request required
- 1 code-owner approval
- dismiss stale reviews
- last pusher cannot approve their own change
- conversation resolution required
- linear history
- no force push
- no branch deletion
- squash-merge only
- direct push, force-push, and deleting `main` apply to
  administrators

GitHub has no per-user ruleset bypass. Organization admins
(`@Anandb71` today) may squash-merge a pull request with no
approvals (`bypass_mode: pull_request`). Maintain collaborators
(`@Akshay0047`) cannot. See [ADR 0008](docs/adr/0008-org-admin-pr-bypass.md).

If GitHub is on fire and an org admin still cannot merge, we
temporarily disable the ruleset in the UI, record why in the next
PR, and turn it back on.

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
