use crate::error::ApiError;
use crate::sync::queue::{JobQueue, JobStatusInfo};
use crate::sync::reset_database;
use crate::sync::pg_config::PgConfig;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::sqlx;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncRequest {
    #[serde(rename = "mailingListSlugs")]
    mailing_list_slugs: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncStartResponse {
    #[serde(rename = "jobIds")]
    job_ids: Vec<i32>,
    message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct QueuedJobInfo {
    id: i32,
    #[serde(rename = "mailingListId")]
    mailing_list_id: i32,
    #[serde(rename = "mailingListSlug")]
    mailing_list_slug: String,
    #[serde(rename = "mailingListName")]
    mailing_list_name: String,
    position: i32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncStatusResponse {
    #[serde(rename = "currentJob")]
    current_job: Option<JobStatusInfo>,
    #[serde(rename = "queuedJobs")]
    queued_jobs: Vec<QueuedJobInfo>,
    #[serde(rename = "isRunning")]
    is_running: bool,
}

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

#[derive(Debug, Serialize, JsonSchema)]
pub struct MessageResponse {
    message: String,
}

/// Start sync for all enabled mailing lists
#[openapi(tag = "Admin")]
#[post("/admin/sync/start")]
pub async fn start_sync(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<MessageResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let job_ids = queue.enqueue_all_enabled()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to enqueue jobs: {}", e)))?;

    Ok(Json(MessageResponse {
        message: format!("Enqueued {} sync jobs", job_ids.len()),
    }))
}

/// Start sync for specific mailing lists
#[openapi(tag = "Admin")]
#[post("/admin/sync/queue", data = "<request>")]
pub async fn queue_sync(
    request: Json<SyncRequest>,
    pool: &State<sqlx::PgPool>,
) -> Result<Json<SyncStartResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());

    // Validate and get mailing list IDs
    let mut job_ids = Vec::new();

    for slug in &request.mailing_list_slugs {
        let list: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM mailing_lists WHERE slug = $1 AND enabled = true"
        )
        .bind(slug)
        .fetch_optional(pool.inner())
        .await?;

        if let Some((id,)) = list {
            let job_id = queue.enqueue_job(id)
                .await
                .map_err(|e| ApiError::InternalError(format!("Failed to enqueue job: {}", e)))?;
            job_ids.push(job_id);
        } else {
            return Err(ApiError::BadRequest(
                format!("Mailing list '{}' not found or disabled", slug)
            ));
        }
    }

    if job_ids.is_empty() {
        return Err(ApiError::BadRequest("No mailing lists specified".to_string()));
    }

    Ok(Json(SyncStartResponse {
        job_ids: job_ids.clone(),
        message: format!("Queued {} sync job(s)", job_ids.len()),
    }))
}

/// Get current sync status (running + queued jobs)
#[openapi(tag = "Admin")]
#[get("/admin/sync/status")]
pub async fn get_sync_status(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<SyncStatusResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let jobs = queue.get_all_jobs()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get sync status: {}", e)))?;

    // Separate current job from queued jobs
    let mut current_job = None;
    let mut queued_jobs = Vec::new();

    for (idx, job) in jobs.iter().enumerate() {
        match job.phase.as_str() {
            "waiting" | "parsing" | "threading" => {
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
            _ => {}
        }
    }

    let is_running = current_job.is_some();

    Ok(Json(SyncStatusResponse {
        current_job,
        queued_jobs,
        is_running,
    }))
}

/// Cancel ALL sync jobs including currently running ones
#[openapi(tag = "Admin")]
#[post("/admin/sync/cancel")]
pub async fn cancel_sync(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<MessageResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let cancelled_count = queue.cancel_all_jobs()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to cancel jobs: {}", e)))?;

    Ok(Json(MessageResponse {
        message: format!("Cancelled {} job(s) (including running jobs)", cancelled_count),
    }))
}

/// Reset the database (drop and recreate all tables)
#[openapi(tag = "Admin")]
#[post("/admin/database/reset")]
pub async fn reset_db(pool: &State<sqlx::PgPool>) -> Result<Json<MessageResponse>, ApiError> {
    reset_database(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Database reset failed: {}", e)))?;

    Ok(Json(MessageResponse {
        message: "Database reset successfully".to_string(),
    }))
}

/// Get database statistics (global, not per mailing list)
/// Queries are parallelized for faster response times
#[openapi(tag = "Admin")]
#[get("/admin/database/status")]
pub async fn get_database_status(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<DatabaseStatusResponse>, ApiError> {
    // Parallelize all independent COUNT queries for better performance
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
                "SELECT MIN(date), MAX(date) FROM emails"
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

/// Get PostgreSQL configuration for monitoring
#[openapi(tag = "Admin")]
#[get("/admin/database/config")]
pub async fn get_database_config(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let config = PgConfig::check_config(pool.inner())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get config: {}", e)))?;

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
