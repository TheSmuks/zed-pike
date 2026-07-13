#!/usr/bin/env python3
"""Headless LSP smoke test for pike-lsp.

Speaks LSP 3.17 over stdio to a `pike-lsp stdio` process and drives a real
initialize -> didOpen -> hover -> shutdown handshake, so an agent (or CI) can
verify the language server actually works without opening Zed.

Usage:
    lsp_smoke.py <pike-lsp-binary> <fixture.pike> [--max-rss-mb N]

Exits 0 on success, non-zero (with a diagnostic on stderr) otherwise.
On Linux it also reports peak VmRSS and enforces --max-rss-mb if given.
"""
import json
import os
import subprocess
import sys
import threading
import time
from pathlib import Path


def frame(payload: dict) -> bytes:
    body = json.dumps(payload).encode("utf-8")
    return f"Content-Length: {len(body)}\r\n\r\n".encode("ascii") + body


class Client:
    def __init__(self, proc: subprocess.Popen):
        self.proc = proc
        self._id = 0
        self._buf = b""

    def send(self, method: str, params=None, is_request=True):
        msg = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        if is_request:
            self._id += 1
            msg["id"] = self._id
            rid = self._id
        else:
            rid = None
        self.proc.stdin.write(frame(msg))
        self.proc.stdin.flush()
        return rid

    def _read_message(self, timeout=10.0):
        deadline = time.time() + timeout
        # read headers
        while b"\r\n\r\n" not in self._buf:
            if time.time() > deadline:
                raise TimeoutError("timed out waiting for LSP header")
            chunk = self.proc.stdout.read1(4096) if hasattr(self.proc.stdout, "read1") else self.proc.stdout.read(1)
            if not chunk:
                raise EOFError("server closed stdout")
            self._buf += chunk
        header, _, rest = self._buf.partition(b"\r\n\r\n")
        length = 0
        for line in header.split(b"\r\n"):
            if line.lower().startswith(b"content-length:"):
                length = int(line.split(b":", 1)[1].strip())
        self._buf = rest
        while len(self._buf) < length:
            if time.time() > deadline:
                raise TimeoutError("timed out reading LSP body")
            chunk = self.proc.stdout.read1(4096) if hasattr(self.proc.stdout, "read1") else self.proc.stdout.read(length - len(self._buf))
            if not chunk:
                raise EOFError("server closed stdout mid-body")
            self._buf += chunk
        body, self._buf = self._buf[:length], self._buf[length:]
        return json.loads(body.decode("utf-8"))

    def await_response(self, rid: int, timeout=10.0):
        """Read messages until we see the response to rid (skipping notifications)."""
        deadline = time.time() + timeout
        while True:
            msg = self._read_message(timeout=max(0.1, deadline - time.time()))
            if msg.get("id") == rid and ("result" in msg or "error" in msg):
                return msg


def peak_rss_kib(pid: int):
    try:
        with open(f"/proc/{pid}/status") as f:
            for line in f:
                if line.startswith("VmHWM:"):
                    return int(line.split()[1])
    except OSError:
        return None
    return None


def main():
    args = sys.argv[1:]
    max_rss_mb = None
    if "--max-rss-mb" in args:
        i = args.index("--max-rss-mb")
        max_rss_mb = int(args[i + 1])
        del args[i:i + 2]
    if len(args) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    binary, fixture = args
    fixture_path = Path(fixture).resolve()
    if not fixture_path.is_file():
        print(f"FAIL: fixture not found: {fixture_path}", file=sys.stderr)
        return 2
    text = fixture_path.read_text()
    uri = fixture_path.as_uri()

    proc = subprocess.Popen(
        [binary, "stdio"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env={**os.environ, "PIKE_LSP_LOG": "warn"},
    )

    # Drain stderr in the background so tracing output never blocks the pipe.
    stderr_lines = []
    def drain():
        for line in proc.stderr:
            stderr_lines.append(line.decode("utf-8", "replace").rstrip())
    threading.Thread(target=drain, daemon=True).start()

    client = Client(proc)
    failures = []
    try:
        rid = client.send("initialize", {
            "processId": os.getpid(),
            "rootUri": fixture_path.parent.as_uri(),
            "capabilities": {},
        })
        resp = client.await_response(rid)
        caps = resp.get("result", {}).get("capabilities", {})
        wanted = ["hoverProvider", "definitionProvider", "referencesProvider",
                  "completionProvider", "documentSymbolProvider"]
        missing = [c for c in wanted if c not in caps]
        if missing:
            failures.append(f"initialize missing capabilities: {missing}")
        server_info = resp.get("result", {}).get("serverInfo", {})
        print(f"  initialize OK: server={server_info.get('name')} "
              f"v{server_info.get('version')} caps={sorted(caps.keys())}")

        client.send("initialized", {}, is_request=False)
        client.send("textDocument/didOpen", {
            "textDocument": {"uri": uri, "languageId": "pike",
                             "version": 1, "text": text},
        }, is_request=False)

        # Hover somewhere inside the document (line 6 col 20 in basic.pike ~ code).
        rid = client.send("textDocument/hover", {
            "textDocument": {"uri": uri},
            "position": {"line": 6, "character": 12},
        })
        hov = client.await_response(rid)
        if "error" in hov:
            failures.append(f"hover returned error: {hov['error']}")
        else:
            print(f"  hover OK (result present={hov.get('result') is not None})")

        rid = client.send("textDocument/documentSymbol", {
            "textDocument": {"uri": uri},
        })
        sym = client.await_response(rid)
        if "error" in sym:
            failures.append(f"documentSymbol returned error: {sym['error']}")
        else:
            n = len(sym.get("result") or [])
            print(f"  documentSymbol OK ({n} symbols)")

        rid = client.send("shutdown")
        client.await_response(rid)
        client.send("exit", is_request=False)
    except (TimeoutError, EOFError, BrokenPipeError) as e:
        failures.append(f"transport error: {e}")
    finally:
        try:
            proc.stdin.close()
        except Exception:
            pass
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()

    rss = peak_rss_kib(proc.pid)
    if rss is not None:
        rss_mb = rss / 1024
        print(f"  peak RSS: {rss_mb:.1f} MiB")
        if max_rss_mb is not None and rss_mb > max_rss_mb:
            failures.append(f"peak RSS {rss_mb:.1f} MiB exceeds budget {max_rss_mb} MiB")

    if stderr_lines:
        tail = "\n    ".join(stderr_lines[-8:])
        print(f"  server stderr (tail):\n    {tail}")

    if failures:
        for f in failures:
            print(f"FAIL: {f}", file=sys.stderr)
        return 1
    print("  LSP smoke: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
