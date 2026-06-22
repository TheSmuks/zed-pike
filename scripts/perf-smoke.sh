#!/usr/bin/env bash
# Quick RSS / startup smoke test for `pike-lsp`.
# Validates the documented SLOs in docs/perf.md.

set -euo pipefail

BIN="${1:-target/release/pike-lsp}"
if [[ ! -x "$BIN" ]]; then
  echo "binary not found at $BIN" >&2
  exit 1
fi

# 1. Startup RSS for a stdio server that does nothing.
(
  # We open stdio and immediately ask the server to shut down.
  printf '{"jsonrpc":"2.0","id":1,"method":"shutdown","params":null}\n{"jsonrpc":"2.0","method":"exit","params":null}\n' \
    | "$BIN" stdio >/dev/null 2>&1 &
  PID=$!
  sleep 0.05
  if [[ -d "/proc/$PID" ]]; then
    RSS_KB=$(awk '/VmRSS/ { print $2 }' "/proc/$PID/status" 2>/dev/null || echo 0)
    echo "stdio RSS at start: ${RSS_KB} KiB"
    kill -9 "$PID" 2>/dev/null || true
  fi
)

# 2. Daemon startup RSS.
TMPDIR=$(mktemp -d)
SOCK="$TMPDIR/pike-lsp.sock"
(
  "$BIN" daemon --socket "$SOCK" --idle-timeout 5 &
  PID=$!
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    [[ -S "$SOCK" ]] && break
    sleep 0.1
  done
  if [[ -S "$SOCK" ]]; then
    RSS_KB=$(awk '/VmRSS/ { print $2 }' "/proc/$PID/status" 2>/dev/null || echo 0)
    echo "daemon RSS at idle: ${RSS_KB} KiB"
  else
    echo "daemon failed to listen" >&2
    kill -9 "$PID" 2>/dev/null || true
    exit 1
  fi
  # Let idle-timeout close it.
  wait "$PID" || true
)
rm -rf "$TMPDIR"

# 3. Forward round-trip smoke.
TMPDIR=$(mktemp -d)
SOCK="$TMPDIR/pike-lsp.sock"
"$BIN" daemon --socket "$SOCK" --idle-timeout 5 &
DPID=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
if [[ -S "$SOCK" ]]; then
  (
    printf '%s' \
      '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null}}' \
      '{"jsonrpc":"2.0","method":"initialized","params":{}}' \
      '{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}' \
      '{"jsonrpc":"2.0","method":"exit","params":null}' \
      | timeout 2 "$BIN" forward --remote "$SOCK" 2>/dev/null
  )
  echo "forward round-trip exit code: $?"
  kill -INT "$DPID" 2>/dev/null || true
  wait "$DPID" 2>/dev/null || true
else
  echo "daemon failed to listen" >&2
  exit 1
fi
rm -rf "$TMPDIR"
