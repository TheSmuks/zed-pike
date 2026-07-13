# Architecture

> **Template / top-level overview.** This is the one-page map of `zed-pike`.
> The detailed, living architecture (component contracts, lifecycle rules,
> SOTA citations) lives in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md); the
> resource/perf contract lives in [`docs/perf.md`](docs/perf.md). Sections
> marked _TBD_ are open decisions to make as the structure firms up.

## 1. Purpose & constraints

Pike **8.0.1116** support for Zed, built to avoid the failure mode of prior
attempts: **high memory use and flaky uptime**. The non-negotiable constraints:

- **Resource budget is a contract** — see [`docs/perf.md`](docs/perf.md).
- **Syntax layer stands alone** — highlighting must work with the LSP absent.
- **Zed owns remoting** — the extension never runs `ssh` itself.
- **Clean-room** — no reuse of any prior Pike tooling.

## 2. System map

```
        Zed editor (host)
        ├── languages/pike/*.scm        Tree-sitter queries (highlight/outline/…)
        └── zed-pike-bridge  (wasm32-wasip2)
                 │  resolves + launches, then gets out of the way
                 ▼  stdio
            pike-lsp  (native binary)
                 ├── transport   stdio (default) · unix · forward · daemon · ssh
                 ├── analysis    tree-sitter-pike + position-stripped "AST shield"
                 └── resource_guard   RSS cap; exits when over budget
```

## 3. Components (summary)

| Component | Path | Responsibility |
|-----------|------|----------------|
| Syntax queries | `languages/pike/` | Highlighting, brackets, indents, outline. Ships standalone. |
| WASM bridge | `crates/zed-pike-bridge/` | Resolve `pike-lsp` (PATH → cache → GitHub release), return launch `Command`. No transport logic. |
| Language server | `crates/pike-lsp/` | The actual LSP: analysis, transports, resource guard. |
| Grammar | `TheSmuks/tree-sitter-pike` (pinned SHA) | Parser; SHA must match in `extension.toml` **and** `Cargo.toml`. |

## 4. Key decisions (current)

- **One stdio process per Zed session** is the default lifecycle. `daemon` /
  `forward` / `unix` are opt-in and must not outlive their sessions.
- **Grammar pinned by SHA**, bumped in two places together.
- **Spec-driven evolution** via `openspec/` for non-trivial changes.

## 5. Open decisions (_TBD — "decide the structure" phase_)

- [ ] Consolidate the two architecture docs (this file vs.
      `docs/ARCHITECTURE.md`) — pick one canonical home.
- [ ] Analysis engine roadmap: how far the position-stripped shield goes before
      a real incremental (Salsa-style) engine is warranted.
- [ ] Which LSP features are in the first shippable milestone (hover / symbols /
      diagnostics / completion) and their perf targets.
- [ ] Windows + Linux-SSH-remote support scope for v1.

## 6. References

- Detailed architecture — [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- Performance SLOs — [`docs/perf.md`](docs/perf.md)
- Contributor workflow — [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Agent operating manual — [`AGENTS.md`](AGENTS.md)
