// Pike LSP: the long-lived language server for Pike 8.0.1116.
//
// This crate ships:
//   - a `pike-lsp` binary that runs the LSP over stdio, a Unix
//     socket, or an SSH reverse-forwarded streamlocal socket;
//   - a `daemon` mode (one process, many sessions) modelled on
//     gopls;
//   - an incremental analysis layer (Salsa-style queries with
//     durable / normal / volatile tiers) over the maintained
//     `TheSmuks/tree-sitter-pike` grammar.
//
// See `../openspec/changes/pike-lsp-foundation/` for the
// requirements and design this crate implements.

pub mod analysis;
pub mod cli;
pub mod daemon;
pub mod forward;
pub mod resource_guard;
pub mod service;
pub mod transport;

pub use service::PikeLanguageServer;
