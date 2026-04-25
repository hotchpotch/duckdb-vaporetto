#!/usr/bin/env bash
set -euo pipefail

model_path="${1:-}"
extension_path="${2:-target/wasm32-unknown-emscripten/release/duckdb_vaporetto.duckdb_extension.wasm}"
extension_version="${EXTENSION_VERSION:-v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)}"
duckdb_capi_version="${DUCKDB_CAPI_VERSION:-v1.2.0}"
duckdb_platform="${DUCKDB_PLATFORM:-wasm_eh}"
emsdk_env="${EMSDK_ENV:-.tmp/emsdk/emsdk_env.sh}"

if [[ -f "$emsdk_env" ]]; then
  # shellcheck source=/dev/null
  source "$emsdk_env" >/dev/null
fi

command -v emcc >/dev/null 2>&1 || {
  echo "emcc not found; install and activate Emscripten first" >&2
  exit 1
}

rustup toolchain install nightly --component rust-src --target wasm32-unknown-emscripten >/dev/null

build_env=()
if [[ -n "$model_path" ]]; then
  build_env+=(DUCKDB_VAPORETTO_EMBED_MODEL="$model_path")
fi

rustflags=(
  -Zunstable-options
  -C panic=immediate-abort
  -C link-arg=-sSIDE_MODULE=2
  -C link-arg=-sDISABLE_EXCEPTION_CATCHING=1
  -C link-arg=-sWASM_BIGINT
)

env \
  RUSTFLAGS="${rustflags[*]}" \
  "${build_env[@]}" \
  cargo +nightly build \
    -Z build-std=std,panic_abort \
    --target wasm32-unknown-emscripten \
    --release

mkdir -p "$(dirname "$extension_path")"
cargo duckdb-ext package \
  --library-path target/wasm32-unknown-emscripten/release/duckdb_vaporetto.wasm \
  --extension-path "$extension_path" \
  --extension-version "$extension_version" \
  --duckdb-platform "$duckdb_platform" \
  --duckdb-capi-version "$duckdb_capi_version"
