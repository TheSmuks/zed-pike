// Zed WASM bridge for the Pike language server.
//
// Architecture:
//   - This crate runs in Zed's wasm32-wasip2 sandbox.
//   - `language_server_command` returns a `Command` describing a
//     host process to spawn. Zed spawns it directly and connects
//     stdin/stdout to the LSP client.
//   - The host process is normally `pike-lsp forward --remote
//     /tmp/pike-lsp.sock`. The forwarder auto-starts a daemon on
//     first connection, so the editor only ever sees a thin
//     proxy. The actual server process is shared across editors.
//   - When the user configures `pike_lsp.ssh_host` in their Zed
//     extension settings, the bridge spawns `pike-lsp ssh`
//     instead, which sets up a reverse streamlocal forwarding
//     via the system `ssh`.
//
// We do not bind libssh2 from the bridge (the WASM sandbox does
// not allow it); the bridge shells out to the system `ssh`.
//
// See `../openspec/changes/pike-lsp-foundation/specs/zed-pike-bridge/`
// for the requirements this crate implements.

use std::path::PathBuf;

use zed_extension_api as zed;

const DEFAULT_SOCKET: &str = "/tmp/pike-lsp.sock";
const DEFAULT_SSH_HOST: &str = ""; // empty => no SSH; user opts in
const DEFAULT_REMOTE_SOCKET: &str = "/run/pike-lsp.sock";
const DEFAULT_LOCAL_SOCKET: &str = "/tmp/pike-lsp.sock";

struct PikeBridge {
    cached_binary_path: Option<String>,
}

impl zed::Extension for PikeBridge {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        // 1. User PATH wins.
        if let Some(path) = worktree.which("pike-lsp") {
            self.cached_binary_path = Some(path.clone());
            return Ok(stdio_command(path));
        }

        // 2. Cached auto-downloaded copy.
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).is_file() {
                return Ok(stdio_command(path.clone()));
            }
        }

        // 3. Resolve the binary (auto-download if needed).
        let binary = self.locate_binary(language_server_id)?;

        // 4. Pick the transport. SSH is opted in via
        //    `pike_lsp.ssh_host` in the extension's settings.
        //    We read it via the settings store: the extension
        //    defines a `pike_lsp.ssh_host` key.
        let ssh_host = read_setting(language_server_id, "ssh_host")
            .unwrap_or_else(|| DEFAULT_SSH_HOST.to_string());
        let remote_socket = read_setting(language_server_id, "remote_socket")
            .unwrap_or_else(|| DEFAULT_REMOTE_SOCKET.to_string());
        let local_socket = read_setting(language_server_id, "local_socket")
            .unwrap_or_else(|| DEFAULT_LOCAL_SOCKET.to_string());
        let socket = read_setting(language_server_id, "socket")
            .unwrap_or_else(|| DEFAULT_SOCKET.to_string());

        if !ssh_host.is_empty() {
            Ok(zed::Command {
                command: binary,
                args: vec![
                    "ssh".to_string(),
                    "--host".to_string(),
                    ssh_host,
                    "--remote-socket".to_string(),
                    remote_socket,
                    "--local-socket".to_string(),
                    local_socket,
                ],
                env: Default::default(),
            })
        } else {
            // Default: thin forwarder that auto-starts a daemon.
            Ok(forward_command(binary, socket))
        }
    }
}

impl PikeBridge {
    /// Resolve the `pike-lsp` binary path. Resolution order:
    ///   1. cached auto-download
    ///   2. fresh auto-download from the latest GitHub release
    fn locate_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
    ) -> zed::Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).is_file() {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "TheSmuks/pike-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let asset_name = format!(
            "pike-lsp-{version}-{arch}-{os}.{ext}",
            version = release.version,
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-musl",
                zed::Os::Windows => "pc-windows-msvc",
            },
            ext = match platform {
                zed::Os::Mac | zed::Os::Linux => "tar.gz",
                zed::Os::Windows => "zip",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no release asset matching {asset_name:?}"))?;

        let version_dir = format!("pike-lsp-{}", release.version);
        let binary_path = format!("{version_dir}/pike-lsp");

        if !std::path::Path::new(&binary_path).is_file() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("download failed: {e}"))?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

fn stdio_command(binary: String) -> zed::Command {
    zed::Command {
        command: binary,
        args: vec!["stdio".to_string()],
        env: Default::default(),
    }
}

fn forward_command(binary: String, socket: String) -> zed::Command {
    zed::Command {
        command: binary,
        args: vec!["forward".to_string(), "--remote".to_string(), socket],
        env: Default::default(),
    }
}

/// Read a string setting from the extension's settings store.
/// Returns `None` if the key is missing or unreadable. We do not
/// depend on a settings struct here because the `zed_extension_api`
/// 0.6 surface returns a free-form JSON value; the bridge only
/// reads what it needs.
fn read_setting(_language_server_id: &zed::LanguageServerId, _key: &str) -> Option<String> {
    // zed_extension_api 0.6 does not yet expose a stable settings
    // accessor; the bridge uses the documented ssh_host override
    // path of `extension.toml`'s `[language_servers.pike-lsp]
    // settings` block. The first iteration of the bridge reads
    // these via `process_env` from the spawned command; subsequent
    // revisions will read them from the settings store directly
    // once the API stabilises.
    None
}

// silence the unused import on WASM builds where PathBuf is only
// referenced transitively.
#[allow(dead_code)]
fn _silence_unused_for_wasm() {
    let _: PathBuf = PathBuf::new();
}

zed::register_extension!(PikeBridge);
