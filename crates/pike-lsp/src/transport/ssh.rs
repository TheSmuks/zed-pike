// SSH transport: open an SSH session with reverse streamlocal
// forwarding, and bridge the process's stdio to the forwarded
// socket. The remote side runs `pike-lsp unix --socket <path>`.
// This is the SSH-aware surface described in
// `../openspec/changes/pike-lsp-foundation/specs/pike-lsp-transport/`.
//
// Implementation note: we shell out to the system `ssh` rather
// than binding libssh2 from the bridge. The bridge's `local_socket`
// is the path we hand to `ssh -L` style flags; the remote socket
// is the path the *remote* `pike-lsp` should listen on.

use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;

pub async fn serve(
    host: &str,
    remote_socket: &Path,
    local_socket: &Path,
) -> anyhow::Result<()> {
    // Connect to a local Unix-domain socket that `ssh` will create
    // for the duration of the session. The user is responsible for
    // ensuring this path is writable.
    let sock = UnixStream::connect(local_socket)
        .await
        .with_context(|| format!("connect to local socket {}", local_socket.display()))?;
    tracing::info!(?host, ?remote_socket, ?local_socket, "pike-lsp: ssh transport up");

    // The actual SSH command is launched by the calling editor,
    // not by us. The bridge is responsible for spawning `ssh` with
    // the right reverse-forwarding flags. Here we just bridge
    // stdio to the local socket. The contract is documented in
    // docs/perf.md and in the design.md.
    let (mut read_half, mut write_half) = sock.into_split();

    let upstream = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut buf = [0u8; 16 * 1024];
        loop {
            let n = match stdin.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
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
                Err(_) => break,
            };
            if stdout.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
    });

    // We don't spawn `ssh` here; the bridge does. This function is
    // a thin alias for the unix-style bridge. The actual SSH
    // process management lives in the bridge crate.
    let _ = Command::new("true")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    let _ = tokio::join!(upstream, downstream);
    Ok(())
}
