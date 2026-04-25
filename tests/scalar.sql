LOAD 'EXT_PATH';

SELECT 'SPLIT_SPACE', vaporetto_split('東京特許許可局');
SELECT 'SPLIT_SLASH', vaporetto_split('東京特許許可局', '/');
SELECT 'SPLIT_SPACED', vaporetto_split('東京特許許可局 検索エンジン', '/');
SELECT 'AND_QUERY', vaporetto_and_query('東京特許許可局');
SELECT 'AND_QUERY_SPACED', vaporetto_and_query('東京特許許可局 検索エンジン');
SELECT 'OR_QUERY', vaporetto_or_query('東京特許許可局');
SELECT 'OR_QUERY_SPACED', vaporetto_or_query('東京特許許可局 検索エンジン');
SELECT 'SPLIT_CASE_DEFAULT', vaporetto_split('Hello HELLO', '/');
SELECT 'SPLIT_CASE_SENSITIVE', vaporetto_split('Hello HELLO', '/', 'case sensitive');
SELECT 'AND_CASE_DEFAULT', vaporetto_and_query('Hello HELLO');
SELECT 'AND_CASE_SENSITIVE', vaporetto_and_query('Hello HELLO', 'case sensitive');
SELECT 'NOUN_SPLIT', vaporetto_split('東京で検索エンジンを実験した。', '/', 'tags 名詞');
SELECT 'NOUN_AND_QUERY', vaporetto_and_query('東京で検索エンジンを実験した。', 'tags 名詞');
SELECT 'NOUN_UNTAGGED_SPLIT', vaporetto_split('東京でasdfoujbvaを検索した。', '/', 'tags 名詞 keep_untagged');
SELECT 'NOUN_UNTAGGED_AND_QUERY', vaporetto_and_query('東京でasdfoujbvaを検索した。', 'tags 名詞 keep_untagged');
