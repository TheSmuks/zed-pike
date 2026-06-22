## Context

The `zed-pike-syntax-mvp` change shipped Tree-sitter syntax
highlighting for Pike 8.0.1116 in Zed, using the maintained
`TheSmuks/tree-sitter-pike` grammar (commit
`adacb8165dc9c7db9ca2f8d15fcb73b3c7ea8980`) and the upstream
`refdoc/` folder as the canonical reference. That is the floor; it is
not the destination. The user has stated the explicit mission for the
language server work:

> performant, very low resource consumption, SSH-aware, and
> intelligently designed; use SOTA LSP implementations as reference
> when possible.

SOTA references that materially inform this design:
- **gopls daemon mode** (https://go.dev/gopls/daemon) — a single
  persistent process serves N editor sessions over a Unix-domain
  socket, with a thin forwarder process per editor. The shared
  analysis cache is the entire reason memory stays bounded. gopls
  itself reports the motivation: "Many separate editor processes →
  many caches → high resource consumption." We adopt the same shape
  for the Pike server.
- **rust-analyzer's Salsa + durable incrementality**
  (https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html,
  https://github.com/salsa-rs/salsa) — incremental, on-demand
  computation with three durability tiers (`durable`, `normal`,
  `volatile`); AST node positions are stripped so the AST acts as a
  "shield" between volatile source text and downstream queries;
  early-cutoff means that re-derivation of an unchanged value is
  avoided. This is the right shape for Pike as well: stdlib is
  `durable`, third-party `.pmod` modules are `normal`, the
  user's project is `volatile`.
- **Zed WASM extension API**
  (https://docs.rs/zed_extension_api) — extensions that need a
  language server run their bridge as a Rust crate compiled to
  `wasm32-wasip2`. The bridge owns the decision of how to launch
  the server; the server itself is a normal native process.

Constraints:
- Pike is 8.0.1116 (release tag
  https://github.com/pikelang/Pike/releases/tag/v8.0.1116, source
  commit `5d216a06d86bf36ec321fff3f82dfe80fd055194`); the
  authoritative language reference is `pikelang/Pike/refdoc/`.
- Prior Pike LSP attempts are out of scope; we do not read,
  vendor, or depend on them.
- Performance and resource targets (subject to empirical
  validation): idle RSS under 80 MiB per single editor session,
  p99 latency under 50 ms for hover/completion on a 10 kLOC
  fixture, p99 latency under 250 ms for workspace symbols over
  the same fixture.
- The Zed extension bridge must compile to `wasm32-wasip2` and
  use only the `zed_extension_api` surface.

## Goals / Non-Goals

**Goals:**
- A single Rust crate `pike-lsp` that talks LSP 3.17 over JSON-RPC
  2.0, with an architecture that supports stdio, local Unix-socket,
  and SSH reverse-forwarded Unix-socket transports.
- A daemon mode (modeled on gopls): one process, many sessions,
  shared analysis cache, idle auto-shutdown.
- An incremental analysis layer (modeled on Salsa) with three
  durability tiers and a position-stripped AST shield.
- A Zed-bridge extension that picks the right transport
  automatically and starts the server.
- A benchmark and CI perf gate against the documented SLO.

**Non-Goals:**
- Implementing the full LSP feature surface. This change lays the
  foundation: transport, daemon, analysis framework, and bridge.
  Feature-specific work (hover, completion, references, rename,
  etc.) lives in follow-up changes.
- Replacing or contributing to `TheSmuks/tree-sitter-pike`. We
  consume the grammar as a pinned dependency.
- A Pike formatter. Separate concern.
- Debugger / DAP integration. Separate concern.
- A web frontend. The server is a normal LSP process; it has no UI.

## Decisions

- **Language:** Rust. Rationale: best-in-class LSP server ecosystem
  (`tower-lsp`, `lsp-server`), first-class async I/O via `tokio`,
  single static binary, predictable memory layout. Alternatives
  considered: Go (would mirror gopls, but the Pike ecosystem has
  no native Go tooling); TypeScript/Node (worse memory floor and
  cold-start); Pike itself (would have to implement an LSP
  server in Pike, which is the point of the project, but is
  out of scope here).
- **Transport:** JSON-RPC 2.0 over a framed byte stream. Three
  concrete transports: (a) stdio (default, no setup); (b) local
  Unix-domain socket (for the daemon case and for SSH reverse
  forwarding); (c) SSH reverse-forwarded streamlocal socket. We
  do **not** add a TCP transport; Unix sockets are sufficient and
  safer.
- **SSH implementation:** spawn `ssh -T -o BatchMode=yes -o
  ExitOnForwardFailure=yes` with `-R /remote/path:streamlocal`
  and wire its stdin/stdout to the LSP forwarder. Rationale:
  shell out to OpenSSH, do not bind libssh2 from the extension
  (avoids a heavyweight native dep in a WASM-constrained
  bridge). For the foundation, the bridge supports the stdio
  transport only and the SSH bridge is a follow-up.
- **Daemon mode:** a `pike-lsp daemon` subcommand that listens on
  a Unix socket, accepts N connections, spawns a `Session` per
  accepted connection, and shuts down after `--idle-timeout`
  (default 60 s) of zero sessions. The forwarder (a separate
  invocation of `pike-lsp` that the editor actually starts) is a
  thin proxy that copies LSP frames over the socket.
- **Analysis engine:** a Salsa-style query engine, with three
  durability tiers. We start by adopting `salsa` from
  `salsa-rs/salsa` (Rust crate). Inputs are file versions and
  source bytes; queries are `parse`, `ast`, `symbols`,
  `references`, `hover`, `completion`, `diagnostics`. The
  position-stripped AST is the key "shield" — strip byte ranges
  from the AST so adding a comment to a file does not invalidate
  downstream queries. The grammar is consumed via
  `TheSmuks/tree-sitter-pike` (pinned, MIT).
- **File watching:** rely on the editor to push `textDocument/...`
  change notifications. We do not spawn our own `inotify` watcher
  in the foundation; the editor is the source of truth for
  "version N of file X." Rationale: matches the gopls model and
  avoids duplicating work.
- **Stdlib durability:** Pike's stdlib lives under `pike_modules`
  in a real installation. We treat any module on the Pike module
  path as `durable`; anything in the worktree as `volatile`; and
  anything in `~/.pike` (third-party) as `normal`. The exact
  resolution is refined in the analysis task.
- **Bridge compile target:** `wasm32-wasip2`. `.cargo/config.toml`
  pins the target. The bridge does not bundle the LSP binary;
  it spawns a host process.

## Risks / Trade-offs

- [Pike is a dynamic language, semantic analysis is hard] →
  The foundation delivers an analysis framework; full type
  resolution is its own follow-up. We start with symbols,
  references-by-identifier, and preprocessor directives.
- [Salsa crate API churn] → pin a known-good version in
  `Cargo.toml`; gate any upgrade on the CI perf benchmark.
- [Spawning `ssh` from WASM is awkward] → for the foundation, the
  bridge does stdio only. The SSH transport is implemented in a
  later change; the design and risk here are documented so the
  follow-up has a clear target.
- [Memory floor in Rust is higher than in Go] → we mitigate with
  the daemon model so the floor is paid once per workspace, not
  once per editor session. The CI perf gate is the source of
  truth; we will tighten the SLO based on measurement.
- [JSON-RPC framing via manual `Content-Length` headers] → use
  `lsp-server` + `tower-lsp` rather than hand-roll framing.
- [WASM bridge cannot use `tokio` features that require
  threads] → keep the bridge crate `no_std`-friendly and use
  only the parts of `tokio` that work in the WASM sandbox.

## Migration Plan

- No migration; the syntax MVP continues to work because the
  bridge is opt-in and the manifest still binds the syntax
  grammar.
- Users opt into LSP by setting `enable_language_server: true`
  for Pike in their Zed settings.
- Rollback: remove the `[language_servers.pike-lsp]` entry from
  `extension.toml`; the syntax MVP keeps working.

## Open Questions

- Should the bridge itself own the daemon lifecycle (auto-start
  on first session, auto-stop on idle), or should it be a
  separate `pike-lsp daemon` invocation? Lean: bridge owns the
  lifecycle for ergonomics, matches gopls `-remote=auto`.
- Do we want a long-lived in-process incremental cache shared
  with the analysis layer, or a per-session one? Lean: shared,
  daemon-shaped.
- Is `tower-lsp` the right base, or do we want a hand-rolled
  loop to keep the binary smaller? Lean: `tower-lsp` first;
  profile before replacing.
- Should the SSH transport shell out to the system `ssh` (zero
  bridge deps) or bind `libssh2` (more control, larger bridge)?
  Lean: shell out; revisit only if there is a real reason.
