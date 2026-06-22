# pike-lsp-daemon Specification

## Purpose
TBD - created by archiving change pike-lsp-foundation. Update Purpose after archive.
## Requirements
### Requirement: A single daemon process serves multiple editor sessions
The Pike LSP server SHALL support a `daemon` subcommand that
listens on a Unix-domain socket and accepts N concurrent client
connections, each as a separate LSP session. The analysis cache
SHALL be shared across all connected sessions.

#### Scenario: Two editors share one daemon
- **WHEN** two clients connect to the same daemon socket within
  the same workspace
- **THEN** both clients receive `initialize` responses and the
  daemon's RSS is at most `single_session_rss + 40 MiB`.

### Requirement: Idle auto-shutdown
The daemon SHALL automatically shut down after an idle period
with no connected sessions. The default idle period is 60
seconds, configurable via `--idle-timeout <duration>`.

#### Scenario: Daemon exits when no clients are connected
- **WHEN** the last client disconnects from the daemon
- **THEN** the daemon process exits within `--idle-timeout`
  seconds and frees its memory.

### Requirement: Auto-start on first connection
MUST: a `pike-lsp` invocation that detects no daemon listening on
the configured socket MUST start one automatically and MUST wait
for it to be ready before forwarding its own stdio to it. This
mirrors gopls's `-remote=auto` behavior.

#### Scenario: First invocation auto-starts the daemon
- **WHEN** the editor starts `pike-lsp forward
  --remote=/tmp/pike-lsp.sock` and the socket does not exist
- **THEN** `pike-lsp` spawns `pike-lsp daemon
  --socket=/tmp/pike-lsp.sock`, waits for the socket to appear,
  and only then begins forwarding.

