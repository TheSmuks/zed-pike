## Context

The repository `/tank/projects/zed-pike` is a Zed extension that today has
the skeleton required for Zed to discover the language (`extension.toml`,
`languages/pike/config.toml`, a highlights query, brackets, indents,
outline, and three small `.pike` fixtures). It does not yet have an
OpenSpec change. Pike currently has no first-class support in Zed; the
existing VS Code extension
(https://github.com/GwennKoi/vscode-pike-lang) is a TextMate-grammar-only
extension, which Zed does not consume on its normal language-extension
path.

The maintained tree-sitter grammar is
https://github.com/TheSmuks/tree-sitter-pike, which targets
**Pike 8.0.1116** (release tag
https://github.com/pikelang/Pike/releases/tag/v8.0.1116, source commit
`5d216a06d86bf36ec321fff3f82dfe80fd055194`). The grammar already covers
Pike 8.0.1116 with a 220/220 corpus pass rate at the time of planning.
We rely on that grammar rather than building a new one.

The canonical reference for what Pike 8.0.1116 actually is, lives in the
upstream `pikelang/Pike` `refdoc/` folder (https://github.com/pikelang/Pike/tree/master/refdoc).
The chapters the extension's queries and fixtures must respect are:

- `refdoc/lexical_grammar.xml`: whitespace, `/* ... */` (non-nested) and
  `// ...` comments, token categories.
- `refdoc/preprocessor.xml`: directive list and predefined defines.
- `refdoc/control_structures.xml`: `if`, `else`, `switch`, `case`, `default`,
  `while`, `for`, `do`, `foreach`, `break`, `continue`, `return`,
  `continue return` (Pike 9.0+).
- `refdoc/declarations.xml`: `constant`, `enum`, `typedef`, `class`,
  `inherit`, modifiers `public`, `protected`, `private`, `local`, `final`,
  `optional`, `extern`, `variant`, `static`, type attributes
  `__attribute__`, `__deprecated__`, `__experimental__`, `__factory__`,
  `__weak__`, `__unused__`, `__generator__`, `__async__`.
- `refdoc/data_types.xml`: `int`, `float`, `string`, `array`, `mapping`,
  `multiset`, `program`, `object`, `function`, `void`, `mixed`,
  `__unknown__`.
- `refdoc/operators.xml`: the full operator table (arithmetic, comparison,
  bitwise, logical, assignment, `..`, `...`, `->`, `->?`, `?->`, `[?`,
  `=>`, `?`, `::`, `++`, `--`).
- `refdoc/pike_bnf.xml`: full BNF for the language.

The Pike 8.0.1116 preprocessor directive set (from `refdoc/preprocessor.xml`,
`<section title="Preprocessor Directives">`) is the authoritative list:

  `#!`, `#line`, `#"..."`, `#(...)`, `#string`, `#include`, `#if`,
  `#ifdef`, `#ifndef`, `#endif`, `#else`, `#elif`, `#elifdef`,
  `#elifndef`, `#define`, `#undef`, `#charset`, `#pike`, `#pragma`,
  `#require`, `#warning`, `#error`.

The Pike 8.0.1116 predefined preprocessor defines (from
`refdoc/preprocessor.xml`, `<section title="Predefined defines">`) are:

  `__VERSION__`, `__MAJOR__`, `__MINOR__`, `__BUILD__`,
  `__REAL_VERSION__`, `__REAL_MAJOR__`, `__REAL_MINOR__`, `__REAL_BUILD__`,
  `__DATE__`, `__TIME__`, `__FILE__`, `__DIR__`, `__LINE__`,
  `__COUNTER__`, `__AUTO_BIGNUM__`, `__NT__`, `__PIKE__`, `__amigaos__`,
  `__APPLE__`, `__HAIKU__`, `_Pragma`, `static_assert`.

Constraints:
- Zed extensions must use the manifest format defined by Zed (TOML with
  `[grammars.<id>]` and `languages/<id>/config.toml`).
- Tree-sitter queries must reference real node names from the pinned
  grammar, not guessed names.
- Zed does not load TextMate grammars on the standard path, so the VS
  Code TextMate grammar is reference material only, not a runtime
  dependency.
- Zed extensions that need only syntax features do not require Rust/WASM
  code.
- Prior Pike LSP attempts are explicitly out of scope for this milestone
  and must not be referenced or built on.

## Goals / Non-Goals

**Goals:**
- Ship a loadable Zed extension that recognizes `.pike`, `.pmod`, and
  `.cmod` files as Pike.
- Use the maintained `TheSmuks/tree-sitter-pike` grammar pinned by commit,
  with the upstream highlight query as the seed and the upstream node
  names as the source of truth.
- Ship `.scm` files for highlight, brackets, indents, and outline.
- Ship a small set of Pike fixtures that parse cleanly against the
  pinned grammar.
- Document how to validate the pinned grammar outside Zed.
- Produce the OpenSpec artifacts (proposal, specs, design, tasks) for
  this change and validate the change.

**Non-Goals:**
- LSP / language server integration.
- Semantic features (hover, completion, go-to-definition, refactor).
- Snippets, themes, icon themes, formatter, debugger.
- Building or maintaining a tree-sitter Pike grammar ourselves.
- Touching prior Pike LSP repos.

## Decisions

- **Use the maintained grammar instead of writing our own.** Rationale:
  The grammar already covers Pike 8.0.1116 with a 220/220 corpus pass
  rate and is being maintained upstream. Alternatives considered:
  (a) write a fresh grammar in this repo (rejected: large scope, no
  advantage over the maintained one); (b) port the VS Code TextMate
  grammar (rejected: Zed does not consume TextMate on the standard
  path).
- **Pin the grammar by commit, not by branch.** Rationale: A 40-char SHA
  in `extension.toml` gives a reproducible build and matches Zed's
  manifest schema. Branch pins drift.
- **Seed `highlights.scm` from the upstream query, then refine.** Rationale:
  The upstream query already targets the right captures
  (`@keyword`, `@string`, `@number`, `@function`, `@type`, etc.) and
  tracks the grammar's real node names. Seeding avoids guessing and
  reduces the surface for bugs.
- **Use Zed's bracket `config.toml` for auto-closing pairs, not just
  `brackets.scm`.** Rationale: `brackets.scm` covers matching, but
  `config.toml` controls auto-closing when the user types a `{}`, `[]`,
  `()`, or quote. The `not_in` field avoids closing inside strings or
  comments.
- **Outdent at the close token, indent at the open block.** Rationale:
  Zed's indentation queries match on the open/close tokens for
  block-shaped nodes; this is enough for the MVP and avoids depending on
  every block-shaped node the grammar may produce.
- **No Rust/WASM crate for this milestone.** Rationale: A pure-syntax
  extension is sufficient and avoids pulling in `zed_extension_api`,
  `cargo`, and `wasm32-wasi` toolchain just to register a language.
- **Fixtures live in `fixtures/syntax/`, not in `examples/`.** Rationale:
  The grammar repo uses `examples/`. Separating the extension's fixtures
  from upstream examples avoids confusion about who owns the file and
  what they prove.

## Risks / Trade-offs

- **Grammar upgrades can change node names.** Mitigation: the change
  procedure is documented in the proposal and README; bump the commit
  in `extension.toml`, rerun the upstream test suite, parse fixtures,
  and adjust queries only if a node name changed.
- **`.cmod` is in the upstream `tree-sitter.json` `file-types`, but the
  upstream `package.json` scope is `source.pike`.** Mitigation: Zed
  reads the extension's own `languages/pike/config.toml`, so we can
  list `.cmod` directly and override upstream defaults. We list
  `.pike`, `.pmod`, and `.cmod` in our config to match the VS Code
  extension's behavior.
- **Tree-sitter query node names are grammar-version-sensitive.**
  Mitigation: seed from the upstream query and verify the parse
  output contains no `ERROR` or `MISSING` for every fixture.
- **Indentation queries are coarse.** Mitigation: the MVP indents at
  every block-shaped node and outdents at the close token. Future
  revisions can refine this once the grammar is stable in Zed.
- **No LSP means no semantic features.** Mitigation: documented in
  the proposal as out of scope; deferred to a future OpenSpec change.

## Migration Plan

- There is no existing user base or published extension; this is the
  initial change. The "migration" is the install path:
  1. `zed: install dev extension` -> select `/tank/projects/zed-pike`.
  2. Open a `.pike`, `.pmod`, or `.cmod` file.
  3. Observe syntax highlighting, brackets, indentation, and outline.
- Rollback: uninstall the dev extension; the extension does not
  modify any global state.

## Open Questions

- Should the extension id remain `pike` (matching the language id) or
  a distinct id such as `zed-pike`? Zed ids cannot be changed casually
  after publication, so the choice must be made before submitting to
  the registry. The MVP uses `pike`; revisit before any registry
  submission.
- Should we ship snippets from the VS Code extension as part of this
  change? Deferred to a future change to keep this MVP small and
  verifiable.
