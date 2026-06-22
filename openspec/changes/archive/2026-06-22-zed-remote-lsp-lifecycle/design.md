## Context

The previous LSP foundation change treated SSH support as an extension-owned SSH tunnel. That is the wrong model for Zed. Zed remote SSH opens a remote worktree and launches language-server commands in that remote worktree context. The extension should therefore return a normal language-server command (`pike-lsp stdio`) and let Zed decide whether that command is local or remote.

The previous default also preferred a shared daemon/forwarder path. A daemon can be useful for large workspaces, but making it the default means a process can outlive the Zed session on an SSH host. That violates the low-resource goal: the default must follow Zed's LSP session lifecycle and die when stdin closes.

## Goals / Non-Goals

**Goals:**
- Make Zed local and Zed SSH remote worktrees use the same default command: `pike-lsp stdio`.
- Ensure the default language-server process lifetime is owned by Zed and ends with the LSP session.
- Remove custom SSH tunnel selection from the bridge path.
- Keep daemon/forwarder mode possible only as an explicit, bounded opt-in.

**Non-Goals:**
- Building a custom SSH client, reverse tunnel, or libssh2 integration for the bridge.
- Making a shared daemon the default transport.
- Solving cross-editor daemon cache sharing in this corrective change.

## Decisions

1. **Zed remote SSH support is stdio in the remote worktree context.**
   - Decision: `language_server_command` returns `pike-lsp stdio` by default.
   - Rationale: Zed already owns remote process launch for SSH worktrees. The extension should not tunnel around Zed.
   - Alternative rejected: `pike-lsp ssh` plus reverse streamlocal forwarding. That creates duplicate remoting logic and does not match the user's requirement.

2. **Daemon/forwarder is opt-in, not default.**
   - Decision: the bridge must not select `forward` unless an explicit supported setting exists. With the current `zed_extension_api` surface, no stable settings accessor is used, so the bridge always defaults to stdio.
   - Rationale: default per-session stdio guarantees process cleanup when Zed closes stdin.
   - Alternative rejected: default auto-start daemon with idle timeout. Even with a TTL, it can remain alive unexpectedly on shared SSH hosts.

3. **Forwarder no longer auto-starts a daemon by default.**
   - Decision: `pike-lsp forward --remote <socket>` connects only to an existing socket and fails clearly if none exists.
   - Rationale: an explicit `daemon` command should be visible to the user or future setting; a proxy command should not silently spawn a detached process.
   - Alternative retained for future: add `forward --auto-start --idle-timeout <duration>` once the bridge has a stable explicit setting and tests prove bounded cleanup.

4. **Custom SSH helper is non-Zed-path code.**
   - Decision: remove bridge references to `pike-lsp ssh`. The CLI helper may be kept temporarily, but it is not part of Zed SSH remote support and must not be used by default.
   - Rationale: this prevents future maintainers from confusing Zed remote SSH with extension-managed tunnels.

## Risks / Trade-offs

- **Loss of shared cache by default** -> accepted. Correct lifecycle and remote-host resource behavior are more important for the initial Zed integration. We can reintroduce explicit daemon mode later.
- **Auto-download on remote worktrees may download on the remote host** -> acceptable because Zed worktree semantics decide the host context. User-installed `worktree.which("pike-lsp")` still wins.
- **No stable Zed settings read path in `zed_extension_api` 0.6** -> the first correction avoids pretending a setting exists. Future settings should be implemented only against a verified API.

## Migration Plan

1. Patch bridge default to `pike-lsp stdio` and remove fake SSH setting/read path.
2. Patch forwarder to fail if the socket is absent instead of spawning a daemon.
3. Update perf smoke to validate stdio lifecycle and existing-socket forwarder behavior.
4. Validate OpenSpec, Rust host build, wasm bridge build, and workspace tests.

Rollback: revert this change and restore daemon/forwarder default only if the user explicitly chooses shared daemon mode and accepts remote-host process lifetime trade-offs.
