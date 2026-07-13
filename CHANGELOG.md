# Changelog

All notable changes to `zed-pike` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
and [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Each release section lists changes by their conventional-commit type:
`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Unreleased]

## [0.0.3] - 2026-06-23

### Fixed
- Extension activation on `.pike`, `.pmod`, and `.cmod` files. The
  published v0.0.1 release shipped an `extension.toml` that did not
  register the Pike LSP under `[language_servers.pike-lsp]`, so Zed
  installed the grammar and language config but had no language
  server entry to wire up. That left files with no language, no
  syntax highlighting, and no semantic features. The fix landed on
  `main` via commits #5 and #6; v0.0.3 ships them under a tagged
  version.
- The WASM bridge is exposed as a workspace-root package so Zed's
  extension builder can find it (the previous layout lived at
  `crates/zed-pike-bridge/`, which Zed's loader did not pick up).
- The bridge now resolves the correct `pike-lsp` asset for Windows
  and Linux remote worktrees, matching the host triples already
  shipped on v0.0.1.
- Tree-sitter query files in `languages/pike/` referenced node names
  that do not exist in the maintained
  `TheSmuks/tree-sitter-pike` grammar at the pinned commit.
  `highlights.scm` queried `(number_literal)`; the real node names
  are `integer_literal` and `float_literal`. `indents.scm` queried
  `(compound_statement)` and `(parameter_list)`; the real node names
  are `block` and `parameters`. The stale queries made Zed reject
  each file at extension load with `Query error: Invalid node type`,
  which silently dropped all syntax highlighting and indent rules.

## [0.0.2] - 2026-06-22

### Added
- `pike-lsp` is now registered in `extension.toml` so Zed invokes the
  WASM bridge for Pike language-server sessions.

### Fixed
- Linux SSH remote fallback now selects the Linux `pike-lsp` release
  asset and Unix executable path from any local Zed UI host instead of
  trying to use a host-local binary in the remote worktree.
- Local Windows fallback now resolves the downloaded binary as
  `pike-lsp.exe`, matching the Windows release archive.
- Latest release verification now checks that a Windows `pike-lsp` release
  asset exists for users installing from Windows.

## [0.0.2] - 2026-06-22

### Added
- Windows host build in CI: the `pike-lsp-x86_64-pc-windows-msvc`
  artifact is now produced on every push and pull request.
- New `.github/workflows/release.yml` triggered by tag push (`v*`)
  publishes a GitHub release with three artifacts:
  `pike-lsp-<version>-x86_64-unknown-linux-gnu.tar.gz`,
  `pike-lsp-<version>-x86_64-pc-windows-msvc.zip`, and
  `zed-pike-bridge-<version>.wasm`. SHA-256 sums for every asset are
  embedded in the release notes.
- The release notes are generated from `out/SHA256SUMS` so a downstream
  consumer can verify what they downloaded without trusting the
  GitHub UI.

### Changed
- Bumped `extension.toml` and workspace `version` from `0.0.1` to
  `0.0.2`.

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

[Unreleased]: https://github.com/TheSmuks/zed-pike/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/TheSmuks/zed-pike/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/TheSmuks/zed-pike/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/TheSmuks/zed-pike/releases/tag/v0.0.1