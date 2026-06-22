## 1. Manifest and language config

- [x] 1.1 Confirm `extension.toml` registers `[grammars.pike]` with
  `repository = "https://github.com/TheSmuks/tree-sitter-pike"` and a
  40-character `commit` SHA, and that `extension.toml` has no
  `[language_servers.*]` table.
- [x] 1.2 Confirm `languages/pike/config.toml` sets `name = "Pike"`,
  `grammar = "pike"`, `path_suffixes = ["pike", "pmod", "cmod"]`,
  `line_comments = ["// "]`, `tab_size = 2`, and bracket auto-close
  entries for `{}`, `[]`, `()`, and both quote pairs with
  `not_in = ["string", "comment"]`.
- [x] 1.3 Confirm `extension.toml` is parseable as TOML.

## 2. Tree-sitter query files

- [x] 2.1 Seed `languages/pike/highlights.scm` from
  `TheSmuks/tree-sitter-pike/queries/highlights.scm` and verify every
  captured node name exists in the pinned grammar's
  `src/node-types.json`.
- [x] 2.2 Add `languages/pike/brackets.scm` covering `{}`, `[]`, and `()`.
- [x] 2.3 Add `languages/pike/indents.scm` that indents at block-shaped
  nodes and outdents at the matching close token.
- [x] 2.4 Add `languages/pike/outline.scm` that captures
  `function_decl`, `class_decl`, `enum_decl`, and `typedef_decl` with
  their `name: (identifier)` label.

## 3. Fixtures

- [x] 3.1 Add `fixtures/syntax/basic.pike` with a class, a function with
  parameters, a return statement, and a top-level `int main`.
- [x] 3.2 Add `fixtures/syntax/preprocessor.pike` with `#include`,
  `#define`, `#ifndef`, `#endif`, and `#warning`.
- [x] 3.3 Add `fixtures/syntax/roxen_component.pike` with `inherit`,
  `constant`, and a `string query_name()` function.
- [x] 3.4 Parse each fixture against the pinned grammar with
  `tree-sitter parse` and confirm the output contains no `ERROR` and
  no `MISSING` nodes.
- [x] 3.5 Verify `fixtures/syntax/preprocessor.pike` exercises the
  Pike 8.0.1116 preprocessor directive set documented in
  `pikelang/Pike/refdoc/preprocessor.xml`
  `<section title="Preprocessor Directives">` (at minimum `#ifndef`,
  `#define`, `#include`, `#endif`, plus `#warning` or `#error`).
- [x] 3.6 Verify the highlight query keyword/type/modifier token lists
  align with the Pike 8.0.1116 keywords, types, and modifiers documented
  in `pikelang/Pike/refdoc/declarations.xml` and
  `pikelang/Pike/refdoc/data_types.xml`.

## 4. Documentation

- [ ] 4.1 README documents how to install the extension as a Zed dev
  extension and how to validate the pinned grammar outside Zed using
  the upstream `bash scripts/generate.sh` and `bunx tree-sitter test`.
- [ ] 4.2 README states that LSP and semantic features are out of scope
  for this milestone.

## 5. OpenSpec change

- [ ] 5.1 `proposal.md` describes the why, what changes, capabilities,
  and impact, and lists the new `zed-pike-syntax` capability.
- [ ] 5.2 `specs/zed-pike-syntax/spec.md` defines the
  `ADDED Requirements` for syntax highlighting, brackets, indents,
  outline, fixtures, and the explicit no-LSP constraint.
- [ ] 5.3 `design.md` captures the decisions, risks, and open
  questions.
- [ ] 5.4 `tasks.md` (this file) lists the implementable tasks grouped
  by area.
- [ ] 5.5 Run `openspec validate zed-pike-syntax-mvp` and confirm it
  is clean.
