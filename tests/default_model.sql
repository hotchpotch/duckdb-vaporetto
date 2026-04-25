LOAD 'EXT_PATH';

SELECT 'DEFAULT_SPLIT', vaporetto_split('東京特許許可局', '/');
SELECT 'DEFAULT_AND_QUERY', vaporetto_and_query('東京特許許可局');

