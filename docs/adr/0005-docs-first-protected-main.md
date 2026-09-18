# ADR 0005: Docs-first, protected main

## Status

Accepted. Amended by [ADR 0008](0008-org-admin-pr-bypass.md):
org admins may squash-merge PRs without reviews; they still
cannot push directly to `main`.

## Context

Empty-runtime repos with a viral README become unmaintainable the
week they hit trending. Two maintainers, no CI yet, high chance of
a force-push “just this once.”

## Decision

1. Spec lands before runtime.
2. `main` is protected: PR, 1 code-owner review, dismiss stale,
   last-pusher cannot approve, linear history, no force push, no
   deleting `main`. Direct push has no admin bypass. PR merge
   bypass for org admins is [ADR 0008](0008-org-admin-pr-bypass.md).
3. CODEOWNERS is `@Anandb71` and `@Akshay0047`.
4. Akshay0047 is a collaborator with Maintain, not org owner.

## Consequences

- First commit is the only unprotected push
- After ruleset enable, even maintainers use PRs
- Docs bugs follow the same path as code bugs
