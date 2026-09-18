# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Releases will follow [SemVer](https://semver.org/) once v0 exists.

## [Unreleased]

### Changed

- Organization admins may squash-merge pull requests without
  reviews. Direct push to `main` stays denied. Maintain
  collaborators still need a review (ADR 0008).

### Added

- Docs-first repository: product contract, skill spec, hook spec,
  threat model, ADRs, GitHub templates, and maintainer rules.
- `SKILL.md` contract for a 30-second install path
  (`npx skills add getArbor-dev/leash`).
- v0 `leash` binary: path canonicalize, diff working set,
  LEASH-SEC-001..005, PreToolUse deny JSON, `leash install` for
  Claude Code.
- Fixture pack: radius, secrets, symlink, manifest.
