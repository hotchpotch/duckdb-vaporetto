# AGENTS.md

Guidance for coding agents working in this repository.

## Project

`duckdb-vaporetto` is a Rust DuckDB loadable extension backed by the
[Vaporetto](https://github.com/daac-tools/vaporetto) Japanese tokenizer.

The extension exposes scalar SQL helper functions that segment Japanese text
before it is stored, inspected, or searched in DuckDB. DuckDB's `fts` extension
does not currently expose a tokenizer hook like SQLite FTS5, so this project
uses a DuckDB-native flow: create a tokenized text column with
`vaporetto_split()`, build DuckDB FTS on that column, and tokenize user input
with the same options before calling `match_bm25()`.

The project also builds DuckDB-Wasm extension artifacts. Wasm artifacts embed a
Vaporetto model so they can run inside DuckDB-Wasm in a web browser without a
local model path.

Temporary build inputs and generated artifacts belong under `.tmp/`, `target/`,
or `dist/`; keep them out of git.

## Build And Test

Use the Makefile for normal workflows:

```sh
make test
make test-extension
make test-embedded
```

`make test` runs Rust unit tests. `make test-extension` downloads a DuckDB CLI
and a Vaporetto model into `.tmp/`, builds a release DuckDB extension with
`cargo duckdb-ext`, loads it with `duckdb -unsigned`, and runs the scalar SQL
smoke tests. `make test-embedded` verifies a native build with the default model
embedded.

For focused Rust checks:

```sh
cargo test
cargo fmt
```

For distributable native extension builds:

```sh
make duckdb-extension
make embedded-release
```

For DuckDB-Wasm builds:

```sh
make wasm-extension
make wasm-extension-small
make wasm-extension-full
```

The wasm build installs Emscripten under `.tmp/emsdk`, fetches Vaporetto models
under `.tmp/models`, and writes local artifacts to:

- `target/wasm32-unknown-emscripten/release/small/duckdb_vaporetto.duckdb_extension.wasm`
- `target/wasm32-unknown-emscripten/release/full/duckdb_vaporetto.duckdb_extension.wasm`

Use `.tmp/web/` for local browser examples and DuckDB-Wasm experiments. This
directory is intentionally ignored by git.

## Development Rules

- Prefer TDD. Add or update SQL/script expectations first for behavior changes,
  then update Rust implementation.
- Keep `.tmp/`, `target/`, `dist/`, downloaded DuckDB CLIs, Emscripten SDKs,
  Vaporetto model files, and packaged extension artifacts out of git.
- Use `apply_patch` or normal source edits for tracked files; do not generate
  repository files under `.tmp/`.
- Keep the public SQL surface stable unless the change is intentional and
  documented:
  - `vaporetto_split(text)`
  - `vaporetto_split(text, separator)`
  - `vaporetto_split(text, separator, options)`
  - `vaporetto_and_query(text)`
  - `vaporetto_and_query(text, options)`
  - `vaporetto_or_query(text)`
  - `vaporetto_or_query(text, options)`
- `vaporetto_split()` is the function to use with DuckDB FTS `match_bm25()`.
  DuckDB's built-in `match_bm25()` expects a plain term string, not the quoted
  boolean syntax produced by `vaporetto_and_query()` or `vaporetto_or_query()`.
- Query helper functions must quote generated tokens and omit whitespace-only
  tokens. Do not replace them with raw `split + join` behavior.
- ASCII letters are case-insensitive by default and are folded to lowercase
  unless `case sensitive` is explicitly specified.
- `tags <prefixes>` is prefix-based POS/tag filtering. No-tag tokens are
  filtered out unless `keep_untagged` is present.
- Treat `keep_untagged` as "include tokens with empty/no POS tag"; do not label
  these tokens as unknown/UNK unless the Vaporetto model actually provides such
  a tag.
- When changing tokenizer options or SQL helper semantics, update `README.md`
  and add tests in both Unix and Windows paths when the behavior is shared.
- Native registration uses `quack-rs`; Wasm registration uses the raw DuckDB C
  API path in `src/lib.rs`. Keep both paths functionally aligned.
- The DuckDB C API/package version is currently `v1.2.0` in build scripts and
  workflows. Change it deliberately and test native and Wasm builds together.

## Tests

Use these files for behavior coverage:

- `tests/scalar.sql`, `tests/scalar.sh`, and `tests/scalar.ps1` for explicit
  model-path scalar helper behavior.
- `tests/default_model.sql`, `tests/default_model.sh`, and
  `tests/default_model.ps1` for embedded-model behavior.
- Rust unit tests in `src/tokenizer.rs` for option parsing, token quoting, and
  tokenizer internals.

When updating integration behavior, keep `.sql`, `.sh`, and `.ps1`
expectations in sync.

## Models And Options

The default native embedded model is
`bccwj-suw+unidic_pos+kana.model.zst`. This model supports POS/tag prediction
and is required for useful `tags` filtering.

The smaller `bccwj-suw_c0.003` and `bccwj-suw_c1.0` models are useful for smoke
tests or small Wasm downloads, but do not assume they support POS/tag filtering.

Configuration can come from an options string:

```sql
SELECT vaporetto_split(
  '東京でasdfoujbvaを検索した。',
  '/',
  'model /path/to/model.zst tags 名詞 keep_untagged'
);
```

or from environment variables:

```sh
DUCKDB_VAPORETTO_MODEL=/path/to/model.zst duckdb -unsigned
```

Supported options and environment variables:

- `model <path>`: Vaporetto `.model` or `.model.zst` file.
- `wsconst <chars>`: Vaporetto/KyTea-style character classes not to segment.
  Defaults to `DGR`.
- `tags <prefixes>`: Comma-separated Vaporetto tag prefixes to keep.
- `keep_untagged`: With `tags`, also keep tokens with no POS/tag data.
- `case sensitive`: Preserve ASCII case distinctions.
- `case insensitive`: Explicitly request the default case-insensitive behavior.
- `DUCKDB_VAPORETTO_MODEL`: Default model path.
- `DUCKDB_VAPORETTO_WSCONST`: Default `wsconst`.
- `DUCKDB_VAPORETTO_TAGS`: Default comma-separated tag prefixes.

## Release Artifacts

GitHub Actions build release artifacts only for tags matching `v*`. Do not add
a plain branch `push` release trigger unless the project owner explicitly asks
for it.

Native release artifacts are built for:

- `linux-x86_64`
- `linux-aarch64`
- `macos-aarch64`
- `windows-x86_64`

Each native platform has two variants:

- unqualified asset name: no bundled model; requires `model <path>` or
  `DUCKDB_VAPORETTO_MODEL`; licensed as `duckdb-vaporetto` under
  `MIT OR Apache-2.0`.
- `-with-model`: embeds `bccwj-suw+unidic_pos+kana.model.zst`; contains
  `duckdb-vaporetto` under `MIT OR Apache-2.0` and the bundled model under
  `BSD-3-Clause`.

DuckDB-Wasm release artifacts are always `-with-model`:

- `wasm_eh-small-bccwj-suw-c1.0-with-model`: embeds
  `bccwj-suw_c1.0.model.zst`; smaller download, no POS/tag filtering support.
- `wasm_eh-full-bccwj-suw-unidic-pos-kana-with-model`: embeds
  `bccwj-suw+unidic_pos+kana.model.zst`; larger download, supports richer
  vocabulary and `tags` filtering.

Tagged release asset names include the release tag, platform, and model
variant, for example:

- `duckdb-vaporetto-v0.2.0-linux-x86_64.tar.gz`
- `duckdb-vaporetto-v0.2.0-linux-x86_64-with-model.tar.gz`
- `duckdb-vaporetto-v0.2.0-wasm_eh-small-bccwj-suw-c1.0-with-model.tar.gz`
- `duckdb-vaporetto-v0.2.0-wasm_eh-full-bccwj-suw-unidic-pos-kana-with-model.tar.gz`

Keep `MODEL_LICENSES.md` and the README license section aligned with any
bundled model changes.

## Release Notes

- Keep not-yet-released notes in `docs/releases/HEAD.md`.
- Keep published release notes in `docs/releases/vX.Y.Z.md`.
- `scripts/release-notes.sh <tag>` reads `docs/releases/HEAD.md` first and
  falls back to `docs/releases/<tag>.md` when `HEAD.md` has no body.
- Before tagging a release, create `docs/releases/<tag>.md` with the intended
  release notes and reset `docs/releases/HEAD.md` to just `# HEAD` in the same
  commit. Then push the commit and tag.
- Verify the tag-triggered GitHub Actions run succeeds and that the GitHub
  Release has native and Wasm assets plus `.sha256` files.

## Important Commands

```sh
cargo test
cargo fmt
make test
make test-extension
make test-embedded
make duckdb-extension
make wasm-extension-small
make wasm-extension-full
```

Run `cargo fmt` before committing Rust changes, and run `git diff --check`
before committing documentation or workflow changes.
