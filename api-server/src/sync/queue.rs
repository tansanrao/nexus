use chrono::{DateTime, Utc};
use rocket_db_pools::sqlx::{self, PgPool};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: i32,
    pub mailing_list_id: i32,
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
            let job_id = self.enqueue_job(list_id).await?;
            job_ids.push(job_id);
        }

        Ok(job_ids)
    }

    /// Enqueue single job
    pub async fn enqueue_job(&self, mailing_list_id: i32) -> Result<i32, sqlx::Error> {
        let (id,): (i32,) = sqlx::query_as(
            r#"INSERT INTO sync_jobs (mailing_list_id, phase)
               VALUES ($1, 'waiting')
               RETURNING id"#,
        )
        .bind(mailing_list_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get next job atomically (SELECT FOR UPDATE SKIP LOCKED)
    pub async fn get_next_job(&self) -> Result<Option<SyncJob>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let job: Option<(i32, i32)> = sqlx::query_as(
            r#"SELECT id, mailing_list_id FROM sync_jobs
               WHERE phase = 'waiting'
               ORDER BY priority DESC, created_at ASC
               LIMIT 1
               FOR UPDATE SKIP LOCKED"#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((id, mailing_list_id)) = job {
            sqlx::query("UPDATE sync_jobs SET phase = 'parsing', started_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;

            Ok(Some(SyncJob {
                id,
                mailing_list_id,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update job phase
    pub async fn update_phase(&self, job_id: i32, phase: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE sync_jobs SET phase = $1 WHERE id = $2")
            .bind(phase)
            .bind(job_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Mark job complete
    pub async fn complete_job(&self, job_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE sync_jobs SET phase = 'done', completed_at = NOW() WHERE id = $1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Mark job failed
    pub async fn fail_job(&self, job_id: i32, error: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE sync_jobs SET phase = 'errored', completed_at = NOW(), error_message = $1 WHERE id = $2"
        )
        .bind(error)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Cancel all queued jobs (waiting only)
    pub async fn cancel_queued_jobs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE sync_jobs
               SET phase = 'done', completed_at = NOW()
               WHERE phase = 'waiting'"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Cancel ALL jobs including currently running ones
    pub async fn cancel_all_jobs(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"UPDATE sync_jobs
               SET phase = 'errored', completed_at = NOW(), error_message = 'Cancelled by user'
               WHERE phase IN ('waiting', 'parsing', 'threading')"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a job was cancelled (for dispatcher to detect cancellation)
    pub async fn is_job_cancelled(&self, job_id: i32) -> Result<bool, sqlx::Error> {
        let result: Option<(String,)> = sqlx::query_as("SELECT phase FROM sync_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|(phase,)| phase == "errored").unwrap_or(false))
    }

    /// Get all jobs (for status endpoint)
    pub async fn get_all_jobs(&self) -> Result<Vec<JobStatusInfo>, sqlx::Error> {
        let jobs: Vec<JobStatusInfo> = sqlx::query_as(
            r#"SELECT
                sj.id,
                sj.mailing_list_id,
                ml.slug,
                ml.name,
                sj.phase,
                sj.priority,
                sj.created_at,
                sj.started_at,
                sj.completed_at,
                sj.error_message
               FROM sync_jobs sj
               JOIN mailing_lists ml ON sj.mailing_list_id = ml.id
               WHERE sj.phase IN ('waiting', 'parsing', 'threading')
               ORDER BY sj.priority DESC, sj.created_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct JobStatusInfo {
    pub id: i32,
    pub mailing_list_id: i32,
    pub slug: String,
    pub name: String,
    pub phase: String,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
