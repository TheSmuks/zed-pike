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
MUST: daemon auto-start MUST be opt-in and bounded. A `pike-lsp forward` invocation MUST NOT spawn a detached daemon unless the user explicitly requests daemon mode via a documented flag or setting. When daemon auto-start is enabled, the spawned daemon MUST use an idle timeout and MUST exit after the last client disconnects.

#### Scenario: Default forwarder does not leave a daemon behind
- **WHEN** `pike-lsp forward --remote=/tmp/pike-lsp.sock` is started and no socket exists
- **THEN** the command fails with a clear socket-not-found error by default
- **AND** it does not spawn a background daemon process.

#### Scenario: Explicit daemon auto-start is bounded
- **WHEN** the user starts a documented daemon opt-in path that auto-starts a daemon
- **THEN** the daemon is started with an idle timeout
- **AND** the daemon exits within that timeout after the last client disconnects.

