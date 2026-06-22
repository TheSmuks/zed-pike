# Zed Pike

Pike language support for Zed.

Current milestone: syntax highlighting via the maintained `TheSmuks/tree-sitter-pike` grammar.
Next milestone: optional LSP integration using a new, modern Pike language server when one is ready.

## Status

- Zed language config: present
- Tree-sitter grammar: pinned to `TheSmuks/tree-sitter-pike`
- Highlight queries: seeded from `TheSmuks/tree-sitter-pike/queries/highlights.scm`
- LSP bridge: planned, not implemented yet

## Development

Install as a Zed dev extension:

1. Open Zed command palette.
2. Run `zed: install dev extension`.
3. Select this repository root.
4. Open a `.pike`, `.pmod`, or `.cmod` file.

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
