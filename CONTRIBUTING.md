# Contributing

Leash is spec-and-code in the same PR. If the spec is wrong, the
code will be wrong on purpose. Change the spec in the same PR as
the code, or first.

## Rules that will not be debated in a drive-by PR

1. One causal product. Budgeted working set → cheaper tokens and
   denied out-of-radius / ruleset patches. No third product line.
2. Markdown does not enforce. Hooks deny.
3. `main` is protected. Every change is a pull request. Direct
   pushes are blocked, including for admins.
4. One approving review from a code owner who is not the last
   pusher. Stale reviews dismiss. Conversations must resolve.
5. Do not add a skill, agent, or MCP that is unrelated to the
   working set or the deny hook.

## Setup

Rust stable and git.

```bash
git clone https://github.com/getArbor-dev/leash.git
cd leash
cargo test --locked
```

## Branch, PR, merge

```bash
git checkout -b docs/short-topic
# or feat/, fix/, security/
```

- Rebase on `main`. Linear history is required.
- Squash-merge only. The squash message is the public history.
- Delete the branch on merge (repo default).
- Fill in the PR template. Link the ADR if you change a decision.

### Commit messages

Imperative, scoped, why not what:

```text
docs(threat-model): record symlink bypass as in-scope

Radius is path-based. Without canonicalization, Edit on a symlink
outside the set is a deny miss.
```

Do not use `git commit --no-verify` unless a hook is broken and the
PR says so.

## Docs PRs

Docs are the product right now. A docs PR must:

- Keep the README one-liner and the 30-second install block accurate
- Update comparison tables if you name a competitor
- Avoid “blocks CVEs” and “sandbox” language
- Add or update an ADR when you change a decision, not when you
  fix a typo

## Code PRs (when they exist)

- Tests for every deny path, including path canonicalization
- A fixture that would have shipped a vuln without Leash
- No new dependency without a lockfile change in the same PR

## Security

See [SECURITY.md](SECURITY.md). Public issues for vulns will be
converted or deleted.

## Code of conduct

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Maintainers: Anand
([@Anandb71](https://github.com/Anandb71)) and Akshay
([@Akshay0047](https://github.com/Akshay0047)).
