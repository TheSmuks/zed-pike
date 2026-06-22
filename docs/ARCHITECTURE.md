# zed-pike architecture

This document is the single source of truth for the architecture of the
`zed-pike` extension. It is intentionally short; details that change during
implementation live in OpenSpec change artifacts under
`openspec/changes/`. After a change is applied and archived, the relevant
decisions are folded back into this document.

## Goals

1. **Zed-native syntax highlighting** for Pike 8.0.1116 on day one, using a
   maintained Tree-sitter grammar and Tree-sitter-first Zed query files.
2. **Optional LSP integration** that follows Zed's own remote-worktree
   process-lifecycle model — never an extension-owned SSH tunnel.
3. **Low-resource, editor-honest language server** for Pike. SOTA-informed
   (gopls daemon mode, rust-analyzer's Salsa incrementality, Zed's WASM
   extension host), not reinvented.

## Components

```
┌──────────────────────────────────────────────────────────────────┐
│                          Zed editor                              │
│                                                                  │
│   ┌────────────────────────────┐    ┌──────────────────────────┐ │
│   │  languages/pike/*.scm      │    │  wasm32-wasip2 extension │ │
│   │  (Tree-sitter queries)     │    │  crates/                 │ │
│   │                            │    │  zed-pike-bridge/        │ │
│   │  highlights, brackets,     │    │                          │ │
│   │  indents, outline          │    │  language_server_        │ │
│   │                            │    │  command() → pike-lsp    │ │
│   └────────────────────────────┘    └────────────┬─────────────┘ │
│                                                  │               │
└──────────────────────────────────────────────────┼───────────────┘
                                                   │ stdio
                                                   ▼
                                       ┌─────────────────────┐
                                       │     pike-lsp        │
                                       │  (native binary)    │
                                       │                     │
                                       │  subcommands:       │
                                       │    stdio (default)  │
                                       │    unix             │
                                       │    forward          │
                                       │    daemon (opt-in)  │
                                       │    ssh (non-Zed)    │
                                       │                     │
                                       │  ├─ analysis        │
                                       │  │  (tree-sitter-pike│
                                       │  │   + position-     │
                                       │  │   stripped AST    │
                                       │  │   shield)         │
                                       │  └─ resource_guard  │
                                       │     (RSS cap, exits │
                                       │      over-limit)    │
                                       └─────────────────────┘
```

### `extension.toml`

Pinned by SHA, not branch. The `[grammars.pike]` table points at
`TheSmuks/tree-sitter-pike`. There is no `[language_servers.*]` table on
the syntax milestone; the bridge is a separate WASM crate that Zed loads
because the workspace includes a `Cargo.toml`.

### `languages/pike/`

Tree-sitter query files. `highlights.scm` is seeded from the upstream
grammar's `queries/highlights.scm` and verified against
`src/node-types.json`. `brackets.scm`, `indents.scm`, and `outline.scm`
are hand-rolled. Auto-close brackets use single-quoted TOML strings for
quote characters (see CONTRIBUTING.md for the trap).

### `crates/zed-pike-bridge/`

WASM extension (`wasm32-wasip2`, `cdylib`). Implements `Extension` and
returns a `Command { command, args: ["stdio"] }`. Resolution order:

1. `worktree.which("pike-lsp")` — Zed remote SSH makes this look in the
   remote worktree's PATH; the bridge never invokes `ssh` itself.
2. Cached auto-downloaded binary.
3. Fresh auto-download from the latest `TheSmuks/zed-pike` GitHub
   release, selected for the worktree execution platform. A Windows UI
   connected to a Linux SSH worktree selects the Linux asset; a local
   Windows worktree selects the Windows `.zip` and `pike-lsp.exe`.

The bridge must not select `forward` or `daemon` by default. Either is
only correct behind an explicit Zed extension setting.

### `crates/pike-lsp/`

Native language server, built from the same workspace. Subcommands:

| Subcommand | Purpose | Default? |
|------------|---------|----------|
| `stdio`    | LSP over stdin/stdout. | yes |
| `unix`     | N concurrent sessions on one Unix-domain socket. | no |
| `forward`  | Stdout/socket bridge; fails if target socket absent. | no |
| `daemon`   | Multi-session server with idle timeout. | no |
| `ssh`      | Bridge to a `ssh -R streamlocal:...` session. | no |

The transport layer uses `tower-lsp`'s `LspService`. The analysis layer
strips byte ranges from the tree-sitter tree so downstream queries are
insulated from raw text changes (the "AST shield"). A RSS resource guard
runs in `main` and exits the process when its memory budget is exceeded.

## Lifecycle: local vs. SSH remote

Zed owns remote remoting. When the user opens a remote SSH worktree, Zed
launches `language_server_command`'s `command` inside that remote
worktree. The bridge therefore returns a plain `pike-lsp stdio` command.
When the local UI host is Windows and the worktree is Linux, fallback
auto-download uses the Linux release asset and the Unix `pike-lsp` filename,
not the Windows `.zip`/`.exe` pair. The bridge must not:

- invoke `ssh`,
- manage reverse `streamlocal:` forwards,
- select a local daemon when the worktree is remote.

Process lifetime: the default is one stdio LSP process per Zed LSP
session. The daemon subcommand exists for users who explicitly want a
shared cache, but it must not be the default and must not outlive its
last session indefinitely.

## SOTA references (cited, not re-derived)

- gopls daemon mode — https://go.dev/gopls/daemon
- rust-analyzer durable incrementality — https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html
- Zed WASM extension API — https://docs.rs/zed_extension_api
- Pike 8.0.1116 refdoc — https://github.com/pikelang/Pike/tree/v8.0.1116/refdoc
- Maintained grammar — https://github.com/TheSmuks/tree-sitter-pike

## Non-goals

- LSP work that duplicates a previously-released Pike language server.
  This repo ships a fresh implementation in `crates/pike-lsp/`.
- An extension-owned SSH transport. Zed already owns it.
- Reference to or vendoring of any prior Pike LSP attempt.
- VS Code-style TextMate grammar import. The Zed path is Tree-sitter.

## OpenSpec-driven evolution

Architectural changes start as OpenSpec changes under
`openspec/changes/`. Once a change is applied and archived, its `design.md`
becomes a citation in this file, and any lasting decisions are folded
into the appropriate section above. `docs/PLAN.md` retains the original
high-level milestone plan but is no longer authoritative once
architecture is captured here.