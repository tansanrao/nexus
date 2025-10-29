use crate::search::embeddings::EmbeddingsClient;
use crate::search::error::SearchError;
use crate::search::models::{AuthorDocument, ThreadDocument};
use chrono::{DateTime, Utc};
use log::{debug, warn};
use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

const TASK_POLL_INTERVAL_MS: u64 = 200;
const TASK_TIMEOUT_MS: u64 = 60_000;
const UPSERT_BATCH_SIZE: usize = 400;

#[derive(Clone)]
pub struct SearchService {
    http: Client,
    base_url: String,
    api_key: Option<String>,
    embeddings: EmbeddingsClient,
    threads_index_uid: String,
    authors_index_uid: String,
    thread_embedder: String,
    default_semantic_ratio: f32,
    thread_embedding_dimensions: usize,
    allow_global_thread_search: bool,
}

#[derive(Debug, Clone)]
pub struct ThreadMailingListFilter {
    pub slug: String,
    pub mailing_list_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct ThreadSearchPayload {
    pub query: String,
    pub page: i64,
    pub size: i64,
    pub semantic_ratio: f32,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub has_patches: Option<bool>,
    pub starter_id: Option<i32>,
    pub participant_ids: Vec<i32>,
    pub series_id: Option<String>,
    pub mailing_lists: Vec<ThreadMailingListFilter>,
    pub sort_expressions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthorSearchPayload {
    pub query: Option<String>,
    pub page: i64,
    pub size: i64,
    pub sort_expression: Option<String>,
    pub mailing_lists: Vec<String>,
}

impl SearchService {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        embeddings_url: impl Into<String>,
        default_semantic_ratio: f32,
        thread_embedding_dimensions: usize,
        allow_global_thread_search: bool,
    ) -> Self {
        let http = Client::builder()
            .build()
            .expect("failed to construct reqwest client");

        let base = base_url.into().trim_end_matches('/').to_string();
        let embeddings = EmbeddingsClient::new(embeddings_url, http.clone());

        Self {
            http,
            base_url: base,
            api_key,
            embeddings,
            threads_index_uid: "threads".to_string(),
            authors_index_uid: "authors".to_string(),
            thread_embedder: "threads-qwen3".to_string(),
            default_semantic_ratio: default_semantic_ratio.clamp(0.0, 1.0),
            thread_embedding_dimensions,
            allow_global_thread_search,
        }
    }

    pub fn default_semantic_ratio(&self) -> f32 {
        self.default_semantic_ratio
    }

    pub fn allow_global_thread_search(&self) -> bool {
        self.allow_global_thread_search
    }

    pub fn threads_index_uid(&self) -> &str {
        &self.threads_index_uid
    }

    pub fn authors_index_uid(&self) -> &str {
        &self.authors_index_uid
    }

    pub fn thread_embedder(&self) -> &str {
        &self.thread_embedder
    }

    pub fn embeddings(&self) -> &EmbeddingsClient {
        &self.embeddings
    }

    pub fn thread_embedding_dimensions(&self) -> usize {
        self.thread_embedding_dimensions
    }

    pub async fn embed_with_fallback(
        &self,
        text: &str,
        expected_dimension: usize,
        context: &str,
    ) -> Result<Vec<f32>, SearchError> {
        if text.trim().is_empty() {
            return Ok(vec![0.0; expected_dimension]);
        }

        match self.embeddings.embed(text).await {
            Ok(mut vector) => {
                let original_len = vector.len();
                if original_len != expected_dimension {
                    warn!(
                        "embedding dimension mismatch for {}: expected {}, got {}. Adjusting output.",
                        context, expected_dimension, original_len
                    );
                    if original_len > expected_dimension {
                        vector.truncate(expected_dimension);
                    } else {
                        vector.resize(expected_dimension, 0.0);
                    }
                }
                Ok(vector)
            }
            Err(err) => {
                warn!(
                    "embedding request failed for {}: {}. Falling back to zero vector.",
                    context, err
                );
                Ok(vec![0.0; expected_dimension])
            }
        }
    }

    fn url_for(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.url_for(path);
        let builder = self.http.request(method, url);
        self.apply_auth(builder)
    }

    async fn send_json<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        payload: &T,
    ) -> Result<reqwest::Response, SearchError> {
        let response = self
            .request(method, path)
            .json(payload)
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error body".to_string());
            return Err(SearchError::meili_status(status, body));
        }

        Ok(response)
    }

    async fn send(&self, method: Method, path: &str) -> Result<reqwest::Response, SearchError> {
        let response = self
            .request(method, path)
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error body".to_string());
            return Err(SearchError::meili_status(status, body));
        }

        Ok(response)
    }

    async fn submit_task<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        payload: &T,
    ) -> Result<u64, SearchError> {
        info!("meilisearch submit_task: {method} {path}");
        let response = self.send_json(method, path, payload).await?;
        let task: TaskInfo = response
            .json()
            .await
            .map_err(SearchError::MeilisearchHttp)?;
        Ok(task.task_uid)
    }

    async fn ensure_vector_features(&self) -> Result<(), SearchError> {
        #[derive(Deserialize, Default)]
        struct ExperimentalFeaturesResponse {
            #[serde(default, rename = "vectorStore")]
            vector_store: Option<bool>,
            #[serde(default, rename = "vectorStoreSetting")]
            vector_store_setting: Option<bool>,
        }

        #[derive(Default, Serialize)]
        struct ExperimentalFeaturesPayload {
            #[serde(skip_serializing_if = "Option::is_none", rename = "vectorStore")]
            vector_store: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none", rename = "vectorStoreSetting")]
            vector_store_setting: Option<bool>,
        }

        let response = self
            .request(Method::GET, "/experimental-features")
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        let status = response.status();

        if status == StatusCode::NOT_FOUND {
            // Older versions without experimental feature endpoint
            return Ok(());
        }

        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error body".to_string());
            return Err(SearchError::meili_status(status, body));
        }

        let features: ExperimentalFeaturesResponse = response
            .json()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        let mut payload = ExperimentalFeaturesPayload::default();

        if let Some(enabled) = features.vector_store_setting {
            if !enabled {
                payload.vector_store_setting = Some(true);
            }
        }

        if let Some(enabled) = features.vector_store {
            if !enabled {
                payload.vector_store = Some(true);
            }
        }

        if payload.vector_store.is_none() && payload.vector_store_setting.is_none() {
            return Ok(());
        }

        let patch_response = self
            .request(Method::PATCH, "/experimental-features")
            .json(&payload)
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        let patch_status = patch_response.status();

        if patch_status.is_success() || patch_status == StatusCode::NOT_FOUND {
            return Ok(());
        }

        let body = patch_response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read error body".to_string());

        Err(SearchError::meili_status(patch_status, body))
    }

    async fn index_exists(&self, index_uid: &str) -> Result<bool, SearchError> {
        let response = self
            .request(Method::GET, &format!("/indexes/{}", index_uid))
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        match response.status() {
            status if status.is_success() => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            other => {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "failed to read error body".to_string());
                Err(SearchError::meili_status(other, body))
            }
        }
    }

    async fn wait_for_task(&self, task_uid: u64) -> Result<(), SearchError> {
        let mut elapsed_ms: u64 = 0;
        info!("meilisearch wait_for_task: awaiting task {task_uid}");

        loop {
            let response = self
                .send(Method::GET, &format!("/tasks/{}", task_uid))
                .await?;

            let status: TaskStatus = response
                .json()
                .await
                .map_err(SearchError::MeilisearchHttp)?;

            match status.status.as_str() {
                "succeeded" => {
                    info!(
                        "meilisearch wait_for_task: task {task_uid} succeeded after {elapsed_ms} ms"
                    );
                    return Ok(());
                }
                "failed" => {
                    let message = status
                        .error
                        .and_then(|err| err.message)
                        .unwrap_or_else(|| "Task failed without error message".to_string());
                    return Err(SearchError::meili_status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        message,
                    ));
                }
                "enqueued" | "processing" | "pending" => {
                    if elapsed_ms == 0 || elapsed_ms % 1000 == 0 {
                        info!(
                            "meilisearch wait_for_task: task {task_uid} still {} (elapsed {} ms)",
                            status.status, elapsed_ms
                        );
                    }
                    if elapsed_ms >= TASK_TIMEOUT_MS {
                        return Err(SearchError::meili_status(
                            StatusCode::REQUEST_TIMEOUT,
                            format!("Task {} timed out", task_uid),
                        ));
                    }
                    sleep(Duration::from_millis(TASK_POLL_INTERVAL_MS)).await;
                    elapsed_ms += TASK_POLL_INTERVAL_MS;
                }
                other => {
                    return Err(SearchError::meili_status(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unexpected task status '{}'", other),
                    ));
                }
            }
        }
    }

    async fn create_index_if_missing(
        &self,
        index_uid: &str,
        primary_key: &str,
    ) -> Result<(), SearchError> {
        if self.index_exists(index_uid).await? {
            return Ok(());
        }

        let payload = CreateIndexRequest {
            uid: index_uid.to_string(),
            primary_key: primary_key.to_string(),
        };

        let response = self
            .request(Method::POST, "/indexes")
            .json(&payload)
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        match response.status() {
            status if status == StatusCode::CONFLICT => Ok(()),
            status if status.is_success() => {
                let task: TaskInfo = response
                    .json()
                    .await
                    .map_err(SearchError::MeilisearchHttp)?;
                self.wait_for_task(task.task_uid).await
            }
            status => {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "failed to read error body".to_string());
                if status == StatusCode::INTERNAL_SERVER_ERROR && body.contains("already exists") {
                    Ok(())
                } else {
                    Err(SearchError::meili_status(status, body))
                }
            }
        }
    }

    async fn drop_index(&self, index_uid: &str) -> Result<(), SearchError> {
        if !self.index_exists(index_uid).await? {
            return Ok(());
        }

        let response = self
            .request(Method::DELETE, &format!("/indexes/{}", index_uid))
            .send()
            .await
            .map_err(SearchError::MeilisearchHttp)?;

        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(());
        }

        if status.is_success() {
            let task: TaskInfo = response
                .json()
                .await
                .map_err(SearchError::MeilisearchHttp)?;
            return self.wait_for_task(task.task_uid).await;
        }

        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "failed to read error body".to_string());

        let body_lower = body.to_ascii_lowercase();

        if status == StatusCode::INTERNAL_SERVER_ERROR && body_lower.contains("not found") {
            return Ok(());
        }

        Err(SearchError::meili_status(status, body))
    }

    pub async fn search_threads(
        &self,
        options: ThreadSearchPayload,
    ) -> Result<ThreadSearchResults, SearchError> {
        let semantic_ratio = options.semantic_ratio.clamp(0.0, 1.0);

        let vector = if semantic_ratio > 0.0 {
            Some(
                self.embed_with_fallback(
                    &options.query,
                    self.thread_embedding_dimensions(),
                    "thread search query",
                )
                .await?,
            )
        } else {
            None
        };

        let filters = build_thread_filters(&options);

        let payload = ThreadSearchRequest {
            q: &options.query,
            limit: options.size as usize,
            offset: ((options.page - 1) * options.size) as usize,
            filter: if filters.is_empty() {
                None
            } else {
                Some(filters)
            },
            attributes_to_highlight: Some(vec!["subject", "discussion_text"]),
            attributes_to_crop: Some(vec!["discussion_text"]),
            crop_length: Some(160),
            vector: vector.clone(),
            hybrid: vector.map(|_| HybridSpec {
                embedder: self.thread_embedder.clone(),
                semantic_ratio: semantic_ratio.clamp(0.0, 1.0),
            }),
            sort: if options.sort_expressions.is_empty() {
                None
            } else {
                Some(options.sort_expressions.clone())
            },
        };

        let response = self
            .send_json(
                Method::POST,
                &format!("/indexes/{}/search", self.threads_index_uid),
                &payload,
            )
            .await?;

        parse_search_response(response).await
    }

    pub async fn search_authors(
        &self,
        options: AuthorSearchPayload,
    ) -> Result<AuthorSearchResults, SearchError> {
        let filters = if options.mailing_lists.is_empty() {
            None
        } else {
            let clauses: Vec<String> = options
                .mailing_lists
                .into_iter()
                .map(|slug| format!("mailing_lists = \"{}\"", slug))
                .collect();

            Some(vec![join_filter_clauses(clauses)])
        };

        let sort_clause = options.sort_expression.map(|expr| vec![expr]);

        let payload = AuthorSearchRequest {
            q: options.query.as_deref(),
            limit: options.size as usize,
            offset: ((options.page - 1) * options.size) as usize,
            filter: filters,
            sort: sort_clause,
        };

        let response = self
            .send_json(
                Method::POST,
                &format!("/indexes/{}/search", self.authors_index_uid),
                &payload,
            )
            .await?;

        parse_author_search_response(response).await
    }

    pub async fn ensure_thread_index(&self) -> Result<(), SearchError> {
        self.ensure_vector_features().await?;
        self.create_index_if_missing(&self.threads_index_uid, "thread_id")
            .await?;

        let searchable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/searchable-attributes",
                    self.threads_index_uid
                ),
                &["subject", "discussion_text", "participants"],
            )
            .await?;
        self.wait_for_task(searchable_task).await?;

        let filterable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/filterable-attributes",
                    self.threads_index_uid
                ),
                &[
                    "mailing_list",
                    "mailing_list_id",
                    "participant_ids",
                    "starter_id",
                    "has_patches",
                    "series_id",
                    "start_ts",
                    "last_ts",
                    "message_count",
                ],
            )
            .await?;
        self.wait_for_task(filterable_task).await?;

        let sortable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/sortable-attributes",
                    self.threads_index_uid
                ),
                &["last_ts", "start_ts", "message_count"],
            )
            .await?;
        self.wait_for_task(sortable_task).await?;

        let distinct_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/distinct-attribute",
                    self.threads_index_uid
                ),
                &"thread_id",
            )
            .await?;
        self.wait_for_task(distinct_task).await?;

        let embedder_payload = EmbeddersPayload {
            threads: EmbedderSpec {
                source: "userProvided".to_string(),
                dimensions: self.thread_embedding_dimensions,
            },
        };
        let embedder_task = self
            .submit_task(
                Method::PATCH,
                &format!("/indexes/{}/settings/embedders", self.threads_index_uid),
                &embedder_payload,
            )
            .await?;
        self.wait_for_task(embedder_task).await?;

        Ok(())
    }

    pub async fn ensure_author_index(&self) -> Result<(), SearchError> {
        self.ensure_vector_features().await?;
        self.create_index_if_missing(&self.authors_index_uid, "author_id")
            .await?;

        let searchable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/searchable-attributes",
                    self.authors_index_uid
                ),
                &["canonical_name", "aliases", "email"],
            )
            .await?;
        self.wait_for_task(searchable_task).await?;

        let filterable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/filterable-attributes",
                    self.authors_index_uid
                ),
                &["mailing_lists"],
            )
            .await?;
        self.wait_for_task(filterable_task).await?;

        let sortable_task = self
            .submit_task(
                Method::PUT,
                &format!(
                    "/indexes/{}/settings/sortable-attributes",
                    self.authors_index_uid
                ),
                &[
                    "last_email_ts",
                    "first_email_ts",
                    "thread_count",
                    "email_count",
                ],
            )
            .await?;
        self.wait_for_task(sortable_task).await?;

        Ok(())
    }

    pub async fn upsert_threads(&self, documents: &[ThreadDocument]) -> Result<(), SearchError> {
        if documents.is_empty() {
            return Ok(());
        }

        debug!(
            "meilisearch upsert_threads: preparing to send {} documents (chunk size {})",
            documents.len(),
            UPSERT_BATCH_SIZE
        );

        for (chunk_index, chunk) in documents.chunks(UPSERT_BATCH_SIZE).enumerate() {
            debug!(
                "meilisearch upsert_threads: submitting chunk #{} ({} documents)",
                chunk_index + 1,
                chunk.len()
            );
            let task = self
                .submit_task(
                    Method::POST,
                    &format!("/indexes/{}/documents", self.threads_index_uid),
                    chunk,
                )
                .await?;
            self.wait_for_task(task).await?;
        }

        debug!(
            "meilisearch upsert_threads: completed {} documents",
            documents.len()
        );

        Ok(())
    }

    pub async fn upsert_authors(&self, documents: &[AuthorDocument]) -> Result<(), SearchError> {
        if documents.is_empty() {
            return Ok(());
        }

        debug!(
            "meilisearch upsert_authors: preparing to send {} documents (chunk size {})",
            documents.len(),
            UPSERT_BATCH_SIZE
        );

        for (chunk_index, chunk) in documents.chunks(UPSERT_BATCH_SIZE).enumerate() {
            debug!(
                "meilisearch upsert_authors: submitting chunk #{} ({} documents)",
                chunk_index + 1,
                chunk.len()
            );
            let task = self
                .submit_task(
                    Method::POST,
                    &format!("/indexes/{}/documents", self.authors_index_uid),
                    chunk,
                )
                .await?;
            self.wait_for_task(task).await?;
        }

        debug!(
            "meilisearch upsert_authors: completed {} documents",
            documents.len()
        );

        Ok(())
    }

    pub async fn delete_threads_by_mailing_list(
        &self,
        mailing_list_id: i32,
    ) -> Result<(), SearchError> {
        let payload = DeleteByFilter {
            filter: format!("mailing_list_id = {}", mailing_list_id),
        };

        let task = self
            .submit_task(
                Method::POST,
                &format!("/indexes/{}/documents/delete", self.threads_index_uid),
                &payload,
            )
            .await?;
        self.wait_for_task(task).await
    }

    pub async fn delete_authors_by_slug(&self, slug: &str) -> Result<(), SearchError> {
        let payload = DeleteByFilter {
            filter: format!("mailing_lists = \"{}\"", slug),
        };

        let task = self
            .submit_task(
                Method::POST,
                &format!("/indexes/{}/documents/delete", self.authors_index_uid),
                &payload,
            )
            .await?;
        self.wait_for_task(task).await
    }

    pub async fn reset_indexes(&self) -> Result<(), SearchError> {
        self.drop_index(&self.threads_index_uid).await?;
        self.drop_index(&self.authors_index_uid).await?;
        self.ensure_thread_index().await?;
        self.ensure_author_index().await?;
        Ok(())
    }

    fn apply_auth(&self, request: RequestBuilder) -> RequestBuilder {
        if let Some(ref key) = self.api_key {
            request
                .header("Authorization", format!("Bearer {}", key))
                .header("X-Meili-API-Key", key)
        } else {
            request
        }
    }
}

fn build_thread_filters(options: &ThreadSearchPayload) -> Vec<String> {
    let mut filters: Vec<String> = Vec::new();

    if !options.mailing_lists.is_empty() {
        let slug_filters: Vec<String> = options
            .mailing_lists
            .iter()
            .map(|ml| format!("mailing_list = \"{}\"", ml.slug))
            .collect();
        if !slug_filters.is_empty() {
            filters.push(join_filter_clauses(slug_filters));
        }

        let id_filters: Vec<String> = options
            .mailing_lists
            .iter()
            .filter_map(|ml| {
                ml.mailing_list_id
                    .map(|id| format!("mailing_list_id = {}", id))
            })
            .collect();
        if !id_filters.is_empty() {
            filters.push(join_filter_clauses(id_filters));
        }
    }

    if let Some(start) = options.start_date {
        filters.push(format!("last_ts >= {}", start.timestamp()));
    }

    if let Some(end) = options.end_date {
        filters.push(format!("start_ts <= {}", end.timestamp()));
    }

    if let Some(has_patches) = options.has_patches {
        filters.push(format!("has_patches = {}", has_patches));
    }

    if let Some(starter_id) = options.starter_id {
        filters.push(format!("starter_id = {}", starter_id));
    }

    if !options.participant_ids.is_empty() {
        let participant_filters: Vec<String> = options
            .participant_ids
            .iter()
            .map(|id| format!("participant_ids = {}", id))
            .collect();
        filters.push(join_filter_clauses(participant_filters));
    }

    if let Some(series_id) = options.series_id.as_ref() {
        filters.push(format!("series_id = \"{}\"", escape_quotes(series_id)));
    }

    filters
}

fn join_filter_clauses(clauses: Vec<String>) -> String {
    if clauses.len() == 1 {
        clauses.into_iter().next().unwrap()
    } else {
        clauses.join(" OR ")
    }
}

fn escape_quotes(input: &str) -> String {
    input.replace('"', "\\\"")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThreadSearchRequest<'a> {
    q: &'a str,
    limit: usize,
    offset: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes_to_highlight: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes_to_crop: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crop_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hybrid: Option<HybridSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HybridSpec {
    embedder: String,
    #[serde(rename = "semanticRatio")]
    semantic_ratio: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorSearchRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    q: Option<&'a str>,
    limit: usize,
    offset: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ThreadSearchResults {
    pub hits: Vec<ThreadHit>,
    pub total: i64,
}

#[derive(Debug)]
pub struct ThreadHit {
    pub document: ThreadDocument,
    pub ranking_score: Option<f32>,
    pub formatted: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct AuthorSearchResults {
    pub hits: Vec<AuthorHit>,
    pub total: i64,
}

#[derive(Debug)]
pub struct AuthorHit {
    pub document: AuthorDocument,
}

#[derive(serde::Deserialize)]
struct MeiliSearchResponse<T> {
    hits: Vec<MeiliHit<T>>,
    #[serde(rename = "estimatedTotalHits")]
    estimated_total_hits: Option<u64>,
    #[serde(rename = "totalHits")]
    total_hits: Option<u64>,
}

#[derive(serde::Deserialize)]
struct MeiliHit<T> {
    #[serde(flatten)]
    document: T,
    #[serde(rename = "_formatted")]
    formatted: Option<serde_json::Value>,
    #[serde(rename = "_rankingScore")]
    ranking_score: Option<f32>,
}

async fn parse_search_response(
    response: reqwest::Response,
) -> Result<ThreadSearchResults, SearchError> {
    let payload: MeiliSearchResponse<ThreadDocument> = response
        .json()
        .await
        .map_err(SearchError::MeilisearchHttp)?;

    let total = payload
        .total_hits
        .or(payload.estimated_total_hits)
        .unwrap_or(payload.hits.len() as u64) as i64;

    let hits = payload
        .hits
        .into_iter()
        .map(|hit| ThreadHit {
            document: hit.document,
            ranking_score: hit.ranking_score,
            formatted: hit.formatted,
        })
        .collect();

    Ok(ThreadSearchResults { hits, total })
}

async fn parse_author_search_response(
    response: reqwest::Response,
) -> Result<AuthorSearchResults, SearchError> {
    let payload: MeiliSearchResponse<AuthorDocument> = response
        .json()
        .await
        .map_err(SearchError::MeilisearchHttp)?;

    let total = payload
        .total_hits
        .or(payload.estimated_total_hits)
        .unwrap_or(payload.hits.len() as u64) as i64;

    let hits = payload
        .hits
        .into_iter()
        .map(|hit| AuthorHit {
            document: hit.document,
        })
        .collect();

    Ok(AuthorSearchResults { hits, total })
}

#[derive(serde::Deserialize)]
struct TaskInfo {
    #[serde(rename = "taskUid")]
    task_uid: u64,
}

#[derive(serde::Deserialize)]
struct TaskStatus {
    status: String,
    error: Option<TaskError>,
}

#[derive(serde::Deserialize)]
struct TaskError {
    message: Option<String>,
}

#[derive(serde::Serialize)]
struct CreateIndexRequest {
    uid: String,
    #[serde(rename = "primaryKey")]
    primary_key: String,
}

#[derive(serde::Serialize)]
struct DeleteByFilter {
    filter: String,
}

#[derive(serde::Serialize)]
struct EmbeddersPayload {
    #[serde(rename = "threads-qwen3")]
    threads: EmbedderSpec,
}

#[derive(serde::Serialize)]
struct EmbedderSpec {
    source: String,
    dimensions: usize,
}
