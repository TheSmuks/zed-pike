## Why

The `v0.0.1` release of `zed-pike` only attached a Linux `pike-lsp` tarball.
Users on Windows cannot run the language server today: the bridge can
auto-download `pike-lsp`, but the release pipeline does not produce a
Windows binary for it to download. The extension itself (Tree-sitter
syntax, brackets, outline) works on Windows; semantic features do not.
Shipping a `pike-lsp-x86_64-pc-windows-msvc.zip` artifact closes that gap.

## What Changes

- Add a `windows` build job to `.github/workflows/ci.yml` that builds
  `pike-lsp` on `windows-latest` and uploads the resulting `pike-lsp.exe`.
- Add a new `.github/workflows/release.yml` triggered by a `v*` tag push
  that:
  - Builds `pike-lsp` on Linux (`x86_64-unknown-linux-gnu`) and Windows
    (`x86_64-pc-windows-msvc`).
  - Builds `zed-pike-bridge` on Linux (`wasm32-wasip2`).
  - Packages all three artifacts under `out/`.
  - Computes SHA-256 sums and embeds them in the release notes.
  - Creates a GitHub release on `TheSmuks/zed-pike` with the artifacts
    attached.
- Update `crates/zed-pike-bridge/src/lib.rs` if needed so the bridge's
  per-platform asset naming matches what `release.yml` publishes (already
  maps `Windows → pc-windows-msvc`, `.zip`).
- Update `CHANGELOG.md` `Unreleased` and add a `0.0.2` section.
- Bump `extension.toml` `version = "0.0.2"` and the workspace package
  version in `Cargo.toml`.

## Capabilities

### New Capabilities

- `release-pipeline`: tag-driven GitHub Actions workflow that builds and
  publishes platform artifacts on `TheSmuks/zed-pike`.

### Modified Capabilities

- `zed-pike-bridge`: no requirement change, but the bridge's
  per-platform asset-name template becomes a supported contract; the
  release pipeline publishes exactly the names the bridge expects.

## Impact

- `.github/workflows/ci.yml`: one new `windows` job.
- `.github/workflows/release.yml`: new file.
- `crates/zed-pike-bridge/src/lib.rs`: only inspected; no code change
  expected.
- `CHANGELOG.md`: `Unreleased` entry and a `0.0.2` section.
- `extension.toml`: `version = "0.0.2"`.
- `Cargo.toml`: workspace `version = "0.0.2"`.
- New tag `v0.0.2` pushed to `TheSmuks/zed-pike`.