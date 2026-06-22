# Contributing

## Workflow

1. Branch off `main` using a **conventional branch name** (see below).
2. Implement the change with **conventional commits** (see below).
3. Open a pull request against `main`. CI runs commitlint, branch-name
   check, `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, the host `pike-lsp` release build, and the
   `wasm32-wasip2` bridge release build.
4. Use OpenSpec for non-trivial changes: `openspec new change <slug>` →
   proposal → specs → design → tasks → apply.

## Conventional branch names

Branch names follow `<type>/<scope>-<short-topic>`:

- `feat/<scope>-<short-topic>` — new user-visible functionality.
- `fix/<scope>-<short-topic>` — bug fixes.
- `docs/<scope>-<short-topic>` — documentation only.
- `refactor/<scope>-<short-topic>` — code change with no behavior change.
- `perf/<scope>-<short-topic>` — performance improvement.
- `test/<scope>-<short-topic>` — test-only change.
- `build/<scope>-<short-topic>` — build system / dependency change.
- `ci/<scope>-<short-topic>` — CI configuration change.
- `chore/<scope>-<short-topic>` — tooling or housekeeping.
- `release/<version>` — release cut (tags + changelog + release notes).

Examples:

- `feat/bridge-ssh-rss-guard`
- `fix/forward-fails-on-missing-socket`
- `docs/add-changelog-and-architecture`

Allowed scopes include `bridge`, `lsp`, `transport`, `analysis`,
`daemon`, `extension`, `docs`, `ci`, `release`, or any short identifier
that makes the branch findable.

`main` is protected. Branches are validated by
`scripts/check-branch-name.sh`, which `pre-push` invokes locally and
which the `ci` workflow also runs.

## Conventional Commits

Commit messages follow [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Allowed types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`,
`test`, `build`, `ci`, `chore`, `revert`.

Rules:

- Subject line ≤ 72 characters, imperative mood, no trailing period.
- Body explains **what** and **why**, separated from subject by a blank
  line.
- Footer may contain `BREAKING CHANGE: <description>`, `Refs: <issue>`,
  or `Refs: openspec/changes/<slug>`.
- `feat` and `fix` are user-visible; everything else is internal.

Examples:

```
feat(bridge): auto-download pike-lsp from latest GitHub release

Adds a fresh release fetch as the last fallback in
language_server_command. The cache is checked first; the cache miss
path calls zed::latest_github_release with require_assets = true.

Refs: openspec/changes/zed-pike-lsp-bridge
```

```
fix(forward): exit non-zero when target socket is absent

The previous behavior auto-started a daemon when the forwarded socket
was missing. That violates the low-resource lifecycle: the daemon can
outlive the editor session. The forward command now exits 2 with a
clear error and points at `pike-lsp daemon` for opt-in shared mode.

Refs: openspec/changes/zed-remote-lsp-lifecycle
```

## Local hooks

The repository ships `.husky/commit-msg`, which runs `commitlint` on
every commit message. To install the hooks locally:

```sh
bunx husky install
```

`scripts/check-branch-name.sh` is invoked by `.husky/pre-push` to
prevent pushing branches that violate the naming convention.

## TOML trap reminder

`languages/<id>/config.toml` parses TOML strings silently. Triple-quoted
`"""` produces a string that starts with `, end = ` — Zed then has the
wrong auto-close pair. Always use single-quoted TOML strings for quote
characters:

```toml
[[brackets]]
start = "'"
end   = "'"

[[brackets]]
start = '"'
end   = '"'
```

Verify by parsing the file:

```sh
python3 -c "import tomllib; print(tomllib.loads(open('languages/pike/config.toml').read()))"
```

The parsed `brackets` array must contain two entries with `start`/`end`
of single characters.

## Releases

`release/<version>` branches are created from `main` at release time.
The release pipeline:

1. Builds `pike-lsp` (host) and `zed-pike-bridge` (`wasm32-wasip2`).
2. Packages artifacts in `out/`.
3. Tags `v<version>` (annotated).
4. Publishes a GitHub release with the artifacts attached.

Releases happen on `TheSmuks/zed-pike` only. No prior Pike LSP repo is
referenced or cross-linked.