#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  echo "usage: $0 <asset-name> <extension-path> <archive-ext> <with-model|without-model>" >&2
  exit 2
fi

asset_name="$1"
extension_path="$2"
archive_ext="$3"
model_variant="$4"
package_dir="dist/$asset_name"

case "$model_variant" in
  with-model|without-model) ;;
  *)
    echo "unsupported model variant: $model_variant" >&2
    exit 2
    ;;
esac

rm -rf "$package_dir"
mkdir -p "$package_dir/tests" "$package_dir/scripts"

cp "$extension_path" "$package_dir/"
cp README.md MODEL_LICENSES.md "$package_dir/"
cp -R docs "$package_dir/"
cp tests/scalar.sql tests/scalar.sh tests/scalar.ps1 "$package_dir/tests/"
cp scripts/fetch-vaporetto-model.sh scripts/fetch-vaporetto-model.ps1 "$package_dir/scripts/"
if [[ "$model_variant" == "with-model" ]]; then
  cp tests/default_model.sql tests/default_model.sh tests/default_model.ps1 "$package_dir/tests/"
fi

case "$archive_ext" in
  tar.gz)
    archive="dist/$asset_name.tar.gz"
    tar -C dist -czf "$archive" "$asset_name"
    ;;
  zip)
    archive="dist/$asset_name.zip"
    (cd dist && zip -qr "$asset_name.zip" "$asset_name")
    ;;
  *)
    echo "unsupported archive extension: $archive_ext" >&2
    exit 2
    ;;
esac

(cd dist && shasum -a 256 "$(basename "$archive")" > "$(basename "$archive").sha256")
