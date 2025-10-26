use reqwest::StatusCode;
use rocket_db_pools::sqlx;
use std::time::Duration;
use thiserror::Error;

/// Errors that occur while interacting with search infrastructure.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("embedding HTTP error: {0}")]
    EmbeddingHttp(reqwest::Error),
    #[error("embedding service returned status {status}: {body}")]
    EmbeddingStatus { status: StatusCode, body: String },
    #[error("embedding response did not include any vectors")]
    EmptyEmbedding,
    #[error("embedding request timed out after {0:?}")]
    EmbeddingTimeout(Duration),
    #[error("meilisearch HTTP error: {0}")]
    MeilisearchHttp(reqwest::Error),
    #[error("meilisearch service returned status {status}: {body}")]
    MeilisearchStatus { status: StatusCode, body: String },
    #[error("database error: {0}")]
    Database(sqlx::Error),
}

impl SearchError {
    pub fn embedding_status(status: StatusCode, body: String) -> Self {
        SearchError::EmbeddingStatus { status, body }
    }

    pub fn meili_status(status: StatusCode, body: String) -> Self {
        SearchError::MeilisearchStatus { status, body }
    }
}
