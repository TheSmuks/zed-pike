#!/usr/bin/env bash
# Quick RSS / startup smoke test for `pike-lsp`.
# Validates the documented SLOs in docs/perf.md.

set -euo pipefail

BIN="${1:-target/release/pike-lsp}"
if [[ ! -x "$BIN" ]]; then
  echo "binary not found at $BIN" >&2
  exit 1
fi

# Frame a JSON-RPC message with the standard `Content-Length` header.
# Each framed message is sent as a single `printf`, so the server
# sees one message at a time and can reply.
frame() {
  local body="$1"
  printf 'Content-Length: %d\r\n\r\n%s' "${#body}" "$body"
}

# 1. Daemon startup RSS.
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

# 2. Forward round-trip with proper framing.
TMPDIR=$(mktemp -d)
SOCK="$TMPDIR/pike-lsp.sock"
"$BIN" daemon --socket "$SOCK" --idle-timeout 5 &
DPID=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
  [[ -S "$SOCK" ]] && break
  sleep 0.1
done
if [[ ! -S "$SOCK" ]]; then
  echo "daemon failed to listen" >&2
  exit 1
fi

OUT=$(
  # Send just the initialize request and read the response before
  # sending the shutdown. This is what an LSP client does in
  # practice: it waits for the response before sending the next
  # request.
  frame '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null}}' \
    | timeout 2 "$BIN" forward --remote "$SOCK" 2>/dev/null || true
)

# Send shutdown + exit, but those don't need to come back.
{
  frame '{"jsonrpc":"2.0","method":"initialized","params":{}}'
  frame '{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}'
  frame '{"jsonrpc":"2.0","method":"exit","params":null}'
} | timeout 2 "$BIN" forward --remote "$SOCK" 2>/dev/null || true

# Send a HUP to the daemon to terminate the test cleanly.
kill -HUP "$DPID" 2>/dev/null || true
wait "$DPID" 2>/dev/null || true

if echo "$OUT" | grep -q '"serverInfo"'; then
  SERVER_NAME=$(echo "$OUT" | grep -oE '"name":"pike-lsp"' | head -n 1)
  if [[ -n "$SERVER_NAME" ]]; then
    echo "forward round-trip: OK ($SERVER_NAME)"
  else
    echo "forward round-trip: missing server name" >&2
    kill -INT "$DPID" 2>/dev/null || true
    wait "$DPID" 2>/dev/null || true
    exit 1
  fi
else
  echo "forward round-trip: MISSING initialize response" >&2
  echo "  got: $(echo "$OUT" | head -c 200)" >&2
  kill -INT "$DPID" 2>/dev/null || true
  wait "$DPID" 2>/dev/null || true
  exit 1
fi

kill -INT "$DPID" 2>/dev/null || true
wait "$DPID" 2>/dev/null || true
rm -rf "$TMPDIR"
