DUCKDB_VERSION ?= v1.5.2
MODEL_VERSION ?= v0.5.0
MODEL_NAME ?= bccwj-suw+unidic_pos+kana
CI_MODEL_NAME ?= bccwj-suw_c0.003
MODEL_FILE := .tmp/models/$(MODEL_NAME)/$(MODEL_NAME).model.zst
CI_MODEL_FILE := .tmp/models/$(CI_MODEL_NAME)/$(CI_MODEL_NAME).model.zst
DUCKDB_CLI := .tmp/duckdb/duckdb
EMSDK_VERSION ?= 5.0.6
EMSDK_DIR ?= .tmp/emsdk

UNAME_S := $(shell uname -s)
RELEASE_EXT := target/release/duckdb_vaporetto.duckdb_extension
WASM_RELEASE_EXT := target/wasm32-unknown-emscripten/release/duckdb_vaporetto.duckdb_extension.wasm

.PHONY: all test build release embedded-release duckdb-extension wasm-extension emsdk duckdb model ci-model test-extension test-embedded fmt clean

all: build

build:
	cargo build

release:
	cargo build --release

embedded-release: model
	DUCKDB_VAPORETTO_EMBED_MODEL="$(abspath $(MODEL_FILE))" cargo build --release

duckdb-extension:
	cargo duckdb-ext build -a v1.2.0 -- --release

wasm-extension: emsdk ci-model
	scripts/build-wasm.sh "$(abspath $(CI_MODEL_FILE))" "$(abspath $(WASM_RELEASE_EXT))"

emsdk:
	if [ ! -d "$(EMSDK_DIR)/.git" ]; then git clone --depth 1 https://github.com/emscripten-core/emsdk.git "$(EMSDK_DIR)"; fi
	"$(EMSDK_DIR)/emsdk" install "$(EMSDK_VERSION)"
	"$(EMSDK_DIR)/emsdk" activate "$(EMSDK_VERSION)"

duckdb:
	scripts/fetch-duckdb-unix.sh

model: $(MODEL_FILE)

$(MODEL_FILE):
	scripts/fetch-vaporetto-model.sh "$(MODEL_NAME)"

ci-model: $(CI_MODEL_FILE)

$(CI_MODEL_FILE):
	scripts/fetch-vaporetto-model.sh "$(CI_MODEL_NAME)"

test:
	cargo test

test-extension: duckdb-extension duckdb model
	DUCKDB_VAPORETTO_MODEL="$(abspath $(MODEL_FILE))" \
	DUCKDB_CLI="$(abspath $(DUCKDB_CLI))" \
	EXT="$(abspath $(RELEASE_EXT))" \
	tests/scalar.sh

test-embedded: embedded-release duckdb
	cargo duckdb-ext package \
	  --library-path target/release/libduckdb_vaporetto.so \
	  --extension-path "$(RELEASE_EXT)" \
	  --extension-version "v$(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)" \
	  --duckdb-platform linux_amd64 \
	  --duckdb-capi-version v1.2.0
	DUCKDB_CLI="$(abspath $(DUCKDB_CLI))" \
	EXT="$(abspath $(RELEASE_EXT))" \
	tests/default_model.sh

fmt:
	cargo fmt

clean:
	cargo clean
	rm -rf .tmp dist
