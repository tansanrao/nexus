use super::SearchMode;
use std::env;
use std::str::FromStr;
use std::time::Duration;

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(default)
}

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_f32(key: &str, default: f32) -> f32 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or(default)
}

fn env_duration_millis(key: &str, default_millis: u64) -> Duration {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(default_millis))
}

fn env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Configuration for the embeddings service client.
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub base_url: String,
    pub model_id: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub request_timeout: Duration,
    pub document_prefix: String,
    pub query_prefix: String,
}

impl EmbeddingConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: env_string("EMBEDDINGS_URL", "http://embeddings:8080"),
            model_id: env_string("EMBEDDINGS_MODEL_ID", "nomic-ai/nomic-embed-text-v1.5"),
            dimension: env_usize("EMBEDDINGS_DIM", 768),
            batch_size: env_usize("EMBEDDINGS_BATCH_SIZE", 32),
            request_timeout: env_duration_millis("EMBEDDINGS_TIMEOUT_MS", 30_000),
            document_prefix: env_string("SEARCH_DOCUMENT_PREFIX", "search_document:"),
            query_prefix: env_string("SEARCH_QUERY_PREFIX", "search_query:"),
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Runtime configuration for search behavior.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub enable_semantic: bool,
    pub default_mode: SearchMode,
    pub hybrid_lexical_weight: f32,
    pub hybrid_semantic_weight: f32,
}

impl SearchConfig {
    pub fn from_env() -> Self {
        let enable_semantic = env_bool("SEARCH_ENABLE_VECTOR", true);
        let mut default_mode = env::var("SEARCH_DEFAULT_MODE")
            .ok()
            .and_then(|value| SearchMode::from_str(&value).ok())
            .unwrap_or_default();

        if !enable_semantic && default_mode != SearchMode::Lexical {
            default_mode = SearchMode::Lexical;
        }

        let lexical_weight = env_f32("SEARCH_HYBRID_LEXICAL_WEIGHT", 0.5);
        let semantic_weight = (1.0 - lexical_weight).clamp(0.0, 1.0);

        Self {
            enable_semantic,
            default_mode,
            hybrid_lexical_weight: lexical_weight,
            hybrid_semantic_weight: semantic_weight,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
