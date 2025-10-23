//! Administrative endpoints for sync scheduling and system status.

use crate::error::ApiError;
use crate::sync::database::{backfill_fts_columns, refresh_search_indexes};
use crate::sync::pg_config::PgConfig;
use crate::sync::queue::{JobQueue, JobStatusInfo};
use crate::sync::reset_database;
use rocket::{State, get, post, serde::json::Json};
use rocket_db_pools::sqlx;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};

/// Request body for enqueuing sync jobs targeting specific mailing lists.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncRequest {
    /// Mailing list slugs to process.
    #[serde(rename = "mailingListSlugs")]
    mailing_list_slugs: Vec<String>,
}

/// Response returned when sync jobs are queued.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncStartResponse {
    /// Identifiers for the queued jobs.
    #[serde(rename = "jobIds")]
    job_ids: Vec<i32>,
    /// Human-readable summary message.
    message: String,
}

/// Simplified representation of a queued job.
#[derive(Debug, Serialize, JsonSchema)]
pub struct QueuedJobInfo {
    /// Job identifier.
    id: i32,
    /// Mailing list identifier.
    #[serde(rename = "mailingListId")]
    mailing_list_id: i32,
    /// Mailing list slug.
    #[serde(rename = "mailingListSlug")]
    mailing_list_slug: String,
    /// Mailing list display name.
    #[serde(rename = "mailingListName")]
    mailing_list_name: String,
    /// Position in the queue (1-based).
    position: i32,
}

/// Response describing the current sync queue.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncStatusResponse {
    /// Currently running job, if any.
    #[serde(rename = "currentJob")]
    current_job: Option<JobStatusInfo>,
    /// Jobs waiting in the queue.
    #[serde(rename = "queuedJobs")]
    queued_jobs: Vec<QueuedJobInfo>,
    /// Indicates whether a job is actively running.
    #[serde(rename = "isRunning")]
    is_running: bool,
}

/// Aggregated statistics about the database state.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DatabaseStatusResponse {
    #[serde(rename = "totalAuthors")]
    total_authors: i64,
    #[serde(rename = "totalEmails")]
    total_emails: i64,
    #[serde(rename = "totalThreads")]
    total_threads: i64,
    #[serde(rename = "totalRecipients")]
    total_recipients: i64,
    #[serde(rename = "totalReferences")]
    total_references: i64,
    #[serde(rename = "totalThreadMemberships")]
    total_thread_memberships: i64,
    #[serde(rename = "dateRangeStart")]
    date_range_start: Option<chrono::NaiveDateTime>,
    #[serde(rename = "dateRangeEnd")]
    date_range_end: Option<chrono::NaiveDateTime>,
}

/// Simple message wrapper for acknowledgement responses.
#[derive(Debug, Serialize, JsonSchema)]
pub struct MessageResponse {
    /// Response text.
    message: String,
}

/// Request payload for manual search index refresh operations.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchRefreshRequest {
    /// Restrict the refresh to a specific mailing list slug; omitted for all lists.
    #[serde(rename = "mailingListSlug")]
    pub mailing_list_slug: Option<String>,
    /// When true, reindex supporting GIN/GIN-trgm/vector indexes after recomputing tsvectors.
    #[serde(default)]
    pub reindex: bool,
}

/// Enqueue sync jobs for every enabled mailing list.
#[openapi(tag = "Admin")]
#[post("/admin/sync/start")]
pub async fn start_sync(pool: &State<sqlx::PgPool>) -> Result<Json<MessageResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let job_ids = queue
        .enqueue_all_enabled()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to enqueue jobs: {e}")))?;

    Ok(Json(MessageResponse {
        message: format!("Enqueued {} sync jobs", job_ids.len()),
    }))
}

/// Enqueue sync jobs for specific mailing lists.
#[openapi(tag = "Admin")]
#[post("/admin/sync/queue", data = "<request>")]
pub async fn queue_sync(
    request: Json<SyncRequest>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<SyncStartResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let mut job_ids = Vec::new();

    for slug in &request.mailing_list_slugs {
        let list: Option<(i32,)> =
            sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1 AND enabled = true")
                .bind(slug)
                .fetch_optional(pool.inner())
                .await?;

        if let Some((id,)) = list {
            let job_id = queue
                .enqueue_job(id)
                .await
                .map_err(|e| ApiError::InternalError(format!("Failed to enqueue job: {e}")))?;
            job_ids.push(job_id);
        } else {
            return Err(ApiError::BadRequest(format!(
                "Mailing list '{slug}' not found or disabled"
            )));
        }
    }

    if job_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "No mailing lists specified".to_string(),
        ));
    }

    Ok(Json(SyncStartResponse {
        job_ids: job_ids.clone(),
        message: format!("Queued {} sync job(s)", job_ids.len()),
    }))
}

/// Retrieve queue status and the currently running job.
#[openapi(tag = "Admin")]
#[get("/admin/sync/status")]
pub async fn get_sync_status(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<SyncStatusResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let jobs = queue
        .get_all_jobs()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get sync status: {e}")))?;

    let mut current_job = None;
    let mut queued_jobs = Vec::new();

    for (idx, job) in jobs.iter().enumerate() {
        if matches!(job.phase.as_str(), "waiting" | "parsing" | "threading") {
            if current_job.is_none() {
                current_job = Some(job.clone());
            } else {
                queued_jobs.push(QueuedJobInfo {
                    id: job.id,
                    mailing_list_id: job.mailing_list_id,
                    mailing_list_slug: job.slug.clone(),
                    mailing_list_name: job.name.clone(),
                    position: (idx + 1) as i32,
                });
            }
        }
    }

    let is_running = current_job.is_some();

    Ok(Json(SyncStatusResponse {
        current_job,
        queued_jobs,
        is_running,
    }))
}

/// Cancel all sync jobs, including the active job if one is running.
#[openapi(tag = "Admin")]
#[post("/admin/sync/cancel")]
pub async fn cancel_sync(pool: &State<sqlx::PgPool>) -> Result<Json<MessageResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let cancelled_count = queue
        .cancel_all_jobs()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to cancel jobs: {e}")))?;

    Ok(Json(MessageResponse {
        message: format!(
            "Cancelled {} job(s) (including running jobs)",
            cancelled_count
        ),
    }))
}

/// Refresh search-derived fields and optionally reindex supporting indexes.
#[openapi(tag = "Admin")]
#[post("/admin/search/index/refresh", data = "<request>")]
pub async fn refresh_search_index(
    request: Json<SearchRefreshRequest>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mailing_list_id = if let Some(slug) = &request.mailing_list_slug {
        let list: Option<(i32,)> = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool.inner())
            .await?;

        match list {
            Some((id,)) => Some(id),
            None => {
                return Err(ApiError::BadRequest(format!(
                    "Mailing list '{}' not found",
                    slug
                )));
            }
        }
    } else {
        None
    };

    let updated_rows = backfill_fts_columns(pool.inner(), mailing_list_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to refresh search vectors: {e}")))?;

    if request.reindex {
        refresh_search_indexes(pool.inner()).await.map_err(|e| {
            ApiError::InternalError(format!("Failed to reindex search structures: {e}"))
        })?;
    }

    let scope = request
        .mailing_list_slug
        .as_deref()
        .unwrap_or("all mailing lists");

    let message = if request.reindex {
        format!(
            "Refreshed search fields for {} ({} row updates and indexes reindexed)",
            scope, updated_rows
        )
    } else {
        format!(
            "Refreshed search fields for {} ({} row updates)",
            scope, updated_rows
        )
    };

    Ok(Json(MessageResponse { message }))
}

/// Drop and recreate the database schema.
#[openapi(tag = "Admin")]
#[post("/admin/database/reset")]
pub async fn reset_db(pool: &State<sqlx::PgPool>) -> Result<Json<MessageResponse>, ApiError> {
    reset_database(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database reset failed: {e}")))?;

    Ok(Json(MessageResponse {
        message: "Database reset successfully".to_string(),
    }))
}

/// Return aggregate statistics about the database.
#[openapi(tag = "Admin")]
#[get("/admin/database/status")]
pub async fn get_database_status(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<DatabaseStatusResponse>, ApiError> {
    let (
        total_authors,
        total_emails,
        total_threads,
        total_recipients,
        total_references,
        total_thread_memberships,
        date_range,
    ) = tokio::try_join!(
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM authors")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM emails")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM threads")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM email_recipients")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM email_references")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM thread_memberships")
                .fetch_one(pool.inner())
                .await
        },
        async {
            sqlx::query_as::<_, (Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>)>(
                "SELECT MIN(date), MAX(date) FROM emails",
            )
            .fetch_one(pool.inner())
            .await
            .or_else(|_| Ok::<_, sqlx::Error>((None, None)))
        }
    )?;

    Ok(Json(DatabaseStatusResponse {
        total_authors: total_authors.0,
        total_emails: total_emails.0,
        total_threads: total_threads.0,
        total_recipients: total_recipients.0,
        total_references: total_references.0,
        total_thread_memberships: total_thread_memberships.0,
        date_range_start: date_range.0,
        date_range_end: date_range.1,
    }))
}

/// Return PostgreSQL configuration details relevant for monitoring.
#[openapi(tag = "Admin")]
#[get("/admin/database/config")]
pub async fn get_database_config(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let config = PgConfig::check_config(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get config: {e}")))?;

    let json = serde_json::json!({
        "max_connections": config.max_connections,
        "shared_buffers": config.shared_buffers,
        "work_mem": config.work_mem,
        "maintenance_work_mem": config.maintenance_work_mem,
        "max_parallel_workers": config.max_parallel_workers,
        "max_worker_processes": config.max_worker_processes,
    });

    Ok(Json(json))
}
