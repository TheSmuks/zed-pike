## MODIFIED Requirements

### Requirement: The transport layer is pluggable
The Pike LSP server SHALL provide two required concrete transport implementations selectable at startup:
1. `stdio` (default) — read JSON-RPC frames from stdin, write to stdout. The editor's own LSP client connects to this directly. This is the required transport for Zed local and Zed SSH-remote worktrees.
2. `unix` — listen on a Unix-domain socket given by a path argument or the `PIKE_LSP_SOCKET` environment variable. The server accepts multiple client connections on the same socket.

Custom SSH tunneling is not required for Zed SSH remote compatibility and MUST NOT be selected by the Zed bridge by default. If a standalone CLI `ssh` helper is retained, it MUST be documented as unrelated to Zed's SSH remote-worktree support.

#### Scenario: stdio transport starts on stdin/stdout
- **WHEN** the server is started as `pike-lsp` with no arguments
- **THEN** it reads LSP frames from stdin and writes LSP frames to stdout, with stderr reserved for diagnostic logs only.

#### Scenario: Unix-socket transport accepts multiple clients
- **WHEN** the server is started as `pike-lsp unix --socket /tmp/pike-lsp.sock` and a second client connects after the first
- **THEN** both clients receive an `initialize` response and the server keeps both sessions open concurrently.

#### Scenario: Zed remote SSH does not require custom ssh transport
- **WHEN** Zed opens a Pike file in an SSH remote worktree
- **THEN** the bridge uses the stdio transport in that worktree context
- **AND** no custom SSH reverse streamlocal forwarding is required for correctness.
