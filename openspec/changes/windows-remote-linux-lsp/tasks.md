## 1. Manifest and bridge behavior

- [x] 1.1 Register `pike-lsp` in `extension.toml` for the Pike language.
- [x] 1.2 Refactor bridge release asset and executable path selection into testable helpers.
- [x] 1.3 Infer Linux worktree execution platform from remote-style root path or shell environment before falling back to local `current_platform()`.
- [x] 1.4 Use OS-specific downloaded executable names, including `pike-lsp.exe` for Windows local downloads.

## 2. Regression tests

- [x] 2.1 Add unit coverage for any-host/Linux-remote fallback selecting the Linux asset and `pike-lsp` path.
- [x] 2.2 Add unit coverage for Windows local fallback selecting the Windows zip asset and `pike-lsp.exe` path.
- [x] 2.3 Add unit coverage that stdio command args remain unchanged.

## 3. Documentation and verification

- [x] 3.1 Update README, architecture, and changelog docs for local Windows plus Linux SSH remote behavior from any Zed host OS.
- [x] 3.2 Verify or create the GitHub release asset for Windows `pike-lsp` installation.
- [x] 3.3 Run formatting, tests, clippy, bridge wasm build, and OpenSpec validation.
- [x] 3.4 Mark tasks complete only after corresponding checks pass.
