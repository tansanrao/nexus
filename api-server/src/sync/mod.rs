pub mod bulk_import;
pub mod git;
pub mod manifest;
pub mod parser;
pub mod pg_config;
pub mod queue;
pub mod worker;

use crate::sync::bulk_import::{BulkImporter, ImportStats};
use crate::sync::git::{GitManager, MailingListSyncConfig};
use crate::sync::parser::{parse_email, ParsedEmail};
use crate::sync::queue::JobQueue;
use rayon::prelude::*;
use rocket_db_pools::sqlx::PgPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Main synchronization orchestrator
pub struct SyncOrchestrator {
    pub git_manager: GitManager,
    pool: PgPool,
    mailing_list_id: i32,
}

impl SyncOrchestrator {
    pub fn new(git_config: MailingListSyncConfig, pool: PgPool, mailing_list_id: i32) -> Self {
        Self {
            git_manager: GitManager::new(git_config),
            pool,
            mailing_list_id,
        }
    }

    /// Load last indexed commits from database for incremental sync
    /// Returns a map of repo_order -> last_indexed_commit_hash
    async fn load_last_indexed_commits(&self) -> Result<std::collections::HashMap<i32, String>, String> {
        let rows: Vec<(i32, Option<String>)> = sqlx::query_as(
            r#"SELECT repo_order, last_indexed_commit
               FROM mailing_list_repositories
               WHERE mailing_list_id = $1"#
        )
        .bind(self.mailing_list_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load last indexed commits: {}", e))?;

        let mut map = std::collections::HashMap::new();
        for (repo_order, last_commit) in rows {
            if let Some(commit_hash) = last_commit {
                log::debug!(
                    "loaded checkpoint: list {} repo {} commit {}",
                    self.mailing_list_id, repo_order, &commit_hash[..8]
                );
                map.insert(repo_order, commit_hash);
            }
        }

        if map.is_empty() {
            log::debug!("no checkpoints found, will do full sync");
        } else {
            log::debug!("found {} checkpoints, will do incremental sync", map.len());
        }

        Ok(map)
    }

    /// Save last indexed commits to database after successful import
    /// Takes a map of repo_order -> latest_commit_hash
    async fn save_last_indexed_commits(&self, commits: &std::collections::HashMap<i32, String>) -> Result<(), String> {
        for (repo_order, commit_hash) in commits {
            log::debug!(
                "saving checkpoint: list {} repo {} commit {}",
                self.mailing_list_id, repo_order, &commit_hash[..8]
            );
            sqlx::query(
                r#"UPDATE mailing_list_repositories
                   SET last_indexed_commit = $1
                   WHERE mailing_list_id = $2 AND repo_order = $3"#
            )
            .bind(commit_hash)
            .bind(self.mailing_list_id)
            .bind(repo_order)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to save last indexed commit for repo {}: {}", repo_order, e))?;
        }

        Ok(())
    }

    /// Load last threaded timestamp from database for incremental threading
    /// Returns None if never threaded before (triggers full threading)
    async fn load_last_threaded_at(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
        let row: Option<(Option<chrono::DateTime<chrono::Utc>>,)> = sqlx::query_as(
            r#"SELECT last_threaded_at FROM mailing_lists WHERE id = $1"#
        )
        .bind(self.mailing_list_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load last threaded timestamp: {}", e))?;

        match row {
            Some((Some(timestamp),)) => {
                log::debug!(
                    "loaded threading checkpoint: list {} at {}",
                    self.mailing_list_id, timestamp
                );
                Ok(Some(timestamp))
            }
            _ => {
                log::debug!("no threading checkpoint found, will do full threading");
                Ok(None)
            }
        }
    }

    /// Save last threaded timestamp to database after successful threading
    async fn save_last_threaded_at(&self, timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), String> {
        log::debug!(
            "saving threading checkpoint: list {} at {}",
            self.mailing_list_id, timestamp
        );
        sqlx::query(
            r#"UPDATE mailing_lists
               SET last_threaded_at = $1
               WHERE id = $2"#
        )
        .bind(timestamp)
        .bind(self.mailing_list_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save last threaded timestamp: {}", e))?;

        Ok(())
    }

    /// Extract the latest commit hash per repository from the list of commits
    fn extract_latest_commits(&self, commits: &[(String, String, i32)]) -> std::collections::HashMap<i32, String> {
        let mut latest = std::collections::HashMap::new();

        // For each repository, store the last (most recent) commit we processed
        // Since commits are in chronological order, the last one is the latest
        for (commit_hash, _path, repo_order) in commits {
            latest.insert(*repo_order, commit_hash.clone());
        }

        latest
    }

    /// Optimized run_sync with 2-epoch sliding window for memory efficiency
    /// Full sync: process epochs sequentially with overlapping windows
    /// Incremental sync: process only last 2 epochs at once
    pub async fn run_sync(&self, job_id: i32, queue: &JobQueue) -> Result<ImportStats, String> {
        log::info!("sync started: list {} (job {})", self.mailing_list_id, job_id);

        // Phase 1: Validate
        queue.update_phase(job_id, "waiting").await
            .map_err(|e| format!("Failed to update phase: {}", e))?;

        self.git_manager.validate_all_mirrors()
            .map_err(|e| format!("Mirror validation failed: {}", e))?;

        let last_commits = self.load_last_indexed_commits().await?;
        let is_full_sync = last_commits.is_empty();

        // Determine which epochs to process
        let epochs_to_process: Vec<i32> = if is_full_sync {
            // Full sync: process ALL epochs sequentially
            self.git_manager.config.repos.iter()
                .map(|r| r.order)
                .collect()
        } else {
            // Incremental: process ONLY last 2 epochs
            let max_epoch = self.git_manager.config.repos.iter()
                .map(|r| r.order)
                .max()
                .unwrap_or(0);
            vec![max_epoch - 1, max_epoch]
                .into_iter()
                .filter(|&e| e >= 0)
                .collect()
        };

        log::info!(
            "sync strategy: {} - processing {} epochs",
            if is_full_sync { "FULL" } else { "INCREMENTAL" },
            epochs_to_process.len()
        );

        // Process epochs with sliding 2-epoch window
        let cumulative_stats = if is_full_sync {
            self.run_full_sync_with_sliding_window(job_id, queue, &last_commits, epochs_to_process).await?
        } else {
            self.run_incremental_sync(job_id, queue, &last_commits, epochs_to_process).await?
        };

        log::info!("sync complete: {} emails, {} threads", cumulative_stats.emails, cumulative_stats.threads);
        Ok(cumulative_stats)
    }

    /// Full sync: process epochs one-by-one with 2-epoch sliding cache window
    async fn run_full_sync_with_sliding_window(
        &self,
        job_id: i32,
        queue: &JobQueue,
        last_commits: &std::collections::HashMap<i32, String>,
        epochs: Vec<i32>,
    ) -> Result<ImportStats, String> {
        let mut cumulative_stats = ImportStats::default();

        for (idx, &current_epoch) in epochs.iter().enumerate() {
            log::info!("processing epoch {} ({}/{})", current_epoch, idx + 1, epochs.len());

            // Phase 2: Discovery + Parsing for this epoch
            queue.update_phase(job_id, "parsing").await
                .map_err(|e| format!("Failed to update phase: {}", e))?;

            let since = last_commits.get(&current_epoch);
            let commits = self.git_manager.get_commits_for_epoch(current_epoch, since.map(|s| s.as_str()))
                .map_err(|e| format!("Failed to get commits for epoch {}: {}", current_epoch, e))?;

            log::info!("epoch {}: discovered {} commits", current_epoch, commits.len());

            if commits.is_empty() {
                log::info!("epoch {}: no new commits, skipping", current_epoch);
                continue;
            }

            let parsed_emails = self.parse_all_parallel(commits.clone()).await?;
            log::info!("epoch {}: parsed {} emails", current_epoch, parsed_emails.len());

            // Tag emails with their epoch
            let emails_with_epoch: Vec<(String, ParsedEmail, i32)> = parsed_emails
                .into_iter()
                .map(|(commit, email)| (commit, email, current_epoch))
                .collect();

            // Phase 3: Import + Threading
            queue.update_phase(job_id, "threading").await
                .map_err(|e| format!("Failed to update phase: {}", e))?;

            // Calculate 2-epoch cache window: current and previous epoch
            let cache_epoch_range = (
                if current_epoch > 0 { current_epoch - 1 } else { 0 },
                current_epoch
            );

            let importer = BulkImporter::new(self.pool.clone(), self.mailing_list_id);
            let stats = importer
                .import_and_thread_epoch(emails_with_epoch, cache_epoch_range)
                .await
                .map_err(|e| format!("Import/thread failed for epoch {}: {}", current_epoch, e))?;

            // Log before merging (stats will be moved)
            log::info!("epoch {}: completed ({} emails, {} threads)",
                current_epoch, stats.emails, stats.threads);

            cumulative_stats.merge(stats);

            // Save checkpoint for this epoch
            let mut epoch_commits = std::collections::HashMap::new();
            if let Some((last_commit, _, _)) = commits.last() {
                epoch_commits.insert(current_epoch, last_commit.clone());
            }
            self.save_last_indexed_commits(&epoch_commits).await?;
            self.save_last_threaded_at(chrono::Utc::now()).await?;
        }

        Ok(cumulative_stats)
    }

    /// Incremental sync: process last 2 epochs together
    async fn run_incremental_sync(
        &self,
        job_id: i32,
        queue: &JobQueue,
        last_commits: &std::collections::HashMap<i32, String>,
        epochs: Vec<i32>,
    ) -> Result<ImportStats, String> {
        // Phase 2: Discovery + Parsing
        queue.update_phase(job_id, "parsing").await
            .map_err(|e| format!("Failed to update phase: {}", e))?;

        // Collect commits with their epoch tags
        let mut all_commits_with_epochs = Vec::new();
        for &epoch in &epochs {
            let since = last_commits.get(&epoch);
            let commits = self.git_manager.get_commits_for_epoch(epoch, since.map(|s| s.as_str()))
                .map_err(|e| format!("Failed to get commits for epoch {}: {}", epoch, e))?;
            log::info!("epoch {}: found {} commits", epoch, commits.len());

            // Tag each commit with its epoch
            for commit in commits {
                all_commits_with_epochs.push((commit, epoch));
            }
        }

        log::info!("discovered {} total commits from epochs {:?}", all_commits_with_epochs.len(), epochs);

        if all_commits_with_epochs.is_empty() {
            log::info!("no new commits found");
            return Ok(ImportStats::default());
        }

        // Parse all commits (extract commit tuples for parsing)
        let commits_for_parsing: Vec<(String, String, i32)> = all_commits_with_epochs
            .iter()
            .map(|((hash, path, _epoch), tagged_epoch)| (hash.clone(), path.clone(), *tagged_epoch))
            .collect();

        let parsed_emails = self.parse_all_parallel(commits_for_parsing.clone()).await?;
        log::info!("parsed {} emails", parsed_emails.len());

        // Tag emails with their epoch (from the commit's repo_order)
        let emails_with_epoch: Vec<(String, ParsedEmail, i32)> = parsed_emails
            .into_iter()
            .zip(commits_for_parsing.iter())
            .map(|((commit, email), (_, _, epoch))| (commit, email, *epoch))
            .collect();

        // Phase 3: Import + Threading
        queue.update_phase(job_id, "threading").await
            .map_err(|e| format!("Failed to update phase: {}", e))?;

        let cache_epoch_range = (epochs[0], *epochs.last().unwrap());

        let importer = BulkImporter::new(self.pool.clone(), self.mailing_list_id);
        let stats = importer
            .import_and_thread_epoch(emails_with_epoch, cache_epoch_range)
            .await
            .map_err(|e| format!("Import/thread failed: {}", e))?;

        // Phase 4: Save checkpoints
        let all_commits_for_checkpoint: Vec<(String, String, i32)> = all_commits_with_epochs
            .iter()
            .map(|((hash, path, epoch), _)| (hash.clone(), path.clone(), *epoch))
            .collect();
        let latest_commits = self.extract_latest_commits(&all_commits_for_checkpoint);
        self.save_last_indexed_commits(&latest_commits).await?;
        self.save_last_threaded_at(chrono::Utc::now()).await?;

        Ok(stats)
    }

    /// Parse all commits in parallel using Rayon
    async fn parse_all_parallel(
        &self,
        commits: Vec<(String, String, i32)>,
    ) -> Result<Vec<(String, ParsedEmail)>, String> {
        let total = commits.len();
        log::info!("parsing {} commits with {} threads", total, num_cpus::get());

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .map_err(|e| format!("Failed to create thread pool: {}", e))?;

        let parse_success = Arc::new(AtomicUsize::new(0));
        let parse_errors = Arc::new(AtomicUsize::new(0));

        let parsed = thread_pool.install(|| {
            commits.par_iter()
                .filter_map(|(commit, path, repo)| {
                    match self.git_manager.get_blob_data(commit, path, *repo) {
                        Ok(blob) => {
                            match parse_email(&blob) {
                                Ok(email) => {
                                    parse_success.fetch_add(1, Ordering::Relaxed);
                                    Some((commit.clone(), email))
                                }
                                Err(e) => {
                                    parse_errors.fetch_add(1, Ordering::Relaxed);
                                    log::warn!("parse error for {}: {}", commit, e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            parse_errors.fetch_add(1, Ordering::Relaxed);
                            log::warn!("blob error for {}: {}", commit, e);
                            None
                        }
                    }
                })
                .collect()
        });

        log::info!("parsing complete: {} ok, {} errors",
            parse_success.load(Ordering::Relaxed),
            parse_errors.load(Ordering::Relaxed));

        Ok(parsed)
    }

}

/// Run database migrations
/// This is idempotent - migrations that have already been applied will be skipped
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    log::info!("running database migrations");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    log::info!("database migrations completed");
    Ok(())
}

/// Reset database by dropping and recreating all tables
/// This will drop ALL mailing list partitions as well
pub async fn reset_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    log::info!("resetting database schema");

    // Drop all existing tables in reverse order of dependencies
    // PostgreSQL CASCADE will handle partitions
    sqlx::query("DROP TABLE IF EXISTS thread_memberships CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS threads CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS email_recipients CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS email_references CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS emails CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS author_mailing_list_activity CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS author_name_aliases CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS authors CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS mailing_list_repositories CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS mailing_lists CASCADE")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS sync_jobs CASCADE")
        .execute(pool)
        .await?;

    // Drop the sqlx migrations tracking table to allow re-running migrations
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations CASCADE")
        .execute(pool)
        .await?;

    log::info!("all tables dropped, running migrations");

    // Run all migrations from scratch
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    log::info!("database schema created via migrations");
    log::info!("call /api/admin/mailing-lists/seed to populate lists");
    Ok(())
}

/// Create all partitions for a specific mailing list
///
/// NOTE: Indexes are NOT created here - they are automatically created by PostgreSQL
/// when you create indexes on the parent partitioned table. This follows PostgreSQL
/// best practices for partitioned tables.
pub async fn create_mailing_list_partitions(pool: &PgPool, list_id: i32, slug: &str) -> Result<(), sqlx::Error> {
    log::debug!("creating partitions: {} (id={})", slug, list_id);

    // Sanitize slug for use in table names (replace hyphens with underscores)
    let safe_slug = slug.replace('-', "_");

    // Authors table is now global (not partitioned) - skip partition creation

    // Create emails partition
    sqlx::query(&format!(
        r#"CREATE TABLE emails_{} PARTITION OF emails
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create threads partition
    sqlx::query(&format!(
        r#"CREATE TABLE threads_{} PARTITION OF threads
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create email_recipients partition
    sqlx::query(&format!(
        r#"CREATE TABLE email_recipients_{} PARTITION OF email_recipients
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create email_references partition
    sqlx::query(&format!(
        r#"CREATE TABLE email_references_{} PARTITION OF email_references
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    // Create thread_memberships partition
    sqlx::query(&format!(
        r#"CREATE TABLE thread_memberships_{} PARTITION OF thread_memberships
           FOR VALUES IN ({})"#, safe_slug, list_id
    ))
    .execute(pool)
    .await?;

    log::debug!("partitions created: {}", slug);
    Ok(())
}

/// Drop all partitions for a specific mailing list
#[allow(dead_code)]
pub async fn drop_mailing_list_partitions(pool: &PgPool, slug: &str) -> Result<(), sqlx::Error> {
    log::debug!("dropping partitions: {}", slug);
    let safe_slug = slug.replace('-', "_");

    // Drop in reverse order of dependencies
    sqlx::query(&format!("DROP TABLE IF EXISTS thread_memberships_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS email_references_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS email_recipients_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS threads_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS emails_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;
    sqlx::query(&format!("DROP TABLE IF EXISTS authors_{} CASCADE", safe_slug))
        .execute(pool)
        .await?;

    log::debug!("partitions dropped: {}", slug);
    Ok(())
}
