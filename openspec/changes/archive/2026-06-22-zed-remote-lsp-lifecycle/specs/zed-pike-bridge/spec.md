## MODIFIED Requirements

### Requirement: The bridge selects a transport in priority order
In `language_server_command`, the bridge SHALL resolve the server binary in this order:
1. A user-configured binary path from Zed settings (`pike_lsp.binary_path`) when the Zed API exposes that setting.
2. `worktree.which("pike-lsp")` — the worktree PATH wins. In a Zed SSH remote worktree this lookup is remote-worktree scoped.
3. Auto-download the latest GitHub release asset for the host platform and cache it in the extension work dir.
4. Return a clear error explaining how to install `pike-lsp` if none of the above succeed.

After resolving the binary, the bridge SHALL default to `pike-lsp stdio`. It SHALL NOT default to `pike-lsp forward`, `pike-lsp ssh`, the system `ssh` command, or any reverse streamlocal forwarding path. Daemon/forwarder mode MAY be added behind an explicit user setting, but it MUST remain opt-in and bounded by daemon idle shutdown.

#### Scenario: User-installed binary wins
- **WHEN** a binary named `pike-lsp` exists on the worktree's PATH and the user has not set `pike_lsp.binary_path`
- **THEN** the bridge uses the user-installed binary, regardless of any cached auto-downloaded copy.

#### Scenario: Auto-download falls back to PATH
- **WHEN** no user-installed binary exists and a release asset is available for the host platform
- **THEN** the bridge downloads the asset, places it in the extension work dir, and uses it on subsequent starts.

#### Scenario: Default command is stdio
- **WHEN** `language_server_command` is called without an explicit daemon opt-in setting
- **THEN** the returned command is `pike-lsp stdio`
- **AND** the command args do not include `forward`, `daemon`, `ssh`, `--remote`, `--remote-socket`, or `--local-socket`.
