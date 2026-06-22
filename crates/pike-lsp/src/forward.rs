// Forwarder: a thin proxy that copies LSP frames in both
// directions between stdio and a Unix-domain socket without
// parsing them.
//
// Important lifecycle rule:
//   `forward` does not spawn a detached daemon by default. That
//   prevents Zed remote SSH sessions from leaving background
//   processes alive on the remote host. Users who want a shared
//   daemon must start `pike-lsp daemon --socket <path>` explicitly
//   (or a future explicit opt-in flag can do so with an idle TTL).

use std::path::Path;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn serve(remote: &Path) -> anyhow::Result<()> {
    if !remote.exists() {
        anyhow::bail!(
            "daemon socket {} does not exist; start `pike-lsp daemon --socket {}` explicitly or use `pike-lsp stdio`",
            remote.display(),
            remote.display()
        );
    }

    let sock = UnixStream::connect(remote)
        .await
        .with_context(|| format!("connect to {}", remote.display()))?;
    tracing::info!(?remote, "pike-lsp: forwarder connected");

    let (mut read_half, mut write_half) = sock.into_split();

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
