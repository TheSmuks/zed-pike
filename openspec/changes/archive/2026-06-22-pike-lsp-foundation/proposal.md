## Why

The `zed-pike-syntax-mvp` change delivered Tree-sitter-backed syntax
highlighting for Pike 8.0.1116 in Zed. That is necessary but not
sufficient: serious Pike development needs semantic features (go to
definition, find references, hover, completion, diagnostics, refactor).
The mission for this work is to ship a Pike language server that is
**performant, very low on resource consumption, SSH-aware, and
intelligently designed**, taking architectural cues from SOTA
implementations (gopls daemon mode, rust-analyzer's Salsa incremental
engine, Zed's WASM extension host) rather than reinventing them.

This change lays the foundation: a fresh `pike-lsp` server, a
separation between a thin Zed-bridge extension and the server itself,
and an SSH-friendly transport. It does not yet implement the full
feature surface. That is reserved for follow-up changes. We do not
touch or build on prior Pike LSP attempts, per standing guidance.

## What Changes

- Add a new crate `crates/pike-lsp/` implementing a JSON-RPC 2.0 LSP
  server in Rust (the canonical LSP transport for the foreseeable
  future). Talk over stdin/stdout by default; the same binary can
  listen on a Unix-domain socket for the daemon/SSH case.
- Add a Zed-bridge extension in `src/lib.rs` (compiled to
  `wasm32-wasip2`) that decides how to start the server: local PATH
  binary, a daemon socket, or an SSH transport.
- Add an SSH transport module: the bridge runs `ssh -T -o
  BatchMode=yes -o ExitOnForwardFailure=yes` with a reverse Unix-socket
  forwarding (`-R streamlocal:...`) and connects the LSP stdio to the
  forwarded socket. The server itself stays a normal native binary on
  the remote end.
- Add a "daemon mode" modeled on gopls: a single persistent process
  serves N editor sessions; idle timeout auto-shuts down. Editor
  processes are thin forwarders; the cache is shared.
- Add an incremental computation layer modeled on rust-analyzer's
  Salsa: inputs and queries are tiered `durable` / `normal` / `volatile`;
  AST node positions are stripped so the AST can shield downstream
  queries from raw text changes; early-cutoff is enabled.
- Add a diagnostic surface for `#include` resolution and preprocessor
  errors sourced from the Pike 8.0.1116 directive list documented in
  `pikelang/Pike/refdoc/preprocessor.xml`.
- Add a CI workflow that benchmarks the server's RSS and p99 latency
  for a representative fixture, with a documented SLO.

## Capabilities

### New Capabilities

- `pike-lsp-transport`: a transport layer that supports stdio, local
  Unix-socket, and SSH reverse-forwarded Unix-socket. This is the
  smallest unit a host (Zed, VS Code, Neovim, Helix) interacts with.
- `pike-lsp-analysis`: an incremental, tiered analysis layer over
  the `TheSmuks/tree-sitter-pike` AST, with symbols, references,
  hover, completion data, and diagnostic sources.
- `pike-lsp-daemon`: a single-process daemon that hosts multiple
  editor sessions against one shared analysis cache, with idle
  timeout and graceful shutdown.
- `zed-pike-bridge`: the Rust/WASM Zed extension that owns the
  decision of which transport to use, and registers the
  `pike-lsp` language server in Zed.

### Modified Capabilities

None. The `REMOVED Requirements` note in `zed-pike-syntax`
("Auto-installation of a Pike language server") records the
intent and points at this change as the migration target; the
`zed-pike-syntax` capability itself does not change here.

## Impact

- New code: `crates/pike-lsp/`, `src/lib.rs` (Zed bridge),
  `Cargo.toml` at the repo root, `.cargo/config.toml` for
  `wasm32-wasip2` target settings, GitHub Actions workflow for
  the perf SLO.
- New dependency: `tower-lsp` or `lsp-server` crate, plus
  `tokio` for async I/O and `ssh2` (libssh2 binding) or direct
  use of the `ssh` CLI for the SSH transport.
- New optional dependency: `capnp` or `flatbuffers` if we end up
  sharing parsed artifacts across processes; default off.
- No prior Pike LSP repos are read, vendored, or referenced.
- SOTA references: gopls daemon mode
  (https://go.dev/gopls/daemon), rust-analyzer's Salsa durability
  (https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html),
  Zed WASM extension API
  (https://docs.rs/zed_extension_api).
- Performance SLO target: idle RSS under 80 MiB for a single
  editor session, p99 latency under 50 ms for hover/completion on
  a 10 kLOC fixture. These targets are subject to empirical
  validation during the change and may be tightened.
