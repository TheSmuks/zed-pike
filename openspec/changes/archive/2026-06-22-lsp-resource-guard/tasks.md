## 1. Resource guard implementation

- [x] 1.1 Add `crates/pike-lsp/src/resource_guard.rs` with config resolution, Linux RSS parsing, and monitor task.
- [x] 1.2 Add global `--max-rss-mb <MiB>` CLI option with env/default precedence.
- [x] 1.3 Start the guard from `main` before transport dispatch.

## 2. Verification

- [x] 2.1 Add unit tests for config precedence and Linux `VmRSS` parsing.
- [x] 2.2 Update `scripts/perf-smoke.sh` to verify the option is accepted and normal stdio startup still works.
- [x] 2.3 Run `openspec validate lsp-resource-guard` successfully.
- [x] 2.4 Run `cargo test --workspace --no-fail-fast` successfully.
- [x] 2.5 Run `cargo check -p zed-pike-bridge --target wasm32-wasip2` successfully.
