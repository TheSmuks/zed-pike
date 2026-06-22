## 1. Workspace setup

- [ ] 1.1 Create the Cargo workspace: top-level `Cargo.toml` with
  members `crates/pike-lsp` and (later) the Zed bridge, and a
  `.cargo/config.toml` pinning `wasm32-wasip2` as the build
  target for the bridge.
- [ ] 1.2 Add a workspace `rust-toolchain.toml` pinning the Rust
  version used for CI.
- [ ] 1.3 Add a `.gitignore` entry for `target/`, `Cargo.lock`
  for the bridge (keep it for the server), and `*.wasm` build
  artifacts.

## 2. pike-lsp-transport

- [ ] 2.1 Add `crates/pike-lsp/` with a `Cargo.toml` and a
  `src/main.rs` that parses the subcommand
  (`stdio | unix | ssh | forward | daemon`).
- [ ] 2.2 Implement the stdio transport: a `LspService` over
  `tokio::io::stdin` and `tokio::io::stdout`, with `Content-Length`
  framing using `lsp-server`/`tower-lsp`.
- [ ] 2.3 Implement the `unix` transport: a `tokio::net::UnixListener`
  that accepts N connections and spawns one LSP session per
  accepted stream.
- [ ] 2.4 Implement the `forward` subcommand: a thin proxy that
  copies LSP frames in both directions between stdio and a
  Unix socket without parsing.
- [ ] 2.5 Implement the `ssh` subcommand: spawn the system `ssh`
  with `-T -o BatchMode=yes -o ExitOnForwardFailure=yes -R
  <remote-socket>:streamlocal:<local-socket>`, then bridge
  stdio to the local socket.
- [ ] 2.6 Add an integration test that runs the server in `unix`
  mode, opens a `forward` proxy on its socket, and verifies
  that an `initialize` request and response survive the round
  trip unmodified.

## 3. pike-lsp-analysis

- [ ] 3.1 Add `salsa` (pinned) as a dependency and define the
  three durability tiers (`durable`, `normal`, `volatile`) on
  every query.
- [ ] 3.2 Implement the `parse` query that produces the raw
  tree-sitter tree and the `ast` query that produces a
  position-stripped "red-green" AST. Add a unit test that
  editing a comment does not invalidate downstream queries.
- [ ] 3.3 Implement the `symbols`, `references`, `hover`, and
  `completion` queries on top of the position-stripped AST.
- [ ] 3.4 Implement the `diagnostics` query for parse errors,
  unresolved `#include`, and unknown preprocessor directives,
  driven by the directive set from
  `pikelang/Pike/refdoc/preprocessor.xml`.
- [ ] 3.5 Wire the analysis layer into the LSP service: map
  `textDocument/definition`, `textDocument/references`,
  `textDocument/hover`, `textDocument/completion`, and
  `textDocument/publishDiagnostics` to the corresponding
  queries.

## 4. pike-lsp-daemon

- [ ] 4.1 Implement the `daemon` subcommand: a process that
  listens on a Unix socket, accepts N connections, hosts one
  LSP session per connection, and shares one `Analysis`
  instance across all sessions.
- [ ] 4.2 Add an idle-timeout auto-shutdown: when the connected
  session count drops to zero for `--idle-timeout` seconds,
  exit cleanly.
- [ ] 4.3 Make the `forward` subcommand auto-start the daemon:
  if the socket does not exist, spawn `pike-lsp daemon
  --socket=<path>` and wait for the socket to appear before
  forwarding.
- [ ] 4.4 Add a benchmark (Criterion) that measures RSS for one
  and two concurrent sessions on a 10 kLOC fixture. The
  per-session overhead target is under 40 MiB.

## 5. zed-pike-bridge

- [ ] 5.1 Add a workspace member `crates/zed-pike-bridge/` with
  `Cargo.toml`, `src/lib.rs`, and `zed-extension.toml` setting
  the `cdylib` crate type.
- [ ] 5.2 Implement `language_server_command` with the
  documented resolution order: user setting, `worktree.which`,
  auto-download via `zed::latest_github_release`, and a clear
  final error.
- [ ] 5.3 Update the repo-root `extension.toml` to add
  `[language_servers.pike-lsp]` with `name = "Pike LSP"` and
  `language = "Pike"`.
- [ ] 5.4 Confirm `cargo build --target wasm32-wasip2` succeeds
  with no warnings.

## 6. Performance and CI

- [ ] 6.1 Add a GitHub Actions workflow that runs the
  transport, analysis, and daemon tests on Linux.
- [ ] 6.2 Add a perf workflow that runs the RSS benchmark
  on a 10 kLOC fixture and fails if the SLO is regressed.
- [ ] 6.3 Document the SLO in `docs/perf.md` and link it from
  the README.

## 7. OpenSpec change

- [ ] 7.1 `proposal.md` describes the why, what changes, the
  four new capabilities, and the impact.
- [ ] 7.2 `specs/pike-lsp-transport/spec.md` defines
  stdio, unix, and ssh transport, plus the forwarder proxy.
- [ ] 7.3 `specs/pike-lsp-analysis/spec.md` defines
  incrementality, the AST shield, the Pike 8.0.1116 alignment,
  and the diagnostic surface.
- [ ] 7.4 `specs/pike-lsp-daemon/spec.md` defines the
  multi-session daemon, idle shutdown, and auto-start.
- [ ] 7.5 `specs/zed-pike-bridge/spec.md` defines the bridge's
  transport resolution order and the wasm32-wasip2 build.
- [ ] 7.6 `design.md` documents the SOTA references
  (gopls daemon mode, rust-analyzer Salsa durability, Zed
  WASM extension API) and the decisions/risks they informed.
- [ ] 7.7 `tasks.md` (this file) lists the implementable
  tasks grouped by area.
- [ ] 7.8 Run `openspec validate pike-lsp-foundation` and
  confirm it is clean.
