## MODIFIED Requirements

### Requirement: Zed remote worktrees launch Pike LSP in the worktree context
The extension SHALL rely on Zed's worktree process-launch semantics for SSH remoting. When Zed is connected to a remote SSH worktree, `language_server_command` SHALL return a command that Zed executes in that remote context, not a locally initiated SSH tunnel. If the bridge must auto-download `pike-lsp` for a Linux SSH remote worktree opened from any local Zed host OS, it SHALL select a Linux command path and Linux release asset rather than a host-local executable.

#### Scenario: Remote SSH worktree starts a remote stdio server
- **WHEN** Zed opens a Pike file from an SSH remote worktree
- **THEN** the bridge returns a `pike-lsp stdio` command resolved via the remote worktree context
- **AND** no `pike-lsp ssh`, `ssh`, reverse-forward, or local socket tunnel command is selected by default.

#### Scenario: Any host remote fallback is Linux runnable
- **WHEN** Zed on Windows, macOS, or Linux opens a Pike file from a Linux SSH remote worktree
- **AND** no remote `pike-lsp` is present on PATH
- **THEN** the bridge fallback selects the Linux release asset
- **AND** the command path uses `pike-lsp` without the Windows `.exe` suffix.

### Requirement: The bridge default lifecycle is one process per LSP session
The extension SHALL default to a single `pike-lsp stdio` process owned by the Zed LSP session lifecycle. The process MUST exit when Zed closes the session or closes stdin. The default stdio process SHALL also enforce the Pike LSP resource guard so runaway RSS on a local or SSH remote worktree terminates the process instead of hoarding resources.

#### Scenario: Closing the Zed session releases resources
- **WHEN** Zed stops the Pike language server for a worktree
- **THEN** the default `pike-lsp stdio` process terminates with the session
- **AND** no shared daemon process remains alive unless daemon mode was explicitly enabled.

#### Scenario: Runaway stdio server self-terminates
- **WHEN** the default `pike-lsp stdio` process exceeds its configured RSS ceiling
- **THEN** the process exits with the resource-guard diagnostic
- **AND** Zed no longer has a runaway Pike LSP process on the worktree host.
