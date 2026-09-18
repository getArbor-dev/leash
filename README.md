# Leash

**Budgeted context for coding agents. Out-of-radius patches don't land.**

A 30-second skill that builds a hard working set from the call graph, refuses writes outside it, and cuts the token bill because the agent stops reading the rest of the repo.

```bash
npx skills add getArbor-dev/leash
cargo install --git https://github.com/getArbor-dev/leash --locked
leash install
```

That is the product. The skill is the contract. The binary is the
deny. There is no daemon to install in v0. If the binary is not on
`PATH`, every skill response starts with `leash: NOT ENFORCING`.

## What it does

One mechanism. Three consequences.

1. **Budgeted context** — the agent receives a ranked, token-capped working set (changed symbols → callers → callees → tests). Not grep. Not embeddings.
2. **Blocked patches** — Edit/Write/Bash that touch files outside that set, or hunks that match a small agent-failure ruleset, are denied. The agent does not get a suggestion. It gets a refusal.
3. **Lower token bills** — the bill drops because the working set is small, not because we rephrase the prompt like a caveman.

If a bullet cannot be traced to that mechanism, it does not ship.

## What it is not

- Not a coding agent.
- Not a kernel sandbox. [nono](https://github.com/nolabs-ai/nono) already owns syscalls.
- Not a knowledge-graph skill clone. [Graphify](https://github.com/Graphify-Labs/graphify) already owns `/graphify`.
- Not a token compressor. [headroom](https://github.com/headroomlabs-ai/headroom) already compresses tool output.
- Not an audit report the agent can ignore. [cloudflare/security-audit-skill](https://github.com/cloudflare/security-audit-skill) already audits. Leash **blocks**.
- Not a 200-skill harness. [ECC](https://github.com/affaan-m/ECC) already is that.

Leash refuses the patch. That is the gap.

## Status

**v1 runtime is in this repository.** The skill plus `leash hook` is
the product. Arbor is optional. There is still no daemon.

| Phase | Ships | Not yet |
| --- | --- | --- |
| Now | Skill, deny hook, Arbor radius when `arbor` is on PATH | Hosted anything |
| v2 | GitHub Action using the same binary | A new coding agent |
| v3 | Optional fleet mode (worktrees + per-worker leash) | OpenClaw clone |

## Docs

Start here, in order:

1. [Why this exists](docs/why.md)
2. [How it works](docs/how-it-works.md)
3. [Install (target UX)](docs/install.md)
4. [Skill contract](docs/skill-spec.md)
5. [Hook contract](docs/hooks.md)
6. [Context budget](docs/context-budget.md)
7. [Vuln gate](docs/vuln-gate.md)
8. [Threat model](docs/threat-model.md)
9. [Architecture](docs/architecture.md)
10. [Non-goals](docs/non-goals.md)
11. [Comparison](docs/comparison.md)
12. [FAQ](docs/faq.md)
13. [Roadmap](ROADMAP.md)
14. [ADRs](docs/adr/)

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). `main` is protected. Changes land through reviewed pull requests. Docs changes follow the same rules as code.

## Security

Do not file public issues for vulnerabilities. Read [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE) © 2026 getArbor

## Maintainers

- [@Anandb71](https://github.com/Anandb71)
- [@Akshay0047](https://github.com/Akshay0047)
