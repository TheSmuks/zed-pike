## 1. Bridge defaults

- [x] 1.1 Remove fake `read_setting` / `ssh_host` selection from `crates/zed-pike-bridge/src/lib.rs`.
- [x] 1.2 Make `language_server_command` return `pike-lsp stdio` by default for both PATH and auto-downloaded binaries.
- [x] 1.3 Ensure bridge comments explicitly state that Zed remote SSH is handled by Zed worktree process launch, not extension-owned SSH.

## 2. Daemon lifecycle

- [x] 2.1 Change `pike-lsp forward --remote <socket>` to fail when the socket is absent instead of auto-starting a detached daemon.
- [x] 2.2 Leave `pike-lsp daemon` available as an explicit command with idle timeout.
- [x] 2.3 Remove or demote custom SSH helper language from the default transport path.

## 3. Verification

- [x] 3.1 Run `openspec validate zed-remote-lsp-lifecycle` successfully.
- [x] 3.2 Run `cargo test --workspace --no-fail-fast` successfully.
- [x] 3.3 Run `cargo check -p zed-pike-bridge --target wasm32-wasip2` successfully.
- [x] 3.4 Run a lifecycle smoke that confirms stdio responds to initialize and exits when stdin closes.
