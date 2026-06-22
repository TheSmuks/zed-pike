// stdio transport: JSON-RPC 2.0 over stdin/stdout.

use tower_lsp::Server;

use crate::service::build_service;

pub async fn serve() -> anyhow::Result<()> {
    let (service, loopback) = build_service();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, loopback)
        .serve(service)
        .await;
    Ok(())
}
