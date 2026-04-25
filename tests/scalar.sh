#!/usr/bin/env bash
set -euo pipefail

: "${DUCKDB_CLI:?DUCKDB_CLI is required}"
: "${EXT:?EXT is required}"
: "${DUCKDB_VAPORETTO_MODEL:?DUCKDB_VAPORETTO_MODEL is required}"

mkdir -p .tmp
tmp_sql="$(mktemp .tmp/scalar.XXXXXX.sql)"
trap 'rm -f "$tmp_sql"' EXIT

sed "s#EXT_PATH#$EXT#g" tests/scalar.sql > "$tmp_sql"
output="$("$DUCKDB_CLI" -unsigned :memory: < "$tmp_sql")"

echo "$output"

grep -q "SPLIT_SPACE.*東京 特許 許可 局" <<<"$output"
grep -q "SPLIT_SLASH.*東京/特許/許可/局" <<<"$output"
grep -q "SPLIT_SPACED.*東京/特許/許可/局/検索/エンジン" <<<"$output"
grep -q 'AND_QUERY.*"東京" AND "特許" AND "許可" AND "局"' <<<"$output"
grep -q 'AND_QUERY_SPACED.*"東京" AND "特許" AND "許可" AND "局" AND "検索" AND "エンジン"' <<<"$output"
grep -q 'OR_QUERY.*"東京" OR "特許" OR "許可" OR "局"' <<<"$output"
grep -q 'OR_QUERY_SPACED.*"東京" OR "特許" OR "許可" OR "局" OR "検索" OR "エンジン"' <<<"$output"
grep -q "SPLIT_CASE_DEFAULT.*hello/hello" <<<"$output"
grep -q "SPLIT_CASE_SENSITIVE.*Hello/HELLO" <<<"$output"
grep -q 'AND_CASE_DEFAULT.*"hello" AND "hello"' <<<"$output"
grep -q 'AND_CASE_SENSITIVE.*"Hello" AND "HELLO"' <<<"$output"
grep -q "NOUN_SPLIT.*東京/検索/エンジン/実験" <<<"$output"
grep -q 'NOUN_AND_QUERY.*"東京" AND "検索" AND "エンジン" AND "実験"' <<<"$output"
grep -q "NOUN_UNTAGGED_SPLIT.*東京/asdfoujbva/検索" <<<"$output"
grep -q 'NOUN_UNTAGGED_AND_QUERY.*"東京" AND "asdfoujbva" AND "検索"' <<<"$output"
