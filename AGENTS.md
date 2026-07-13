# AGENTS.md

Operating manual for AI agents (and humans acting like them) working in
`zed-pike`. Read this before touching code. It is deliberately opinionated:
this is the **third** attempt at Pike editor tooling by this author (twice for
VS Code, once for Zed), and the previous attempts all died the same death —
**high resource consumption and ~85% broken uptime**. Everything here exists to
not repeat that.

## What this project is

Pike **8.0.1116** language support for the Zed editor, in two layers:

1. **Syntax layer** — a Tree-sitter grammar (`TheSmuks/tree-sitter-pike`,
   pinned by SHA) plus Zed query files in `languages/pike/`. This is the
   shipping milestone and must always work on its own.
2. **LSP layer** — a fresh, native Pike language server (`crates/pike-lsp/`)
   wired into Zed by a thin WASM bridge (`crates/zed-pike-bridge/`). Optional,
   opt-in, and must never degrade the syntax layer.

## Prime directives (do not violate)

1. **Low resource, or it doesn't ship.** The perf/RSS budget in
   [`docs/perf.md`](docs/perf.md) is a contract, not an aspiration. Idle stdio
   RSS < 80 MiB; per-extra-session daemon overhead < 40 MiB. If a change moves
   these numbers, it must justify it and update `docs/perf.md` in the same PR.
2. **Editor-honest.** One stdio LSP process per Zed session is the default.
   `daemon`/`forward`/`unix` are opt-in and must never be selected by default
   or outlive their last session indefinitely.
3. **Zed owns remoting.** The bridge must never invoke `ssh`, manage reverse
   `streamlocal:` forwards, or pick a daemon for a remote worktree. See
   [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) → *Lifecycle*.
4. **No vendoring prior attempts.** Do not reference, import, or cross-link any
   previous Pike LSP/VS Code work. This repo is a clean implementation.
5. **Reliability over features.** A missing feature is fine; a crashing or
   memory-leaking one is not. Prefer degrading gracefully to failing loud.

## Repo layout

```
extension.toml            Zed extension manifest (grammar SHA + language server id)
Cargo.toml                Workspace root; the WASM bridge is the default member
languages/pike/           Tree-sitter query files + config.toml (see TOML trap)
crates/zed-pike-bridge/   wasm32-wasip2 cdylib; resolves + launches pike-lsp
crates/pike-lsp/          Native language server (analysis, transport, guard)
docs/ARCHITECTURE.md      Detailed living architecture (source of truth)
docs/perf.md              Performance/resource SLOs (source of truth)
openspec/                 Spec-driven change workflow (proposals → specs → archive)
scripts/                  check-branch-name.sh, perf-smoke.sh
.github/workflows/        ci.yml, release.yml
```

## Build / check / test (verified working on this toolchain)

Toolchain is pinned in `rust-toolchain.toml` (stable + rustfmt, clippy;
targets `wasm32-wasip1`, `wasm32-wasip2`, `x86_64-unknown-linux-gnu`).

```sh
# Native language server
cargo check   -p pike-lsp --all-targets
cargo clippy  --workspace --all-targets -- -D warnings
cargo test    --workspace
cargo fmt --all -- --check

# WASM bridge (what Zed actually loads)
cargo build -p zed-pike-bridge --target wasm32-wasip2 --release

# Performance / resource budget (must stay within docs/perf.md)
cargo bench -p pike-lsp --bench analysis
bash scripts/perf-smoke.sh
```

CI (`.github/workflows/ci.yml`) runs commitlint, branch-name check, `fmt`,
`clippy -D warnings`, `test --workspace`, the host `pike-lsp` release build, and
the `wasm32-wasip2` bridge build. **Match CI locally before opening a PR.**

Load into Zed for manual verification: command palette →
`zed: install dev extension` → select repo root → open a `.pike`/`.pmod`/`.cmod`
file.

## Conventions (enforced by hooks + CI)

- **Conventional Commits 1.0.0** — `<type>(<scope>): <subject>`. Types and
  scopes are enum-enforced by `.commitlintrc.json`. Subject is lower-case,
  imperative, no trailing period. Full rules + examples in
  [`CONTRIBUTING.md`](CONTRIBUTING.md).
- **Conventional branch names** — `<type>/<scope>-<short-topic>`; `main` is
  protected. Validated by `scripts/check-branch-name.sh`.
- **Local hooks** — `.husky/commit-msg` runs commitlint; `.husky/pre-push` runs
  the branch-name check. Install with `bunx husky install`.
- **Non-trivial changes go through OpenSpec** — `openspec/changes/<slug>/`
  (proposal → specs → design → tasks → apply → archive). Reference the change in
  the commit footer: `Refs: openspec/changes/<slug>`.

## Traps and gotchas

- **TOML quote trap** in `languages/pike/config.toml`: use single-quoted TOML
  strings for quote characters, never `"""`. Verify with
  `python3 -c "import tomllib; print(tomllib.loads(open('languages/pike/config.toml').read()))"`.
- **Grammar SHA lives in FOUR places — bump them together or things silently
  diverge:** (1) `extension.toml` `[grammars.pike].commit` (what Zed compiles),
  (2) `crates/pike-lsp/build.rs` `GRAMMAR_COMMIT` (what the LSP actually links —
  it downloads the tarball and compiles `parser.c`), (3) `Cargo.toml`
  `tree-sitter-pike` `rev`, and (4) the same commit's doc comment in `build.rs`.
  Note (3) is currently **unused by any crate** yet still resolved by cargo — an
  old rev there pins the whole workspace's `tree-sitter` version (this is what
  held us at 0.25). Keep it aligned or drop it deliberately.
- **`tree-sitter` runtime version must match the grammar's ABI** and, ideally,
  Zed's own (0.26.x, ABI 15). The grammar crate declares the required
  `tree-sitter` version; bumping the grammar pin can force a `tree-sitter` bump.
- **The bridge target is `wasm32-wasip2`** — it must not pull in host-only tokio
  features (`process`, `net`). Those are gated for `pike-lsp` only.
- **Query node names** must match the pinned grammar's `node-types.json`. A
  grammar bump can silently break `highlights.scm`/`outline.scm`.

## When you finish a change

1. `cargo fmt` + `clippy -D warnings` + `cargo test` all clean.
2. Both targets build (native + wasm).
3. Perf budget still holds (`docs/perf.md`) — rerun the smoke/bench if you
   touched `pike-lsp`.
4. Docs updated if behavior or architecture changed.
5. Conventional commit + branch name; OpenSpec reference if non-trivial.
