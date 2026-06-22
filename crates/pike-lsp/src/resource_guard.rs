use std::time::Duration;

use tokio::time::sleep;

pub const DEFAULT_MAX_RSS_MB: u64 = 256;
pub const EXIT_CODE_RESOURCE_LIMIT: i32 = 137;
const CHECK_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceGuardConfig {
    pub max_rss_mb: u64,
}

impl ResourceGuardConfig {
    pub fn resolve(cli_max_rss_mb: Option<u64>, env_max_rss_mb: Option<&str>) -> Self {
        let max_rss_mb = cli_max_rss_mb
            .or_else(|| env_max_rss_mb.and_then(parse_u64))
            .unwrap_or(DEFAULT_MAX_RSS_MB);
        Self { max_rss_mb }
    }

    pub fn disabled(self) -> bool {
        self.max_rss_mb == 0
    }

    pub fn max_rss_kib(self) -> u64 {
        self.max_rss_mb.saturating_mul(1024)
    }
}

pub fn spawn(config: ResourceGuardConfig) {
    if config.disabled() {
        tracing::info!("pike-lsp resource guard disabled");
        return;
    }

    match current_rss_kib() {
        Ok(rss_kib) => {
            tracing::info!(
                max_rss_mb = config.max_rss_mb,
                current_rss_kib = rss_kib,
                "pike-lsp resource guard enabled"
            );
        }
        Err(err) => {
            tracing::warn!(%err, "pike-lsp resource guard unavailable");
            eprintln!("pike-lsp resource guard unavailable: {err}");
            return;
        }
    }

    tokio::spawn(async move {
        loop {
            sleep(CHECK_INTERVAL).await;
            match current_rss_kib() {
                Ok(rss_kib) if rss_kib > config.max_rss_kib() => {
                    let rss_mb = div_ceil(rss_kib, 1024);
                    let msg = format!(
                        "pike-lsp resource guard: RSS {rss_mb} MiB ({rss_kib} KiB) exceeded limit {} MiB ({} KiB); exiting with code {EXIT_CODE_RESOURCE_LIMIT}",
                        config.max_rss_mb,
                        config.max_rss_kib()
                    );
                    tracing::error!(
                        rss_kib,
                        max_rss_kib = config.max_rss_kib(),
                        exit_code = EXIT_CODE_RESOURCE_LIMIT,
                        "pike-lsp resource guard limit exceeded"
                    );
                    eprintln!("{msg}");
                    std::process::exit(EXIT_CODE_RESOURCE_LIMIT);
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(%err, "pike-lsp resource guard measurement failed; disabling guard");
                    eprintln!("pike-lsp resource guard measurement failed; disabling guard: {err}");
                    return;
                }
            }
        }
    });
}

pub fn current_rss_kib() -> anyhow::Result<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status")?;
        parse_linux_status_rss_kib(&status)
            .ok_or_else(|| anyhow::anyhow!("/proc/self/status did not contain VmRSS"))
    }

    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("RSS measurement is not implemented on this platform")
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn parse_linux_status_rss_kib(status: &str) -> Option<u64> {
    status.lines().find_map(|line| {
        let value = line.strip_prefix("VmRSS:")?.trim();
        let number = value.split_whitespace().next()?;
        number.parse::<u64>().ok()
    })
}

fn parse_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

fn div_ceil(n: u64, d: u64) -> u64 {
    n / d + u64::from(n % d != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_default_without_overrides() {
        assert_eq!(
            ResourceGuardConfig::resolve(None, None),
            ResourceGuardConfig {
                max_rss_mb: DEFAULT_MAX_RSS_MB
            }
        );
    }

    #[test]
    fn config_uses_environment_override() {
        assert_eq!(
            ResourceGuardConfig::resolve(None, Some("128")),
            ResourceGuardConfig { max_rss_mb: 128 }
        );
    }

    #[test]
    fn cli_override_wins_over_environment() {
        assert_eq!(
            ResourceGuardConfig::resolve(Some(64), Some("128")),
            ResourceGuardConfig { max_rss_mb: 64 }
        );
    }

    #[test]
    fn zero_disables_guard() {
        assert!(ResourceGuardConfig::resolve(Some(0), None).disabled());
    }

    #[test]
    fn invalid_environment_falls_back_to_default() {
        assert_eq!(
            ResourceGuardConfig::resolve(None, Some("not-a-number")),
            ResourceGuardConfig {
                max_rss_mb: DEFAULT_MAX_RSS_MB
            }
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_linux_vmrss() {
        let status = "Name:\tpike-lsp\nVmPeak:\t  100000 kB\nVmRSS:\t   12345 kB\nThreads:\t1\n";
        assert_eq!(parse_linux_status_rss_kib(status), Some(12345));
    }
}
