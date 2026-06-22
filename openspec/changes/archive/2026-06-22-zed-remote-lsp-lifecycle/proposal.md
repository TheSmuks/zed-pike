## Why

The current LSP bridge misinterprets SSH support as a custom SSH tunnel initiated by the extension. The required behavior is Zed remote-worktree compatibility: when Zed is connected to an SSH host, the extension must let Zed launch `pike-lsp` in that remote worktree context.

The default daemon/forwarder path also risks leaving a shared daemon alive on the remote host after editor sessions end, hoarding resources. The safe default is one stdio language-server process per Zed LSP session; daemon mode must be explicit and bounded.

## What Changes

- Remove custom SSH-tunnel selection from the Zed bridge; SSH remoting is owned by Zed, not by this extension.
- Change the bridge default command to `pike-lsp stdio`, relying on `worktree.which("pike-lsp")` and Zed's remote worktree process launch semantics.
- Keep daemon/forwarder as an optional transport only for users who explicitly opt in, with a short idle TTL and no indefinite process lifetime.
- Tighten lifecycle docs and tests so a daemon process cannot be silently treated as the normal/default path.

## Capabilities

### New Capabilities

- `zed-remote-lifecycle`: Zed remote-worktree process-launch behavior and bounded Pike LSP process lifecycle.

### Modified Capabilities

- `zed-pike-bridge`: The bridge default transport changes from daemon/forwarder plus custom SSH mode to Zed-managed stdio process launch.
- `pike-lsp-daemon`: Daemon mode becomes explicitly opt-in and bounded; it must not be the bridge default.
- `pike-lsp-transport`: Custom SSH reverse-forwarding is not required for Zed remote SSH compatibility and must not be selected by default.

## Impact

- `crates/zed-pike-bridge/src/lib.rs`: default language-server command becomes `pike-lsp stdio`; custom SSH setting path is removed.
- `crates/pike-lsp/src/forward.rs`: auto-start daemon behavior is disabled or guarded behind explicit opt-in.
- `crates/pike-lsp/src/transport/ssh.rs`: custom SSH tunnel support is removed from the Zed path; any retained CLI support is non-default and documented as not needed for Zed remote SSH.
- `docs/perf.md` and `scripts/perf-smoke.sh`: lifecycle/perf evidence reflects stdio default plus optional bounded daemon checks.
