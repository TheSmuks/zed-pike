## ADDED Requirements

### Requirement: Tag-driven release pipeline publishes Linux and Windows pike-lsp artifacts

The repository MUST publish a tag-driven release pipeline that, on a
push of a tag matching `v*`, builds and uploads the host `pike-lsp`
binary for `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc`,
plus the `wasm32-wasip2` `zed-pike-bridge` module, to a GitHub release
on `TheSmuks/zed-pike`.

#### Scenario: Tag push creates a release with the expected assets
- **WHEN** a maintainer pushes the annotated tag `v<semver>` to
  `TheSmuks/zed-pike`
- **THEN** the release workflow creates (or updates) a GitHub release
  for that tag and attaches exactly three assets:
  `pike-lsp-<version>-x86_64-unknown-linux-gnu.tar.gz`,
  `pike-lsp-<version>-x86_64-pc-windows-msvc.zip`, and
  `zed-pike-bridge-<version>.wasm`.

#### Scenario: Release notes contain SHA-256 sums for every asset
- **WHEN** the release workflow completes successfully
- **THEN** the published release body MUST include a table that lists
  every attached asset and its SHA-256 sum, so a downstream consumer
  can verify what they downloaded.

### Requirement: CI builds the Windows host pike-lsp on every push and pull request

The CI workflow MUST build `pike-lsp` on `windows-latest` for every
push to a branch and every pull request targeting `main`, and MUST
upload the resulting `pike-lsp.exe` as a build artifact named
`pike-lsp-x86_64-pc-windows-msvc`.

#### Scenario: Pull request builds the Windows binary
- **WHEN** a pull request is opened against `main`
- **THEN** CI runs the Windows build job alongside the existing Linux
  build job and uploads `pike-lsp.exe` as an artifact, without
  publishing a release.

### Requirement: The bridge asset-name contract matches what the release pipeline publishes

The bridge in `crates/zed-pike-bridge/src/lib.rs` MUST map each
supported platform to the asset name that the release pipeline
publishes. Concretely: Linux → `unknown-linux-gnu` and `.tar.gz`,
Windows → `pc-windows-msvc` and `.zip`, macOS → `apple-darwin` and
`.tar.gz`.

#### Scenario: Windows bridge downloads the Windows zip from a release
- **WHEN** Zed is running on Windows and the bridge's auto-download
  path runs against a release whose tag is `v<semver>`
- **THEN** the bridge computes the asset name
  `pike-lsp-<version>-x86_64-pc-windows-msvc.zip`, finds it in the
  release, and downloads/extracts it into the version directory.