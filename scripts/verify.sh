#!/usr/bin/env bash
# Headless verification harness for the zed-pike extension — "coding eyes".
#
# Reproduces the checks Zed performs at dev-extension install time, plus a real
# LSP handshake, WITHOUT launching the Zed GUI. Every stage is fail-fast and
# prints what it verified. Exit 0 == the extension is sound to install.
#
# Stages:
#   1. Manifests      extension.toml + languages/pike/config.toml parse; TOML trap
#   2. Bridge (wasm)  cargo build --target wasm32-wasip2 (exact Zed cmd) + validate
#                     the component embeds a released zed_extension_api version and
#                     exports `init-extension`
#   3. LSP            build pike-lsp + drive initialize/didOpen/hover/documentSymbol
#                     over stdio; enforce the RSS budget from docs/perf.md
#   4. Grammar        (best-effort) tree-sitter parse of fixtures/ if tooling present
#
# Usage: scripts/verify.sh [--no-lsp] [--no-grammar]
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RSS_BUDGET_MB="${PIKE_LSP_MAX_RSS_MB:-80}"   # docs/perf.md: idle stdio < 80 MiB
DO_LSP=1; DO_GRAMMAR=1
for a in "$@"; do
  case "$a" in
    --no-lsp) DO_LSP=0 ;;
    --no-grammar) DO_GRAMMAR=0 ;;
  esac
done

pass() { printf '  \033[32m✓\033[0m %s\n' "$1"; }
fail() { printf '  \033[31m✗\033[0m %s\n' "$1"; FAILED=1; }
stage() { printf '\n\033[1m[%s] %s\033[0m\n' "$1" "$2"; }
FAILED=0

# ---------------------------------------------------------------------------
stage 1 "Manifests"
python3 - <<'PY' && pass "extension.toml + config.toml valid; bracket quotes correct" || fail "manifest validation failed"
import sys, tomllib
try:
    ext = tomllib.load(open("extension.toml", "rb"))
    assert ext.get("id"), "extension.toml missing id"
    assert "pike" in ext.get("grammars", {}), "no [grammars.pike]"
    cfg = tomllib.load(open("languages/pike/config.toml", "rb"))
    quotes = [b for b in cfg.get("brackets", [])
              if b.get("start") in ('"', "'") and len(b.get("start","")) == 1]
    assert len(quotes) == 2, f"TOML quote trap: expected 2 single-char quote brackets, got {len(quotes)}"
except Exception as e:
    print(f"    {e}", file=sys.stderr); sys.exit(1)
PY

# ---------------------------------------------------------------------------
stage 2 "Bridge (wasm32-wasip2)"
if cargo build --release --target wasm32-wasip2 >/tmp/zpk_wasm.log 2>&1; then
  pass "cargo build --release --target wasm32-wasip2 (exact Zed build command)"
else
  fail "wasm build failed"; tail -20 /tmp/zpk_wasm.log
fi
WASM="target/wasm32-wasip2/release/zed_pike_bridge.wasm"
if [ -f "$WASM" ]; then
  # Note: avoid `grep -q`/`head` here — they short-circuit and SIGPIPE `strings`,
  # which under `set -o pipefail` would falsely fail the pipeline. Use grep -c.
  syms="$(strings -n 5 "$WASM")"
  api="$(printf '%s\n' "$syms" | grep -oE 'zed_extension_api-[0-9]+\.[0-9]+\.[0-9]+' | sort -u | tr '\n' ' ')"
  if [ -n "$api" ]; then pass "component built against ${api% }"; else fail "no zed_extension_api version marker in wasm"; fi
  if [ "$(printf '%s\n' "$syms" | grep -c 'init-extension')" -gt 0 ]; then pass "exports init-extension"; else fail "missing init-extension export"; fi
else
  fail "wasm artifact not produced at $WASM"
fi

# ---------------------------------------------------------------------------
if [ "$DO_LSP" = 1 ]; then
  stage 3 "LSP (headless stdio handshake)"
  if cargo build --release -p pike-lsp >/tmp/zpk_lsp.log 2>&1; then
    pass "cargo build --release -p pike-lsp"
    BIN="target/release/pike-lsp"
    FIX="fixtures/syntax/basic.pike"
    if python3 scripts/lsp_smoke.py "$BIN" "$FIX" --max-rss-mb "$RSS_BUDGET_MB"; then
      pass "LSP handshake + RSS budget (<= ${RSS_BUDGET_MB} MiB)"
    else
      fail "LSP smoke test failed"
    fi
  else
    fail "pike-lsp build failed"; tail -20 /tmp/zpk_lsp.log
  fi
else
  stage 3 "LSP (skipped: --no-lsp)"
fi

# ---------------------------------------------------------------------------
if [ "$DO_GRAMMAR" = 1 ]; then
  stage 4 "Grammar (best-effort)"
  GDIR=""
  for c in ../tree-sitter-pike ./tree-sitter-pike; do
    [ -f "$c/src/parser.c" ] && GDIR="$c" && break
  done
  if [ -z "$GDIR" ]; then
    pass "skipped: no local tree-sitter-pike checkout with generated src/ found"
  elif ! command -v tree-sitter >/dev/null 2>&1; then
    pass "skipped: tree-sitter CLI not on PATH"
  else
    ok=1
    for f in fixtures/syntax/*.pike; do
      if tree-sitter parse -q "$f" >/dev/null 2>&1 \
         || (cd "$GDIR" && tree-sitter parse -q "$ROOT/$f" >/dev/null 2>&1); then :; else ok=0; echo "    parse failed: $f"; fi
    done
    [ "$ok" = 1 ] && pass "tree-sitter parsed all fixtures/syntax/*.pike" || fail "grammar failed to parse some fixtures"
  fi
else
  stage 4 "Grammar (skipped: --no-grammar)"
fi

# ---------------------------------------------------------------------------
echo
if [ "$FAILED" = 0 ]; then
  printf '\033[1;32mVERIFY: PASS\033[0m — extension is sound to install.\n'
  exit 0
else
  printf '\033[1;31mVERIFY: FAIL\033[0m — see failures above.\n'
  exit 1
fi
