# duckdb-vaporetto

`duckdb-vaporetto` is a DuckDB loadable extension that exposes Japanese
tokenization helpers powered by [Vaporetto](https://github.com/daac-tools/vaporetto).

DuckDB does not expose the same FTS5 tokenizer hook that SQLite uses, so this
extension provides scalar functions that can be used directly in SQL:

```sql
LOAD './duckdb_vaporetto.duckdb_extension';

SELECT vaporetto_split('東京特許許可局', '/');
-- 東京/特許/許可/局

SELECT vaporetto_and_query('東京特許許可局');
-- "東京" AND "特許" AND "許可" AND "局"
```

Builds without an embedded model require either `DUCKDB_VAPORETTO_MODEL` or an
options string containing `model /path/to/model.zst`:

```sql
SELECT vaporetto_split(
  '東京特許許可局',
  '/',
  'model /path/to/bccwj-suw_c0.003.model.zst'
);
```

The options string accepts the same core arguments as `sqlite-vaporetto`:

- `model <path>`
- `wsconst <chars>`
- `tags <comma-separated-prefixes>`
- `case sensitive` or `case insensitive`

`vaporetto_split(text)`, `vaporetto_split(text, separator)`, and
`vaporetto_split(text, separator, options)` return tokenized text.

`vaporetto_and_query(text[, options])` and `vaporetto_or_query(text[, options])`
return quoted boolean query strings that match DuckDB FTS-style term syntax.

## Development

Temporary downloads are kept under `./.tmp/`.

```sh
make test
make duckdb-extension
make test-extension
```

`make test-extension` downloads a DuckDB CLI and the full tag-capable Vaporetto
model under `./.tmp/`, builds a `.duckdb_extension`, and loads it with `duckdb
-unsigned`.
