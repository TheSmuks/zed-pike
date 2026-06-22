## Why

The Pike LSP must not be able to run away on a Zed SSH remote host and hoard memory indefinitely. A defensive resource guard gives the editor and user a predictable failure mode if analysis or parsing consumption goes to the roof.

## What Changes

- Add a process-level RSS guard that monitors the `pike-lsp` process and terminates it when memory exceeds a configured limit.
- Make the guard enabled by default with a conservative ceiling, configurable via CLI and environment variable.
- Ensure Zed's default `pike-lsp stdio` lifecycle benefits from the guard without daemon mode.
- Document smoke/test evidence for stdio and explicit daemon modes.

## Capabilities

### New Capabilities

- `pike-lsp-resource-guard`: Process-level resource limit behavior for Pike LSP.

### Modified Capabilities

- `zed-remote-lifecycle`: Zed-owned stdio process lifecycle now includes a self-termination guard for runaway RSS.
- `pike-lsp-daemon`: Explicit daemon mode now includes the same RSS guard so optional shared mode cannot hoard memory indefinitely.

## Impact

- `crates/pike-lsp/src/cli.rs`: add global resource guard options.
- `crates/pike-lsp/src/main.rs`: start the guard before dispatching transports.
- `crates/pike-lsp/src/resource_guard.rs`: new RSS monitor implementation.
- `scripts/perf-smoke.sh`: add evidence that the guard option is accepted and defaults do not interfere with normal startup.
