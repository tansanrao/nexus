use crate::db::NexusDb;
use crate::error::ApiError;
use crate::sync::queue::{JobQueue, JobStatusInfo};
use crate::sync::reset_database;
use crate::sync::pg_config::PgConfig;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::{sqlx, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    mailing_list_slugs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncStartResponse {
    job_ids: Vec<i32>,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct QueuedJobInfo {
    id: i32,
    mailing_list_id: i32,
    mailing_list_slug: String,
    mailing_list_name: String,
    position: i32,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    current_job: Option<JobStatusInfo>,
    queued_jobs: Vec<QueuedJobInfo>,
    is_running: bool,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStatusResponse {
    total_authors: i64,
    total_emails: i64,
    total_threads: i64,
    total_recipients: i64,
    total_references: i64,
    total_thread_memberships: i64,
    date_range_start: Option<chrono::NaiveDateTime>,
    date_range_end: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    message: String,
}

/// Start sync for all enabled mailing lists
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

/// Cancel all queued sync jobs
#[post("/admin/sync/cancel")]
pub async fn cancel_sync(
    pool: &State<sqlx::PgPool>,
) -> Result<Json<MessageResponse>, ApiError> {
    let queue = JobQueue::new(pool.inner().clone());
    let cancelled_count = queue.cancel_queued_jobs()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to cancel jobs: {}", e)))?;

    Ok(Json(MessageResponse {
        message: format!("Cancelled {} queued job(s)", cancelled_count),
    }))
}

/// Reset the database (drop and recreate all tables)
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
#[get("/admin/database/status")]
pub async fn get_database_status(
    mut db: Connection<NexusDb>,
) -> Result<Json<DatabaseStatusResponse>, ApiError> {
    let total_authors: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM authors")
        .fetch_one(&mut **db)
        .await?;

    let total_emails: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM emails")
        .fetch_one(&mut **db)
        .await?;

    let total_threads: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threads")
        .fetch_one(&mut **db)
        .await?;

    let total_recipients: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_recipients")
        .fetch_one(&mut **db)
        .await?;

    let total_references: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM email_references")
        .fetch_one(&mut **db)
        .await?;

    let total_thread_memberships: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM thread_memberships")
        .fetch_one(&mut **db)
        .await?;

    let date_range: (Option<chrono::NaiveDateTime>, Option<chrono::NaiveDateTime>) =
        sqlx::query_as("SELECT MIN(date), MAX(date) FROM emails")
            .fetch_one(&mut **db)
            .await
            .unwrap_or((None, None));

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
