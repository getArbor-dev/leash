# Security policy

## This is a security product

Leash denies tool calls. A bug in Leash is a bypass: an agent writes
outside the working set, a secret lands, or a deny is silently
skipped. Treat spec changes to hooks, rules, and radius the same way
you treat auth code.

## Supported versions

| Version | Supported |
| --- | --- |
| `main` (unreleased contract) | Yes |
| tagged releases | Yes, latest minor only |

There is no runtime on `main` yet. The skill contract, hook contract,
and threat model are still in scope: a misleading deny story in docs
is a vulnerability.

## What we will never claim

- “Leash stops CVEs.”
- “Leash replaces a sandbox.”
- “Leash replaces SAST / CodeQL / Semgrep.”
- “Prompt instructions are enforcement.”

Enforcement is a hook or binary that can return deny. Markdown is a
hint for the model. If the hook is absent, Leash must fail closed in
product UX (tell the user it is not enforcing) and fail open in the
model (the model cannot actually stop a tool call).

## Report a vulnerability

**Do not open a public issue.**

Use GitHub’s private vulnerability reporting on this repository
(Security tab → Report a vulnerability).

If that flow is unavailable, email `security@getarbor.dev` with:

- a short title
- affected surface (skill, hook, radius, ruleset, docs)
- repro, or why it is theoretical
- impact if an agent is hostile or prompt-injected

You should get an acknowledgement within 3 business days.

## Scope

In scope:

- Fail-open enforcement (hook missing, deny ignored, path canonicalization bugs)
- Working-set bypass (symlink, case fold, `..`, absolute vs repo-relative)
- Ruleset bypass that lets a blocked hunk apply
- Secret leakage through working-set packing or denial messages
- Supply-chain issues in the skill install path

Out of scope:

- The underlying model following bad instructions when no hook is installed (document it, don’t file it as a Leash bypass)
- Kernel escape (not our layer)
- Generic vuln classes in user repos Leash did not touch

## Safe harbor

Good-faith research against your own clone, without accessing other
users’ data or degrading GitHub, is welcome. Do not test denial
bypasses against third-party production agents you do not own.
