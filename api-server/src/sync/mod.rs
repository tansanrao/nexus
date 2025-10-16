//! Mailing list synchronization system.
//!
//! This module provides a complete pipeline for synchronizing mailing list archives from
//! public-inbox Git repositories into a PostgreSQL database with email threading.
//!
//! # Architecture Overview
//!
//! The sync system operates in a job-based architecture with the following components:
//!
//! ## Core Components
//!
//! - **`dispatcher`**: Orchestrates the entire sync lifecycle through a job queue system.
//!   Claims jobs, coordinates all sync phases, and handles error recovery.
//!
//! - **`git`**: Manages Git repository operations including mirror validation, commit
//!   discovery, and blob retrieval from public-inbox v2 format repositories.
//!
//! - **`parser`**: Parses raw email content from Git blobs into structured data with
//!   proper header extraction, sanitization, and subject normalization for threading.
//!
//! - **`import`**: Handles bulk database imports with optimized batch operations,
//!   author deduplication, and threading cache population.
//!
//! - **`database`**: Provides database utilities for partition management, checkpoints,
//!   migrations, and cache persistence.
//!
//! - **`queue`**: Manages the sync job queue with job claiming, status updates, phase
//!   tracking, and cancellation support.
//!
//! ## Data Flow
//!
//! The synchronization process follows this pipeline:
//!
//! 1. **Job Claiming**: Dispatcher claims a sync job from the queue
//! 2. **Git Discovery**: Discover commits from mirrored repositories (per epoch)
//! 3. **Parallel Parsing**: Parse emails using Rayon thread pool (CPU-bound)
//! 4. **Batch Import**: Import emails to database in 25K chunks, populate threading cache
//! 5. **Threading**: Run JWZ algorithm on complete cache to build thread hierarchy
//! 6. **Persistence**: Save cache to disk and update database checkpoints
//!
//! ## Synchronization Modes
//!
//! - **Full Sync**: Process all epochs from scratch with empty cache (initial sync)
//! - **Incremental Sync**: Process only last 2 epochs using existing cache (updates)
//!
//! ## Performance Characteristics
//!
//! - Rayon parallelization for CPU-bound parsing (uses all available cores)
//! - 6 database connection pool for I/O-bound imports
//! - 25,000 email chunks to balance memory usage and connection timeout prevention
//! - Unified in-memory cache eliminates epoch merging overhead
//! - SHA256 membership hashing for efficient thread change detection
//!
//! ## Epoch-Based Model
//!
//! Emails are organized into "epochs" (sequential repository orders). Each mailing list
//! can have multiple epoch repositories, processed sequentially to maintain chronological
//! ordering and enable checkpoint recovery.

pub mod bulk_import;
pub mod database;
pub mod dispatcher;
pub mod git;
pub mod import;
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

// Re-export database functions for backward compatibility
pub use database::{
    create_mailing_list_partitions, drop_mailing_list_partitions, load_last_indexed_commits,
    reset_database, run_migrations, save_last_indexed_commits, save_last_threaded_at,
};

/// Main synchronization orchestrator for email parsing operations.
///
/// Coordinates Git blob retrieval and parallel email parsing using Rayon.
/// This struct is used by the dispatcher during the parsing phase of sync jobs.
///
/// # Fields
///
/// - `git_manager`: Handles Git operations (commit discovery, blob retrieval)
/// - `pool`: Database connection pool for potential future operations
/// - `mailing_list_id`: Target mailing list identifier
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


    /// Parse all commits in parallel using Rayon thread pool.
    ///
    /// This is the CPU-bound parsing phase that processes email blobs in parallel
    /// to maximize throughput. Uses all available CPU cores via Rayon.
    ///
    /// # Arguments
    ///
    /// - `commits`: Vector of (commit_hash, path, repo_order) tuples to parse
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<(String, ParsedEmail)>)`: Successfully parsed emails with their commit hashes
    /// - `Err(String)`: Fatal error (e.g., failed to create thread pool)
    ///
    /// # Performance
    ///
    /// - Creates a Rayon thread pool with `num_cpus::get()` threads
    /// - Processes commits in parallel using `par_iter()`
    /// - Retrieves blobs from Git repositories (I/O per worker thread)
    /// - Parses email content (CPU-intensive MIME parsing)
    /// - Filters out parse errors (logged but not fatal)
    ///
    /// # Error Handling
    ///
    /// Individual email parse failures are logged as warnings but don't stop processing.
    /// Only thread pool creation failures return an error. Parse success/error counts
    /// are tracked and logged for monitoring.
    ///
    /// # Example Flow
    ///
    /// For 10,000 commits on an 8-core machine:
    /// - Creates 8 worker threads
    /// - Each thread processes ~1,250 commits
    /// - Each thread: git blob retrieval → MIME parsing → validation
    /// - Failed parses are skipped, successful ones collected
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
