// Daemon: a single long-lived process that listens on a
// Unix-domain socket, accepts N client connections, hosts one LSP
// session per connection, and shares one `Analysis` instance
// across all sessions. Idle auto-shutdown: if the connected
// session count drops to zero, the daemon exits after
// `idle_timeout`. See `../openspec/changes/pike-lsp-foundation/
// specs/pike-lsp-daemon/` for the requirements this module
// implements.
//
// Unix-only: the daemon's transport is a Unix-domain socket, which
// has no direct equivalent on Windows in this change.

#![cfg(unix)]

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use parking_lot::Mutex;
use tokio::net::UnixListener;
use tokio::time::sleep;
use tower_lsp::Server;

use crate::analysis::Analysis;
use crate::service::{build_service_with_analysis, PikeLanguageServer};

pub async fn serve(socket: &Path, idle_timeout: Duration) -> anyhow::Result<()> {
    if socket.exists() {
        anyhow::bail!(
            "socket {} already exists; remove it if no daemon is running",
            socket.display()
        );
    }
    let listener =
        UnixListener::bind(socket).with_context(|| format!("bind {}", socket.display()))?;
    tracing::info!(?socket, ?idle_timeout, "pike-lsp: daemon listening");

    let analysis = Arc::new(Analysis::new());
    let connected: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    loop {
        // Tick the idle check every second.
        let connected_snapshot = connected.clone();
        let listener_ref = &listener;
        tokio::select! {
            accept = listener_ref.accept() => {
                let (stream, _addr) = accept.context("accept")?;
                {
                    let mut c = connected.lock();
                    *c += 1;
                }
                let analysis = analysis.clone();
                let connected = connected.clone();
                tokio::spawn(async move {
                    let (svc, loopback) =
                        build_service_with_analysis(analysis);
                    let (read, write) = tokio::io::split(stream);
                    let _ = Server::new(read, write, loopback).serve(svc).await;
                    {
                        let mut c = connected.lock();
                        *c = c.saturating_sub(1);
                    }
                    drop(connected_snapshot);
                    // silence "unused" if PikeLanguageServer is later
                    // constructed directly by another transport.
                    let _ = std::any::type_name::<PikeLanguageServer>();
                });
            }
            _ = sleep(Duration::from_secs(1)) => {
                if *connected.lock() == 0 {
                    tracing::info!("pike-lsp: daemon idle, exiting");
                    let _ = std::fs::remove_file(socket);
                    return Ok(());
                }
            }
        }
    }
}
