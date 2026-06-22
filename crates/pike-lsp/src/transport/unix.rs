// Unix-socket transport: JSON-RPC 2.0 over a Unix-domain socket.
// N clients share the same `Analysis` cache (one LSP session per
// accepted connection). See `service::build_service` for the
// shared service factory.

use std::path::Path;

use anyhow::Context;
use tokio::net::UnixListener;
use tower_lsp::Server;

use crate::service::build_service;

pub async fn serve(socket: &Path) -> anyhow::Result<()> {
    if socket.exists() {
        anyhow::bail!(
            "socket {} already exists; remove it if no daemon is running",
            socket.display()
        );
    }
    let listener =
        UnixListener::bind(socket).with_context(|| format!("bind {}", socket.display()))?;
    tracing::info!(?socket, "pike-lsp: unix-socket listening");

    loop {
        let (stream, _addr) = listener.accept().await.context("accept")?;
        let (service, loopback) = build_service();
        let (read, write) = tokio::io::split(stream);
        tokio::spawn(async move {
            let _ = Server::new(read, write, loopback).serve(service).await;
        });
    }
}
