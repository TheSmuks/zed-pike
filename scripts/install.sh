#!/usr/bin/env bash
# Cross-platform (Linux/macOS) installer for the zed-pike dev extension.
#
# Strategy: build + verify + drive Zed. We do NOT poke Zed's internal extension
# database (that format is version-specific and fragile). Instead we:
#   1. Preflight the toolchain Zed needs to compile the extension.
#   2. Build the wasm bridge + pike-lsp and run the headless verify harness,
#      so a broken tree fails HERE, not silently inside Zed.
#   3. Hand off to Zed's own, supported dev-extension install (which downloads
#      wasi-sdk, compiles the grammar, and registers + rebuilds the extension).
#
# Usage:
#   scripts/install.sh              # build, verify, then open Zed for install
#   scripts/install.sh --no-open    # build + verify only; print the manual step
#   scripts/install.sh --skip-verify
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
OPEN=1; VERIFY=1
for a in "$@"; do case "$a" in
  --no-open) OPEN=0 ;;
  --skip-verify) VERIFY=0 ;;
  -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
esac; done

say() { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
die() { printf '\033[1;31mERROR:\033[0m %s\n' "$1" >&2; exit 1; }

# --- 1. preflight -----------------------------------------------------------
say "Preflight: toolchain"
command -v cargo >/dev/null || die "cargo not found — install Rust (https://rustup.rs)"
command -v rustc >/dev/null || die "rustc not found"
if ! rustup target list --installed 2>/dev/null | grep -q wasm32-wasip2; then
  say "Adding wasm32-wasip2 target"; rustup target add wasm32-wasip2
fi
ZED_BIN="$(command -v zed || true)"
[ -z "$ZED_BIN" ] && [ -x "$HOME/.local/bin/zed" ] && ZED_BIN="$HOME/.local/bin/zed"
echo "    cargo: $(cargo --version)"
echo "    zed:   ${ZED_BIN:-<not found on PATH>}"

# --- 2. build + verify ------------------------------------------------------
say "Build: wasm bridge (exact Zed build command)"
cargo build --release --target wasm32-wasip2
say "Build: pike-lsp (native language server)"
cargo build --release -p pike-lsp

if [ "$VERIFY" = 1 ]; then
  say "Verify: headless harness"
  ./scripts/verify.sh
else
  say "Verify: skipped (--skip-verify)"
fi

# --- 3. drive Zed's supported install --------------------------------------
cat <<EOF

$(printf '\033[1;32mArtifacts built and verified.\033[0m')

To install into Zed (one-time, then Zed auto-rebuilds on launch):
  1. Open the command palette (Cmd/Ctrl-Shift-P)
  2. Run:  zed: install dev extension
  3. Select this directory:  $ROOT
  4. Open a .pike / .pmod / .cmod file.

Tip: run Zed with logs visible to watch the compile/install:
  ${ZED_BIN:-zed} --foreground
EOF

if [ "$OPEN" = 1 ] && [ -n "$ZED_BIN" ]; then
  say "Opening Zed on the project (run the palette command above)"
  "$ZED_BIN" "$ROOT" >/dev/null 2>&1 &
fi
