# ADR 0006: Rust for the v0 hook

## Status

Accepted

## Context

ADR 0005 left the language open. The hook budget is milliseconds.
Arbor is Rust. A Python daemon is already a non-goal.

## Decision

v0 is a Rust crate named `leash` that ships one binary. The same
library decides for tests, the CLI, and later a GitHub Action.

No extra CLI framework. argv parsing stays in-tree so a PreToolUse
spawn does not pay for clap.

## Consequences

- Contributors need a Rust toolchain
- `npx skills add` still only installs the skill; the binary is a
  second step (`cargo install --git …` or a later release artifact)
- Hook latency stays on the Rust side of the trade
