## Context

Zed remote SSH keeps the UI on the local host but runs language servers, terminals, and tasks on the remote server. The extension's WASM bridge can be installed by Zed on any supported UI host, but the native `pike-lsp` executable must match where Zed will launch it. The bridge currently calls `worktree.which("pike-lsp")` first, which is correct because Zed scopes that lookup to the worktree. The failure path is the fallback: `zed::current_platform()` describes the local extension host, so any local UI host can select a host-local release asset for a Linux SSH worktree. The downloaded command is then not runnable in the Linux worktree. The current manifest also lacks `[language_servers.pike-lsp]`, so Zed has no manifest-level LSP registration.

## Goals / Non-Goals

**Goals:**

- Keep Zed's remote model intact: no extension-owned `ssh`, reverse forwarding, daemon, or local proxy by default.
- Register the Pike LSP in `extension.toml`.
- Choose auto-download assets for the worktree execution platform when the worktree clearly looks like Linux, regardless of whether the local Zed UI runs on Windows, macOS, or Linux.
- Preserve native Windows local support by using the `.zip` asset and `pike-lsp.exe` path for Windows worktrees.
- Preserve platform-neutral extension installation: the bridge remains `wasm32-wasip2`; only `pike-lsp` is OS-native.
- Cover the behavior with pure unit tests that do not require an actual Zed SSH session.

**Non-Goals:**

- Detect every possible remote OS. This change targets Linux remote worktrees and local Windows worktrees. Other local hosts keep their current `current_platform()` fallback unless the worktree clearly identifies Linux.
- Add daemon/forwarder configuration. The default remains stdio.
- Install `pike-lsp` on the remote host outside Zed's extension-managed download path.

## Decisions

1. **Infer target platform from worktree context before falling back to `current_platform()`.**
   - Rationale: `worktree.shell_env()` and `worktree.root_path()` are the only available API inputs that describe the worktree execution environment in `zed_extension_api` 0.6. A Linux remote worktree exposes Unix-style roots and shell variables such as `HOME=/home/...`, `SHELL=/bin/...`, and colon-separated `PATH`; a local Windows worktree exposes drive roots or Windows-specific variables such as `OS=Windows_NT` or `COMSPEC`.
   - Alternative considered: require users to preinstall `pike-lsp` remotely. Rejected because auto-download is part of the extension contract and should work for dev extensions.
   - Alternative considered: use extension-owned SSH. Rejected because it fights Zed's remote lifecycle and existing specs explicitly forbid it.

2. **Model release asset selection as pure helper functions.**
   - Rationale: The Zed `Worktree` resource cannot be instantiated in tests. Keeping platform inference, asset naming, downloaded binary naming, and archive type as pure helpers allows deterministic regression coverage.

3. **Use executable filename by target OS.**
   - Rationale: Windows release archives contain `pike-lsp.exe`; Unix archives contain `pike-lsp`. The old hard-coded `pike-lsp` path broke Windows local fallback even when the right `.zip` asset was selected.

4. **Register the language server in the manifest.**
   - Rationale: Zed requires a `[language_servers.<id>]` entry with `name` and `languages` to invoke extension language-server code for a language.

## Risks / Trade-offs

- **Heuristic platform detection may misclassify unusual local MSYS paths** → Keep Windows-specific env markers (`OS=Windows_NT`, `COMSPEC`, drive roots) as Windows and only infer Linux from clear Unix remote signals.
- **Remote macOS is not fully inferred** → Current remote target is Linux; unknown Unix defaults to Linux only when the root/env strongly indicates Linux. macOS can still work when `pike-lsp` is on remote PATH or when the local host/platform fallback matches the worktree.
- **Download location is extension-managed** → If Zed stores downloaded files locally but executes commands remotely for an SSH worktree, the user-installed remote `pike-lsp` path remains the most reliable path. This change ensures the fallback no longer selects a Windows binary for a Linux execution context.

## Migration Plan

- Existing users with `pike-lsp` on PATH keep that path unchanged.
- Existing downloaded Windows cache remains valid for local Windows worktrees and now resolves to `pike-lsp.exe`.
- Linux SSH remote worktrees opened from any local Zed host OS will first use remote PATH and otherwise request the Linux release asset.
