## MODIFIED Requirements

### Requirement: The bridge registers the Pike LSP in Zed
The Zed extension SHALL declare a `[language_servers.pike-lsp]` table in `extension.toml` whose `name` is `Pike LSP` and whose `languages` list contains `Pike`.

#### Scenario: Extension manifest lists the language server
- **WHEN** `extension.toml` is read by Zed
- **THEN** it contains a `[language_servers.pike-lsp]` table
- **AND** that table has `name = "Pike LSP"`
- **AND** that table has `languages = ["Pike"]`.

### Requirement: The bridge selects a transport in priority order
In `language_server_command`, the bridge SHALL resolve the server binary in this order:
1. A user-configured binary path from Zed settings (`pike_lsp.binary_path`) when the Zed API exposes that setting.
2. `worktree.which("pike-lsp")` — the worktree PATH wins. In a Zed SSH remote worktree this lookup is remote-worktree scoped.
3. Auto-download the latest GitHub release asset for the worktree execution platform and cache it in the extension work dir. When the worktree is a Linux SSH remote opened from any local Zed host OS, the bridge SHALL select the Linux asset, not a host-local asset. When the worktree is a local Windows worktree, the bridge SHALL select the Windows asset and executable path.
4. Return a clear error explaining how to install `pike-lsp` if none of the above succeed.

After resolving the binary, the bridge SHALL default to `pike-lsp stdio`. It SHALL NOT default to `pike-lsp forward`, `pike-lsp ssh`, the system `ssh` command, or any reverse streamlocal forwarding path. Daemon/forwarder mode MAY be added behind an explicit user setting, but it MUST remain opt-in and bounded by daemon idle shutdown.

#### Scenario: User-installed binary wins
- **WHEN** a binary named `pike-lsp` exists on the worktree's PATH and the user has not set `pike_lsp.binary_path`
- **THEN** the bridge uses the user-installed binary, regardless of any cached auto-downloaded copy.

#### Scenario: Auto-download selects worktree platform
- **WHEN** no user-installed binary exists and a release asset is available for the worktree execution platform
- **THEN** the bridge downloads the asset for the worktree execution platform, places it in the extension work dir, and uses it on subsequent starts.

#### Scenario: Linux remote auto-download is host independent
- **WHEN** no user-installed binary exists
- **AND** the worktree execution platform is Linux
- **AND** the local Zed UI host platform is Windows, macOS, or Linux
- **THEN** the bridge downloads the Linux `.tar.gz` asset
- **AND** the resolved command path ends with `pike-lsp` without the Windows `.exe` suffix.

#### Scenario: Windows auto-download uses exe path
- **WHEN** no user-installed binary exists and the worktree execution platform is Windows
- **THEN** the bridge downloads the Windows `.zip` asset
- **AND** the resolved command path ends with `pike-lsp.exe`.

#### Scenario: Default command is stdio
- **WHEN** `language_server_command` is called without an explicit daemon opt-in setting
- **THEN** the returned command is `pike-lsp stdio`
- **AND** the command args do not include `forward`, `daemon`, `ssh`, `--remote`, `--remote-socket`, or `--local-socket`.

### Requirement: The bridge compiles to wasm32-wasip2
The bridge crate SHALL compile to the `wasm32-wasip2` target without warnings. `.cargo/config.toml` SHALL document that CI builds the bridge with `cargo build -p zed-pike-bridge --target wasm32-wasip2` so Zed can load the generated `.wasm` artifact.

#### Scenario: Bridge build is target-clean
- **WHEN** `cargo build -p zed-pike-bridge --target wasm32-wasip2` is run from the repo root
- **THEN** the build produces a `.wasm` artifact and emits no warnings.
