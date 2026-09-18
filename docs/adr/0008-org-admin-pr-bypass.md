# ADR 0008: Org-admin pull-request bypass

## Status

Accepted. Amends [ADR 0005](0005-docs-first-protected-main.md).

## Context

GitHub repository rulesets have no per-user bypass actor. The
split we need is: [@Anandb71](https://github.com/Anandb71) (a
getArbor-dev organization admin) can squash-merge a pull request
with zero approvals; [@Akshay0047](https://github.com/Akshay0047)
(outside collaborator, Maintain, not an org member) cannot.

ADR 0005 said “no admin bypass.” That blocked Anand from landing
a PR when Akshay was not available. Direct push to `main` must
stay blocked for everyone, including org admins.

`bypass_mode: always` would let org admins push to `main` without
a PR. That is out.

## Decision

1. Ruleset **Protect main** (id `23650014`) lists one bypass
   actor: `OrganizationAdmin` with `bypass_mode: pull_request`.
   Snapshot: [`.github/ruleset-main.json`](../../.github/ruleset-main.json).
2. Organization admins may squash-merge a PR without approving
   reviews, code-owner review, or last-push approval.
3. Direct push, force-push, and deleting `main` stay denied for
   everyone, including organization admins.
4. Maintain collaborators, org members who are not admins, and
   everyone else still need one code-owner review from someone
   who is not the last pusher.
5. This is not a named-user exception. Any future getArbor-dev
   **Owner** or **Admin** gets the same merge bypass. Do not add
   Akshay to the org as Owner or Admin if the split must hold.

## Consequences

- Anand can merge without Akshay. Akshay cannot merge without a
  review.
- Adding another org admin expands the bypass set. That is the
  cost of GitHub having no User actor type.
- The “disable the ruleset if GitHub is on fire” procedure is no
  longer the only unblock for an org admin.
- Hostile PR authors still cannot merge unless they are org
  admins.
