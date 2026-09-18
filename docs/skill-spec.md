# Skill contract

Canonical file: [`/SKILL.md`](../SKILL.md).

This document is the test plan for that file. If they drift, this
doc is wrong or the skill is wrong; fix in one PR.

## Frontmatter

| Field | Rule |
| --- | --- |
| `name` | `leash` |
| `description` | Must mention working set, deny, and token budget. Must not say CVE or sandbox. |
| `compatibility` | Harnesses we actually hook. No fiction. |
| `metadata.status` | `v0` with a semver; was `contract` before the binary |

## Body obligations

The skill must tell the model:

1. Markdown is not enforcement.
2. Working-set schema (paths, reasons, budget).
3. When to activate.
4. How to expand the set.
5. How to report denials (rule id, path, action).

It must not:

- Dump a 200-line style guide
- Instruct the model to “be careful with security” as a substitute
  for the ruleset
- Reference a daemon

## Activation

Preferred: user or harness invokes `leash` at session start.
Acceptable: skill description matches “about to edit” / “token
budget” queries so the harness auto-loads it.

## Versioning

Edits to `SKILL.md` that change deny semantics are breaking, even
before v0. Call them out in `CHANGELOG.md`.
