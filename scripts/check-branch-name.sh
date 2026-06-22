#!/usr/bin/env bash
# Validate that the current branch name follows the project's
# conventional-branches convention:
#   type[/scope]-<short-topic>
# Allowed types match commitlint.config.js plus release.
# Allowed scopes are advisory; anything lowercase-kebab after the type
# is accepted.
set -euo pipefail

ALLOWED_TYPES='feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert|release'
SCOPE_RE='[a-z0-9][a-z0-9-]*'
TOPIC_RE='[a-z0-9][a-z0-9-]*'
BRANCH_RE="^(${ALLOWED_TYPES})(/${SCOPE_RE})?(-${TOPIC_RE})?$"

branch="${1:-$(git rev-parse --abbrev-ref HEAD)}"

if [ "$branch" = "HEAD" ]; then
  echo "refusing to validate detached HEAD" >&2
  exit 0
fi

if [ "$branch" = "main" ] || [ "$branch" = "master" ]; then
  echo "ok: protected branch $branch" >&2
  exit 0
fi

if [[ "$branch" =~ $BRANCH_RE ]]; then
  exit 0
fi

cat >&2 <<EOF
branch name "$branch" does not match the conventional-branches pattern.

expected: <type>[/<scope>]-<short-topic>
examples:
  feat(bridge)-auto-download-pike-lsp
  fix(forward)-fail-on-missing-socket
  docs-add-changelog
  release-0.0.1

allowed types: $ALLOWED_TYPES
EOF
exit 1