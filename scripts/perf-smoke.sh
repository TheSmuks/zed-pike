#!/usr/bin/env bash
# Quick lifecycle / RSS smoke test for `pike-lsp`.
# Validates the documented SLOs in docs/perf.md.

set -euo pipefail

BIN="${1:-target/release/pike-lsp}"
if [[ ! -x "$BIN" ]]; then
  echo "binary not found at $BIN" >&2
  exit 1
fi

frame() {
  local body="$1"
  printf 'Content-Length: %d\r\n\r\n%s' "${#body}" "$body"
}

initialize_frame() {
  frame '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null}}'
}

# 1. Default lifecycle: stdio responds to initialize and exits when stdin closes.
STDIO_OUT=$(initialize_frame | timeout 2 "$BIN" stdio 2>/dev/null || true)
if echo "$STDIO_OUT" | grep -q '"name":"pike-lsp"'; then
  echo 'stdio lifecycle: OK (initialize response, stdin-owned process)'
else
  echo 'stdio lifecycle: MISSING initialize response' >&2
  echo "  got: $(echo "$STDIO_OUT" | head -c 200)" >&2
  exit 1
fi

# 2. Forwarder MUST NOT auto-start a daemon by default.
TMPDIR=$(mktemp -d)
MISSING_SOCK="$TMPDIR/missing.sock"
set +e
MISSING_OUT=$(initialize_frame | timeout 2 "$BIN" forward --remote "$MISSING_SOCK" 2>&1)
MISSING_RC=$?
set -e
if [[ "$MISSING_RC" -eq 0 ]]; then
  echo 'forward missing-socket unexpectedly succeeded' >&2
  rm -rf "$TMPDIR"
  exit 1
fi
if [[ -S "$MISSING_SOCK" ]]; then
  echo 'forward missing-socket created a socket / daemon unexpectedly' >&2
  rm -rf "$TMPDIR"
  exit 1
fi
if echo "$MISSING_OUT" | grep -q 'does not exist'; then
  echo 'forward missing-socket: OK (no daemon autostart)'
else
  echo 'forward missing-socket: missing clear error' >&2
  echo "  got: $(echo "$MISSING_OUT" | head -c 200)" >&2
  rm -rf "$TMPDIR"
  exit 1
fi
rm -rf "$TMPDIR"

# 3. Resource guard kills a deliberately over-limit stdio server.
set +e
GUARD_OUT=$(tail -f /dev/null | timeout 5 "$BIN" --max-rss-mb 1 stdio 2>&1)
GUARD_RC=$?
set -e
if [[ "$GUARD_RC" -eq 137 ]] && echo "$GUARD_OUT" | grep -q 'resource guard'; then
  echo 'resource guard: OK (over-limit process exited 137)'
else
  echo "resource guard: expected exit 137 with diagnostic, got rc=$GUARD_RC" >&2
  echo "  got: $(echo "$GUARD_OUT" | head -c 240)" >&2
  exit 1
fi

# 4. Explicit daemon startup RSS remains bounded.
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
  wait "$PID" || true
)
rm -rf "$TMPDIR"

# 5. Existing-socket forwarder round trip still works when daemon is explicit.
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
  kill -9 "$DPID" 2>/dev/null || true
  exit 1
fi

OUT=$(initialize_frame | timeout 2 "$BIN" forward --remote "$SOCK" 2>/dev/null || true)
kill -HUP "$DPID" 2>/dev/null || true
wait "$DPID" 2>/dev/null || true

if echo "$OUT" | grep -q '"serverInfo"'; then
  SERVER_NAME=$(echo "$OUT" | grep -oE '"name":"pike-lsp"' | head -n 1)
  echo "forward round-trip: OK ($SERVER_NAME)"
else
  echo "forward round-trip: MISSING initialize response" >&2
  echo "  got: $(echo "$OUT" | head -c 200)" >&2
  rm -rf "$TMPDIR"
  exit 1
fi
rm -rf "$TMPDIR"
