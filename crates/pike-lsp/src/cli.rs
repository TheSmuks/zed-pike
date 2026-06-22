// CLI surface for `pike-lsp`. Mirrors the transport spec:
//   pike-lsp                              # stdio (default)
//   pike-lsp unix --socket /path.sock     # listen on a unix-socket
//   pike-lsp ssh  --host user@h --remote-socket /run/pike-lsp.sock
//   pike-lsp forward --remote /path.sock  # thin proxy
//   pike-lsp daemon --socket /path.sock   # shared analysis cache

use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "pike-lsp",
    version,
    about = "Pike language server (LSP 3.17 over JSON-RPC 2.0)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Read LSP frames from stdin, write to stdout. Default transport.
    Stdio,
    /// Listen on a Unix-domain socket; N clients share the analysis cache.
    Unix {
        #[arg(long)]
        socket: PathBuf,
    },
    /// Open an SSH session with reverse streamlocal forwarding and
    /// bridge stdio to the forwarded socket.
    Ssh {
        #[arg(long)]
        host: String,
        #[arg(long)]
        remote_socket: PathBuf,
        /// Local Unix-socket the bridge uses to talk to the SSH process.
        #[arg(long)]
        local_socket: PathBuf,
    },
    /// Thin proxy: copy LSP frames in both directions between stdio
    /// and a Unix-socket without parsing them. Used by `daemon` and
    /// by editors that want to share an existing daemon.
    Forward {
        #[arg(long)]
        remote: PathBuf,
    },
    /// Shared daemon: listen on a Unix-socket, accept N client
    /// connections, host one LSP session per connection, share one
    /// analysis cache. Auto-shutdown after `--idle-timeout` of
    /// zero connected sessions.
    Daemon {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long, default_value = "60s", value_parser = parse_duration)]
        idle_timeout: Duration,
    },
}

fn parse_duration(s: &str) -> Result<Duration, std::num::ParseIntError> {
    // Accept "60s" or bare seconds. Cheap parser; no need for full humantime.
    let trimmed = s.trim();
    if let Some(num) = trimmed.strip_suffix('s') {
        num.parse::<u64>().map(Duration::from_secs)
    } else {
        trimmed.parse::<u64>().map(Duration::from_secs)
    }
}
