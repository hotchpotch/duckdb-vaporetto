use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use vaporetto::{CharacterBoundary, CharacterType, Model, Predictor, Sentence};
use vaporetto_rules::{
    SentenceFilter, StringFilter,
    sentence_filters::{ConcatGraphemeClustersFilter, KyteaWsConstFilter, SplitLinebreaksFilter},
    string_filters::KyteaFullwidthFilter,
};

include!(concat!(env!("OUT_DIR"), "/embedded_model.rs"));

#[derive(Debug, Clone, PartialEq, Eq)]
struct TokenizerConfig {
    model_path: Option<PathBuf>,
    wsconst: String,
    tags: Vec<String>,
    keep_untagged: bool,
    case_sensitive: bool,
}

struct ScalarTokenizer {
    predictor: Predictor,
    prefilter: KyteaFullwidthFilter,
    postfilters: Vec<Arc<dyn SentenceFilter>>,
    tags: Vec<String>,
    keep_untagged: bool,
    case_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    surface: String,
    start: usize,
    end: usize,
}

thread_local! {
    static TOKENIZER_CACHE: RefCell<HashMap<String, Rc<ScalarTokenizer>>> =
        RefCell::new(HashMap::new());
}

fn build_post_filters(wsconst: &str) -> Result<Vec<Arc<dyn SentenceFilter>>, String> {
    let mut postfilters: Vec<Arc<dyn SentenceFilter>> = vec![Arc::new(SplitLinebreaksFilter)];
    for c in wsconst.chars() {
        postfilters.push(match c {
            'D' => Arc::new(KyteaWsConstFilter::new(CharacterType::Digit)),
            'R' => Arc::new(KyteaWsConstFilter::new(CharacterType::Roman)),
            'H' => Arc::new(KyteaWsConstFilter::new(CharacterType::Hiragana)),
            'T' => Arc::new(KyteaWsConstFilter::new(CharacterType::Katakana)),
            'K' => Arc::new(KyteaWsConstFilter::new(CharacterType::Kanji)),
            'O' => Arc::new(KyteaWsConstFilter::new(CharacterType::Other)),
            'G' => Arc::new(ConcatGraphemeClustersFilter),
            _ => return Err(format!("invalid wsconst character: {c}")),
        });
    }
    Ok(postfilters)
}

fn load_model_from_bytes(bytes: &[u8], source: &str, compressed: bool) -> Result<Model, String> {
    let mut buf = Vec::new();
    if compressed {
        let mut reader = Cursor::new(bytes);
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut reader)
            .map_err(|e| format!("failed to create zstd decoder for {source}: {e}"))?;
        decoder
            .read_to_end(&mut buf)
            .map_err(|e| format!("failed to decompress model {source}: {e}"))?;
    } else {
        buf.extend_from_slice(bytes);
    }

    Model::read(&mut buf.as_slice()).map_err(|e| format!("failed to parse model {source}: {e}"))
}

fn load_model(path: &PathBuf) -> Result<Model, String> {
    let file = File::open(path).map_err(|e| format!("failed to open model {path:?}: {e}"))?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();

    if path.extension().is_some_and(|ext| ext == "zst") {
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut reader)
            .map_err(|e| format!("failed to create zstd decoder for {path:?}: {e}"))?;
        decoder
            .read_to_end(&mut buf)
            .map_err(|e| format!("failed to decompress model {path:?}: {e}"))?;
    } else {
        reader
            .read_to_end(&mut buf)
            .map_err(|e| format!("failed to read model {path:?}: {e}"))?;
    }

    Model::read(&mut buf.as_slice()).map_err(|e| format!("failed to parse model {path:?}: {e}"))
}

fn load_embedded_model() -> Result<Model, String> {
    let bytes = EMBEDDED_MODEL_BYTES.ok_or(
        "missing Vaporetto model path; pass an options string like 'model /path/to/model.zst', set DUCKDB_VAPORETTO_MODEL, or build with DUCKDB_VAPORETTO_EMBED_MODEL",
    )?;
    let compressed = bytes.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]);
    load_model_from_bytes(bytes, "embedded default model", compressed)
}

fn parse_tag_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_config_values(values: &[String]) -> Result<TokenizerConfig, String> {
    let mut model_path = std::env::var("DUCKDB_VAPORETTO_MODEL")
        .ok()
        .map(PathBuf::from);
    let mut wsconst =
        std::env::var("DUCKDB_VAPORETTO_WSCONST").unwrap_or_else(|_| "DGR".to_string());
    let mut tags = std::env::var("DUCKDB_VAPORETTO_TAGS")
        .ok()
        .map(|value| parse_tag_list(&value))
        .unwrap_or_default();
    let mut keep_untagged = false;
    let mut case_sensitive = false;

    let mut i = 0;
    while i < values.len() {
        match values.get(i).map(String::as_str) {
            Some("model") => {
                i += 1;
                let value = values.get(i).ok_or("options 'model ...' requires a path")?;
                model_path = Some(PathBuf::from(value));
            }
            Some("wsconst") => {
                i += 1;
                wsconst = values
                    .get(i)
                    .ok_or("options 'wsconst ...' requires a value")?
                    .to_owned();
            }
            Some("tags") => {
                i += 1;
                let value = values.get(i).ok_or("options 'tags ...' requires a value")?;
                tags.extend(parse_tag_list(value));
            }
            Some("case") => {
                i += 1;
                let value = values.get(i).ok_or("options 'case ...' requires a value")?;
                case_sensitive = match value.as_str() {
                    "sensitive" => true,
                    "insensitive" => false,
                    _ => {
                        return Err(format!(
                            "unknown vaporetto case option: {value}; expected sensitive or insensitive"
                        ));
                    }
                };
            }
            Some("keep_untagged") => {
                keep_untagged = true;
            }
            Some(value) if model_path.is_none() => {
                model_path = Some(PathBuf::from(value));
            }
            Some(value) => {
                return Err(format!("unknown vaporetto option: {value}"));
            }
            None => {}
        }
        i += 1;
    }

    Ok(TokenizerConfig {
        model_path,
        wsconst,
        tags,
        keep_untagged,
        case_sensitive,
    })
}

fn config_from_options(options: Option<&str>) -> Result<TokenizerConfig, String> {
    let values = options
        .unwrap_or_default()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    parse_config_values(&values)
}

fn config_cache_key(config: &TokenizerConfig) -> String {
    let model = config
        .model_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<embedded>".to_string());
    format!(
        "model={model}\nwsconst={}\ntags={}\nkeep_untagged={}\ncase_sensitive={}",
        config.wsconst,
        config.tags.join(","),
        config.keep_untagged,
        config.case_sensitive
    )
}

fn build_predictor(config: &TokenizerConfig) -> Result<Predictor, String> {
    let model = match &config.model_path {
        Some(path) => load_model(path)?,
        None => load_embedded_model()?,
    };
    Predictor::new(model, !config.tags.is_empty()).map_err(|e| e.to_string())
}

fn tokenize_text(
    original_text: &str,
    predictor: &Predictor,
    prefilter: &KyteaFullwidthFilter,
    postfilters: &[Arc<dyn SentenceFilter>],
    tags: &[String],
    keep_untagged: bool,
    case_sensitive: bool,
) -> Result<Vec<Token>, String> {
    if original_text.is_empty() {
        return Ok(Vec::new());
    }

    let prefiltered_text = prefilter.filter(original_text);
    let mut sentence = Sentence::from_raw(prefiltered_text).map_err(|e| e.to_string())?;

    predictor.predict(&mut sentence);
    for filter in postfilters {
        filter.filter(&mut sentence);
    }
    if !tags.is_empty() {
        sentence.fill_tags();
        if sentence.n_tags() == 0 {
            return Err("tag filtering requires a model with tag prediction data".to_string());
        }
    }
    let keep_tokens: Vec<bool> = if tags.is_empty() {
        Vec::new()
    } else {
        sentence
            .iter_tokens()
            .map(|token| {
                let mut token_tags = token.tags().iter().flatten().peekable();
                if token_tags.peek().is_none() {
                    return keep_untagged;
                }
                token_tags.any(|tag| tags.iter().any(|expected| tag.starts_with(expected)))
            })
            .collect()
    };

    let mut boundaries = Vec::with_capacity(sentence.boundaries().len() + 1);
    let mut char_indices = original_text.char_indices();
    char_indices.next();
    for ((idx, _), &boundary) in char_indices.zip(sentence.boundaries()) {
        if boundary == CharacterBoundary::WordBoundary {
            boundaries.push(idx);
        }
    }
    boundaries.push(original_text.len());

    let mut tokens = Vec::new();
    let mut start = 0usize;
    for (word_index, end) in boundaries.into_iter().enumerate() {
        if end <= start {
            continue;
        }
        if !tags.is_empty() && !keep_tokens.get(word_index).copied().unwrap_or(false) {
            start = end;
            continue;
        }
        let surface = &original_text[start..end];
        tokens.push(Token {
            surface: if case_sensitive {
                surface.to_string()
            } else {
                surface.to_ascii_lowercase()
            },
            start,
            end,
        });
        start = end;
    }

    Ok(tokens)
}

fn scalar_tokenizer(config: &TokenizerConfig) -> Result<Rc<ScalarTokenizer>, String> {
    let key = config_cache_key(config);
    TOKENIZER_CACHE.with(|cache| {
        if let Some(tokenizer) = cache.borrow().get(&key) {
            return Ok(Rc::clone(tokenizer));
        }

        let tokenizer = Rc::new(ScalarTokenizer {
            predictor: build_predictor(config)?,
            prefilter: KyteaFullwidthFilter,
            postfilters: build_post_filters(&config.wsconst)?,
            tags: config.tags.clone(),
            keep_untagged: config.keep_untagged,
            case_sensitive: config.case_sensitive,
        });
        cache.borrow_mut().insert(key, Rc::clone(&tokenizer));
        Ok(tokenizer)
    })
}

pub fn scalar_tokens(text: &str, options: Option<&str>) -> Result<Vec<String>, String> {
    let config = config_from_options(options)?;
    let tokenizer = scalar_tokenizer(&config)?;
    tokenize_text(
        text,
        &tokenizer.predictor,
        &tokenizer.prefilter,
        &tokenizer.postfilters,
        &tokenizer.tags,
        tokenizer.keep_untagged,
        tokenizer.case_sensitive,
    )
    .map(|tokens| {
        tokens
            .into_iter()
            .filter_map(|token| {
                let surface = token.surface.trim().to_string();
                (!surface.is_empty()).then_some(surface)
            })
            .collect()
    })
}

pub fn split(text: &str, separator: &str, options: Option<&str>) -> Result<String, String> {
    scalar_tokens(text, options).map(|tokens| tokens.join(separator))
}

fn quote_duckdb_fts_token(token: &str) -> String {
    format!("\"{}\"", token.replace('"', "\"\""))
}

pub fn fts_query(text: &str, options: Option<&str>, operator: &str) -> Result<String, String> {
    let tokens = scalar_tokens(text, options)?;
    let separator = format!(" {operator} ");
    Ok(tokens
        .iter()
        .map(|token| quote_duckdb_fts_token(token))
        .collect::<Vec<_>>()
        .join(&separator))
}

pub fn and_query(text: &str, options: Option<&str>) -> Result<String, String> {
    fts_query(text, options, "AND")
}

pub fn or_query(text: &str, options: Option<&str>) -> Result<String, String> {
    fts_query(text, options, "OR")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_default_config_from_empty_options() {
        let config = parse_config_values(&[]).expect("config parses");
        assert_eq!(config.wsconst, "DGR");
        assert!(config.tags.is_empty());
        assert!(!config.keep_untagged);
        assert!(!config.case_sensitive);
    }

    #[test]
    fn parses_options_in_sqlite_compatible_order() {
        let config = parse_config_values(&strings(&[
            "model",
            ".tmp/model.zst",
            "wsconst",
            "DG",
            "tags",
            "名詞,動詞",
            "keep_untagged",
            "case",
            "sensitive",
        ]))
        .expect("config parses");

        assert_eq!(config.model_path, Some(PathBuf::from(".tmp/model.zst")));
        assert_eq!(config.wsconst, "DG");
        assert_eq!(config.tags, strings(&["名詞", "動詞"]));
        assert!(config.keep_untagged);
        assert!(config.case_sensitive);
    }

    #[test]
    fn rejects_unknown_case_option() {
        let err = parse_config_values(&strings(&["case", "upper"])).expect_err("case rejects");
        assert!(err.contains("expected sensitive or insensitive"));
    }

    #[test]
    fn quotes_query_tokens_for_duckdb_fts_syntax() {
        assert_eq!(quote_duckdb_fts_token("東京"), "\"東京\"");
        assert_eq!(quote_duckdb_fts_token("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn drops_whitespace_around_token_surfaces() {
        let tokens = vec![
            Token {
                surface: "hello".to_string(),
                start: 0,
                end: 5,
            },
            Token {
                surface: " hello".to_string(),
                start: 5,
                end: 11,
            },
            Token {
                surface: " ".to_string(),
                start: 11,
                end: 12,
            },
        ];

        let surfaces = tokens
            .into_iter()
            .filter_map(|token| {
                let surface = token.surface.trim().to_string();
                (!surface.is_empty()).then_some(surface)
            })
            .collect::<Vec<_>>();

        assert_eq!(surfaces, strings(&["hello", "hello"]));
    }
}
