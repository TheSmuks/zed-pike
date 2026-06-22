## MODIFIED Requirements

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
