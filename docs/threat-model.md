# Threat model

## Assets

- The user’s repository
- Secrets on disk and in env
- The integrity of the working set (if it lies, denies lie)
- The user’s token bill (integrity of budget accounting)

## Actors

| Actor | Goal |
| --- | --- |
| Helpful agent, sloppy | Orthogonal writes, huge reads |
| Prompt-injected agent | Exfil, wide writes, hide a hunk |
| Malicious skill/MCP on the same harness | Teach the model to ignore Leash |
| Hostile PR author | Social-engineer a Leash ruleset weaken |

## Trust boundary

**The hook process is trusted. The model is not.** The skill file
is untrusted input to the model. A compromised `SKILL.md` can
misbrief the model; it must not disable the hook.

Install path (`npx skills add`) is supply chain. Pin and review.
Snyk’s 2026 ToxicSkills work is assumed true: public skills can be
payloads. Treat this repo as a supply-chain artifact.

## Threats and mitigations

| Threat | Mitigation |
| --- | --- |
| Write via symlink / `..` / case fold | Canonicalize in hook; fixture |
| Model ignores skill | Fail-visible `NOT ENFORCING`; hook is the real gate |
| Hook not installed | Same |
| Ruleset bypass by splitting hunks | Radius still applies; ruleset is extra |
| Secret echoed in deny message | Never echo matching bytes |
| Working set silently grows | Expand is explicit + audit line |
| Maintainer force-push to neuter denies | Ruleset on `main`; org-admin bypass is PR-merge only, not force-push |
| Org-admin merge without review | Accepted residual ([ADR 0008](adr/0008-org-admin-pr-bypass.md)): org admins can land a PR without Akshay; Maintain cannot |
| PR lands a ruleset hunk | v2 Action runs `scan_hunk` on `BASE...HEAD` and fails the job |
| Dependency confusion on install | Package name not claimed until v0; document |

## Out of scope

- Kernel escape, LSM, microVMs (nono, E2B, NemoClaw)
- Prompt injection into the model that does not result in a tool
  call the hook sees
- Correctness of Arbor’s graph (tracked in arbor; Leash must label
  fallback vs graph)

## Residual risk

A determined injected agent with an allowed path can still write a
novel vuln that the five rules miss. Users who need more run
CodeQL/Semgrep/Cloudflare’s audit skill **in addition**. Leash is
the seatbelt, not the crash cage.
