// Zed WASM bridge for the Pike language server.
//
// Correct remote model:
//   - Zed owns SSH remoting. When the user opens a remote SSH
//     worktree, Zed executes the language-server command in that
//     remote worktree context.
//   - Therefore this bridge returns a normal `pike-lsp stdio`
//     command. It does not invoke `ssh`, create reverse forwards,
//     or select a local daemon by default.
//
// Lifecycle model:
//   - The default LSP server is one native process per Zed LSP
//     session. Zed closes stdin when the session ends, and the
//     stdio server exits with it.
//   - Shared daemon/forwarder mode is intentionally not the
//     default because it can leave a process alive on an SSH host
//     after the editor session ends.

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
        // 1. User/worktree PATH wins. In Zed remote SSH mode,
        //    this lookup is scoped to the remote worktree context;
        //    no extension-owned SSH tunnel is needed.
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

        // 3. Auto-download. The downloaded binary is still launched
        //    as stdio so the process lifetime remains owned by the
        //    Zed LSP session.
        let binary = self.locate_binary(language_server_id)?;
        Ok(stdio_command(binary))
    }
}

impl PikeBridge {
    /// Resolve the `pike-lsp` binary path. Resolution order:
    ///   1. cached auto-download
    ///   2. fresh auto-download from the latest GitHub release
    fn locate_binary(&mut self, language_server_id: &zed::LanguageServerId) -> zed::Result<String> {
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

zed::register_extension!(PikeBridge);
