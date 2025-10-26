use crate::search::error::SearchError;
use log::{debug, warn};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use std::time::Duration;
use tokio::time::{sleep, timeout};

const EMBEDDING_REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const EMBEDDING_MAX_RETRIES: usize = 3;
const EMBEDDING_RETRY_BACKOFF_MS: u64 = 750;
const EMBEDDING_RETRY_BACKOFF_FACTOR: u64 = 2;

#[derive(Debug, Clone)]
pub struct EmbeddingsClient {
    base_url: String,
    http: Client,
}

impl EmbeddingsClient {
    pub fn new(base_url: impl Into<String>, http: Client) -> Self {
        let base = base_url.into().trim_end_matches('/').to_string();
        Self {
            base_url: base,
            http,
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, SearchError> {
        let mut backoff_ms = EMBEDDING_RETRY_BACKOFF_MS;
        for attempt in 1..=EMBEDDING_MAX_RETRIES {
            debug!(
                "embeddings: dispatching request (attempt {} of {})",
                attempt, EMBEDDING_MAX_RETRIES
            );

            let result = timeout(
                EMBEDDING_REQUEST_TIMEOUT,
                self.dispatch_embedding_request(text),
            )
            .await;

            match result {
                Ok(Ok(vector)) => return Ok(vector),
                Ok(Err(err)) => {
                    warn!("embeddings: request attempt {} failed: {}", attempt, err);
                    if attempt == EMBEDDING_MAX_RETRIES {
                        return Err(err);
                    }
                }
                Err(_) => {
                    warn!(
                        "embeddings: request attempt {} timed out after {:?}",
                        attempt, EMBEDDING_REQUEST_TIMEOUT
                    );
                    if attempt == EMBEDDING_MAX_RETRIES {
                        return Err(SearchError::EmbeddingTimeout(EMBEDDING_REQUEST_TIMEOUT));
                    }
                }
            }

            if attempt < EMBEDDING_MAX_RETRIES {
                let delay = Duration::from_millis(backoff_ms);
                debug!(
                    "embeddings: retrying after {:?} backoff (attempt {} of {})",
                    delay,
                    attempt + 1,
                    EMBEDDING_MAX_RETRIES
                );
                sleep(delay).await;
                backoff_ms = backoff_ms.saturating_mul(EMBEDDING_RETRY_BACKOFF_FACTOR);
            }
        }

        Err(SearchError::EmbeddingTimeout(EMBEDDING_REQUEST_TIMEOUT))
    }

    async fn dispatch_embedding_request(&self, text: &str) -> Result<Vec<f32>, SearchError> {
        #[derive(Serialize)]
        struct EmbeddingRequest<'a> {
            inputs: &'a [&'a str],
        }

        #[derive(Serialize)]
        struct OpenAiCompatibleRequest<'a> {
            input: &'a [&'a str],
        }

        let url = format!("{}/embeddings", self.base_url);
        let payload = EmbeddingRequest { inputs: &[text] };

        let primary_response = self
            .http
            .post(url.clone())
            .json(&payload)
            .send()
            .await
            .map_err(SearchError::EmbeddingHttp)?;

        if primary_response.status().is_success() {
            return Self::parse_embedding_response(primary_response).await;
        }

        let status = primary_response.status();
        let body = primary_response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read error body".to_string());

        // Some deployments expose an OpenAI-compatible API that expects an `input` field instead
        // of `inputs`. Retry once using that schema when the server explicitly reports the missing
        // `input` field.
        if status == StatusCode::UNPROCESSABLE_ENTITY && body.contains("missing field `input`") {
            let fallback_payload = OpenAiCompatibleRequest { input: &[text] };
            let fallback_response = self
                .http
                .post(url)
                .json(&fallback_payload)
                .send()
                .await
                .map_err(SearchError::EmbeddingHttp)?;

            if fallback_response.status().is_success() {
                return Self::parse_embedding_response(fallback_response).await;
            }

            let fallback_status = fallback_response.status();
            let fallback_body = fallback_response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error body".to_string());
            return Err(SearchError::embedding_status(
                fallback_status,
                fallback_body,
            ));
        }

        Err(SearchError::embedding_status(status, body))
    }

    async fn parse_embedding_response(
        response: reqwest::Response,
    ) -> Result<Vec<f32>, SearchError> {
        let parsed: EmbeddingResponse =
            response.json().await.map_err(SearchError::EmbeddingHttp)?;

        parsed
            .data
            .into_iter()
            .next()
            .map(|entry| entry.embedding)
            .ok_or(SearchError::EmptyEmbedding)
    }
}

#[derive(serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(serde::Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}
