#!/usr/bin/env bash
set -euo pipefail

tag="${1:?tag is required}"
release_dir="${2:-docs/releases}"
head_notes="$release_dir/HEAD.md"
tag_notes="$release_dir/$tag.md"

read_notes() {
  local file="$1"

  if [[ ! -f "$file" ]]; then
    return 0
  fi

  awk 'NR == 1 && /^# / { next } { print }' "$file" | sed '/^[[:space:]]*$/d'
}

notes="$(read_notes "$head_notes")"

if [[ -z "$notes" ]]; then
  notes="$(read_notes "$tag_notes")"
fi

if [[ -n "$notes" ]]; then
  printf '%s\n' "$notes"
else
  printf 'Release %s\n' "$tag"
fi

