// Zed WASM bridge: registers the Pike LSP server in Zed.
//
// The bridge itself runs inside Zed's wasm32-wasip2 sandbox and
// owns the decision of how to launch the server. The actual
// `pike-lsp` binary is a normal native process spawned via
// `Command::new(...)` from the bridge.
//
// See ../openspec/changes/pike-lsp-foundation/specs/zed-pike-bridge/
// for the requirements this bridge implements.

use zed_extension_api as zed;

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
        // 1. User PATH (per spec, the user's binary wins).
        if let Some(path) = worktree.which("pike-lsp") {
            self.cached_binary_path = Some(path.clone());
            return Ok(zed::Command {
                command: path,
                args: vec!["stdio".to_string()],
                env: Default::default(),
            });
        }

        // 2. Cached auto-downloaded copy from a previous launch.
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).is_file() {
                return Ok(zed::Command {
                    command: path.clone(),
                    args: vec!["stdio".to_string()],
                    env: Default::default(),
                });
            }
        }

        // 3. Auto-install. The release asset naming follows
        //    `pike-lsp-<version>-<arch>-<os>.<ext>` (see docs/perf.md).
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
        Ok(zed::Command {
            command: binary_path,
            args: vec!["stdio".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(PikeBridge);
