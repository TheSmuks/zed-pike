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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TargetPlatform {
    os: zed::Os,
    arch: zed::Architecture,
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
        //    Zed LSP session. For SSH worktrees, choose the asset for
        //    the worktree execution platform rather than blindly using
        //    the local UI host platform.
        let binary = self.locate_binary(language_server_id, worktree)?;
        Ok(stdio_command(binary))
    }
}

impl PikeBridge {
    /// Resolve the `pike-lsp` binary path. Resolution order:
    ///   1. cached auto-download
    ///   2. fresh auto-download from the latest GitHub release
    fn locate_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
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
            "TheSmuks/zed-pike",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (host_os, host_arch) = zed::current_platform();
        let target = infer_target_platform(
            host_os,
            host_arch,
            &worktree.root_path(),
            &worktree.shell_env(),
        );
        let asset_name = release_asset_name(&release.version, target);

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no release asset matching {asset_name:?}"))?;

        let version_dir = format!("pike-lsp-{}", release.version);
        let binary_path = downloaded_binary_path(&version_dir, target.os);

        if !std::path::Path::new(&binary_path).is_file() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &version_dir,
                downloaded_file_type(target.os),
            )
            .map_err(|e| format!("download failed: {e}"))?;

            if matches!(target.os, zed::Os::Mac | zed::Os::Linux) {
                zed::make_file_executable(&binary_path)
                    .map_err(|e| format!("failed to make {binary_path:?} executable: {e}"))?;
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

fn infer_target_platform(
    host_os: zed::Os,
    host_arch: zed::Architecture,
    worktree_root: &str,
    shell_env: &[(String, String)],
) -> TargetPlatform {
    let env = EnvLookup(shell_env);

    let os = if has_windows_signal(worktree_root, &env) {
        zed::Os::Windows
    } else if has_linux_signal(worktree_root, &env) {
        zed::Os::Linux
    } else {
        host_os
    };

    TargetPlatform {
        os,
        arch: host_arch,
    }
}

fn has_windows_signal(worktree_root: &str, env: &EnvLookup<'_>) -> bool {
    let root = worktree_root.replace('/', "\\");
    root.as_bytes().get(1) == Some(&b':')
        || env.eq_ignore_ascii_case("OS", "Windows_NT")
        || env.get("COMSPEC").is_some()
        || env.get("ComSpec").is_some()
}

fn has_linux_signal(worktree_root: &str, env: &EnvLookup<'_>) -> bool {
    worktree_root.starts_with("/home/")
        || worktree_root.starts_with("/workspaces/")
        || worktree_root.starts_with("/workspace/")
        || worktree_root.starts_with("/mnt/")
        || env.get("WSL_DISTRO_NAME").is_some()
        || env
            .get("SHELL")
            .is_some_and(|shell| shell.starts_with("/bin/") || shell.starts_with("/usr/bin/"))
        || env
            .get("HOME")
            .is_some_and(|home| home.starts_with("/home/") || home == "/root")
}

fn release_asset_name(version: &str, target: TargetPlatform) -> String {
    format!(
        "pike-lsp-{version}-{arch}-{os}.{ext}",
        arch = match target.arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X86 => "x86",
            zed::Architecture::X8664 => "x86_64",
        },
        os = match target.os {
            zed::Os::Mac => "apple-darwin",
            zed::Os::Linux => "unknown-linux-gnu",
            zed::Os::Windows => "pc-windows-msvc",
        },
        ext = match target.os {
            zed::Os::Mac | zed::Os::Linux => "tar.gz",
            zed::Os::Windows => "zip",
        },
    )
}

fn downloaded_binary_path(version_dir: &str, os: zed::Os) -> String {
    match os {
        zed::Os::Windows => format!("{version_dir}/pike-lsp.exe"),
        zed::Os::Mac | zed::Os::Linux => format!("{version_dir}/pike-lsp"),
    }
}

fn downloaded_file_type(os: zed::Os) -> zed::DownloadedFileType {
    match os {
        zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
        zed::Os::Windows => zed::DownloadedFileType::Zip,
    }
}

fn stdio_command(binary: String) -> zed::Command {
    zed::Command {
        command: binary,
        args: vec!["stdio".to_string()],
        env: Default::default(),
    }
}

struct EnvLookup<'a>(&'a [(String, String)]);

impl EnvLookup<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }

    fn eq_ignore_ascii_case(&self, key: &str, expected: &str) -> bool {
        self.get(key)
            .is_some_and(|value| value.eq_ignore_ascii_case(expected))
    }
}

zed::register_extension!(PikeBridge);

#[cfg(test)]
mod tests {
    use super::*;

    fn env(vars: &[(&str, &str)]) -> Vec<(String, String)> {
        vars.iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn windows_host_linux_remote_selects_linux_asset_and_binary() {
        let target = infer_target_platform(
            zed::Os::Windows,
            zed::Architecture::X8664,
            "/home/dev/project",
            &env(&[("HOME", "/home/dev"), ("SHELL", "/bin/bash")]),
        );

        assert_eq!(target.os, zed::Os::Linux);
        assert_eq!(
            release_asset_name("0.0.2", target),
            "pike-lsp-0.0.2-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            downloaded_binary_path("pike-lsp-0.0.2", target.os),
            "pike-lsp-0.0.2/pike-lsp"
        );
    }

    #[test]
    fn windows_local_worktree_selects_windows_zip_and_exe() {
        let target = infer_target_platform(
            zed::Os::Windows,
            zed::Architecture::X8664,
            r"C:\Users\me\project",
            &env(&[
                ("OS", "Windows_NT"),
                ("COMSPEC", r"C:\Windows\System32\cmd.exe"),
            ]),
        );

        assert_eq!(target.os, zed::Os::Windows);
        assert_eq!(
            release_asset_name("0.0.2", target),
            "pike-lsp-0.0.2-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            downloaded_binary_path("pike-lsp-0.0.2", target.os),
            "pike-lsp-0.0.2/pike-lsp.exe"
        );
    }

    #[test]
    fn stdio_command_keeps_default_transport() {
        let command = stdio_command("/home/dev/.local/bin/pike-lsp".to_string());

        assert_eq!(command.command, "/home/dev/.local/bin/pike-lsp");
        assert_eq!(command.args, ["stdio"]);
        assert!(command.env.is_empty());
    }
}
