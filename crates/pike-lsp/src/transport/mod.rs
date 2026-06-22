//! Transport layer for `pike-lsp`.
//!
//! Three concrete transports:
//!   - `stdio` — JSON-RPC 2.0 over the process's stdin/stdout.
//!   - `unix` — JSON-RPC 2.0 over a Unix-domain socket; N clients
//!     share the same `Analysis` cache.
//!   - `ssh` — opens an SSH session with a reverse streamlocal
//!     forwarding and bridges stdio to the forwarded socket.
//!
//! All three use the same `tower-lsp` `LspService` built by
//! [`crate::service::build_service`].

pub mod ssh;
pub mod stdio;
pub mod unix;

use std::sync::OnceLock;

use tree_sitter::Language;

/// `tree-sitter-pike` C-language pointer. The actual function is
/// defined by the C parser built in `build.rs` and re-exported by
/// the `pike_grammar` static library.
extern "C" {
    fn tree_sitter_pike() -> *const ();
}

static PIKE_LANGUAGE: OnceLock<Language> = OnceLock::new();

/// The tree-sitter `Language` for Pike 8.0.1116, sourced from the
/// `TheSmuks/tree-sitter-pike` grammar pinned in
/// `extension.toml` and the build script.
pub fn pike_language() -> Language {
    PIKE_LANGUAGE
        .get_or_init(|| unsafe {
            // tree_sitter_pike() returns the raw `*const TSLanguage`
            // from the C ABI; tree-sitter's `Language::from_raw`
            // takes a `*const TSLanguage`. The cast is an identity
            // cast at the ABI level.
            Language::from_raw(tree_sitter_pike() as *const tree_sitter::ffi::TSLanguage)
        })
        .clone()
}
