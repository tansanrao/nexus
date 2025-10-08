use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "Queued"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetrics {
    pub emails_parsed: usize,
    pub parse_errors: usize,
    pub authors_imported: usize,
    pub emails_imported: usize,
    pub threads_created: usize,
}

impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            emails_parsed: 0,
            parse_errors: 0,
            authors_imported: 0,
            emails_imported: 0,
            threads_created: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub current_step: String,
    pub phase_details: Option<String>,
    pub processed: usize,
    pub total: Option<usize>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            current_step: String::new(),
            phase_details: None,
            processed: 0,
            total: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobData {
    pub mailing_list_slug: String,
    pub progress: JobProgress,
    pub metrics: SyncMetrics,
}

impl Default for JobData {
    fn default() -> Self {
        Self {
            mailing_list_slug: String::new(),
            progress: JobProgress::default(),
            metrics: SyncMetrics::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: i32,
    pub mailing_list_id: i32,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub job_data: JobData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    pub id: i32,
    pub mailing_list_id: i32,
    pub mailing_list_slug: String,
    pub mailing_list_name: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSyncStatus {
    pub current_job: Option<SyncJob>,
    pub queued_jobs: Vec<QueuedJob>,
    pub is_running: bool,
}

/// Job queue manager that stores jobs in PostgreSQL
pub struct JobQueue {
    pool: PgPool,
    /// In-memory state for the currently running job
    current_job_state: Arc<Mutex<Option<(i32, JobData)>>>,
}

impl JobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            current_job_state: Arc::new(Mutex::new(None)),
        }
    }

    /// Add jobs to the queue for multiple mailing lists
    pub async fn enqueue_jobs(&self, mailing_list_ids: Vec<i32>) -> Result<Vec<i32>, sqlx::Error> {
        let mut job_ids = Vec::new();

        for mailing_list_id in mailing_list_ids {
            // Get the slug for the mailing list
            let slug: (String,) = sqlx::query_as(
                "SELECT slug FROM mailing_lists WHERE id = $1"
            )
            .bind(mailing_list_id)
            .fetch_one(&self.pool)
            .await?;

            let job_data = JobData {
                mailing_list_slug: slug.0,
                progress: JobProgress::default(),
                metrics: SyncMetrics::default(),
            };

            let job_id: (i32,) = sqlx::query_as(
                r#"INSERT INTO sync_jobs (mailing_list_id, status, job_data)
                   VALUES ($1, $2, $3)
                   RETURNING id"#
            )
            .bind(mailing_list_id)
            .bind(JobStatus::Queued.to_string())
            .bind(serde_json::to_value(&job_data).unwrap())
            .fetch_one(&self.pool)
            .await?;

            job_ids.push(job_id.0);
        }

        Ok(job_ids)
    }

    /// Get the next job from the queue and mark it as running
    pub async fn get_next_job(&self) -> Result<Option<SyncJob>, sqlx::Error> {
        // Use a transaction to atomically get and update the next job
        let mut tx = self.pool.begin().await?;

        let job: Option<(i32, i32, String, DateTime<Utc>, sqlx::types::JsonValue)> = sqlx::query_as(
            r#"SELECT id, mailing_list_id, status, created_at, job_data
               FROM sync_jobs
               WHERE status = 'Queued'
               ORDER BY created_at ASC
               LIMIT 1
               FOR UPDATE SKIP LOCKED"#
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((id, mailing_list_id, _status, created_at, job_data_value)) = job {
            // Update status to Running
            sqlx::query(
                r#"UPDATE sync_jobs
                   SET status = 'Running', started_at = NOW()
                   WHERE id = $1"#
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            let job_data: JobData = serde_json::from_value(job_data_value)
                .unwrap_or_default();

            // Store in current job state
            let mut state = self.current_job_state.lock().await;
            *state = Some((id, job_data.clone()));

            Ok(Some(SyncJob {
                id,
                mailing_list_id,
                status: JobStatus::Running,
                created_at,
                started_at: Some(Utc::now()),
                completed_at: None,
                error_message: None,
                job_data,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update the progress of the current running job
    pub async fn update_progress(&self, progress: JobProgress) -> Result<(), sqlx::Error> {
        let state = self.current_job_state.lock().await;
        if let Some((job_id, job_data)) = state.as_ref() {
            let mut updated_data = job_data.clone();
            updated_data.progress = progress;

            sqlx::query(
                r#"UPDATE sync_jobs
                   SET job_data = $1
                   WHERE id = $2"#
            )
            .bind(serde_json::to_value(&updated_data).unwrap())
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Update the metrics of the current running job
    pub async fn update_metrics(&self, metrics: SyncMetrics) -> Result<(), sqlx::Error> {
        let state = self.current_job_state.lock().await;
        if let Some((job_id, job_data)) = state.as_ref() {
            let mut updated_data = job_data.clone();
            updated_data.metrics = metrics;

            sqlx::query(
                r#"UPDATE sync_jobs
                   SET job_data = $1
                   WHERE id = $2"#
            )
            .bind(serde_json::to_value(&updated_data).unwrap())
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Mark the current job as completed
    pub async fn complete_job(&self) -> Result<(), sqlx::Error> {
        let mut state = self.current_job_state.lock().await;
        if let Some((job_id, _)) = state.take() {
            sqlx::query(
                r#"UPDATE sync_jobs
                   SET status = 'Completed', completed_at = NOW()
                   WHERE id = $1"#
            )
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Mark the current job as failed
    pub async fn fail_job(&self, error: String) -> Result<(), sqlx::Error> {
        let mut state = self.current_job_state.lock().await;
        if let Some((job_id, _)) = state.take() {
            sqlx::query(
                r#"UPDATE sync_jobs
                   SET status = 'Failed', completed_at = NOW(), error_message = $1
                   WHERE id = $2"#
            )
            .bind(error)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Cancel all queued jobs
    pub async fn cancel_queued_jobs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE sync_jobs
               SET status = 'Cancelled', completed_at = NOW()
               WHERE status = 'Queued'"#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get the current global sync status
    pub async fn get_global_status(&self) -> Result<GlobalSyncStatus, sqlx::Error> {
        // Get currently running job
        let running_job: Option<(i32, i32, String, DateTime<Utc>, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<String>, sqlx::types::JsonValue)> = sqlx::query_as(
            r#"SELECT id, mailing_list_id, status, created_at, started_at, completed_at, error_message, job_data
               FROM sync_jobs
               WHERE status = 'Running'
               LIMIT 1"#
        )
        .fetch_optional(&self.pool)
        .await?;

        let current_job = running_job.map(|(id, mailing_list_id, status, created_at, started_at, completed_at, error_message, job_data_value)| {
            let job_data: JobData = serde_json::from_value(job_data_value)
                .unwrap_or_default();

            SyncJob {
                id,
                mailing_list_id,
                status: match status.as_str() {
                    "Running" => JobStatus::Running,
                    "Queued" => JobStatus::Queued,
                    "Completed" => JobStatus::Completed,
                    "Failed" => JobStatus::Failed,
                    "Cancelled" => JobStatus::Cancelled,
                    _ => JobStatus::Queued,
                },
                created_at,
                started_at,
                completed_at,
                error_message,
                job_data,
            }
        });

        // Get queued jobs with mailing list info
        let queued: Vec<(i32, i32, String, String)> = sqlx::query_as(
            r#"SELECT sj.id, sj.mailing_list_id, ml.slug, ml.name
               FROM sync_jobs sj
               JOIN mailing_lists ml ON sj.mailing_list_id = ml.id
               WHERE sj.status = 'Queued'
               ORDER BY sj.created_at ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let queued_jobs: Vec<QueuedJob> = queued
            .into_iter()
            .enumerate()
            .map(|(idx, (id, mailing_list_id, slug, name))| QueuedJob {
                id,
                mailing_list_id,
                mailing_list_slug: slug,
                mailing_list_name: name,
                position: (idx + 1) as i32,
            })
            .collect();

        Ok(GlobalSyncStatus {
            is_running: current_job.is_some(),
            current_job,
            queued_jobs,
        })
    }

    /// Check if there's a running job
    pub async fn is_job_running(&self) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sync_jobs WHERE status = 'Running'"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }
}
