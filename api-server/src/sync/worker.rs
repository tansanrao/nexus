use crate::sync::{SyncOrchestrator, queue::JobQueue, git::{MailingListSyncConfig, RepoConfig}};
use rocket_db_pools::sqlx::PgPool;
use std::time::Duration;

pub struct SyncWorker {
    pool: PgPool,
    queue: JobQueue,
}

impl SyncWorker {
    pub fn new(pool: PgPool) -> Self {
        let queue = JobQueue::new(pool.clone());
        Self { pool, queue }
    }

    /// Run worker loop forever
    pub async fn run(self) -> ! {
        log::info!("sync worker started");

        loop {
            // Get next job
            let job = match self.queue.get_next_job().await {
                Ok(Some(j)) => {
                    log::info!("worker: claimed job {} for list {}", j.id, j.mailing_list_id);
                    j
                }
                Ok(None) => {
                    // No jobs available, sleep and retry
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(e) => {
                    log::error!("worker: failed to get job: {}", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            // Process job
            if let Err(e) = self.process_job(job).await {
                log::error!("worker: job processing failed: {}", e);
            }
        }
    }

    async fn process_job(&self, job: crate::sync::queue::SyncJob) -> Result<(), String> {
        let job_id = job.id;
        let list_id = job.mailing_list_id;

        // Load mailing list configuration
        let (slug, repos) = match self.load_mailing_list_config(list_id).await {
            Ok(config) => config,
            Err(e) => {
                let error_msg = format!("Failed to load config: {}", e);
                let _ = self.queue.fail_job(job_id, error_msg.clone()).await;
                return Err(error_msg);
            }
        };

        log::info!("job {}: processing mailing list '{}' with {} repos",
            job_id, slug, repos.len());

        // Create sync configuration
        let git_config = MailingListSyncConfig::new(list_id, slug, repos);

        // Create orchestrator
        let orchestrator = SyncOrchestrator::new(
            git_config,
            self.pool.clone(),
            list_id,
        );

        // Run sync with queue updates
        match orchestrator.run_sync(job_id, &self.queue).await {
            Ok(stats) => {
                log::info!("job {}: complete - {} emails, {} threads",
                    job_id, stats.emails, stats.threads);

                if let Err(e) = self.queue.complete_job(job_id).await {
                    log::error!("Failed to mark job {} as complete: {}", job_id, e);
                    return Err(format!("Failed to mark job complete: {}", e));
                }
            }
            Err(e) => {
                log::error!("job {}: failed - {}", job_id, e);

                if let Err(err) = self.queue.fail_job(job_id, e.clone()).await {
                    log::error!("Failed to mark job {} as failed: {}", job_id, err);
                }

                return Err(e);
            }
        }

        Ok(())
    }

    async fn load_mailing_list_config(&self, list_id: i32)
        -> Result<(String, Vec<RepoConfig>), sqlx::Error> {

        // Get mailing list slug
        let (slug,): (String,) = sqlx::query_as(
            "SELECT slug FROM mailing_lists WHERE id = $1"
        )
        .bind(list_id)
        .fetch_one(&self.pool)
        .await?;

        // Get repositories ordered by repo_order
        let repos: Vec<(String, i32)> = sqlx::query_as(
            "SELECT repo_url, repo_order FROM mailing_list_repositories
             WHERE mailing_list_id = $1 ORDER BY repo_order"
        )
        .bind(list_id)
        .fetch_all(&self.pool)
        .await?;

        let repo_configs = repos.into_iter()
            .map(|(url, order)| RepoConfig { url, order })
            .collect();

        Ok((slug, repo_configs))
    }
}
