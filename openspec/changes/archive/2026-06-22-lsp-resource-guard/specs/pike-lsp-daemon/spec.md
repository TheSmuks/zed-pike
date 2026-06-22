## MODIFIED Requirements

### Requirement: Idle auto-shutdown
The daemon SHALL automatically shut down after an idle period with no connected sessions. The default idle period is 60 seconds, configurable via `--idle-timeout <duration>`. The daemon SHALL also enforce the Pike LSP resource guard so explicit shared daemon mode cannot hoard memory indefinitely.

#### Scenario: Daemon exits when no clients are connected
- **WHEN** the last client disconnects from the daemon
- **THEN** the daemon process exits within `--idle-timeout` seconds and frees its memory.

#### Scenario: Runaway daemon self-terminates
- **WHEN** an explicit daemon process exceeds its configured RSS ceiling
- **THEN** the daemon writes the resource-guard diagnostic to stderr
- **AND** exits with code 137.
