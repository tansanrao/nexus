use crate::db::LinuxKbDb;
use crate::error::ApiError;
use crate::sync::git::{MailingListSyncConfig, RepoConfig};
use crate::sync::queue::{GlobalSyncStatus, JobQueue};
use crate::sync::{reset_database, SyncOrchestrator};
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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

/// Start sync jobs for multiple mailing lists (adds them to the queue)
#[post("/admin/sync/queue", data = "<request>")]
pub async fn queue_sync(
    request: Json<SyncRequest>,
    pool: &State<sqlx::PgPool>,
    job_queue: &State<Arc<Mutex<JobQueue>>>,
) -> Result<Json<SyncStartResponse>, ApiError> {
    let queue = job_queue.inner().lock().await;

    // Validate and get mailing list IDs
    let mut mailing_list_ids = Vec::new();

    for slug in &request.mailing_list_slugs {
        let list: Option<(i32,)> = sqlx::query_as(
            "SELECT id FROM mailing_lists WHERE slug = $1 AND enabled = true"
        )
        .bind(slug)
        .fetch_optional(pool.inner())
        .await?;

        if let Some((id,)) = list {
            mailing_list_ids.push(id);
        } else {
            return Err(ApiError::BadRequest(
                format!("Mailing list '{}' not found or disabled", slug)
            ));
        }
    }

    if mailing_list_ids.is_empty() {
        return Err(ApiError::BadRequest("No mailing lists specified".to_string()));
    }

    // Enqueue jobs
    let job_ids = queue.enqueue_jobs(mailing_list_ids)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to enqueue jobs: {}", e)))?;

    drop(queue); // Release lock

    // Start processing if not already running
    let queue_clone = Arc::clone(job_queue.inner());
    let pool_clone = pool.inner().clone();
    tokio::spawn(async move {
        process_queue(queue_clone, pool_clone).await;
    });

    Ok(Json(SyncStartResponse {
        job_ids: job_ids.clone(),
        message: format!("Queued {} sync job(s)", job_ids.len()),
    }))
}

/// Process the job queue (runs in background)
async fn process_queue(job_queue: Arc<Mutex<JobQueue>>, pool: sqlx::PgPool) {
    loop {
        let next_job = {
            let queue = job_queue.lock().await;
            queue.get_next_job().await
        };

        match next_job {
            Ok(Some(job)) => {
                log::info!("Processing sync job {} for mailing list {}", job.id, job.mailing_list_id);

                // Load repositories for this mailing list
                let repos: Vec<(String, i32)> = match sqlx::query_as(
                    "SELECT repo_url, repo_order FROM mailing_list_repositories
                     WHERE mailing_list_id = $1 ORDER BY repo_order"
                )
                .bind(job.mailing_list_id)
                .fetch_all(&pool)
                .await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("Failed to load repositories: {}", e);
                        let queue = job_queue.lock().await;
                        let _ = queue.fail_job(format!("Failed to load repositories: {}", e)).await;
                        continue;
                    }
                };

                if repos.is_empty() {
                    log::error!("No repositories configured for mailing list {}", job.mailing_list_id);
                    let queue = job_queue.lock().await;
                    let _ = queue.fail_job("No repositories configured".to_string()).await;
                    continue;
                }

                // Build repo configs
                let repo_configs: Vec<RepoConfig> = repos
                    .into_iter()
                    .map(|(url, order)| RepoConfig { url, order })
                    .collect();

                // Create sync config
                let git_config = MailingListSyncConfig::new(
                    job.mailing_list_id,
                    job.job_data.mailing_list_slug.clone(),
                    repo_configs
                );

                // Run sync
                let orchestrator = SyncOrchestrator::new(git_config, pool.clone(), job.mailing_list_id);
                let result = orchestrator.run_sync_with_queue(Arc::clone(&job_queue)).await;

                let queue = job_queue.lock().await;
                match result {
                    Ok(stats) => {
                        log::info!(
                            "Sync job {} completed successfully: {} authors, {} emails, {} threads",
                            job.id, stats.authors, stats.emails, stats.threads
                        );
                        let _ = queue.complete_job().await;
                    }
                    Err(e) => {
                        log::error!("Sync job {} failed: {}", job.id, e);
                        let _ = queue.fail_job(e).await;
                    }
                }
            }
            Ok(None) => {
                // No more jobs in queue
                break;
            }
            Err(e) => {
                log::error!("Failed to get next job: {}", e);
                break;
            }
        }
    }
}

/// Get the global sync status (current job + queue)
#[get("/admin/sync/status")]
pub async fn get_sync_status(
    job_queue: &State<Arc<Mutex<JobQueue>>>,
) -> Result<Json<GlobalSyncStatus>, ApiError> {
    let queue = job_queue.inner().lock().await;
    let status = queue.get_global_status()
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to get sync status: {}", e)))?;
    Ok(Json(status))
}

/// Cancel all queued sync jobs
#[post("/admin/sync/cancel")]
pub async fn cancel_sync(
    job_queue: &State<Arc<Mutex<JobQueue>>>,
) -> Result<Json<MessageResponse>, ApiError> {
    let queue = job_queue.inner().lock().await;
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
    mut db: Connection<LinuxKbDb>,
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
