pub mod git;
pub mod importer;
pub mod jobs;
pub mod parser;
pub mod queue;

use crate::sync::git::{GitManager, MailingListSyncConfig};
use crate::sync::importer::{Importer, ImportStats};
use crate::sync::jobs::{JobManager, JobStatus};
use crate::sync::parser::{parse_email, ParsedEmail};
use crate::sync::queue::{JobQueue, JobProgress, SyncMetrics};
use sqlx::PgPool;
use std::sync::Arc;

/// Main synchronization orchestrator
pub struct SyncOrchestrator {
    git_manager: GitManager,
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
                map.insert(repo_order, commit_hash);
            }
        }

        Ok(map)
    }

    /// Save last indexed commits to database after successful import
    /// Takes a map of repo_order -> latest_commit_hash
    async fn save_last_indexed_commits(&self, commits: &std::collections::HashMap<i32, String>) -> Result<(), String> {
        for (repo_order, commit_hash) in commits {
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

    /// Run incremental sync: discover new commits -> parse emails -> import to database
    /// Git mirroring is handled externally by grokmirror
    pub async fn run_sync(&self, job_manager: Arc<tokio::sync::Mutex<JobManager>>) -> Result<ImportStats, String> {
        log::info!("=== Starting incremental sync for mailing list {} ===", self.mailing_list_id);

        // Phase 1: Validate mirrors and load last indexed commits
        log::info!("Phase 1: Validating mirrors for mailing list {}", self.mailing_list_id);
        {
            let mgr = job_manager.lock().await;
            if mgr.is_cancelled() {
                return Err("Sync cancelled".to_string());
            }
            mgr.update_status(JobStatus::Syncing, "Validating git mirrors".to_string()).await;
            mgr.update_phase_details("Checking that grokmirror has synced repositories...".to_string()).await;
        }

        // Validate that all mirrors exist
        self.git_manager
            .validate_all_mirrors()
            .map_err(|e| {
                log::error!("Mirror validation failed: {}", e);
                format!("Mirror validation failed: {}", e)
            })?;

        // Load last indexed commits from database
        let last_commits = self.load_last_indexed_commits().await?;

        // Phase 2: Incremental Commit Discovery
        let is_incremental = !last_commits.is_empty();
        log::info!("Phase 2: Discovering {} email commits across all repositories",
            if is_incremental { "new" } else { "all" });
        {
            let mgr = job_manager.lock().await;
            if mgr.is_cancelled() {
                return Err("Sync cancelled".to_string());
            }
            let phase_msg = if is_incremental {
                format!("Discovering new commits since last sync ({} repos tracked)...", last_commits.len())
            } else {
                "Discovering all commits (first sync)...".to_string()
            };
            mgr.update_phase_details(phase_msg).await;
        }

        let commits = if is_incremental {
            self.git_manager
                .get_new_commits_since(&last_commits)
                .map_err(|e| {
                    log::error!("Failed to get new commits: {}", e);
                    format!("Failed to get new commits: {}", e)
                })?
        } else {
            self.git_manager
                .get_all_email_commits()
                .map_err(|e| {
                    log::error!("Failed to get commits: {}", e);
                    format!("Failed to get commits: {}", e)
                })?
        };

        let total_commits = commits.len();
        log::info!("Found {} email commits to process", total_commits);

        // Phase 3: Email Parsing (single-threaded, I/O bound)
        log::info!("Phase 3: Parsing {} emails", total_commits);
        {
            let mgr = job_manager.lock().await;
            mgr.update_status(JobStatus::Parsing, "Parsing emails".to_string()).await;
            mgr.update_progress(0, Some(total_commits)).await;
        }

        let mut parsed_emails: Vec<(String, ParsedEmail)> = Vec::new();
        let mut parse_success_count = 0;
        let mut parse_error_count = 0;

        for (idx, (commit_hash, path, repo_order)) in commits.iter().enumerate() {
            // Check for cancellation and update progress every 100 emails
            if idx % 100 == 0 {
                let mgr = job_manager.lock().await;
                if mgr.is_cancelled() {
                    log::warn!("Sync cancelled during parsing at email {}/{}", idx, total_commits);
                    return Err("Sync cancelled".to_string());
                }

                // Update progress
                mgr.update_progress(idx, Some(total_commits)).await;
                mgr.update_phase_details(format!(
                    "Parsed {}/{} emails ({} successful, {} failed)",
                    idx, total_commits, parse_success_count, parse_error_count
                )).await;
            }

            // Get blob data
            let blob_data = match self.git_manager.get_blob_data(commit_hash, path, *repo_order) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Failed to get blob data for commit {}: {}", commit_hash, e);
                    parse_error_count += 1;
                    continue;
                }
            };

            // Parse email
            match parse_email(&blob_data) {
                Ok(email) => {
                    parse_success_count += 1;
                    parsed_emails.push((commit_hash.clone(), email));
                },
                Err(e) => {
                    parse_error_count += 1;
                    log::warn!("Failed to parse email at commit {}: {}", commit_hash, e);
                }
            }
        }

        // Final progress update
        {
            let mgr = job_manager.lock().await;
            mgr.update_progress(total_commits, Some(total_commits)).await;
        }

        let final_parse_success = parse_success_count;
        let final_parse_errors = parse_error_count;

        log::info!(
            "Email parsing complete: {} successful, {} failed out of {} total",
            final_parse_success, final_parse_errors, total_commits
        );

        {
            let mgr = job_manager.lock().await;
            if mgr.is_cancelled() {
                return Err("Sync cancelled".to_string());
            }
            mgr.update_metrics(|metrics| {
                metrics.emails_parsed = final_parse_success;
                metrics.parse_errors = final_parse_errors;
            }).await;

            if final_parse_errors > 0 {
                mgr.add_warning(format!(
                    "Failed to parse {} out of {} emails",
                    final_parse_errors, total_commits
                )).await;
            }
        }

        // Phase 4: Database Import
        log::info!("Phase 4: Importing {} parsed emails to database", parsed_emails.len());
        {
            let mgr = job_manager.lock().await;
            mgr.update_status(JobStatus::Importing, "Importing to database".to_string()).await;
            mgr.update_phase_details("Importing authors, emails, and references...".to_string()).await;
        }

        let importer = Importer::new(self.pool.clone(), self.mailing_list_id);
        let stats = importer
            .import_emails(parsed_emails)
            .await
            .map_err(|e| {
                log::error!("Database import failed: {}", e);
                format!("Database import failed: {}", e)
            })?;

        {
            let mgr = job_manager.lock().await;
            if mgr.is_cancelled() {
                return Err("Sync cancelled".to_string());
            }
            mgr.update_metrics(|metrics| {
                metrics.authors_imported = stats.authors;
                metrics.emails_imported = stats.emails;
                metrics.threads_created = stats.threads;
            }).await;
        }

        // Phase 5: Save last indexed commits for incremental future syncs
        log::info!("Phase 5: Saving last indexed commits for future incremental syncs");
        let latest_commits = self.extract_latest_commits(&commits);
        self.save_last_indexed_commits(&latest_commits).await?;

        log::info!(
            "Sync complete: {} authors, {} emails, {} threads. Updated {} repository checkpoints.",
            stats.authors, stats.emails, stats.threads, latest_commits.len()
        );

        Ok(stats)
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

    /// Run incremental sync with the JobQueue (for queue-based processing)
    /// Git mirroring is handled externally by grokmirror
    pub async fn run_sync_with_queue(&self, job_queue: Arc<tokio::sync::Mutex<JobQueue>>) -> Result<ImportStats, String> {
        log::info!("=== Starting incremental sync for mailing list {} (queue mode) ===", self.mailing_list_id);

        // Phase 1: Validate mirrors and load last indexed commits
        log::info!("Phase 1: Validating mirrors for mailing list {}", self.mailing_list_id);
        {
            let queue = job_queue.lock().await;
            let _ = queue.update_progress(JobProgress {
                current_step: "Validating git mirrors".to_string(),
                phase_details: Some("Checking that grokmirror has synced repositories...".to_string()),
                processed: 0,
                total: None,
                errors: Vec::new(),
                warnings: Vec::new(),
            }).await;
        }

        // Validate that all mirrors exist
        self.git_manager
            .validate_all_mirrors()
            .map_err(|e| {
                log::error!("Mirror validation failed: {}", e);
                format!("Mirror validation failed: {}", e)
            })?;

        // Load last indexed commits from database
        let last_commits = self.load_last_indexed_commits().await?;

        // Phase 2: Incremental Commit Discovery
        let is_incremental = !last_commits.is_empty();
        log::info!("Phase 2: Discovering {} email commits across all repositories",
            if is_incremental { "new" } else { "all" });
        {
            let queue = job_queue.lock().await;
            let phase_msg = if is_incremental {
                format!("Discovering new commits since last sync ({} repos tracked)...", last_commits.len())
            } else {
                "Discovering all commits (first sync)...".to_string()
            };
            let _ = queue.update_progress(JobProgress {
                current_step: "Discovering commits".to_string(),
                phase_details: Some(phase_msg),
                processed: 0,
                total: None,
                errors: Vec::new(),
                warnings: Vec::new(),
            }).await;
        }

        let commits = if is_incremental {
            self.git_manager
                .get_new_commits_since(&last_commits)
                .map_err(|e| {
                    log::error!("Failed to get new commits: {}", e);
                    format!("Failed to get new commits: {}", e)
                })?
        } else {
            self.git_manager
                .get_all_email_commits()
                .map_err(|e| {
                    log::error!("Failed to get commits: {}", e);
                    format!("Failed to get commits: {}", e)
                })?
        };

        let total_commits = commits.len();
        log::info!("Found {} email commits to process", total_commits);

        // Phase 3: Email Parsing
        log::info!("Phase 3: Parsing {} emails", total_commits);
        {
            let queue = job_queue.lock().await;
            let _ = queue.update_progress(JobProgress {
                current_step: "Parsing emails".to_string(),
                phase_details: Some(format!("Parsing {} emails...", total_commits)),
                processed: 0,
                total: Some(total_commits),
                errors: Vec::new(),
                warnings: Vec::new(),
            }).await;
        }

        let mut parsed_emails: Vec<(String, ParsedEmail)> = Vec::new();
        let mut parse_success_count = 0;
        let mut parse_error_count = 0;

        for (idx, (commit_hash, path, repo_order)) in commits.iter().enumerate() {
            // Update progress every 100 emails
            if idx % 100 == 0 {
                let queue = job_queue.lock().await;
                let _ = queue.update_progress(JobProgress {
                    current_step: "Parsing emails".to_string(),
                    phase_details: Some(format!(
                        "Parsed {}/{} emails ({} successful, {} failed)",
                        idx, total_commits, parse_success_count, parse_error_count
                    )),
                    processed: idx,
                    total: Some(total_commits),
                    errors: Vec::new(),
                    warnings: Vec::new(),
                }).await;
            }

            // Get blob data
            let blob_data = match self.git_manager.get_blob_data(commit_hash, path, *repo_order) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Failed to get blob data for commit {}: {}", commit_hash, e);
                    parse_error_count += 1;
                    continue;
                }
            };

            // Parse email
            match parse_email(&blob_data) {
                Ok(email) => {
                    parse_success_count += 1;
                    parsed_emails.push((commit_hash.clone(), email));
                },
                Err(e) => {
                    parse_error_count += 1;
                    log::warn!("Failed to parse email at commit {}: {}", commit_hash, e);
                }
            }
        }

        // Update final parsing stats
        {
            let queue = job_queue.lock().await;
            let _ = queue.update_metrics(SyncMetrics {
                emails_parsed: parse_success_count,
                parse_errors: parse_error_count,
                authors_imported: 0,
                emails_imported: 0,
                threads_created: 0,
            }).await;
        }

        log::info!(
            "Email parsing complete: {} successful, {} failed out of {} total",
            parse_success_count, parse_error_count, total_commits
        );

        // Phase 4: Database Import
        log::info!("Phase 4: Importing {} parsed emails to database", parsed_emails.len());
        {
            let queue = job_queue.lock().await;
            let _ = queue.update_progress(JobProgress {
                current_step: "Importing to database".to_string(),
                phase_details: Some("Importing authors, emails, and references...".to_string()),
                processed: 0,
                total: None,
                errors: Vec::new(),
                warnings: Vec::new(),
            }).await;
        }

        let importer = Importer::new(self.pool.clone(), self.mailing_list_id);
        let stats = importer
            .import_emails(parsed_emails)
            .await
            .map_err(|e| {
                log::error!("Database import failed: {}", e);
                format!("Database import failed: {}", e)
            })?;

        // Update final metrics
        {
            let queue = job_queue.lock().await;
            let _ = queue.update_metrics(SyncMetrics {
                emails_parsed: parse_success_count,
                parse_errors: parse_error_count,
                authors_imported: stats.authors,
                emails_imported: stats.emails,
                threads_created: stats.threads,
            }).await;
        }

        // Phase 5: Save last indexed commits for incremental future syncs
        log::info!("Phase 5: Saving last indexed commits for future incremental syncs");
        let latest_commits = self.extract_latest_commits(&commits);
        self.save_last_indexed_commits(&latest_commits).await?;

        log::info!(
            "Sync complete: {} authors, {} emails, {} threads. Updated {} repository checkpoints.",
            stats.authors, stats.emails, stats.threads, latest_commits.len()
        );

        Ok(stats)
    }
}

/// Reset database by dropping and recreating all tables
/// This will drop ALL mailing list partitions as well
pub async fn reset_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    log::info!("Resetting database schema...");

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

    // Create metadata tables (not partitioned)

    // Table 0: sync_jobs (job queue)
    sqlx::query(
        r#"CREATE TABLE sync_jobs (
            id SERIAL PRIMARY KEY,
            mailing_list_id INTEGER NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('Queued', 'Running', 'Completed', 'Failed', 'Cancelled')),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            started_at TIMESTAMPTZ,
            completed_at TIMESTAMPTZ,
            error_message TEXT,
            job_data JSONB
        )"#
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX idx_sync_jobs_status ON sync_jobs(status)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_sync_jobs_mailing_list_id ON sync_jobs(mailing_list_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_sync_jobs_created_at ON sync_jobs(created_at)")
        .execute(pool)
        .await?;

    // Table 1: mailing_lists
    sqlx::query(
        r#"CREATE TABLE mailing_lists (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT UNIQUE NOT NULL,
            description TEXT,
            enabled BOOLEAN DEFAULT true,
            sync_priority INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            last_synced_at TIMESTAMPTZ
        )"#
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX idx_mailing_lists_slug ON mailing_lists(slug)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_mailing_lists_enabled ON mailing_lists(enabled)")
        .execute(pool)
        .await?;

    // Table 2: mailing_list_repositories (supports multiple repos per mailing list)
    sqlx::query(
        r#"CREATE TABLE mailing_list_repositories (
            id SERIAL PRIMARY KEY,
            mailing_list_id INTEGER REFERENCES mailing_lists(id) ON DELETE CASCADE,
            repo_url TEXT NOT NULL,
            repo_order INTEGER NOT NULL DEFAULT 0,
            last_indexed_commit TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE(mailing_list_id, repo_order)
        )"#
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX idx_mailing_list_repos_list_id ON mailing_list_repositories(mailing_list_id)")
        .execute(pool)
        .await?;

    // Create partitioned parent tables

    // Table 3: authors (GLOBAL - not partitioned)
    sqlx::query(
        r#"CREATE TABLE authors (
            id SERIAL PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            canonical_name TEXT,
            first_seen TIMESTAMPTZ DEFAULT NOW(),
            last_seen TIMESTAMPTZ DEFAULT NOW()
        )"#
    )
    .execute(pool)
    .await?;

    // Table 3a: author_name_aliases (track name variations)
    sqlx::query(
        r#"CREATE TABLE author_name_aliases (
            id SERIAL PRIMARY KEY,
            author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            usage_count INTEGER DEFAULT 1,
            first_seen TIMESTAMPTZ DEFAULT NOW(),
            last_seen TIMESTAMPTZ DEFAULT NOW(),
            UNIQUE (author_id, name)
        )"#
    )
    .execute(pool)
    .await?;

    // Table 3b: author_mailing_list_activity (per-list stats)
    sqlx::query(
        r#"CREATE TABLE author_mailing_list_activity (
            author_id INTEGER NOT NULL REFERENCES authors(id) ON DELETE CASCADE,
            mailing_list_id INTEGER NOT NULL REFERENCES mailing_lists(id) ON DELETE CASCADE,
            first_email_date TIMESTAMPTZ,
            last_email_date TIMESTAMPTZ,
            email_count BIGINT DEFAULT 0,
            thread_count BIGINT DEFAULT 0,
            PRIMARY KEY (author_id, mailing_list_id)
        )"#
    )
    .execute(pool)
    .await?;

    // Table 4: emails (partitioned by mailing_list_id)
    sqlx::query(
        r#"CREATE TABLE emails (
            id SERIAL,
            mailing_list_id INTEGER NOT NULL,
            message_id TEXT NOT NULL,
            git_commit_hash TEXT NOT NULL,
            author_id INTEGER NOT NULL,
            subject TEXT NOT NULL,
            normalized_subject TEXT,
            date TIMESTAMPTZ NOT NULL,
            in_reply_to TEXT,
            body TEXT,
            series_id TEXT,
            series_number INTEGER,
            series_total INTEGER,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            PRIMARY KEY (id, mailing_list_id),
            UNIQUE (mailing_list_id, message_id),
            UNIQUE (mailing_list_id, git_commit_hash)
        ) PARTITION BY LIST (mailing_list_id)"#
    )
    .execute(pool)
    .await?;

    // Table 5: threads (partitioned by mailing_list_id)
    sqlx::query(
        r#"CREATE TABLE threads (
            id SERIAL,
            mailing_list_id INTEGER NOT NULL,
            root_message_id TEXT NOT NULL,
            subject TEXT NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            last_date TIMESTAMPTZ NOT NULL,
            message_count INTEGER DEFAULT 0,
            PRIMARY KEY (id, mailing_list_id),
            UNIQUE (mailing_list_id, root_message_id)
        ) PARTITION BY LIST (mailing_list_id)"#
    )
    .execute(pool)
    .await?;

    // Table 6: email_recipients (partitioned by mailing_list_id)
    sqlx::query(
        r#"CREATE TABLE email_recipients (
            id SERIAL,
            mailing_list_id INTEGER NOT NULL,
            email_id INTEGER NOT NULL,
            author_id INTEGER NOT NULL,
            recipient_type TEXT CHECK (recipient_type IN ('to', 'cc')),
            PRIMARY KEY (id, mailing_list_id)
        ) PARTITION BY LIST (mailing_list_id)"#
    )
    .execute(pool)
    .await?;

    // Table 7: email_references (partitioned by mailing_list_id)
    sqlx::query(
        r#"CREATE TABLE email_references (
            mailing_list_id INTEGER NOT NULL,
            email_id INTEGER NOT NULL,
            referenced_message_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            PRIMARY KEY (mailing_list_id, email_id, referenced_message_id)
        ) PARTITION BY LIST (mailing_list_id)"#
    )
    .execute(pool)
    .await?;

    // Table 8: thread_memberships (partitioned by mailing_list_id)
    sqlx::query(
        r#"CREATE TABLE thread_memberships (
            mailing_list_id INTEGER NOT NULL,
            thread_id INTEGER NOT NULL,
            email_id INTEGER NOT NULL,
            depth INTEGER DEFAULT 0,
            PRIMARY KEY (mailing_list_id, thread_id, email_id)
        ) PARTITION BY LIST (mailing_list_id)"#
    )
    .execute(pool)
    .await?;

    log::info!("Base schema created successfully");

    // Create indexes on parent tables
    // PostgreSQL will automatically create matching indexes on all partitions
    log::info!("Creating indexes on parent partitioned tables...");

    // Authors indexes (no longer needed - email is UNIQUE, id is PRIMARY KEY)
    // Author name aliases indexes
    sqlx::query("CREATE INDEX idx_author_name_aliases_author_id ON author_name_aliases(author_id)")
        .execute(pool)
        .await?;

    // Author activity indexes
    sqlx::query("CREATE INDEX idx_author_activity_mailing_list ON author_mailing_list_activity(mailing_list_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_author_activity_last_date ON author_mailing_list_activity(last_email_date)")
        .execute(pool)
        .await?;

    // Emails indexes
    sqlx::query("CREATE INDEX idx_emails_author_id ON emails(author_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_emails_date ON emails(date)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_emails_in_reply_to ON emails(in_reply_to)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_emails_normalized_subject ON emails(normalized_subject)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_emails_series_id ON emails(series_id)")
        .execute(pool)
        .await?;

    // Threads indexes
    sqlx::query("CREATE INDEX idx_threads_start_date ON threads(start_date)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_threads_last_date ON threads(last_date)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_threads_message_count ON threads(message_count)")
        .execute(pool)
        .await?;

    // Email recipients indexes
    sqlx::query("CREATE INDEX idx_email_recipients_email_id ON email_recipients(email_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_email_recipients_author_id ON email_recipients(author_id)")
        .execute(pool)
        .await?;

    // Email references indexes
    sqlx::query("CREATE INDEX idx_email_references_email_id ON email_references(email_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_email_references_ref_msg_id ON email_references(referenced_message_id)")
        .execute(pool)
        .await?;

    // Thread memberships indexes
    sqlx::query("CREATE INDEX idx_thread_memberships_thread_id ON thread_memberships(thread_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX idx_thread_memberships_email_id ON thread_memberships(email_id)")
        .execute(pool)
        .await?;

    log::info!("Indexes created on parent tables (will auto-propagate to partitions)");

    log::info!("Database schema created successfully!");
    log::info!("To populate mailing lists, call the /api/admin/mailing-lists/seed endpoint");
    Ok(())
}

/// Create all partitions for a specific mailing list
///
/// NOTE: Indexes are NOT created here - they are automatically created by PostgreSQL
/// when you create indexes on the parent partitioned table. This follows PostgreSQL
/// best practices for partitioned tables.
pub async fn create_mailing_list_partitions(pool: &PgPool, list_id: i32, slug: &str) -> Result<(), sqlx::Error> {
    log::info!("Creating partitions for mailing list: {} (id={})", slug, list_id);

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

    log::info!("Partitions created successfully for {} - indexes will be auto-created by parent table", slug);
    Ok(())
}

/// Drop all partitions for a specific mailing list
pub async fn drop_mailing_list_partitions(pool: &PgPool, slug: &str) -> Result<(), sqlx::Error> {
    log::info!("Dropping partitions for mailing list: {}", slug);
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

    log::info!("Partitions dropped successfully for {}", slug);
    Ok(())
}
