## ADDED Requirements

### Requirement: Zed remote worktrees launch Pike LSP in the worktree context
The extension SHALL rely on Zed's worktree process-launch semantics for SSH remoting. When Zed is connected to a remote SSH worktree, `language_server_command` SHALL return a command that Zed executes in that remote context, not a locally initiated SSH tunnel.

#### Scenario: Remote SSH worktree starts a remote stdio server
- **WHEN** Zed opens a Pike file from an SSH remote worktree
- **THEN** the bridge returns a `pike-lsp stdio` command resolved via the remote worktree context
- **AND** no `pike-lsp ssh`, `ssh`, reverse-forward, or local socket tunnel command is selected by default.

### Requirement: The bridge default lifecycle is one process per LSP session
The extension SHALL default to a single `pike-lsp stdio` process owned by the Zed LSP session lifecycle. The process MUST exit when Zed closes the session or closes stdin.

#### Scenario: Closing the Zed session releases resources
- **WHEN** Zed stops the Pike language server for a worktree
- **THEN** the default `pike-lsp stdio` process terminates with the session
- **AND** no shared daemon process remains alive unless daemon mode was explicitly enabled.
