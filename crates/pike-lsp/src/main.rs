// Pike LSP: thin wrapper around the lib crate. All real
// implementation lives in `pike_lsp::*`.

#[cfg(unix)]
use pike_lsp::cli::{Cli, Command};
#[cfg(not(unix))]
use pike_lsp::cli::{Cli, Command as _Command};
use pike_lsp::resource_guard;
use pike_lsp::resource_guard::ResourceGuardConfig;
#[cfg(unix)]
use pike_lsp::{daemon, forward, transport};

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let guard = ResourceGuardConfig::resolve(
        cli.max_rss_mb,
        std::env::var("PIKE_LSP_MAX_RSS_MB").ok().as_deref(),
    );
    resource_guard::spawn(guard);

    #[cfg(unix)]
    match cli.command {
        Command::Stdio => {
            tracing::info!("pike-lsp: starting stdio transport");
            transport::stdio::serve().await.context("stdio transport")?;
        }
        Command::Unix { socket } => {
            tracing::info!(?socket, "pike-lsp: starting unix-socket transport");
            transport::unix::serve(&socket)
                .await
                .with_context(|| format!("unix transport on {}", socket.display()))?;
        }
        Command::Ssh {
            host,
            remote_socket,
            local_socket,
        } => {
            tracing::info!(
                ?host,
                ?remote_socket,
                ?local_socket,
                "pike-lsp: starting ssh transport"
            );
            transport::ssh::serve(&host, &remote_socket, &local_socket)
                .await
                .context("ssh transport")?;
        }
        Command::Forward { remote } => {
            tracing::info!(?remote, "pike-lsp: starting forwarder proxy");
            forward::serve(&remote)
                .await
                .with_context(|| format!("forwarder to {}", remote.display()))?;
        }
        Command::Daemon {
            socket,
            idle_timeout,
        } => {
            tracing::info!(?socket, ?idle_timeout, "pike-lsp: starting daemon");
            daemon::serve(&socket, idle_timeout)
                .await
                .with_context(|| format!("daemon on {}", socket.display()))?;
        }
    }

    #[cfg(not(unix))]
    {
        // Windows ships only the stdio transport. Anything else is
        // an explicit clap error (the variant doesn't exist on this
        // target), so we only need to handle the `Stdio` arm here.
        match cli.command {
            _Command::Stdio => {
                tracing::info!("pike-lsp: starting stdio transport");
                pike_lsp::transport::stdio::serve()
                    .await
                    .context("stdio transport")?;
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("PIKE_LSP_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
