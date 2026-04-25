#!/usr/bin/env bash
set -euo pipefail

model_name="${1:-${MODEL_NAME:-bccwj-suw_c0.003}}"
model_version="${MODEL_VERSION:-v0.5.0}"
root="${MODEL_TMP_DIR:-.tmp/models}"
archive=".tmp/$model_name.tar.xz"
model_dir="$root/$model_name"
model_file="$model_dir/$model_name.model.zst"
url="https://github.com/daac-tools/vaporetto-models/releases/download/$model_version/$model_name.tar.xz"

mkdir -p ".tmp" "$model_dir"

if [[ ! -f "$model_file" ]]; then
  curl -L "$url" -o "$archive"
  tar -xJf "$archive" -C "$model_dir" --strip-components=1
fi

test -f "$model_file"

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  model_path="$(cd "$model_dir" && pwd)/$model_name.model.zst"
  if command -v cygpath >/dev/null 2>&1; then
    model_path="$(cygpath -w "$model_path")"
  fi
  echo "model_path=$model_path" >> "$GITHUB_OUTPUT"
fi

