## Context

`v0.0.1` shipped with a single host binary (`x86_64-unknown-linux-gnu`)
attached to the GitHub release. The bridge in
`crates/zed-pike-bridge/src/lib.rs` already maps Windows to
`pc-windows-msvc` and `.zip`, so the bridge's runtime behavior is
ready: it will look for `pike-lsp-<version>-x86_64-pc-windows-msvc.zip`
on a release. The release pipeline does not produce that asset, which
means a Windows user installing the extension today gets syntax
highlighting but no LSP, because the auto-download path cannot find a
matching asset.

The fix is two workflow edits and one new workflow file:

1. Add a `windows` job to `.github/workflows/ci.yml` so every PR builds
   and uploads `pike-lsp.exe` for the matrix, but does not publish.
2. Add `.github/workflows/release.yml` so pushing the `v*` tag triggers
   a release that bundles all three artifacts with SHA-256 sums in the
   notes.
3. Bump `extension.toml` and the workspace `Cargo.toml` version, and
   cut `v0.0.2` from a `release/0.0.2` branch.

## Goals / Non-Goals

**Goals:**
- Build `pike-lsp` on Linux and Windows in CI; build
  `zed-pike-bridge` on `wasm32-wasip2`.
- Publish a GitHub release with three assets:
  `pike-lsp-<version>-x86_64-unknown-linux-gnu.tar.gz`,
  `pike-lsp-<version>-x86_64-pc-windows-msvc.zip`, and
  `zed-pike-bridge-<version>.wasm`.
- Embed SHA-256 sums in the release body.
- Bump versions and ship `v0.0.2`.

**Non-Goals:**
- Cross-compiling Windows binaries from Linux runners; the Windows job
  runs on `windows-latest`.
- macOS or Linux ARM64 binaries. Tracked separately.
- Code signing the Windows `.exe`. Tracked separately.
- A separate installer (MSI/inno setup). Tracked separately.

## Decisions

1. **Linux host build stays in the existing `ci` workflow; Windows host
   build joins it.**
   - Rationale: the existing `build artifacts` job already covers
     Linux. Adding a parallel `windows` job keeps the matrix readable
     and lets either job fail without blocking the other.
   - Alternative rejected: a separate `build-windows.yml`. Splits the
     build signals across two places for no benefit.

2. **Release workflow is a new file (`release.yml`) triggered by tag
   push.**
   - Rationale: keeps `ci.yml` for PR-time checks and `release.yml` for
     tag-time publishing. The split avoids accidentally publishing from
     a PR.
   - Alternative rejected: extending `ci.yml` with a publish step on
     tag. Risk: accidentally publishing on a PR tag, harder to gate.

3. **SHA-256 sums are computed at packaging time and pasted into the
   release body via `gh release create --notes-file`.**
   - Rationale: keeps the bridge's existing asset-name contract the
     only consumer-facing contract; SHA-256 is a release-time artifact,
     not a build-time constraint.
   - Alternative rejected: a separate `checksums.txt` asset. Splitting
     sums and assets across files makes the contract harder to verify.

4. **The bridge code does not change.**
   - Rationale: the bridge already maps each platform to the right
     target triple and archive extension. This change only fills the
     missing artifacts on the release side.
   - Alternative rejected: rewrite the bridge to use a manifest. Premature
     for two platforms.

5. **`v0.0.2` is cut from a `release/0.0.2` branch off `main`, not
   directly from `main`.**
   - Rationale: keeps the merge history linear and gives a place to add
     last-minute release notes before tagging.
   - Alternative rejected: tag `main` directly. Faster, but no
     per-release branch to amend.

## Risks / Trade-offs

- **Windows runner has higher latency and a cold cache** → cache the
  Cargo target dir keyed by OS + Rust toolchain (`Swatinem/rust-cache@v2`
  already supports this; no extra config).
- **The release workflow uploads to GitHub, which needs
  `permissions: contents: write`** → the workflow declares
  `permissions: contents: write` at the top level.
- **Tag retag policy** → if a maintainer re-tags, `gh release create`
  exits non-zero. The workflow uses `--fail-on-no-changes` only for
  draft re-runs; the first publish uses `--title` + `--notes-file`.
- **Bridge asset-name drift** → the release pipeline writes the exact
  strings the bridge expects. A change to either side without the other
  is a release-time bug; both are pinned in
  `openspec/specs/release-pipeline/spec.md`.

## Migration Plan

1. Land the new `windows` CI job and the `release.yml` workflow on a
   feature branch via PR.
2. Cut `release/0.0.2` from `main`, bump versions, push the branch.
3. Push the `v0.0.2` tag from the release branch.
4. `release.yml` builds the artifacts and creates the GitHub release.
5. Update `CHANGELOG.md` `Unreleased` and add a `0.0.2` section.
6. Rollback: delete the tag locally and remotely, then `gh release
   delete v0.0.2 --yes`. The CI workflow does not need a rollback; the
   new jobs are additive.

## Open Questions

- None. `extension.toml`'s `version` and the workspace `Cargo.toml`'s
  `version` are the only two numeric fields that need to move from
  `0.0.1` to `0.0.2`; the bridge's CLI version (`pike-lsp --version`)
  is sourced from `CARGO_PKG_VERSION` and tracks the workspace version.