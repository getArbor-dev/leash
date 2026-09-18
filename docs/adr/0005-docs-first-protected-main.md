# ADR 0005: Docs-first, protected main

## Status

Accepted

## Context

Empty-runtime repos with a viral README become unmaintainable the
week they hit trending. Two maintainers, no CI yet, high chance of
a force-push “just this once.”

## Decision

1. Spec lands before runtime.
2. `main` is protected with no admin bypass: PR, 1 code-owner
   review, dismiss stale, last-pusher cannot approve, linear
   history, no force push, no deleting `main`.
3. CODEOWNERS is `@Anandb71` and `@Akshay0047`.
4. Akshay0047 is a collaborator with Maintain, not org owner.

## Consequences

- First commit is the only unprotected push
- After ruleset enable, even maintainers use PRs
- Docs bugs follow the same path as code bugs
