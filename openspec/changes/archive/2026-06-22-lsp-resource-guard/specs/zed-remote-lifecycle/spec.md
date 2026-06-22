## MODIFIED Requirements

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
