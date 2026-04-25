#!/usr/bin/env bash
set -euo pipefail

: "${DUCKDB_CLI:?DUCKDB_CLI is required}"
: "${EXT:?EXT is required}"

mkdir -p .tmp
tmp_sql="$(mktemp .tmp/default-model.XXXXXX.sql)"
trap 'rm -f "$tmp_sql"' EXIT

sed "s#EXT_PATH#$EXT#g" tests/default_model.sql > "$tmp_sql"
output="$(env -u DUCKDB_VAPORETTO_MODEL -u DUCKDB_VAPORETTO_TAGS "$DUCKDB_CLI" -unsigned :memory: < "$tmp_sql")"

echo "$output"

grep -q "DEFAULT_SPLIT.*東京/特許/許可/局" <<<"$output"
grep -q 'DEFAULT_AND_QUERY.*"東京" AND "特許" AND "許可" AND "局"' <<<"$output"
