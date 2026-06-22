# zed-pike-syntax Specification

## Purpose
TBD - created by archiving change zed-pike-syntax-mvp. Update Purpose after archive.
## Requirements
### Requirement: Zed registers Pike as a supported language
The Zed extension SHALL register Pike as a language in `extension.toml` and
MUST publish a `languages/pike/config.toml` whose `grammar` field references
the grammar named in the extension's `[grammars]` section.

#### Scenario: Zed opens a `.pike` file as Pike
- **WHEN** a user opens `foo.pike` in Zed with the extension installed as a
  dev extension
- **THEN** Zed reports the buffer language as Pike and the syntax
  highlight query file is loaded for the buffer.

#### Scenario: Zed opens a `.pmod` and `.cmod` file as Pike
- **WHEN** a user opens `foo.pmod` or `foo.cmod` in Zed with the extension
  installed
- **THEN** Zed reports the buffer language as Pike.

### Requirement: The extension pins a known-good tree-sitter grammar
The `extension.toml` `[grammars.pike]` entry MUST pin the maintained
`TheSmuks/tree-sitter-pike` repository to a specific commit SHA. The
`languages/pike/config.toml` `grammar` field MUST match the grammar name
declared in `extension.toml`.

#### Scenario: Extension manifest references the pinned grammar
- **WHEN** `extension.toml` is read by Zed
- **THEN** the `[grammars.pike]` table has a `repository` pointing at
  `https://github.com/TheSmuks/tree-sitter-pike` and a `commit` value that
  is a 40-character hex SHA.

### Requirement: Syntax highlighting is sourced from Tree-sitter queries
The extension MUST provide `languages/pike/highlights.scm` whose predicates
match the named nodes produced by the pinned grammar, and the extension
MUST NOT include any Rust/WASM code for this milestone.

#### Scenario: Highlight query references real grammar node names
- **WHEN** `languages/pike/highlights.scm` is loaded
- **THEN** every captured node name (for example `comment`,
  `string_literal`, `number_literal`, `function_decl`,
  `class_decl`, `identifier`) corresponds to a named node in the pinned
  grammar's `src/node-types.json`.

#### Scenario: Pinned grammar passes its own corpus tests
- **WHEN** the pinned grammar is generated and tested with the upstream
  test command
- **THEN** the test suite reports 0 failed parses.

### Requirement: Bracket matching and basic indentation are configured
The extension MUST provide `languages/pike/brackets.scm` covering the
bracket pairs `{}`, `[]`, and `()`, and MUST provide
`languages/pike/indents.scm` that indents content inside block-like nodes
and de-indents at their closing punctuation.

#### Scenario: Typing a block produces paired brackets
- **WHEN** a user types `{` in a Pike buffer
- **THEN** Zed auto-inserts a matching `}` and positions the cursor
  between them.

### Requirement: Outline surfaces Pike declarations
The extension MUST provide `languages/pike/outline.scm` that captures
top-level declarations of `function_decl`, `class_decl`, `enum_decl`, and
`typedef_decl` with their identifier name as the outline label.

#### Scenario: Outline lists a class and a function
- **WHEN** a fixture file contains `class Demo { ... }` and
  `int main() { ... }`
- **THEN** Zed's outline panel shows `Demo` and `main` as items.

### Requirement: Repository ships parseable Pike fixtures
The repository MUST include at least three `.pike` files under
`fixtures/syntax/` that the pinned grammar parses without producing any
`ERROR` or `MISSING` nodes, and at least one of them MUST exercise the
Pike 8.0.1116 preprocessor directive set documented in
`pikelang/Pike/refdoc/preprocessor.xml` (for example `#if`, `#ifdef`,
`#ifndef`, `#endif`, `#else`, `#elif`, `#define`, `#undef`, `#include`,
`#warning`, `#error`, `#pragma`).

#### Scenario: Fixtures parse cleanly with the pinned grammar
- **WHEN** each fixture in `fixtures/syntax/*.pike` is parsed with
  `tree-sitter parse` against the pinned grammar
- **THEN** the parse output contains no `ERROR` and no `MISSING` nodes.

#### Scenario: Preprocessor fixture uses Pike 8.0.1116 directives
- **WHEN** `fixtures/syntax/preprocessor.pike` is opened
- **THEN** it contains at least one occurrence of `#ifndef`, `#define`,
  `#include`, and `#endif`, and at least one of `#warning` or `#error`,
  matching the directives enumerated in `refdoc/preprocessor.xml`
  `<section title="Preprocessor Directives">`.

### Requirement: LSP and semantic features are out of scope
The extension MUST NOT declare any `[language_servers]` section in
`extension.toml` and MUST NOT include `Cargo.toml` or `src/lib.rs` for
this milestone. Any future LSP work SHALL be introduced as a separate
OpenSpec change and MUST NOT reference prior Pike LSP attempts.

#### Scenario: Manifest has no language server declaration
- **WHEN** `extension.toml` is read
- **THEN** it does not contain any `[language_servers.*]` table.

