// SSH transport: open an SSH session with reverse streamlocal
// forwarding and bridge the process's stdio to the forwarded
// socket. The remote side runs `pike-lsp unix --socket <path>`.
//
// Implementation note: we shell out to the system `ssh` rather
// than binding libssh2 from the bridge. The bridge is responsible
// for selecting the transport; the server's `ssh` subcommand is
// the thin client that talks to a `pike-lsp` already listening on
// the local forwarded socket.

use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn serve(host: &str, remote_socket: &Path, local_socket: &Path) -> anyhow::Result<()> {
    // Connect to the local socket that the `ssh` process is
    // listening on. The caller (the bridge) is responsible for
    // spawning `ssh` with the right `-R streamlocal:...` flag; the
    // server only sees the local end of the bridge.
    let sock = UnixStream::connect(local_socket)
        .await
        .with_context(|| format!("connect to local socket {}", local_socket.display()))?;
    tracing::info!(
        ?host,
        ?remote_socket,
        ?local_socket,
        "pike-lsp: ssh transport up"
    );

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

    let _ = tokio::join!(upstream, downstream);
    Ok(())
}

/// Spawn the `ssh` process that reverse-forwards `remote_socket`
/// to a local socket. Returns the `Child` so the caller can wait
/// on it or kill it. This is the server-side helper used by the
/// CLI `pike-lsp ssh --host ... --remote-socket ... --local-socket ...`
/// invocation; the bridge uses the same shape but spawns from
/// WASM-unreachable code so it shells out to a host process.
pub async fn spawn_ssh_reverse_forward(
    host: &str,
    remote_socket: &Path,
    local_socket: &Path,
) -> anyhow::Result<tokio::process::Child> {
    use tokio::process::Command;
    let mut cmd = Command::new("ssh");
    cmd.arg("-T")
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-R")
        .arg(format!(
            "{}:streamlocal:{}",
            remote_socket.display(),
            local_socket.display()
        ))
        .arg(host)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    let child = cmd.spawn().context("spawn ssh")?;
    Ok(child)
}
