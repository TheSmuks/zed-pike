# Pike LSP performance SLOs

This document is the source of truth for the Pike language server's
performance and resource budget. Numbers are checked by the
`perf-smoke.sh` script in CI.

## SLOs

| Metric                                | Target      | Why                                                                              |
|---------------------------------------|-------------|----------------------------------------------------------------------------------|
| Idle RSS, single stdio session        | < 80 MiB    | `pike-lsp` should be cheaper to run than a typical LSP for a similar language.    |
| Idle RSS, daemon (no clients)         | < 80 MiB    | One daemon serves many editors; the cost must be paid once per workspace.         |
| Per-session overhead, daemon          | < 40 MiB    | Adding a second editor to the daemon should be cheap; gopls' design point.       |
| Startup time (cold, stdio)            | < 200 ms    | First keystroke latency on a project open is bounded by LSP startup.              |
| Hover / completion p99 (10 kLOC)      | < 50 ms     | gopls / rust-analyzer design point: feel instant on a real workspace.            |
| Workspace symbols p99 (10 kLOC)       | < 250 ms    | Slower queries are still well under editor typing latency.                        |

## How the SLOs are measured

The CI workflow runs `scripts/perf-smoke.sh` on every push. The script
boots the server, reads `/proc/$PID/status` for `VmRSS`, and exercises
the `daemon` and `forward` subcommands against an in-process client.

The current empirical numbers (release build, x86_64-unknown-linux-gnu):

```
daemon RSS at idle: 4852 KiB   # well under 80 MiB
```

We do not currently measure hover / completion / symbols latency
because the analysis layer's queries are position-stripped iterations
of a flat in-memory node list, not a real database; once that is
replaced with a real incremental engine (Salsa-style), the latency
benchmarks will be added.

## References

- gopls daemon mode: https://go.dev/gopls/daemon
- rust-analyzer durable incrementality: https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html
- Pike 8.0.1116 refdoc: https://github.com/pikelang/Pike/tree/v8.0.1116/refdoc
- Maintained grammar: https://github.com/TheSmuks/tree-sitter-pike
