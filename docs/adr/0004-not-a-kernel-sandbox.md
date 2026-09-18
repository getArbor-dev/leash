# ADR 0004: Not a kernel sandbox

## Status

Accepted

## Context

nono, E2B, NemoClaw, and Microsoft’s governance toolkit occupy
syscall / microVM / policy-passport layers. Building that in a
30-second skill is a year of CVEs (nono already had CVE-2026-47128).

## Decision

Leash operates at **intent**: repo-relative paths, hunks, and a
token budget. Users who need kernel isolation should run nono
*and* Leash.

## Consequences

- No Landlock/Seatbelt/seccomp in this repo’s v0–v2
- Threat model lists kernel escape as out of scope
- Complementary, not competitive, with nono
