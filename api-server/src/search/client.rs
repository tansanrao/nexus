use super::config::EmbeddingConfig;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur while interacting with the embeddings service.
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("embedding HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("embedding service returned status {status}: {body}")]
    Service { status: StatusCode, body: String },
    #[error("failed to decode embedding response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("embedding response count mismatch: expected {expected}, got {actual}")]
    CountMismatch { expected: usize, actual: usize },
    #[error("embedding vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

#[derive(Clone)]
pub struct EmbeddingClient {
    http: reqwest::Client,
    config: EmbeddingConfig,
}

impl EmbeddingClient {
    pub fn new(config: EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let timeout = config.request_timeout;
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .user_agent("nexus-search/0.1")
            .build()
            .map_err(EmbeddingError::Http)?;

        Ok(Self {
            http: client,
            config,
        })
    }

    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }

    pub async fn healthcheck(&self) -> Result<(), EmbeddingError> {
        let url = format!("{}/health", self.config.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(EmbeddingError::Http)?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(EmbeddingError::Service { status, body })
        }
    }

    pub async fn embed_documents(
        &self,
        documents: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.embed_with_prefix(&self.config.document_prefix, documents)
            .await
    }

    pub async fn embed_queries(&self, queries: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.embed_with_prefix(&self.config.query_prefix, queries)
            .await
    }

    async fn embed_with_prefix(
        &self,
        prefix: &str,
        items: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(items.len());
        let endpoint = format!("{}/embed", self.config.base_url.trim_end_matches('/'));
        let chunk_size = self.config.batch_size.max(1);

        for chunk in items.chunks(chunk_size) {
            let prefixed: Vec<String> = chunk
                .iter()
                .map(|item| format!("{prefix}{}", item))
                .collect();

            let payload = EmbeddingRequest {
                inputs: prefixed,
                truncate: Some(true),
                normalize: Some(true),
            };

            let response = self
                .http
                .post(&endpoint)
                .json(&payload)
                .send()
                .await
                .map_err(EmbeddingError::Http)?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(EmbeddingError::Service { status, body });
            }

            let body = response.bytes().await.map_err(EmbeddingError::Http)?;
            let parsed: EmbeddingResponse = serde_json::from_slice(&body)?;
            let embeddings = parsed.into_embeddings();

            if embeddings.len() != chunk.len() {
                return Err(EmbeddingError::CountMismatch {
                    expected: chunk.len(),
                    actual: embeddings.len(),
                });
            }

            for embedding in embeddings {
                if embedding.len() != self.config.dimension {
                    return Err(EmbeddingError::DimensionMismatch {
                        expected: self.config.dimension,
                        actual: embedding.len(),
                    });
                }
                results.push(embedding);
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    inputs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    normalize: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EmbeddingResponse {
    Bare(Vec<Vec<f32>>),
    Wrapped { embeddings: Vec<Vec<f32>> },
}

impl EmbeddingResponse {
    fn into_embeddings(self) -> Vec<Vec<f32>> {
        match self {
            EmbeddingResponse::Bare(values) => values,
            EmbeddingResponse::Wrapped { embeddings } => embeddings,
        }
    }
}
