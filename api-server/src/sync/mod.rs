pub mod bulk_import;
pub mod dispatcher;
pub mod git;
pub mod manifest;
pub mod parser;
pub mod pg_config;
pub mod queue;

use crate::sync::git::{GitManager, MailingListSyncConfig};
use crate::sync::parser::{parse_email, ParsedEmail};
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
