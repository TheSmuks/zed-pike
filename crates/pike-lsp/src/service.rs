// Shared LSP service. Used by every transport: stdio, unix, ssh,
// and the daemon. The service uses `tower-lsp`'s `LspService`
// with a single shared `Analysis` instance per process.
//
// On non-Unix targets, only the `stdio` transport is compiled in,
// and it serves an analysis-free stub. The stub still speaks
// LSP 3.17 (responds to `initialize`, `initialized`, `shutdown`)
// so the bridge can confirm the binary is alive; it just does
// not advertise any document capabilities.

#[cfg(unix)]
use std::sync::Arc;

#[cfg(unix)]
use tower_lsp::jsonrpc::Result as JsonRpcResult;
#[cfg(unix)]
use tower_lsp::lsp_types::*;
#[cfg(unix)]
use tower_lsp::{Client, ClientSocket, LanguageServer, LspService};

#[cfg(not(unix))]
use tower_lsp::{ClientSocket, LspService};

#[cfg(unix)]
use crate::analysis::Analysis;

#[cfg(unix)]
pub struct PikeLanguageServer {
    client: Client,
    analysis: Arc<Analysis>,
}

#[cfg(unix)]
#[tower_lsp::async_trait]
impl LanguageServer for PikeLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> JsonRpcResult<InitializeResult> {
        tracing::info!(?params.capabilities, "initialize");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "pike-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("initialized");
        let _ = self
            .client
            .log_message(MessageType::INFO, "pike-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        tracing::info!("shutdown");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        self.analysis.open(&uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.analysis
                .update(params.text_document.uri.as_ref(), change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.analysis.close(params.text_document.uri.as_ref());
    }

    async fn hover(&self, params: HoverParams) -> JsonRpcResult<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let pos = params.text_document_position_params.position;
        let line = pos.line as usize;
        let col = pos.character as usize;
        Ok(self.analysis.hover(&uri, line, col))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> JsonRpcResult<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let pos = params.text_document_position_params.position;
        let line = pos.line as usize;
        let col = pos.character as usize;
        Ok(self.analysis.definition(&uri, line, col))
    }

    async fn references(&self, params: ReferenceParams) -> JsonRpcResult<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let pos = params.text_document_position.position;
        let line = pos.line as usize;
        let col = pos.character as usize;
        Ok(Some(self.analysis.references(&uri, line, col)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> JsonRpcResult<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        Ok(self.analysis.document_symbols(&uri))
    }
}

#[cfg(unix)]
pub fn build_service() -> (LspService<PikeLanguageServer>, ClientSocket) {
    let analysis = Arc::new(Analysis::new());
    LspService::new(move |client| PikeLanguageServer {
        client,
        analysis: analysis.clone(),
    })
}

/// Build a `(LspService, ClientSocket)` for an existing `Analysis`
/// instance. Used by the daemon to share one `Analysis` across
/// every connected session.
#[cfg(unix)]
pub fn build_service_with_analysis(
    analysis: Arc<Analysis>,
) -> (LspService<PikeLanguageServer>, ClientSocket) {
    LspService::new(move |client| PikeLanguageServer {
        client,
        analysis: analysis.clone(),
    })
}

#[cfg(unix)]
impl PikeLanguageServer {
    /// Construct a new server bound to the given analysis. This is
    /// the canonical way for transports to assemble a server with
    /// a shared analysis (e.g. the daemon).
    pub fn new(client: Client, analysis: Arc<Analysis>) -> Self {
        Self { client, analysis }
    }
}

// ------------------------------------------------------------------
// Windows stub. The Windows binary ships only the `stdio`
// transport and never instantiates an `Analysis`. To keep the
// `LspService<...>` machinery in `tower-lsp` happy we provide a
// minimal `PikeLanguageServer` that responds to the three core
// LSP lifecycle requests (`initialize`, `initialized`,
// `shutdown`) and reports no document capabilities.
// ------------------------------------------------------------------
#[cfg(not(unix))]
use tower_lsp::lsp_types::{
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    ServerInfo,
};

#[cfg(not(unix))]
pub struct PikeLanguageServer {
    client: tower_lsp::Client,
}

#[cfg(not(unix))]
#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for PikeLanguageServer {
    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // No document capabilities. The bridge is free to
                // fall back to syntax-only editor features.
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "pike-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        let _ = self
            .client
            .log_message(MessageType::INFO, "pike-lsp ready (windows stdio stub)")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}

#[cfg(not(unix))]
pub fn build_service() -> (LspService<PikeLanguageServer>, ClientSocket) {
    LspService::new(|client| PikeLanguageServer { client })
}
