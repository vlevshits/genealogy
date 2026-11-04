#!/usr/bin/env bash
# Verifies that repo paths referenced from docs/memory-bank/ still exist.
#
# Checks:
#   1. Inline-code paths starting with a known repo root, e.g. `backend/src/error.rs`
#      (an optional `::symbol`, `#anchor` or `:line` suffix is ignored).
#   2. Relative markdown link targets, e.g. [spec](../../features/02-rbac-access-control.md).
# Paths containing glob or placeholder characters (* < > … NN {) are skipped.
#
# Usage: scripts/check-memory-bank.sh   (from anywhere inside the repo)
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
BANK="$ROOT/docs/memory-bank"
[ -d "$BANK" ] || { echo "memory bank not found at $BANK" >&2; exit 2; }

missing=0
checked=0

is_placeholder() {
  case "$1" in
    *'*'*|*'<'*|*'>'*|*'…'*|*NN*|*'{'*|*'...'*) return 0 ;;
  esac
  return 1
}

while IFS= read -r file; do
  dir="$(dirname "$file")"

  # 1. inline code paths
  while IFS= read -r ref; do
    path="${ref%%::*}"; path="${path%%#*}"; path="${path%%:*}"; path="${path%/}"
    is_placeholder "$path" && continue
    checked=$((checked + 1))
    if [ ! -e "$ROOT/$path" ]; then
      echo "MISSING  ${file#$ROOT/}: \`$ref\`"
      missing=$((missing + 1))
    fi
  done < <(grep -oE '`(backend|frontend|docs|scripts|\.agents|fixtures|test-data)/[^` ]+`' "$file" | tr -d '`' || true)

  # 2. relative markdown links
  while IFS= read -r target; do
    case "$target" in http*|mailto:*|\#*|file:*|/*) continue ;; esac
    path="${target%%#*}"
    [ -z "$path" ] && continue
    is_placeholder "$path" && continue
    checked=$((checked + 1))
    if [ ! -e "$dir/$path" ]; then
      echo "MISSING  ${file#$ROOT/}: ($target)"
      missing=$((missing + 1))
    fi
  done < <(grep -oE '\]\([^)]+\)' "$file" | sed -E 's/^\]\(([^) ]+).*/\1/' || true)
done < <(find "$BANK" -name '*.md' -type f | sort)

echo "memory-bank check: $checked references checked, $missing missing"
[ "$missing" -eq 0 ]

