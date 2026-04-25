# 🛥 duckdb-vaporetto

Japanese tokenization extension for DuckDB full-text search, powered by
[Vaporetto](https://github.com/daac-tools/vaporetto).

`duckdb-vaporetto` is a DuckDB loadable extension. It adds SQL scalar functions
that tokenize Japanese text with Vaporetto before you store, inspect, or search
text in DuckDB.

Release packages are available for native DuckDB and DuckDB-Wasm. The Wasm
packages embed a Vaporetto model, so the same tokenization functions can also
run inside DuckDB-Wasm in a web browser.

DuckDB's `fts` extension is a compact full-text search engine with BM25 ranking,
but it does not currently expose a tokenizer hook like SQLite FTS5.
`duckdb-vaporetto` combines DuckDB FTS with Vaporetto by following a
DuckDB-native flow: create a tokenized text column with Vaporetto, build a
DuckDB FTS index on that column, and tokenize user input with the same Vaporetto
settings before calling `match_bm25()`.

[Vaporetto](https://github.com/daac-tools/vaporetto) を利用した、DuckDB の
全文検索向け日本語 tokenization extension です。

`duckdb-vaporetto` は DuckDB loadable extension です。日本語テキストを
DuckDB に保存、確認、検索する前に、Vaporetto で tokenization する SQL scalar
function を追加します。

native DuckDB 向けの配布物に加えて、DuckDB-Wasm 向けの配布物も提供しています。
Wasm 版は Vaporetto model を同梱しているため、web browser 上の DuckDB-Wasm でも
同じ tokenization function を利用できます。

DuckDB の `fts` extension は BM25 ranking を備えた小さな全文検索エンジンです
が、SQLite FTS5 のような tokenizer hook は現在公開されていません。
`duckdb-vaporetto` は DuckDB らしい流れで DuckDB FTS と Vaporetto を組み合わせ
ます。Vaporetto で tokenized text column を作り、その column に DuckDB FTS index
を作成し、user query も同じ Vaporetto 設定で tokenize してから `match_bm25()` に
渡します。

## Quick Start

Download the package for your operating system and CPU architecture from the
[Releases page](../../releases). If you are not sure which Vaporetto model to
use, choose the package whose name ends with `-with-model`. It includes
[`bccwj-suw+unidic_pos+kana.model.zst`](https://github.com/daac-tools/vaporetto-models/releases),
so the examples below work without a model path.

Extract the package, then start DuckDB with unsigned extension loading enabled
and load the extension:

```sh
duckdb -unsigned
```

```sql
LOAD './duckdb_vaporetto.duckdb_extension';
```

Check tokenization first:

```sql
SELECT vaporetto_split('東京特許許可局', '/');
-- 東京/特許/許可/局
```

For full-text search, load DuckDB's `fts` extension and store a tokenized search
column. The `tags 名詞` option keeps noun-like tokens and removes many particles
and punctuation marks from the FTS input:

```sql
INSTALL fts;
LOAD fts;

CREATE TABLE docs(
  id INTEGER,
  body VARCHAR,
  body_tokens VARCHAR
);

INSERT INTO docs
SELECT id, body, vaporetto_split(body, ' ', 'tags 名詞')
FROM (VALUES
  (1, '東京特許許可局で検索エンジンの実験をした。'),
  (2, '大阪で検索エンジンの実験をした。'),
  (3, '東京で特許の申請をして、別の日に許可局へ行った。'),
  (4, '札幌で全文検索の実験をした。')
) AS v(id, body);
```

Build a DuckDB FTS index on the tokenized column:

```sql
PRAGMA create_fts_index(
  'docs',
  'id',
  'body_tokens',
  stemmer = 'none',
  stopwords = 'none',
  lower = 0
);
```

Tokenize the user query with the same options and rank matches with BM25.
DuckDB FTS uses `conjunctive := 1` when every query term must be present:

```sql
SELECT id, body, score
FROM (
  SELECT
    id,
    body,
    fts_main_docs.match_bm25(
      id,
      vaporetto_split('東京 検索エンジン', ' ', 'tags 名詞'),
      conjunctive := 1
    ) AS score
  FROM docs
) sq
WHERE score IS NOT NULL
ORDER BY score DESC, id;

-- 1|東京特許許可局で検索エンジンの実験をした。|...
```

For broader recall, omit `conjunctive := 1` and let DuckDB rank documents that
contain any query term:

```sql
SELECT id, body, score
FROM (
  SELECT
    id,
    body,
    fts_main_docs.match_bm25(
      id,
      vaporetto_split('東京 検索エンジン', ' ', 'tags 名詞')
    ) AS score
  FROM docs
) sq
WHERE score IS NOT NULL
ORDER BY score DESC, id
LIMIT 10;
```

DuckDB FTS indexes are not updated automatically when the input table changes.
Recreate the FTS index after changing indexed rows.

Packages without `-with-model` are smaller, but they require an explicit model
path through `DUCKDB_VAPORETTO_MODEL` or the `model <path>` option:

```sh
export DUCKDB_VAPORETTO_MODEL=/path/to/bccwj-suw+unidic_pos+kana.model.zst
duckdb -unsigned
```

```sql
LOAD './duckdb_vaporetto.duckdb_extension';

SELECT vaporetto_split(
  '東京特許許可局',
  '/',
  'model /path/to/bccwj-suw+unidic_pos+kana.model.zst'
);
```

### DuckDB-Wasm

Release artifacts whose name contains `wasm_eh` are for DuckDB-Wasm. They
bundle a Vaporetto model because browser-side DuckDB cannot read a local model
path from your machine in the same way as the native CLI.

Choose one of the two wasm builds:

- `wasm_eh-small-bccwj-suw-c1.0-with-model`: embeds
  `bccwj-suw_c1.0.model.zst`. This is the smaller build and is useful when
  download size matters, but it does not include part-of-speech tags.
- `wasm_eh-full-bccwj-suw-unidic-pos-kana-with-model`: embeds
  `bccwj-suw+unidic_pos+kana.model.zst`. This is the larger build and supports
  richer vocabulary and tag filtering such as `tags 名詞`.

DuckDB-Wasm requires unsigned extension loading for this project:

```js
await db.open({
  allowUnsignedExtensions: true,
  filesystem: { allowFullHTTPReads: true },
});

await conn.query(
  "LOAD 'http://127.0.0.1:4173/extensions/duckdb_vaporetto.duckdb_extension.wasm'"
);
await conn.query("SELECT vaporetto_split('東京特許許可局', '/')");
```

## Usage

Use `vaporetto_split()` to create the text that DuckDB FTS should index and to
create the query string passed to `match_bm25()`:

```sql
SELECT vaporetto_split('東京特許許可局 検索エンジン');
-- 東京 特許 許可 局 検索 エンジン

SELECT vaporetto_split('東京で検索エンジンを実験した。', '/', 'tags 名詞');
-- 東京/検索/エンジン/実験
```

The helper functions can filter by Vaporetto tags. The option string uses the
same syntax as `sqlite-vaporetto`:

```sql
SELECT vaporetto_split('東京で検索エンジンを実験した。', ' ', 'tags 名詞');
-- 東京 検索 エンジン 実験
```

To use `tags`, choose a model with tag prediction data, such as
[`bccwj-suw+unidic_pos+kana.model.zst`](https://github.com/daac-tools/vaporetto-models/releases).
The tag match is prefix-based, so `tags 名詞` keeps tags such as
`名詞-普通名詞-一般` and `名詞-固有名詞-地名-一般`.

Multiple tags can be comma-separated:

```sql
SELECT vaporetto_split(
  '東京で新しい検索エンジンを実験した。',
  ' ',
  'tags 名詞,形容詞'
);
```

Use `keep_untagged` with `tags` when you want selected tagged words plus tokens
that have no POS/tag data, such as ASCII identifiers or product codes outside
the model's tag prediction:

```sql
SELECT vaporetto_split(
  '東京でasdfoujbvaを検索した。',
  '/',
  'tags 名詞 keep_untagged'
);
-- 東京/asdfoujbva/検索
```

A practical full-text search setup is to keep content-word tags and untagged
tokens:

```sql
SELECT vaporetto_split(
  '東京で新しいDuckDB検索エンジンを実験した。',
  ' ',
  'tags 名詞,動詞,形容詞,副詞,接頭辞,接尾辞 keep_untagged'
);
-- 東京 新しい duckdb 検索 エンジン 実験 し
```

ASCII letters are case-insensitive by default. The returned token is folded to
lowercase unless `case sensitive` is specified:

```sql
SELECT vaporetto_split('Hello HELLO', '/');
-- hello/hello

SELECT vaporetto_split('Hello HELLO', '/', 'case sensitive');
-- Hello/HELLO
```

Builds can optionally embed
[`bccwj-suw+unidic_pos+kana.model.zst`](https://github.com/daac-tools/vaporetto-models/releases),
so `vaporetto_split()` works without a model path. Builds without an embedded
model require an explicit `model <path>` option or `DUCKDB_VAPORETTO_MODEL`. An
explicit `model <path>` option or `DUCKDB_VAPORETTO_MODEL` overrides the
embedded default.

Vaporetto model files are available from
[daac-tools/vaporetto-models releases](https://github.com/daac-tools/vaporetto-models/releases).

Optional arguments:

- `model <path>`: Vaporetto `.model` or `.model.zst` file. Overrides the
  embedded default model when present.
- `wsconst <chars>`: Vaporetto/KyTea-style character classes not to segment.
  Defaults to `DGR`.
- `tags <prefixes>`: Comma-separated Vaporetto tag prefixes to keep. When
  omitted, all tokens are returned.
- `keep_untagged`: With `tags`, also keep tokens that have no POS/tag data.
  This is useful for ASCII identifiers, product codes, and other tokens outside
  the model's tag prediction.
- `case sensitive`: Preserve ASCII uppercase/lowercase distinctions.
- `case insensitive`: Explicitly request the default ASCII case-insensitive
  behavior.

Environment variables:

- `DUCKDB_VAPORETTO_MODEL`: Default model path.
- `DUCKDB_VAPORETTO_WSCONST`: Default `wsconst`.
- `DUCKDB_VAPORETTO_TAGS`: Default comma-separated tag prefixes.

SQL helper functions:

- `vaporetto_split(text)`: Tokenize `text` and join tokens with spaces.
- `vaporetto_split(text, separator)`: Tokenize `text` and join tokens with
  `separator`.
- `vaporetto_split(text, separator, options)`: Tokenize with options such as
  `tags 名詞`, `model /path/to/model.zst`, or `case sensitive`.
- `vaporetto_and_query(text)`: Build a quoted boolean query string joined with
  `AND`.
- `vaporetto_and_query(text, options)`: Build an `AND` query string with
  tokenizer options.
- `vaporetto_or_query(text)`: Build a quoted boolean query string joined with
  `OR`.
- `vaporetto_or_query(text, options)`: Build an `OR` query string with
  tokenizer options.

`vaporetto_and_query()` and `vaporetto_or_query()` quote every generated token
and omit whitespace-only tokens. DuckDB's built-in `match_bm25()` expects a
plain term string rather than this boolean syntax, so use `vaporetto_split()`
for DuckDB FTS examples like the ones above.

## Developer Build

```sh
make build
```

For a distributable DuckDB extension:

```sh
make duckdb-extension
```

To build a native library with the default model embedded:

```sh
make embedded-release
```

Development builds only have an embedded default when built with
`DUCKDB_VAPORETTO_EMBED_MODEL`, or via:

```sh
DUCKDB_VAPORETTO_EMBED_MODEL=/path/to/bccwj-suw+unidic_pos+kana.model.zst \
  cargo build --release
```

To build both DuckDB-Wasm extensions:

```sh
make wasm-extension
```

The wasm build installs Emscripten under `.tmp/emsdk`, fetches models under
`.tmp/models`, and writes:

- `target/wasm32-unknown-emscripten/release/small/duckdb_vaporetto.duckdb_extension.wasm`
- `target/wasm32-unknown-emscripten/release/full/duckdb_vaporetto.duckdb_extension.wasm`

To build only one variant:

```sh
make wasm-extension-small
make wasm-extension-full
```

## Test With DuckDB

Temporary downloads are kept under `./.tmp/`.

`make test-extension` downloads a DuckDB CLI and a Vaporetto distribution model
into `.tmp/`, builds `duckdb_vaporetto.duckdb_extension`, and loads it with
`duckdb -unsigned`:

```sh
make test-extension
```

To test a build with the default model embedded:

```sh
make test-embedded
```

Core Rust tests can be run with:

```sh
make test
```

## Author

Yuichi Tateno ([@hotchpotch](https://github.com/hotchpotch))

## License

The `duckdb-vaporetto` extension is licensed under `MIT OR Apache-2.0`.

Release artifacts without `-with-model` do not bundle a Vaporetto model and use
the `duckdb-vaporetto` license. Release artifacts with `-with-model`
additionally bundle a Vaporetto model. The small wasm build bundles
`bccwj-suw_c1.0.model.zst`, licensed under `MIT OR Apache-2.0`. The full wasm
build and native `-with-model` builds bundle
`bccwj-suw+unidic_pos+kana.model.zst`, licensed under `BSD-3-Clause`.

See [MODEL_LICENSES.md](MODEL_LICENSES.md) for the bundled model notice and
license text.

## Acknowledgements

- [Vaporetto](https://github.com/daac-tools/vaporetto)
- [Vaporetto models](https://github.com/daac-tools/vaporetto-models/releases)
- [DuckDB FTS extension](https://duckdb.org/docs/current/core_extensions/full_text_search.html)
