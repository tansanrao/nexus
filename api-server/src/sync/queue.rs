use chrono::{DateTime, Utc};
use rocket_db_pools::sqlx::{self, PgPool};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "job_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Import,
    IndexMaintenance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: i32,
    pub job_type: JobType,
    pub mailing_list_id: Option<i32>,
    pub payload: Value,
}

pub struct JobQueue {
    pool: PgPool,
}

impl JobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Enqueue jobs for all enabled mailing lists
    pub async fn enqueue_all_enabled(&self) -> Result<Vec<i32>, sqlx::Error> {
        let list_ids: Vec<(i32,)> = sqlx::query_as(
            "SELECT id FROM mailing_lists WHERE enabled = true ORDER BY sync_priority DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut job_ids = Vec::new();
        for (list_id,) in list_ids {
            let job_id = self
                .enqueue_job(JobType::Import, Some(list_id), Value::Null, 0)
                .await?;
            job_ids.push(job_id);
        }

        Ok(job_ids)
    }

    /// Enqueue a single import job for a mailing list
    pub async fn enqueue_import_job(&self, mailing_list_id: i32) -> Result<i32, sqlx::Error> {
        self.enqueue_job(JobType::Import, Some(mailing_list_id), Value::Null, 0)
            .await
    }

    /// Enqueue single job (generic)
    pub async fn enqueue_job(
        &self,
        job_type: JobType,
        mailing_list_id: Option<i32>,
        payload: Value,
        priority: i32,
    ) -> Result<i32, sqlx::Error> {
        let (id,): (i32,) = sqlx::query_as(
            r#"INSERT INTO jobs (job_type, mailing_list_id, payload, priority)
               VALUES ($1, $2, $3, $4)
               RETURNING id"#,
        )
        .bind(job_type)
        .bind(mailing_list_id)
        .bind(payload)
        .bind(priority)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get next job atomically (SELECT FOR UPDATE SKIP LOCKED)
    pub async fn get_next_job(&self) -> Result<Option<Job>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let job: Option<(i32, JobType, Option<i32>, Value)> = sqlx::query_as(
            r#"SELECT id, job_type, mailing_list_id, payload FROM jobs
               WHERE status = 'queued'
               ORDER BY priority DESC, created_at ASC
               LIMIT 1
               FOR UPDATE SKIP LOCKED"#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((id, job_type, mailing_list_id, payload)) = job {
            sqlx::query(
                "UPDATE jobs SET status = 'running', started_at = COALESCE(started_at, NOW()), last_heartbeat = NOW() WHERE id = $1",
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            Ok(Some(Job {
                id,
                job_type,
                mailing_list_id,
                payload,
            }))
        } else {
            Ok(None)
        }
    }

    /// Mark job complete
    pub async fn complete_job(&self, job_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE jobs SET status = 'succeeded', completed_at = NOW(), last_heartbeat = NOW() WHERE id = $1",
        )
            .bind(job_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Mark job failed
    pub async fn fail_job(&self, job_id: i32, error: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE jobs SET status = 'failed', completed_at = NOW(), error_message = $1, last_heartbeat = NOW() WHERE id = $2",
        )
        .bind(error)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Record a heartbeat/progress update for a running job.
    pub async fn heartbeat(&self, job_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE jobs SET last_heartbeat = NOW() WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Cancel all queued jobs (waiting only)
    pub async fn cancel_queued_jobs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE jobs
               SET status = 'cancelled', completed_at = NOW(), error_message = 'Cancelled by user'
               WHERE status = 'queued'"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Cancel ALL jobs including currently running ones
    pub async fn cancel_all_jobs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE jobs
               SET status = 'cancelled', completed_at = NOW(), error_message = 'Cancelled by user'
               WHERE status IN ('queued', 'running')"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a job was cancelled (for dispatcher to detect cancellation)
    pub async fn is_job_cancelled(&self, job_id: i32) -> Result<bool, sqlx::Error> {
        let result: Option<(JobStatus,)> = sqlx::query_as("SELECT status FROM jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result
            .map(|(status,)| status == JobStatus::Cancelled)
            .unwrap_or(false))
    }

    /// Get all jobs (for status endpoint)
    pub async fn get_all_jobs(&self) -> Result<Vec<JobStatusInfo>, sqlx::Error> {
        let jobs: Vec<JobStatusInfo> = sqlx::query_as(
            r#"SELECT
                j.id,
                j.mailing_list_id,
                ml.slug,
                ml.name,
                j.job_type,
                j.status,
                j.priority,
                j.created_at,
                j.started_at,
                j.completed_at,
                j.error_message
               FROM jobs j
               LEFT JOIN mailing_lists ml ON j.mailing_list_id = ml.id
               WHERE j.status IN ('queued', 'running')
               ORDER BY j.priority DESC, j.created_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct JobStatusInfo {
    pub id: i32,
    #[serde(rename = "mailingListId")]
    pub mailing_list_id: Option<i32>,
    pub slug: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "jobType")]
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
