## Why

Pike currently has no first-class language support in the Zed editor. The
existing VS Code extension (https://github.com/GwennKoi/vscode-pike-lang) only
ships a TextMate grammar, which Zed does not consume on its normal
language-extension path. We need a Zed extension that uses the maintained
`TheSmuks/tree-sitter-pike` grammar (which targets Pike 8.0.1116 from
https://github.com/pikelang/Pike/releases/tag/v8.0.1116) so that Zed users
get fast, accurate syntax highlighting, outline, and indentation for
Pike on day one. The Pike 8.0.1116 `refdoc/` folder is the canonical
authority for what the language actually is and is what the extension's
queries and fixtures must respect. LSP integration is intentionally
deferred to a separate, future change so this milestone is small,
verifiable, and unblocks early Zed adoption.

## What Changes

- Add a Zed extension manifest (`extension.toml`) that pins the maintained
  tree-sitter Pike grammar by commit.
- Add a `languages/pike/config.toml` that registers Pike, `.pmod`, and
  `.cmod` files.
- Add Zed Tree-sitter query files for syntax highlighting, bracket
  matching, indentation, and outline, seeded from the upstream grammar's
  `queries/highlights.scm` and refined against the grammar's real node
  names.
- Ship a small set of Pike fixtures (`.pike` files) under `fixtures/syntax`
  that exercise the supported constructs.
- Document how to validate the pinned grammar outside Zed (clone, install,
  generate, test, parse) in the README.

## Capabilities

### New Capabilities

- `zed-pike-syntax`: Zed extension for Pike syntax highlighting, brackets,
  indentation, and outline, backed by `TheSmuks/tree-sitter-pike`. No
  language server is started; semantic features are explicitly out of
  scope for this change.

### Modified Capabilities

- None.

## Impact

- New files only: `extension.toml`, `languages/pike/**`, `fixtures/syntax/**`,
  `docs/PLAN.md`, and OpenSpec artifacts under
  `openspec/changes/zed-pike-syntax-mvp/`.
- No existing source files are modified.
- One new indirect dependency: the maintained tree-sitter Pike grammar at
  https://github.com/TheSmuks/tree-sitter-pike, pinned to commit
  `adacb8165dc9c7db9ca2f8d15fcb73b3c7ea8980` (MIT licensed). The grammar
  targets Pike 8.0.1116
  (https://github.com/pikelang/Pike/releases/tag/v8.0.1116, source commit
  `5d216a06d86bf36ec321fff3f82dfe80fd055194`); the canonical language
  reference is the `refdoc/` folder of
  https://github.com/pikelang/Pike (lexical grammar, preprocessor,
  control structures, declarations, data types, operators, BNF).
- Zed extensions are loaded from `extension.toml` plus a `languages/<id>/`
  tree, so registering this extension does not affect any other editor.
- Grammar upgrade strategy: bump the `commit` field in `extension.toml`,
  re-run the upstream test suite, re-parse the fixtures, and adjust
  queries if any node names change.
- Out of scope: LSP, semantic features, formatter integration, debugger
  integration, snippets, themes, and icon themes. Any LSP work must be a
  new OpenSpec change and must not reference or build on prior Pike LSP
  attempts.
