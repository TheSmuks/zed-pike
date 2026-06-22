## Why

Zed users should be able to install the Pike extension on any supported local UI host and get LSP support both for local Windows worktrees and for Linux SSH remote worktrees. Today the bridge can fall through from remote `worktree.which("pike-lsp")` to an auto-download selected from the local UI platform, which can produce a host-local binary that cannot run in the worktree execution context; the manifest also does not register the Pike language server at all.

## What Changes

- Register `pike-lsp` in `extension.toml` so Zed actually asks the WASM bridge for a Pike language server command.
- Make bridge asset selection target the worktree execution platform, not blindly the local UI platform, when the worktree environment/root path indicates Linux.
- Fix downloaded binary paths so Windows local downloads use `pike-lsp.exe` while Linux/macOS downloads use `pike-lsp`.
- Add regression tests for Windows-local and any-host/Linux-remote asset/path selection.
- Update user-facing docs for Windows local usage plus SSH remote Linux setup and fallback behavior from any Zed host OS.
- Ensure the release path includes a Windows `pike-lsp` asset so Windows local fallback can install the language server.

## Capabilities

### New Capabilities

- `zed-remote-platform-resolution`: Detects the worktree execution platform for language-server binary resolution and covers local Windows worktrees plus Linux SSH remote worktrees opened from any Zed host OS.

### Modified Capabilities

- `zed-pike-bridge`: The bridge must be manifest-registered and must resolve auto-download assets for the worktree execution platform, including correct executable filenames per target OS.
- `zed-remote-lifecycle`: Remote SSH worktrees must remain Zed-owned stdio launches, with fallback downloads choosing a remote-runnable Linux binary instead of a host-local binary.
- `release-pipeline`: Published releases must include the Windows `pike-lsp` asset consumed by Windows local fallback.

## Impact

- `extension.toml`
- `crates/zed-pike-bridge/src/lib.rs`
- `README.md`, `docs/ARCHITECTURE.md`, `CHANGELOG.md`
- OpenSpec specs for bridge registration/platform resolution and remote lifecycle
