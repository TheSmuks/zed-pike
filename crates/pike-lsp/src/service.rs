// Shared LSP service. Used by every transport: stdio, unix, ssh,
// and the daemon. The service uses `tower-lsp`'s `LspService`
// with a single shared `Analysis` instance per process.

use std::sync::Arc;

use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, ClientSocket, LanguageServer, LspService};

use crate::analysis::Analysis;

pub struct PikeLanguageServer {
    client: Client,
    analysis: Arc<Analysis>,
}

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
pub fn build_service_with_analysis(
    analysis: Arc<Analysis>,
) -> (LspService<PikeLanguageServer>, ClientSocket) {
    LspService::new(move |client| PikeLanguageServer {
        client,
        analysis: analysis.clone(),
    })
}

impl PikeLanguageServer {
    /// Construct a new server bound to the given analysis. This is
    /// the canonical way for transports to assemble a server with
    /// a shared analysis (e.g. the daemon).
    pub fn new(client: Client, analysis: Arc<Analysis>) -> Self {
        Self { client, analysis }
    }
}
