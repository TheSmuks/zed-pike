#!/usr/bin/env bash
# Authoritative headless Zed-extension validation.
#
# Builds/uses Zed's official `zed-extension` CLI (the `extension_cli` crate from
# the Zed repo — the same tool the extensions registry CI runs) and validates
# THIS extension exactly the way Zed does on install:
#   - compile the wasm bridge (+ download wasi-sdk, compile the grammar, release)
#   - load each grammar .wasm into tree-sitter's WasmStore (real ABI check)
#   - compile every languages/**/*.scm query against the grammar
#   - package archive.tar.gz + manifest.json
#
# If this exits 0, the extension installs in Zed. This is the ground truth that
# scripts/verify.sh only approximates.
#
# The `zed-extension` binary is NOT shipped in the Zed app; it is built from the
# Zed source, pinned to the installed Zed's version so validation matches. The
# first run is a heavy build (~Zed dep subset incl. wasmtime); it is cached.
#
# Usage:
#   scripts/zed-extension-check.sh                 # build/cache tool, then check
#   ZED_REF=v1.7.2 scripts/zed-extension-check.sh  # pin the Zed source ref
#   ZED_EXTENSION_BIN=/path/to/zed-extension scripts/zed-extension-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/zed-extension-cli"
# Match the installed Zed version so validation semantics line up.
detect_ref() {
  local v
  v="$(zed --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)" || true
  [ -n "$v" ] && echo "v$v" || echo "main"
}
ZED_REF="${ZED_REF:-$(detect_ref)}"

find_bin() {
  if [ -n "${ZED_EXTENSION_BIN:-}" ] && [ -x "$ZED_EXTENSION_BIN" ]; then echo "$ZED_EXTENSION_BIN"; return; fi
  command -v zed-extension 2>/dev/null && return
  [ -x "$CACHE/bin/zed-extension" ] && echo "$CACHE/bin/zed-extension"
}

BIN="$(find_bin || true)"
if [ -z "$BIN" ]; then
  echo "==> Building Zed's zed-extension CLI (extension_cli @ $ZED_REF) — first run is slow, then cached"
  mkdir -p "$CACHE"
  # `cargo install` from the Zed git workspace; --locked uses Zed's Cargo.lock.
  cargo install \
    --git https://github.com/zed-industries/zed \
    --tag "$ZED_REF" \
    --locked \
    --root "$CACHE" \
    extension_cli 2>&1 | tail -5 || {
      echo "==> tag $ZED_REF not found; retrying against main" >&2
      cargo install --git https://github.com/zed-industries/zed --locked --root "$CACHE" extension_cli 2>&1 | tail -5
    }
  BIN="$CACHE/bin/zed-extension"
fi
echo "==> zed-extension: $BIN"

OUT="$(mktemp -d)"; SCRATCH="$(mktemp -d)"
trap 'rm -rf "$OUT" "$SCRATCH"' EXIT
echo "==> Validating $ROOT the way Zed does on install"
if "$BIN" --source-dir "$ROOT" --output-dir "$OUT" --scratch-dir "$SCRATCH"; then
  echo "ZED-EXTENSION CHECK: PASS — installs in Zed. Packaged: $(ls "$OUT")"
else
  rc=$?
  echo "ZED-EXTENSION CHECK: FAIL (exit $rc) — Zed would reject this on install." >&2
  exit "$rc"
fi
