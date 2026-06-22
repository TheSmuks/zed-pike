# pike-lsp-analysis

## ADDED Requirements

### Requirement: The analysis layer is incremental
The Pike LSP analysis layer SHALL use an on-demand incremental
computation engine, with each query tagged with a durability tier
of `durable`, `normal`, or `volatile`. Inputs whose tier is
`volatile` SHALL NOT cause invalidation of `durable` queries
unless the input's own tier is `durable` or higher.

#### Scenario: Editing a user file does not re-validate stdlib
- **WHEN** a file in the worktree is edited and the server is
  re-queried for a fact about a Pike stdlib symbol
- **THEN** the analysis layer returns the cached result for the
  stdlib symbol and does not re-run its parse or resolution.

### Requirement: The AST shields downstream queries from text changes
MUST: the position-stripped AST (a `red-green` node tree with no
byte ranges) is the canonical internal representation. The raw
tree-sitter syntax tree SHALL be private to the `parse` query and
SHALL NOT be exposed to other queries.

#### Scenario: Adding a comment does not invalidate symbol queries
- **WHEN** a comment is added to a function declaration and the
  client requests `textDocument/documentSymbol` for that file
- **THEN** the symbol result is returned from cache without
  re-running `parse` or `symbols` (verified by the unit test in
  `crates/pike-lsp/src/analysis/shield_test.rs`).

### Requirement: The analysis layer targets Pike 8.0.1116
The analysis layer MUST be aligned with the Pike 8.0.1116
language reference. In particular:
- The set of recognized preprocessor directives is the set
  documented in `pikelang/Pike/refdoc/preprocessor.xml`
  `<section title="Preprocessor Directives">`.
- The set of recognized modifiers is the set documented in
  `pikelang/Pike/refdoc/declarations.xml` `<section
  title="Modifiers">`.
- The set of recognized basic types is the set documented in
  `pikelang/Pike/refdoc/data_types.xml` `<section
  title="Basic types">`.

#### Scenario: Preprocessor fixture is recognized without errors
- **WHEN** `fixtures/syntax/preprocessor.pike` is opened in
  Zed with the language server enabled
- **THEN** the server reports zero diagnostic errors for that
  file's preprocessor directives.

### Requirement: Diagnostics cover the supported surface
The server SHALL publish diagnostics for:
- Parse errors reported by `TheSmuks/tree-sitter-pike` for the
  current buffer.
- `#include` and module-resolution failures where the file is
  not found on the Pike module path.
- Preprocessor directive errors (unknown directive, malformed
  macro).

#### Scenario: A bad include produces a diagnostic
- **WHEN** the buffer contains `#include <does-not-exist.pike>`
  and the file is not on the module path
- **THEN** the server publishes a `Diagnostic` whose
  `code` is `PIKE0001` and whose `message` mentions the missing
  file.

## REMOVED Requirements

### Requirement: Eager file watching from the server side
**Reason**: gopls and other SOTA LSPs delegate file watching to
the editor. The server should be told what changed via
`workspace/didChangeWatchedFiles` and `textDocument/didChange`,
not by spawning its own `inotify` watcher.
**Migration**: If a future need arises for in-server watching,
add it as a new capability rather than reviving this
requirement.
