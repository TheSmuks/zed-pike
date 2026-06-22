## ADDED Requirements

### Requirement: Worktree execution platform is inferred for downloads
The bridge SHALL select auto-download release assets for the worktree execution platform when the worktree root path or shell environment clearly identifies a different execution platform than the local Zed UI host.

#### Scenario: Windows host opens Linux SSH worktree
- **WHEN** the local Zed UI host platform is Windows
- **AND** the worktree root path is Unix-style or the worktree shell environment indicates Linux
- **AND** `worktree.which("pike-lsp")` does not find a user-installed remote binary
- **THEN** the bridge selects the `x86_64-unknown-linux-gnu.tar.gz` release asset for auto-download
- **AND** the returned command path ends with `/pike-lsp`, not `/pike-lsp.exe`.

#### Scenario: Windows local worktree keeps Windows binary
- **WHEN** the local Zed UI host platform is Windows
- **AND** the worktree root path or shell environment indicates a Windows local worktree
- **AND** `worktree.which("pike-lsp")` does not find a user-installed binary
- **THEN** the bridge selects the `x86_64-pc-windows-msvc.zip` release asset for auto-download
- **AND** the returned command path ends with `/pike-lsp.exe`.
