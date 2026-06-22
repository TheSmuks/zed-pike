# zed-pike-bridge Specification

## Purpose
TBD - created by archiving change pike-lsp-foundation. Update Purpose after archive.
## Requirements
### Requirement: The bridge registers the Pike LSP in Zed
The Zed extension SHALL declare a `[language_servers.pike-lsp]`
table in `extension.toml` whose `name` is `Pike LSP` and whose
`language` is `Pike`.

#### Scenario: Extension manifest lists the language server
- **WHEN** `extension.toml` is read by Zed
- **THEN** it contains a `[language_servers.pike-lsp]` table
  with the required keys.

### Requirement: The bridge selects a transport in priority order
In `language_server_command`, the bridge SHALL resolve the
command in this order:
1. A user-configured binary path from Zed settings
   (`pike_lsp.binary_path`).
2. `worktree.which("pike-lsp")` — the user's PATH wins.
3. Auto-download the latest GitHub release asset for the host
   platform and cache it in the extension work dir.
4. Return a clear error explaining how to install `pike-lsp` if
   none of the above succeed.

#### Scenario: User-installed binary wins
- **WHEN** a binary named `pike-lsp` exists on the worktree's
  PATH and the user has not set `pike_lsp.binary_path`
- **THEN** the bridge uses the user-installed binary, regardless
  of any cached auto-downloaded copy.

#### Scenario: Auto-download falls back to PATH
- **WHEN** no user-installed binary exists and a release asset
  is available for the host platform
- **THEN** the bridge downloads the asset, places it in the
  extension work dir, and uses it on subsequent starts.

### Requirement: The bridge compiles to wasm32-wasip2
The bridge crate SHALL compile to the `wasm32-wasip2` target
without warnings. `.cargo/config.toml` SHALL pin this target so
`cargo build` produces a `.wasm` artifact that Zed can load.

#### Scenario: Bridge build is target-clean
- **WHEN** `cargo build --target wasm32-wasip2` is run from the
  repo root
- **THEN** the build produces a `.wasm` artifact and emits no
  warnings.

