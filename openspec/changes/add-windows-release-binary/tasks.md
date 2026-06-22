## 1. CI Windows build job

- [ ] 1.1 Add a `windows` job to `.github/workflows/ci.yml` that runs on
  `windows-latest`, installs the Rust stable toolchain via `dtolnay/rust-toolchain@stable`,
  adds the `x86_64-pc-windows-msvc` target, and runs
  `cargo build -p pike-lsp --release`.
- [ ] 1.2 In the same job, upload `target/release/pike-lsp.exe` as a
  build artifact named `pike-lsp-x86_64-pc-windows-msvc`.
- [ ] 1.3 Verify by opening a PR: the new job appears in the CI run and
  uploads the Windows `.exe` artifact successfully.

## 2. Release workflow

- [ ] 2.1 Create `.github/workflows/release.yml` triggered by tag push
  matching `v*`, with `permissions: contents: write`.
- [ ] 2.2 Add a Linux build step in `release.yml`:
  `cargo build -p pike-lsp --release` and
  `cargo build -p zed-pike-bridge --target wasm32-wasip2 --release`.
- [ ] 2.3 Add a Windows build step in `release.yml`:
  `cargo build -p pike-lsp --release` on `windows-latest`.
- [ ] 2.4 In `release.yml`, after builds complete, package the Linux
  artifacts into `out/pike-lsp-<version>-x86_64-unknown-linux-gnu.tar.gz`
  and `out/zed-pike-bridge-<version>.wasm`. Compute SHA-256 sums.
- [ ] 2.5 Package the Windows artifact into
  `out/pike-lsp-<version>-x86_64-pc-windows-msvc.zip` and add its
  SHA-256 sum.
- [ ] 2.6 Generate a notes file from the SHA-256 sums and asset names,
  then call `gh release create "$TAG" --repo TheSmuks/zed-pike --title
  "$TAG" --notes-file out/RELEASE.md -- <assets>`.

## 3. Version bump and changelog

- [ ] 3.1 Bump `extension.toml` `version = "0.0.2"`.
- [ ] 3.2 Bump workspace `version = "0.0.2"` in `Cargo.toml`.
- [ ] 3.3 Add a `0.0.2` section to `CHANGELOG.md` summarizing the
  Windows binary and the new release workflow, and move the relevant
  `Unreleased` bullets into it.
- [ ] 3.4 Update the `[Unreleased]` compare link at the bottom of
  `CHANGELOG.md` to point at the new tag.

## 4. Bridge asset-name audit

- [ ] 4.1 Confirm `crates/zed-pike-bridge/src/lib.rs` already maps
  `zed::Os::Windows` to `pc-windows-msvc` and `.zip`. No change
  expected; this is a checkpoint.
- [ ] 4.2 If the bridge needs to be touched, keep the change minimal and
  add a unit test asserting the asset-name contract.

## 5. Release cut

- [ ] 5.1 Cut a `release/0.0.2` branch from `main` after the PR from
  Task 1 lands.
- [ ] 5.2 Push the annotated tag `v0.0.2` to `TheSmuks/zed-pike`.
- [ ] 5.3 Confirm the release workflow runs end-to-end and the
  `v0.0.2` release page lists all three assets with their SHA-256 sums
  in the notes.
- [ ] 5.4 If the release fails, fix forward; do not delete the tag
  unless the user explicitly asks.