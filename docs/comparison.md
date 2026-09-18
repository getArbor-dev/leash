# Comparison

Star counts are approximate, observed 18 Sep 2026. They move.

| Project | Stars (then) | Job | Overlap with Leash | Gap we keep |
| --- | --- | --- | --- | --- |
| [Graphify](https://github.com/Graphify-Labs/graphify) | 119k | Graph skill, queryable map | Context from structure | They visualize/query. We deny writes. |
| [getArbor-dev/arbor](https://github.com/getArbor-dev/arbor) | 158 | Deterministic graph + PR radius | v1 engine | Arbor is the map. Leash is the leash. |
| [headroom](https://github.com/headroomlabs-ai/headroom) | 73k | Compress tool output / JSON | Tokens | They compress a dump. We avoid the dump. |
| [caveman](https://github.com/JuliusBrussee/caveman) | 106k | 65% tokens via diction | Tokens (meme) | Voice is not a working set. |
| [context-mode](https://github.com/mksglu/context-mode) | 23.5k | Clip tool output, session memory | Tokens | Clipping ≠ radius deny. |
| [cloudflare/security-audit-skill](https://github.com/cloudflare/security-audit-skill) | 11.5k | Multi-phase audit, machine-readable findings | Security | Audit after. We deny before apply. |
| [nolabs-ai/nono](https://github.com/nolabs-ai/nono) | 4.1k | Kernel least-privilege sandbox | Security | Syscalls vs intent. Complementary. |
| [affaan-m/ECC](https://github.com/affaan-m/ECC) | 261k | Harness pack: skills, memory, AgentShield | Kitchen sink | AgentShield scans agent config. We scan the hunk. |
| [obra/superpowers](https://github.com/obra/superpowers) | 288k | Methodology skill pack | Skills distribution | Process, not a deny hook. |
| [alibaba/open-code-review](https://github.com/alibaba/open-code-review) | 36k | Deterministic + LLM review | Review | PR review vs pre-apply hook. v2 Action may sit beside it. |

## Positioning sentence

Leash is the seatbelt: a hard working set and a deny. It is not the
car (the coding agent), not the map (Arbor/Graphify), not the crash
cage (nono), and not the post-crash report (Cloudflare audit).
