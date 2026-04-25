#!/usr/bin/env bash
set -euo pipefail

duckdb_version="${DUCKDB_VERSION:-v1.5.2}"
root="${DUCKDB_TMP_DIR:-.tmp/duckdb}"
uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "${DUCKDB_CLI_ASSET:-}" in
  "") ;;
  *)
    asset="$DUCKDB_CLI_ASSET"
    ;;
esac

if [[ -z "${asset:-}" ]]; then
  case "$uname_s:$uname_m" in
    Linux:x86_64) asset="duckdb_cli-linux-amd64.zip" ;;
    Linux:aarch64|Linux:arm64) asset="duckdb_cli-linux-arm64.zip" ;;
    Darwin:x86_64) asset="duckdb_cli-osx-amd64.zip" ;;
    Darwin:arm64|Darwin:aarch64) asset="duckdb_cli-osx-arm64.zip" ;;
    *) echo "unsupported DuckDB CLI platform: $uname_s $uname_m" >&2; exit 2 ;;
  esac
fi

archive="$root/$asset"
cli="$root/duckdb"
stamp="$root/.asset"
url="https://github.com/duckdb/duckdb/releases/download/$duckdb_version/$asset"

mkdir -p "$root"
if [[ ! -x "$cli" || ! -f "$stamp" || "$(cat "$stamp")" != "$asset" ]]; then
  rm -f "$cli"
  curl -L "$url" -o "$archive"
  unzip -o "$archive" -d "$root"
  chmod +x "$cli"
  printf '%s\n' "$asset" > "$stamp"
fi

test -x "$cli"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  echo "duckdb_cli=$(cd "$root" && pwd)/duckdb" >> "$GITHUB_OUTPUT"
fi
