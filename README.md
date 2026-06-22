# Zed Pike

Pike language support for Zed.

Current milestone: syntax highlighting via the maintained `TheSmuks/tree-sitter-pike` grammar.
Next milestone: optional LSP integration using a new, modern Pike language server when one is ready.

## Status

- Zed language config: present
- Tree-sitter grammar: pinned to `TheSmuks/tree-sitter-pike`
- Highlight queries: seeded from `TheSmuks/tree-sitter-pike/queries/highlights.scm`
- Pike LSP bridge: registered as `pike-lsp`, launched with stdio
- Windows host + Linux SSH remote: supported through Zed's worktree launch model

## Development

Install as a Zed dev extension:

1. Open Zed command palette.
2. Run `zed: install dev extension`.
3. Select this repository root.
4. Open a `.pike`, `.pmod`, or `.cmod` file.

### Local Windows and Linux SSH remote worktrees

Use Zed's built-in SSH remote workflow. The extension does not start `ssh` itself:

1. Connect with Zed to the Linux remote workspace from Windows, macOS, or Linux.
2. Open a Pike file from that remote worktree.
3. The bridge first checks the remote worktree PATH for `pike-lsp`.
4. If `pike-lsp` is not on PATH, the fallback selects the Linux release asset (`x86_64-unknown-linux-gnu.tar.gz`) for the remote worktree.

For local Windows worktrees, the fallback selects the Windows release asset (`x86_64-pc-windows-msvc.zip`) and runs `pike-lsp.exe`. The Zed extension itself remains a platform-neutral WASM bridge; only the `pike-lsp` executable is OS-native.

Validate the pinned grammar outside Zed:

```sh
git clone https://github.com/TheSmuks/tree-sitter-pike /tmp/tree-sitter-pike
cd /tmp/tree-sitter-pike
bun install
bash scripts/generate.sh
bunx tree-sitter test
bunx tree-sitter parse /path/to/zed-pike/fixtures/syntax/basic.pike
```

## Source material

- Syntax grammar: https://github.com/TheSmuks/tree-sitter-pike
- Initial VS Code/TextMate reference for file associations and snippets: https://github.com/GwennKoi/vscode-pike-lang
