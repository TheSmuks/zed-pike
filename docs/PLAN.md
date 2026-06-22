# Zed Pike Extension Implementation Plan

> For Hermes: implement this with OpenSpec if the target repo has `openspec/config.yaml`; otherwise use this plan directly. Keep commits small and verify every step.

Goal: Build a Zed IDE extension for Pike. The first shippable milestone is syntax highlighting; later milestones add a modern LSP integration only after a reliable, editor-agnostic Pike language server exists.

Architecture: Zed language support is Tree-sitter-first for syntax highlighting, brackets, outline, indentation, and injections. Use the maintained `TheSmuks/tree-sitter-pike` grammar, pinned by commit in `extension.toml`. The VS Code extension at https://github.com/GwennKoi/vscode-pike-lang is only reference material for file associations, snippets, and legacy TextMate token coverage; do not port its TextMate grammar directly as the Zed syntax path. LSP support is explicitly out of the first milestone and should be a separate Rust/WASM extension layer later. Do not reference or build on previous Pike LSP attempts.

Tech stack:
- Zed extension manifest: `extension.toml`
- Zed language config: `languages/pike/config.toml`
- Syntax parsing/highlighting: `TheSmuks/tree-sitter-pike` + Zed `.scm` queries
- Later LSP bridge: Rust crate using `zed_extension_api`, backed by a new/verified modern Pike language server only
- Test fixtures: real Pike examples plus snippets adapted from the VS Code extension

Source findings:
- `vscode-pike-lang/package.json` registers language id `pike`, extensions `.pike` and `.pmod`, and grammar scope `source.pike`.
- Its TextMate grammar also lists `.cmod`; include `.cmod` in Zed unless the project decides to exclude it.
- `language-configuration.json` has line comments `//`, block comments `/* */`, and bracket pairs `{}`, `[]`, `()`, `<>`.
- `TheSmuks/tree-sitter-pike` is the maintained grammar; observed pinned commit during planning: `adacb8165dc9c7db9ca2f8d15fcb73b3c7ea8980`.
- `TheSmuks/tree-sitter-pike` reports Pike 8.0.1116 coverage, 210/210 corpus tests, and ships `queries/highlights.scm`, `queries/locals.scm`, and `queries/tags.scm`.

Repository shape:

```text
zed-pike/
  extension.toml
  README.md
  LICENSE
  languages/
    pike/
      config.toml
      highlights.scm
      brackets.scm
      indents.scm
      outline.scm
      injections.scm          # optional after MVP
  fixtures/
    syntax/
      basic.pike
      preprocessor.pike
      roxen_component.pike
  docs/
    PLAN.md
  Cargo.toml                  # add only when LSP integration starts
  src/
    lib.rs                    # add only when LSP integration starts
```

Phase 1: Syntax Highlighting MVP

Task 1: Create the Zed extension skeleton

Objective: Create a loadable Zed extension with Pike metadata and the maintained grammar pinned.

Files:
- Create: `extension.toml`
- Create: `languages/pike/config.toml`
- Create: `README.md`

Implementation:

```toml
# extension.toml
id = "pike"
name = "Pike"
description = "Pike language support for Zed."
version = "0.0.1"
schema_version = 1
authors = ["TheSmuks"]
repository = "https://github.com/TheSmuks/zed-pike"

[grammars.pike]
repository = "https://github.com/TheSmuks/tree-sitter-pike"
commit = "adacb8165dc9c7db9ca2f8d15fcb73b3c7ea8980"
```

```toml
# languages/pike/config.toml
name = "Pike"
grammar = "pike"
path_suffixes = ["pike", "pmod", "cmod"]
line_comments = ["// "]
tab_size = 2
autoclose_before = ";:.,=}])>"
brackets = [
  { start = "{", end = "}", close = true, newline = true },
  { start = "[", end = "]", close = true, newline = true },
  { start = "(", end = ")", close = true, newline = true },
  { start = "\"", end = "\"", close = true, newline = false, not_in = ["string", "comment"] },
  { start = "'", end = "'", close = true, newline = false, not_in = ["string", "comment"] },
]
```

Verification:
- Install as a Zed dev extension: `zed: install dev extension` and select the repo root.
- Expected: extension appears in Zed extensions UI.

Commit:

```bash
git add extension.toml languages/pike/config.toml README.md
git commit -m "feat: add Pike Zed extension skeleton"
```

Task 2: Seed highlighting from `tree-sitter-pike`

Objective: Use the maintained grammar’s query coverage as the starting point for Zed highlighting.

Files:
- Create: `languages/pike/highlights.scm` from `TheSmuks/tree-sitter-pike/queries/highlights.scm`
- Create: `fixtures/syntax/basic.pike`
- Create: `fixtures/syntax/preprocessor.pike`
- Create: `fixtures/syntax/roxen_component.pike`

Verification:
```bash
git clone https://github.com/TheSmuks/tree-sitter-pike /tmp/tree-sitter-pike
cd /tmp/tree-sitter-pike
bun install
bash scripts/generate.sh
bunx tree-sitter test
bunx tree-sitter parse /tank/projects/zed-pike/fixtures/syntax/basic.pike
```
Expected: grammar tests pass and the fixture parses without parser failure.

Commit:
```bash
git add languages/pike/highlights.scm fixtures/syntax
git commit -m "feat: add Pike highlight queries and fixtures"
```

Task 3: Add bracket matching, indentation, and outline queries

Objective: Make Pike feel native in Zed beyond token colors.

Files:
- Create: `languages/pike/brackets.scm`
- Create: `languages/pike/indents.scm`
- Create: `languages/pike/outline.scm`

Use node names from `TheSmuks/tree-sitter-pike/src/node-types.json`, not guessed names. Initial outline targets: `function_decl`, `class_decl`, `enum_decl`, `typedef_decl`.

Verification:
- Open fixture in Zed.
- Confirm bracket pairing, basic block indentation, and outline entries for classes/functions.
- Run Zed from terminal with `zed --foreground` if extension errors occur.

Commit:
```bash
git add languages/pike/brackets.scm languages/pike/indents.scm languages/pike/outline.scm
git commit -m "feat: add Pike editor queries"
```

Phase 2: Polish Syntax and UX

Task 4: Add fixture-driven query refinement

Objective: Keep the extension aligned with real Pike and Roxen code.

Files:
- Modify: `fixtures/syntax/*.pike`
- Modify: `languages/pike/*.scm`

Verification:
- Parse fixtures with `TheSmuks/tree-sitter-pike`.
- Open fixtures in Zed and inspect highlight/outline behavior.

Task 5: Add optional snippets

Objective: Port low-risk snippets from VS Code after syntax is stable.

Start with: `main`, `class`, `for`, `foreach`, `while`, `switch`, `lambda`.

Phase 3: Modern LSP Integration (future)

Rules:
- Do not build on or reference prior Pike LSP attempts.
- Treat LSP as a new/verified backend decision.
- Add Rust/WASM extension code only after syntax support is stable.
- Start with PATH discovery of a user-installed server. Add auto-download only after stable release assets exist.

Future acceptance criteria:
- Zed registers a Pike language server for Pike.
- The extension starts the server over stdio using documented arguments.
- Zed logs show successful initialization.
- At least one semantic feature is proven in Zed: diagnostics, hover, completion, go-to-definition, or symbols.

Acceptance criteria for milestone 1:
- Zed recognizes `.pike`, `.pmod`, and `.cmod` as Pike.
- Syntax highlighting uses `TheSmuks/tree-sitter-pike`.
- Highlighting covers comments, strings, numbers, keywords, types, modifiers, constants, preprocessor directives, declarations, operators, and punctuation.
- Bracket pairing works for `{}`, `[]`, `()`, quotes.
- Basic indentation works inside block-like nodes.
- The extension can be installed as a Zed dev extension without Rust code.
- README clearly states LSP is planned but not part of the first syntax-only milestone.

Recommended next action:
1. Keep `/tank/projects/zed-pike` as the Zed extension repo.
2. Validate current syntax-only extension in Zed as a dev extension.
3. Refine Zed queries against real Pike fixtures.
4. Only after syntax is solid, design a fresh modern LSP backend/bridge.
