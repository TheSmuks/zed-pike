## Context

The Zed bridge now uses per-session `pike-lsp stdio` by default, which fixes the daemon-hoarding risk for normal remote worktrees. A separate risk remains: a parser or analysis bug could still cause the single LSP process to consume excessive memory until Zed or the OS intervenes.

The resource guard should be simple, dependency-light, and active for every transport path. The immediate target is Linux SSH hosts, where `/proc/self/status` exposes `VmRSS` reliably. Unsupported platforms should fail open with a warning rather than breaking the editor.

## Goals / Non-Goals

**Goals:**
- Monitor current process RSS for stdio, unix, forward, ssh, and explicit daemon commands.
- Default to a 256 MiB RSS ceiling.
- Let users override via CLI (`--max-rss-mb`) or environment (`PIKE_LSP_MAX_RSS_MB`).
- Exit with code 137 and a clear diagnostic when the ceiling is exceeded.

**Non-Goals:**
- Implement cgroup limits or OS-level sandboxing.
- Kill child processes by process tree; the current default is a single process.
- CPU throttling in this change. RSS is the highest-impact remote-host hoarding risk.

## Decisions

1. **Process-local monitor task.**
   - Start a Tokio background task from `main` after CLI parsing and before transport dispatch.
   - The task samples RSS every two seconds.
   - On exceed, it prints to stderr and calls `std::process::exit(137)`.

2. **Configuration precedence.**
   - CLI `--max-rss-mb` wins.
   - Else `PIKE_LSP_MAX_RSS_MB` wins.
   - Else default is 256 MiB.
   - Value `0` disables the monitor.

3. **Linux-first measurement.**
   - Implement RSS by parsing `/proc/self/status` `VmRSS:` in KiB.
   - If the file or field is unavailable, log once and do not kill.
   - This covers Zed SSH remotes on Linux, which is the primary safety target.

4. **No new dependency.**
   - Avoid `sysinfo` for now; the guard can be implemented with std + tokio.
   - This keeps the server small and avoids extra platform-specific behavior.

## Risks / Trade-offs

- **False positive kills on very large projects** -> users can raise the ceiling with `--max-rss-mb` or disable with `0`.
- **Unsupported OS has no guard** -> warning is explicit; future changes can add macOS/Windows RSS sources if needed.
- **Exit code 137 resembles SIGKILL** -> intentional: it communicates resource exhaustion to wrappers and users.
