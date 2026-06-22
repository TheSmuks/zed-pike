// Forwarder: a thin proxy that copies LSP frames in both
// directions between stdio and a Unix-domain socket without
// parsing them. Mirrors gopls's `-remote=<addr>` behavior. The
// forwarder is what the editor actually starts; the daemon
// (long-lived) and the editor (short-lived) are decoupled by
// this proxy.
//
// See `../openspec/changes/pike-lsp-foundation/specs/pike-lsp-transport/`
// for the requirements this module implements.

use std::path::Path;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn serve(remote: &Path) -> anyhow::Result<()> {
    let sock = UnixStream::connect(remote)
        .await
        .with_context(|| format!("connect to {}", remote.display()))?;
    tracing::info!(?remote, "pike-lsp: forwarder connected");

    let (mut read_half, mut write_half) = sock.into_split();

    // stdio <-> socket in two directions, on two tasks.
    let upstream = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = match stdin.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!(?e, "forwarder stdin read");
                    break;
                }
            };
            if write_half.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
    });

    let downstream = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = match read_half.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!(?e, "forwarder socket read");
                    break;
                }
            };
            if stdout.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
    });

    let _ = tokio::join!(upstream, downstream);
    Ok(())
}
