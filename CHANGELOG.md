# Changelog

All notable changes to `zed-pike` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Each release section lists changes by their conventional-commit type:
`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Unreleased]

### Changed
- Adopted Keep-a-Changelog format and Conventional Commits.
- Adopted the conventional-branches workflow documented in `CONTRIBUTING.md`
  (branch names of the form `type/<scope>-<short-topic>`; protected `main`).

### Added
- `CHANGELOG.md` — single source of truth for user-visible change history.
- `docs/ARCHITECTURE.md` — single source of truth for system architecture.
- `CONTRIBUTING.md` — branching, commit, and review conventions.
- `commitlint.config.js` — Conventional Commits enforcement in CI.
- `.commitlintrc.json` (legacy alias) — same configuration.
- `scripts/check-branch-name.sh` — pre-push guard.
- `.husky/` — local commit-msg hook invoking `commitlint`.

## [0.0.1] - 2026-06-22

First releaseable cut of the Zed extension. Source data was lifted from the
OpenSpec change archives at `openspec/changes/archive/2026-06-22-*/`. The
extension does not include any prior Pike LSP work and does not link to any
previously-released Pike language server.

### Added
- Zed extension manifest (`extension.toml`) pinning the maintained
  `TheSmuks/tree-sitter-pike` grammar at commit
  `adacb8165dc9c7db9ca2f8d15fcb73b3c7ea8980` (Pike 8.0.1116).
- `languages/pike/` config and Tree-sitter query files: `config.toml`,
  `highlights.scm`, `brackets.scm`, `indents.scm`, `outline.scm`.
- `fixtures/syntax/` exercising basic Pike, preprocessor directives, and a
  Roxen-style component.
- `crates/pike-lsp/` — a fresh, from-scratch Pike language server in Rust
  (LSP 3.17 over JSON-RPC 2.0). Supports `stdio`, `unix`, `forward`, `daemon`,
  and (non-default) `ssh` transports. Diagnostics cover tree-sitter parse
  errors, unresolved `#include`, and unknown preprocessor directives sourced
  from `pikelang/Pike/refdoc/preprocessor.xml`.
- `crates/zed-pike-bridge/` — Zed WASM bridge (`wasm32-wasip2`) that resolves
  the `pike-lsp` binary via `worktree.which`, a cached auto-download, or a
  fresh auto-download from the latest `TheSmuks/zed-pike` GitHub release.
- A per-process RSS resource guard configurable via `--max-rss-mb` and
  `PIKE_LSP_MAX_RSS_MB` (set to `0` to disable; default 256 MiB).
- A Criterion benchmark harness and a `scripts/perf-smoke.sh` script that
  exercises the stdio lifecycle, the existing-socket forwarder, and the
  resource guard.
- A GitHub Actions `ci` workflow running `cargo fmt`, `cargo clippy`,
  `cargo test`, the host `pike-lsp` release build, and the WASM bridge
  release build.

### Changed
- The bridge default transport is `pike-lsp stdio`. Zed remote SSH is
  handled by Zed's worktree process launch; the extension does not own an
  SSH tunnel.
- The `forward` subcommand fails clearly when its target socket is absent
  instead of auto-starting a daemon. Daemon mode is opt-in only.

### Fixed
- Bridge now downloads `pike-lsp` from `TheSmuks/zed-pike` releases (Linux
  asset suffix `x86_64-unknown-linux-gnu`).

[Unreleased]: https://github.com/TheSmuks/zed-pike/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/TheSmuks/zed-pike/releases/tag/v0.0.1