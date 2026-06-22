# pike-lsp-transport

## ADDED Requirements

### Requirement: Pike LSP speaks LSP 3.17 over JSON-RPC 2.0
The Pike LSP server SHALL speak the Language Server Protocol version
3.17 over JSON-RPC 2.0, and SHALL correctly frame messages using
`Content-Length` headers followed by `\r\n\r\n` and a UTF-8 JSON body.

#### Scenario: Initialize handshake
- **WHEN** a client sends `initialize` with `processId`, `rootUri`,
  and `capabilities`
- **THEN** the server replies with a non-null `serverInfo` whose
  `name` is `pike-lsp` and a `capabilities` object that includes
  at least `textDocumentSync`, `hoverProvider`, and
  `definitionProvider`.

### Requirement: The transport layer is pluggable
The Pike LSP server SHALL provide three concrete transport
implementations selectable at startup:
1. `stdio` (default) — read JSON-RPC frames from stdin, write to
   stdout. The editor's own LSP client connects to this directly.
2. `unix` — listen on a Unix-domain socket given by a path argument
   or the `PIKE_LSP_SOCKET` environment variable. The server
   accepts multiple client connections on the same socket.
3. `ssh` — initiate an SSH session to a remote host, set up a
   reverse streamlocal forwarding, and bridge the local stdio to
   the forwarded socket. CLI surface is documented in the README.

#### Scenario: stdio transport starts on stdin/stdout
- **WHEN** the server is started as `pike-lsp` with no arguments
- **THEN** it reads LSP frames from stdin and writes LSP frames to
  stdout, with stderr reserved for diagnostic logs only.

#### Scenario: Unix-socket transport accepts multiple clients
- **WHEN** the server is started as `pike-lsp unix
  --socket /tmp/pike-lsp.sock` and a second client connects after
  the first
- **THEN** both clients receive an `initialize` response and the
  server keeps both sessions open concurrently.

#### Scenario: SSH transport uses reverse streamlocal forwarding
- **WHEN** the server is started as `pike-lsp ssh --host
  build@pikelang.org --remote-socket /run/pike-lsp.sock`
- **THEN** it opens an SSH session, requests reverse streamlocal
  forwarding of `/run/pike-lsp.sock` on the remote, waits for the
  remote-side `pike-lsp unix` to listen, and bridges stdio to the
  forwarded socket.

### Requirement: Forwarder mode is a thin proxy
The `pike-lsp` binary SHALL support a `forward` subcommand that
takes a `--remote` argument pointing at a Unix-socket path, opens
the socket, and copies LSP frames in both directions between its
own stdio and the socket without parsing them.

#### Scenario: Forwarder proxy is frame-transparent
- **WHEN** a `forward --remote /tmp/pike-lsp.sock` instance
  receives a `textDocument/definition` request on stdin
- **THEN** the request bytes appear unmodified on the daemon side
  and the response bytes appear unmodified on the editor side.

## REMOVED Requirements

### Requirement: Plain TCP transport
**Reason**: Unix-domain sockets cover the local and SSH cases with
better security semantics and no port collisions. Adding a TCP
listener would expand the attack surface for no incremental
benefit.
**Migration**: If a future need arises for TCP, add it as a new
transport rather than reviving this requirement.
