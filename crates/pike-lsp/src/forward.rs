// Forwarder: a thin proxy that copies LSP frames in both
// directions between stdio and a Unix-domain socket without
// parsing them. Mirrors gopls's `-remote=auto` behavior.
//
// Auto-start: if the requested socket does not exist, the
// forwarder spawns `pike-lsp daemon --socket <path>` itself,
// waits for the socket to appear, and only then begins
// forwarding. This matches the gopls design point: a single
// persistent server is shared across editor sessions, and the
// editor never has to know about daemon lifecycle.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;
use tokio::time::{sleep, timeout};

const SOCKET_WAIT: Duration = Duration::from_secs(10);

pub async fn serve(remote: &Path) -> anyhow::Result<()> {
    // 1. If the socket already exists, just connect.
    if !remote.exists() {
        // 2. Otherwise try to start a daemon ourselves. The
        //    forwarder binary and the daemon binary are the same
        //    `pike-lsp`; we resolve via PATH (so a user-installed
        //    binary wins) and fall back to the same argv[0].
        let resolved = resolve_self_binary();
        if let Some(bin) = resolved {
            spawn_daemon(&bin, remote).await?;
            wait_for_socket(remote).await?;
        } else {
            anyhow::bail!(
                "socket {} does not exist and `pike-lsp` is not on PATH",
                remote.display()
            );
        }
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

/// Resolve the path to the `pike-lsp` binary that owns the
/// auto-start logic. We try `which pike-lsp` on the current
/// process's PATH; if that fails, we fall back to `argv[0]`
/// (which is what `cargo run` uses during development).
fn resolve_self_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PIKE_LSP_BIN") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("pike-lsp");
            if candidate.is_file() {
                return Some(candidate);
            }
            let candidate_exe = dir.join("pike-lsp.exe");
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
        }
    }
    let argv0 = std::env::args().next()?;
    let p = PathBuf::from(argv0);
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

async fn spawn_daemon(bin: &Path, socket: &Path) -> anyhow::Result<()> {
    tracing::info!(?bin, ?socket, "pike-lsp: spawning daemon for auto-start");
    let child = Command::new(bin)
        .arg("daemon")
        .arg("--socket")
        .arg(socket)
        .arg("--idle-timeout")
        .arg("60s")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn {} daemon", bin.display()))?;
    // Detach: we don't wait on the daemon. It will exit on idle.
    std::mem::forget(child);
    Ok(())
}

async fn wait_for_socket(socket: &Path) -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    while start.elapsed() < SOCKET_WAIT {
        if socket.exists() {
            // Give the daemon a beat to actually accept() on the
            // listening socket. The kernel can return from
            // bind() before listen() in rare cases.
            sleep(Duration::from_millis(50)).await;
            return Ok(());
        }
        sleep(Duration::from_millis(50)).await;
    }
    let _ = timeout(SOCKET_WAIT, async {
        // last-chance: try a connect to force the error path
        if let Ok(s) = UnixStream::connect(socket).await {
            drop(s);
        }
    })
    .await;
    if socket.exists() {
        Ok(())
    } else {
        anyhow::bail!(
            "daemon did not create socket {} within {:?}",
            socket.display(),
            SOCKET_WAIT
        )
    }
}
