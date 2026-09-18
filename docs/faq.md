# FAQ

**Is this Arbor?**
No. Arbor maps. Leash budgets and denies. v1 calls Arbor when
present.

**Can I use it without Arbor?**
Yes. Git-diff working set, labeled `engine: diff`. Callers and
callees are omitted until Arbor runs.

**Does `arbor map` become the working set?**
No. Map is a repo skeleton. Leash uses `arbor diff` and one-hop
callers/callees. See [ADR 0007](adr/0007-arbor-subprocess.md).

**Does this replace CodeQL?**
No.

**Does the skill block writes by itself?**
No. The hook blocks writes. The skill briefs the model.

**Why MIT?**
Same as Arbor. Forks should be easy. The hook is the product, not
the license maze.

**Why not a kernel sandbox?**
Already exists, already has CVEs, not a 30-second skill. See
[non-goals.md](non-goals.md).

**Why docs before code?**
Because a fail-open deny tool is worse than no tool. The contract
is the test plan.

**Who maintains this?**
[@Anandb71](https://github.com/Anandb71) and
[@Akshay0047](https://github.com/Akshay0047). See
[GOVERNANCE.md](../GOVERNANCE.md).
