//! Sync job dispatcher and orchestration.
//!
//! This module contains the `SyncDispatcher` which orchestrates the complete lifecycle
//! of mailing list synchronization jobs. It operates as a long-running worker that
//! continuously claims jobs from a queue and processes them through multiple phases.
//!
//! # Job Processing Lifecycle
//!
//! The dispatcher processes each sync job through 6 distinct phases:
//!
//! 1. **Configuration Loading**: Load mailing list config (slug, repositories/epochs)
//! 2. **Cache Initialization**: Load existing cache (incremental) or create empty (full sync)
//! 3. **Parsing & Import**: Sequentially process each epoch:
//!    - Discover commits from Git repository
//!    - Parse emails in parallel using Rayon
//!    - Import to database in 25K chunks
//!    - Populate threading cache with email metadata
//! 4. **Threading**: Run JWZ algorithm on complete cache to build thread hierarchy
//! 5. **Persistence**: Save cache to disk for future incremental syncs
//! 6. **Finalization**: Update author statistics and save checkpoints
//!
//! # Synchronization Modes
//!
//! ## Full Sync
//! - Triggered when no checkpoints exist for mailing list
//! - Processes all epochs from epoch 0 to latest
//! - Starts with empty threading cache
//! - Takes longer but builds complete dataset
//!
//! ## Incremental Sync
//! - Triggered when checkpoints exist
//! - Processes only last 2 epochs (for overlap safety)
//! - Loads existing cache from disk or database
//! - Much faster for regular updates
//!
//! # Error Handling & Cancellation
//!
//! - Jobs can be cancelled by setting `cancelled = true` in database
//! - Cancellation is checked periodically during long operations
//! - Errors fail the job and update status in queue
//! - Each phase error is logged with context
//!
//! # Performance Optimizations
//!
//! - **Parallel Parsing**: Uses Rayon with all CPU cores
//! - **Chunked Imports**: 25K email batches prevent timeouts
//! - **Unified Cache**: No epoch merging overhead
//! - **Change Detection**: SHA256 membership hashing skips unchanged threads
//! - **Checkpoint Recovery**: Resume from last successful epoch

use crate::sync::{SyncOrchestrator, queue::JobQueue, git::{MailingListSyncConfig, RepoConfig}};
use crate::sync::bulk_import::BulkImporter;
use crate::sync::database::checkpoint;
use crate::sync::parser::ParsedEmail;
use crate::threading::{MailingListCache, build_email_threads};
use crate::threading::container::ThreadInfo;
use rocket_db_pools::sqlx::{PgPool, Acquire};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Dispatcher that orchestrates the complete sync job lifecycle.
///
/// This is the main worker component that runs continuously, claiming sync jobs
/// from the queue and processing them through all phases. Previously named
/// `SyncWorker`, renamed to `SyncDispatcher` to better reflect its orchestration role.
///
/// # Fields
///
/// - `pool`: Database connection pool for all operations
/// - `queue`: Job queue manager for claiming/updating jobs
pub struct SyncDispatcher {
    pool: PgPool,
    queue: JobQueue,
}

impl SyncDispatcher {
    pub fn new(pool: PgPool) -> Self {
        let queue = JobQueue::new(pool.clone());
        Self { pool, queue }
    }

    /// Run dispatcher loop forever
    pub async fn run(self) -> ! {
        log::info!("SyncDispatcher started");

        loop {
            // Get next job
            let job = match self.queue.get_next_job().await {
                Ok(Some(j)) => {
                    log::info!("dispatcher: claimed job {} for list {}", j.id, j.mailing_list_id);
                    j
                }
                Ok(None) => {
                    // No jobs available, sleep and retry
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(e) => {
                    log::error!("dispatcher: failed to get job: {}", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            // Process job
            if let Err(e) = self.process_sync_job(job).await {
                log::error!("dispatcher: job processing failed: {}", e);
            }
        }
    }

    /// Process a complete sync job through all 6 phases.
    ///
    /// This is the main orchestration method that coordinates the entire sync lifecycle
    /// from configuration loading through final persistence.
    ///
    /// # Arguments
    ///
    /// - `job`: The sync job to process (contains job_id and mailing_list_id)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Job completed successfully (marked as complete in queue)
    /// - `Err(String)`: Job failed (marked as failed in queue with error message)
    ///
    /// # Phases
    ///
    /// 1. **Configuration**: Load mailing list slug and repository configs from database
    /// 2. **Cache Init**: Determine sync mode (full vs incremental) and initialize cache
    /// 3. **Parse & Import**: For each epoch to process:
    ///    - Get commits from Git (respecting checkpoint if incremental)
    ///    - Parse emails in parallel (Rayon)
    ///    - Import in 25K chunks to database
    ///    - Populate threading cache simultaneously
    ///    - Save checkpoint after each epoch
    /// 4. **Threading**: Run JWZ algorithm on complete cache and insert to database
    /// 5. **Persistence**: Save cache to disk for next incremental sync
    /// 6. **Finalization**: Update author statistics and save final checkpoints
    ///
    /// # Sync Mode Determination
    ///
    /// - **Full Sync**: No checkpoints exist → process all epochs with empty cache
    /// - **Incremental Sync**: Checkpoints exist → process last 2 epochs with loaded cache
    ///
    /// The "last 2 epochs" strategy ensures we catch any emails added to the previous
    /// epoch after the last sync (public-inbox can append to older epochs).
    ///
    /// # Cancellation
    ///
    /// Job cancellation is checked:
    /// - Before each epoch processing
    /// - Every 5 chunks during import (to avoid excessive DB queries)
    /// - Before threading phase
    ///
    /// # Error Handling
    ///
    /// All errors are propagated to fail the job. Non-fatal errors (e.g., cache save
    /// failures) are logged as warnings but don't fail the job.
    async fn process_sync_job(&self, job: crate::sync::queue::SyncJob) -> Result<(), String> {
        let job_id = job.id;
        let list_id = job.mailing_list_id;

        // Phase 0: Load mailing list configuration
        let (slug, repos) = match self.load_mailing_list_configuration(list_id).await {
            Ok(config) => config,
            Err(e) => {
                let error_msg = format!("Failed to load config: {}", e);
                let _ = self.queue.fail_job(job_id, error_msg.clone()).await;
                return Err(error_msg);
            }
        };

        log::info!("job {}: processing mailing list '{}' with {} repos (epochs)",
            job_id, slug, repos.len());

        // Create sync configuration
        let git_config = MailingListSyncConfig::new(list_id, slug.clone(), repos.clone());

        // Phase 1: Initialize threading cache (determines full vs incremental sync)
        let (cache, epochs_to_process, _is_full_sync) =
            self.initialize_cache_for_sync(job_id, list_id, &repos).await?;

        // Phase 2: Parse and import all epochs
        let (total_emails_imported, epoch_checkpoints) = self.parse_and_import_epochs(
            job_id,
            list_id,
            git_config,
            &epochs_to_process,
            &cache,
        ).await?;

        // Phase 3: Build threads and insert to database
        let (total_threads, total_memberships) = self.build_and_insert_threads(
            job_id,
            list_id,
            &cache,
        ).await?;

        // Phase 4: Persist cache to disk for future incremental syncs
        self.persist_cache_to_storage(job_id, list_id, &cache).await;

        // Phase 5: Update author statistics
        self.update_author_statistics(job_id, list_id).await?;

        // Phase 6: Save checkpoints
        self.save_sync_checkpoints(job_id, list_id, &epoch_checkpoints).await?;

        // Complete job
        self.queue.complete_job(job_id).await
            .map_err(|e| format!("Failed to mark job complete: {}", e))?;

        log::info!("job {}: complete - {} emails, {} threads, {} memberships",
            job_id, total_emails_imported, total_threads, total_memberships);
        Ok(())
    }

    /// Import emails to database and populate threading cache in optimized chunks.
    ///
    /// This method handles the import phase where parsed emails are written to the database
    /// and simultaneously populate the in-memory threading cache. Uses chunking to balance
    /// memory usage, connection stability, and throughput.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for cancellation checking
    /// - `mailing_list_id`: Target mailing list
    /// - `parsed_emails`: Vector of (commit_hash, ParsedEmail) tuples from parsing phase
    /// - `epoch`: Current epoch number being processed
    /// - `cache`: Reference to unified threading cache (populated during import)
    ///
    /// # Returns
    ///
    /// - `Ok(usize)`: Number of emails successfully imported
    /// - `Err(String)`: Import failure with context
    ///
    /// # Chunking Strategy
    ///
    /// Emails are processed in chunks of 25,000 because:
    /// - Prevents PostgreSQL connection timeouts on large imports
    /// - Limits memory usage from prepared statement parameters
    /// - Allows periodic cancellation checks without excessive queries
    /// - Provides progress visibility through logging
    ///
    /// # Process Flow
    ///
    /// For each chunk:
    /// 1. Check for job cancellation (every 5th chunk to reduce overhead)
    /// 2. Call `BulkImporter::import_chunk_with_epoch_cache()`
    ///    - Deduplicates authors (inserts new, gets existing IDs)
    ///    - Bulk inserts emails using UNNEST + ON CONFLICT
    ///    - Populates cache with (message_id, subject, date, references)
    /// 3. Accumulates import statistics
    ///
    /// # Performance
    ///
    /// - Each chunk is a single database transaction
    /// - Uses PostgreSQL UNNEST for efficient bulk inserts
    /// - Cache population is done in-memory (no additional DB queries)
    ///
    /// # Cancellation
    ///
    /// Checked every 5 chunks (e.g., every 125K emails). More frequent checks would
    /// add unnecessary database load, less frequent would delay cancellation response.
    async fn import_epoch_emails_to_database_and_cache(
        &self,
        job_id: i32,
        mailing_list_id: i32,
        parsed_emails: Vec<(String, ParsedEmail)>,
        epoch: i32,
        cache: &MailingListCache,
    ) -> Result<usize, String> {
        const CHUNK_SIZE: usize = 25_000;

        let importer = BulkImporter::new(self.pool.clone(), mailing_list_id);
        let total = parsed_emails.len();

        log::info!("importing {} emails for epoch {} in chunks of {}", total, epoch, CHUNK_SIZE);

        // Tag emails with their epoch
        let emails_with_epoch: Vec<(String, ParsedEmail, i32)> = parsed_emails
            .into_iter()
            .map(|(commit, email)| (commit, email, epoch))
            .collect();

        // Process in chunks to avoid connection timeouts and memory issues
        let mut total_imported = 0;
        let num_chunks = (total + CHUNK_SIZE - 1) / CHUNK_SIZE;

        for (chunk_idx, chunk) in emails_with_epoch.chunks(CHUNK_SIZE).enumerate() {
            // Check for cancellation every 5 chunks (to avoid too many DB queries)
            if chunk_idx % 5 == 0 && self.queue.is_job_cancelled(job_id).await.unwrap_or(false) {
                log::warn!("job {}: cancelled during import at chunk {}/{}", job_id, chunk_idx + 1, num_chunks);
                return Err("Job cancelled by user during import".to_string());
            }

            log::debug!("importing chunk {}/{} ({} emails)", chunk_idx + 1, num_chunks, chunk.len());

            let stats = importer
                .import_chunk_with_epoch_cache(chunk, cache)
                .await
                .map_err(|e| format!("Import failed for epoch {} chunk {}: {}", epoch, chunk_idx + 1, e))?;

            total_imported += stats.emails;
        }

        log::info!("epoch {}: imported {} emails in {} chunks", epoch, total_imported, num_chunks);
        Ok(total_imported)
    }

    /// Build email threads from the unified cache using the JWZ algorithm.
    ///
    /// This is the threading phase that takes the complete populated cache and builds
    /// the thread hierarchy using the Jamie Zawinski (JWZ) threading algorithm. The
    /// unified cache approach eliminates the need for epoch merging.
    ///
    /// # Arguments
    ///
    /// - `mailing_list_id`: Target mailing list
    /// - `cache`: Unified threading cache populated during import phase
    ///
    /// # Returns
    ///
    /// - `Ok((thread_count, membership_count))`: Number of threads and memberships created
    /// - `Err(String)`: Threading or database insertion failure
    ///
    /// # Process Flow
    ///
    /// 1. **Extract Data**: Get all email data and references from unified cache
    ///    - Email data: (email_id, message_id, subject, date)
    ///    - References: (message_id, reference_id) for In-Reply-To and References headers
    ///
    /// 2. **Run JWZ Algorithm**: Call `build_email_threads()` which:
    ///    - Builds a container tree structure from references
    ///    - Groups emails into threads by reference chains
    ///    - Handles missing parents (creates dummy containers)
    ///    - Performs subject-based grouping as fallback
    ///    - Computes thread root and hierarchy depth
    ///    - Uses Rayon internally for parallel processing
    ///
    /// 3. **Insert to Database**: Call `insert_thread_batch_with_memberships()` to:
    ///    - Compute membership hashes for change detection
    ///    - Bulk insert/update threads
    ///    - Bulk insert thread memberships with depth info
    ///
    /// # Why Unified Cache?
    ///
    /// Previous implementation processed each epoch separately and merged results, which:
    /// - Required complex cross-epoch reference resolution
    /// - Had epoch boundary edge cases
    /// - Needed expensive merge operations
    ///
    /// Unified cache processes everything together:
    /// - Single JWZ pass over all data
    /// - Natural cross-epoch threading
    /// - No merging overhead
    /// - Simpler and faster
    ///
    /// # Performance
    ///
    /// - JWZ algorithm is O(n log n) where n = number of emails
    /// - Uses Rayon for parallel container processing
    /// - Database insertion uses bulk operations (UNNEST)
    async fn build_threads_from_cache(
        &self,
        mailing_list_id: i32,
        cache: &MailingListCache,
    ) -> Result<(usize, usize), String> {
        log::info!("Running threading on unified cache");

        // Step 1: Get all data from unified cache (no merging needed!)
        let (all_email_data, all_references) = cache.get_all_for_threading();

        log::info!("Threading data: {} emails, {} reference entries",
            all_email_data.len(), all_references.len());

        // Step 2: Run JWZ algorithm (Rayon handles internal parallelism)
        log::info!("Running JWZ threading algorithm");

        let threads_to_create = build_email_threads(all_email_data, all_references);

        log::info!("JWZ complete: {} threads identified", threads_to_create.len());

        // Step 3: Bulk insert threads and memberships
        log::info!("Bulk inserting {} threads to database", threads_to_create.len());

        let (thread_count, membership_count) = self.insert_thread_batch_with_memberships(
            mailing_list_id,
            threads_to_create,
        ).await?;

        log::info!("Threading complete: {} threads, {} memberships inserted",
            thread_count, membership_count);

        Ok((thread_count, membership_count))
    }

    /// Prepare thread data by computing membership hashes and statistics.
    ///
    /// This is a pure function that transforms `ThreadInfo` into prepared thread data
    /// with membership hashes for change detection.
    ///
    /// # Arguments
    ///
    /// - `threads_to_create`: Vector of `ThreadInfo` from JWZ algorithm
    ///
    /// # Returns
    ///
    /// Vector of tuples: (root_message_id, subject, start_date, last_date,
    /// message_count, membership_hash, membership_map)
    ///
    /// # Process
    ///
    /// For each thread:
    /// 1. Build membership map (email_id → depth in hierarchy)
    /// 2. Compute SHA256 hash of sorted email_ids for change detection
    /// 3. Calculate thread statistics (message_count, dates)
    fn prepare_thread_batch_data(
        threads_to_create: Vec<ThreadInfo>,
    ) -> Vec<(String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, i32, Vec<u8>, HashMap<i32, i32>)> {
        use sha2::{Sha256, Digest};

        let mut prepared_threads = Vec::new();

        for thread_info in threads_to_create {
            // Build membership map from JWZ results
            // Maps email_id → depth in thread hierarchy (0 = root, 1 = reply, etc.)
            let mut membership_map = HashMap::new();
            for (email_id, depth) in thread_info.emails {
                membership_map.entry(email_id).or_insert(depth);
            }

            // Compute deterministic SHA256 hash of thread membership
            // Used for change detection: if hash matches, thread hasn't changed
            let mut sorted_email_ids: Vec<i32> = membership_map.keys().copied().collect();
            sorted_email_ids.sort_unstable();
            let mut hasher = Sha256::new();
            for email_id in sorted_email_ids {
                hasher.update(email_id.to_le_bytes());
            }
            let membership_hash = hasher.finalize().to_vec();

            // Compute thread statistics
            let message_count = membership_map.len() as i32;
            let start_date = thread_info.start_date;
            let last_date = thread_info.start_date; // JWZ doesn't compute last_date separately

            prepared_threads.push((
                thread_info.root_message_id,
                thread_info.subject,
                start_date,
                last_date,
                message_count,
                membership_hash,
                membership_map,
            ));
        }

        prepared_threads
    }

    /// Insert thread batch with memberships using efficient change detection.
    ///
    /// This method handles the database insertion of threads with an optimization:
    /// it computes a SHA256 hash of thread memberships to detect which threads
    /// have changed since the last sync, avoiding unnecessary updates and rebuilds
    /// of unchanged threads.
    ///
    /// # Arguments
    ///
    /// - `mailing_list_id`: Target mailing list
    /// - `threads_to_create`: Vector of `ThreadInfo` from JWZ algorithm
    ///
    /// # Returns
    ///
    /// - `Ok((thread_count, membership_count))`: Total threads processed and memberships inserted
    /// - `Err(String)`: Database operation failure
    ///
    /// # Change Detection Strategy
    ///
    /// For each thread:
    /// 1. Extract email_id list from thread members
    /// 2. Sort email_ids deterministically
    /// 3. Compute SHA256 hash of sorted IDs
    /// 4. Compare hash with existing thread's hash in database
    /// 5. Skip if hash matches (thread unchanged)
    /// 6. Update if hash differs (membership changed)
    ///
    /// This is efficient because:
    /// - Hash comparison is fast (32 bytes vs full membership set)
    /// - Only changed threads trigger updates
    /// - Membership table updates are expensive (deletes + inserts)
    /// - Incremental syncs typically have few changed threads
    ///
    /// # Process Flow
    ///
    /// 1. **Prepare Thread Data**: Build membership maps and compute hashes
    /// 2. **Bulk Check Existing**: Query database for existing threads
    /// 3. **Filter Unchanged**: Compare hashes to skip unchanged threads
    /// 4. **Bulk Upsert Threads**: Insert/update changed threads
    /// 5. **Bulk Insert Memberships**: Insert thread memberships
    ///
    /// # Performance Impact
    ///
    /// Example: Incremental sync with 1000 threads, 3 changed:
    /// - Without optimization: 1000 thread updates + 50K membership operations
    /// - With optimization: 3 thread updates + 150 membership operations
    /// - ~300x reduction in database operations for typical incremental syncs
    async fn insert_thread_batch_with_memberships(
        &self,
        mailing_list_id: i32,
        threads_to_create: Vec<ThreadInfo>,
    ) -> Result<(usize, usize), String> {
        if threads_to_create.is_empty() {
            return Ok((0, 0));
        }

        let mut conn = self.pool.acquire().await
            .map_err(|e| format!("Failed to acquire connection: {}", e))?;

        let mut tx = conn.begin().await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Step 1: Prepare thread data (compute hashes and statistics)
        log::debug!("Preparing {} threads for bulk insert", threads_to_create.len());
        let prepared_threads = Self::prepare_thread_batch_data(threads_to_create);
        let thread_count = prepared_threads.len();

        // Step 2: Query database for existing threads
        log::debug!("Checking existing threads");

        let root_message_ids: Vec<String> = prepared_threads.iter()
            .map(|(root_msg_id, ..)| root_msg_id.clone())
            .collect();

        let existing_threads: Vec<(String, i32, Option<Vec<u8>>)> = sqlx::query_as(
            r#"SELECT root_message_id, id, membership_hash
               FROM threads
               WHERE mailing_list_id = $1 AND root_message_id = ANY($2)"#
        )
        .bind(mailing_list_id)
        .bind(&root_message_ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| format!("Failed to check existing threads: {}", e))?;

        let existing_map: HashMap<String, (i32, Option<Vec<u8>>)> = existing_threads
            .into_iter()
            .map(|(root_msg_id, id, hash)| (root_msg_id, (id, hash)))
            .collect();

        // Step 3: Filter unchanged threads using hash comparison
        // This is the key optimization that skips unchanged threads in incremental syncs
        let mut threads_to_upsert = Vec::new();
        let mut thread_id_map: HashMap<String, i32> = HashMap::new(); // Preserves existing IDs
        let mut skipped_count = 0;

        for thread in prepared_threads {
            let (ref root_msg_id, .., ref hash, _) = thread;
            if let Some((existing_id, Some(existing_hash))) = existing_map.get(root_msg_id) {
                // Thread exists in database, compare membership hashes
                if existing_hash == hash {
                    // Hash match = membership unchanged = skip entire update
                    // This saves updating threads table + deleting/reinserting memberships
                    skipped_count += 1;
                    continue;
                }
                // Hash mismatch = membership changed = need to update
                // Preserve existing thread_id for membership updates
                thread_id_map.insert(root_msg_id.clone(), *existing_id);
            }
            // Either new thread or changed thread - needs upsert
            threads_to_upsert.push(thread);
        }

        log::debug!("Skipped {} unchanged threads, upserting {} threads",
            skipped_count, threads_to_upsert.len());

        if threads_to_upsert.is_empty() {
            log::info!("All threads unchanged, skipping insert");
            tx.commit().await
                .map_err(|e| format!("Failed to commit transaction: {}", e))?;
            return Ok((thread_count, 0));
        }

        // Step 4: Bulk insert/update changed threads
        log::debug!("Bulk inserting {} threads", threads_to_upsert.len());

        let mut list_ids = Vec::new();
        let mut root_msg_ids = Vec::new();
        let mut subjects = Vec::new();
        let mut start_dates = Vec::new();
        let mut last_dates = Vec::new();
        let mut message_counts = Vec::new();
        let mut membership_hashes = Vec::new();

        for (root_msg_id, subject, start_date, last_date, message_count, membership_hash, _) in &threads_to_upsert {
            list_ids.push(mailing_list_id);
            root_msg_ids.push(root_msg_id.clone());
            subjects.push(subject.clone());
            start_dates.push(*start_date);
            last_dates.push(*last_date);
            message_counts.push(*message_count);
            membership_hashes.push(membership_hash.clone());
        }

        let thread_ids_from_insert: Vec<(String, i32)> = sqlx::query_as(
            r#"INSERT INTO threads
               (mailing_list_id, root_message_id, subject, start_date, last_date, message_count, membership_hash)
               SELECT * FROM UNNEST($1::int[], $2::text[], $3::text[], $4::timestamptz[], $5::timestamptz[], $6::int[], $7::bytea[])
               ON CONFLICT (mailing_list_id, root_message_id)
               DO UPDATE SET
                   subject = EXCLUDED.subject,
                   start_date = EXCLUDED.start_date,
                   last_date = EXCLUDED.last_date,
                   message_count = EXCLUDED.message_count,
                   membership_hash = EXCLUDED.membership_hash
               RETURNING root_message_id, id"#
        )
        .bind(&list_ids)
        .bind(&root_msg_ids)
        .bind(&subjects)
        .bind(&start_dates)
        .bind(&last_dates)
        .bind(&message_counts)
        .bind(&membership_hashes)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| format!("Failed to bulk insert threads: {}", e))?;

        log::debug!("Bulk insert returned {} thread IDs", thread_ids_from_insert.len());

        // Merge returned thread IDs with existing thread IDs
        for (root_msg_id, thread_id) in thread_ids_from_insert {
            thread_id_map.insert(root_msg_id, thread_id);
        }

        // Step 5: Bulk insert thread memberships
        log::debug!("Preparing memberships for bulk insert");

        let mut membership_list_ids = Vec::new();
        let mut membership_thread_ids = Vec::new();
        let mut membership_email_ids = Vec::new();
        let mut membership_depths = Vec::new();

        for (root_msg_id, .., membership_map) in &threads_to_upsert {
            if let Some(&thread_id) = thread_id_map.get(root_msg_id) {
                for (email_id, depth) in membership_map {
                    membership_list_ids.push(mailing_list_id);
                    membership_thread_ids.push(thread_id);
                    membership_email_ids.push(*email_id);
                    membership_depths.push(*depth);
                }
            }
        }

        let membership_count = membership_email_ids.len();

        if membership_count > 0 {
            log::debug!("Bulk inserting {} memberships", membership_count);

            sqlx::query(
                r#"INSERT INTO thread_memberships (mailing_list_id, thread_id, email_id, depth)
                   SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::int[])
                   ON CONFLICT (mailing_list_id, thread_id, email_id) DO NOTHING"#
            )
            .bind(&membership_list_ids)
            .bind(&membership_thread_ids)
            .bind(&membership_email_ids)
            .bind(&membership_depths)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to bulk insert memberships: {}", e))?;
        }

        // Commit transaction
        tx.commit().await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok((thread_count, membership_count))
    }

    /// Build email threads and insert to database.
    ///
    /// Runs the threading phase which applies the JWZ algorithm to the populated cache
    /// and inserts the resulting thread hierarchy to the database.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging and cancellation checks
    /// - `list_id`: Mailing list ID
    /// - `cache`: Populated threading cache
    ///
    /// # Returns
    ///
    /// - `Ok((thread_count, membership_count))`: Number of threads and memberships created
    /// - `Err(String)`: Threading failure
    async fn build_and_insert_threads(
        &self,
        job_id: i32,
        list_id: i32,
        cache: &MailingListCache,
    ) -> Result<(usize, usize), String> {
        // Check if job was cancelled before threading
        if self.queue.is_job_cancelled(job_id).await.unwrap_or(false) {
            log::warn!("job {}: cancelled by user before threading, stopping", job_id);
            return Err("Job cancelled by user".to_string());
        }

        log::info!("job {}: starting threading phase", job_id);
        self.queue.update_phase(job_id, "threading").await
            .map_err(|e| format!("Failed to update phase: {}", e))?;

        let (total_threads, total_memberships) = self.build_threads_from_cache(
            list_id,
            cache,
        ).await?;

        log::info!("job {}: threading complete - {} threads, {} memberships",
            job_id, total_threads, total_memberships);

        Ok((total_threads, total_memberships))
    }

    /// Persist the threading cache to disk for future incremental syncs.
    ///
    /// Saves the cache to disk. Errors are logged as warnings but don't fail the job
    /// since the cache can be reconstructed from the database if needed.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging
    /// - `list_id`: Mailing list ID
    /// - `cache`: Threading cache to save
    async fn persist_cache_to_storage(
        &self,
        job_id: i32,
        list_id: i32,
        cache: &MailingListCache,
    ) {
        log::info!("job {}: saving unified cache to disk", job_id);

        let cache_dir = std::env::var("THREADING_CACHE_BASE_PATH")
            .unwrap_or_else(|_| "./cache".to_string());

        let _ = cache.save_to_disk(&PathBuf::from(&cache_dir))
            .map_err(|e| {
                log::warn!("job {}: failed to save cache (non-fatal): {}", job_id, e);
            });
    }

    /// Update author activity statistics.
    ///
    /// Recalculates author statistics (message counts, date ranges) for the mailing list.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging
    /// - `list_id`: Mailing list ID
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Statistics updated successfully
    /// - `Err(String)`: Update failure
    async fn update_author_statistics(
        &self,
        job_id: i32,
        list_id: i32,
    ) -> Result<(), String> {
        log::info!("job {}: updating author activity", job_id);

        let importer = BulkImporter::new(self.pool.clone(), list_id);
        importer.update_author_activity().await
            .map_err(|e| format!("Failed to update author activity: {}", e))?;

        Ok(())
    }

    /// Save sync checkpoints to database.
    ///
    /// Persists the last processed commit hash for each epoch and the threading timestamp,
    /// enabling incremental syncs to resume from the correct position.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging
    /// - `list_id`: Mailing list ID
    /// - `epoch_checkpoints`: Map of epoch → last commit hash
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Checkpoints saved successfully
    /// - `Err(String)`: Save failure
    async fn save_sync_checkpoints(
        &self,
        job_id: i32,
        list_id: i32,
        epoch_checkpoints: &HashMap<i32, String>,
    ) -> Result<(), String> {
        log::info!("job {}: saving checkpoints", job_id);

        checkpoint::save_last_indexed_commits(&self.pool, list_id, epoch_checkpoints).await?;
        checkpoint::save_last_threaded_at(&self.pool, list_id).await?;

        Ok(())
    }

    /// Parse and import emails for all epochs to be processed.
    ///
    /// Orchestrates the sequential processing of each epoch: fetching commits from Git,
    /// parsing emails in parallel, and importing to the database while populating the
    /// threading cache.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging and cancellation checks
    /// - `list_id`: Mailing list ID
    /// - `git_config`: Git configuration with repository information
    /// - `epochs_to_process`: List of epoch numbers to process
    /// - `cache`: Threading cache to populate during import
    ///
    /// # Returns
    ///
    /// - `Ok((total_emails, epoch_checkpoints))`: Number of emails imported and checkpoint map
    /// - `Err(String)`: Processing failure
    ///
    /// # Process
    ///
    /// For each epoch:
    /// 1. Check for job cancellation
    /// 2. Get commits from Git (respecting checkpoints)
    /// 3. Parse emails in parallel using Rayon
    /// 4. Import emails in chunks to database and cache
    /// 5. Save checkpoint with last commit hash
    async fn parse_and_import_epochs(
        &self,
        job_id: i32,
        list_id: i32,
        git_config: MailingListSyncConfig,
        epochs_to_process: &[i32],
        cache: &MailingListCache,
    ) -> Result<(usize, HashMap<i32, String>), String> {
        log::info!("job {}: starting sequential parsing & import phase", job_id);
        self.queue.update_phase(job_id, "parsing").await
            .map_err(|e| format!("Failed to update phase: {}", e))?;

        let orchestrator = SyncOrchestrator::new(git_config, self.pool.clone(), list_id);

        // Load existing checkpoints to determine where to resume
        let last_commits = checkpoint::load_last_indexed_commits(&self.pool, list_id).await?;

        let mut total_emails_imported = 0;
        let mut epoch_checkpoints = HashMap::new();

        for &epoch in epochs_to_process {
            // Check if job was cancelled
            if self.queue.is_job_cancelled(job_id).await.unwrap_or(false) {
                log::warn!("job {}: cancelled by user, stopping", job_id);
                return Err("Job cancelled by user".to_string());
            }

            log::info!("job {}: processing epoch {}", job_id, epoch);

            // Get commits for this epoch
            let since = last_commits.get(&epoch).map(|s| s.as_str());
            let commits = orchestrator.git_manager
                .get_commits_for_epoch(epoch, since)
                .map_err(|e| format!("Failed to get commits for epoch {}: {}", epoch, e))?;

            if commits.is_empty() {
                log::info!("job {}: epoch {} - no new commits", job_id, epoch);
                continue;
            }

            log::info!("job {}: epoch {} - {} commits", job_id, epoch, commits.len());

            // Parse emails (Rayon parallel)
            let parsed = orchestrator.parse_all_parallel(commits.clone()).await?;
            log::info!("job {}: epoch {} - parsed {} emails", job_id, epoch, parsed.len());

            // Import and populate unified cache
            let emails_imported = self.import_epoch_emails_to_database_and_cache(
                job_id,
                list_id,
                parsed,
                epoch,
                cache,
            ).await?;

            total_emails_imported += emails_imported;

            log::info!("job {}: epoch {} - imported {} emails, populated cache",
                job_id, epoch, emails_imported);

            // Save checkpoint for this epoch
            if let Some((last_commit, _, _)) = commits.last() {
                epoch_checkpoints.insert(epoch, last_commit.clone());
            }
        }

        log::info!("job {}: parsing & import complete - {} total emails", job_id, total_emails_imported);

        Ok((total_emails_imported, epoch_checkpoints))
    }

    /// Initialize the threading cache for a sync job.
    ///
    /// Determines whether to perform a full or incremental sync based on checkpoint
    /// existence, and loads/creates the appropriate cache.
    ///
    /// # Arguments
    ///
    /// - `job_id`: Current job ID for logging
    /// - `list_id`: Mailing list ID
    /// - `repos`: Repository configurations with epoch ordering
    ///
    /// # Returns
    ///
    /// - `Ok((cache, epochs_to_process, is_full_sync))`: Initialized cache and epoch list
    /// - `Err(String)`: Initialization failure
    ///
    /// # Sync Type Determination
    ///
    /// - **Full Sync**: No checkpoints exist → process all epochs with empty cache
    /// - **Incremental Sync**: Checkpoints exist → process last 2 epochs with loaded cache
    async fn initialize_cache_for_sync(
        &self,
        job_id: i32,
        list_id: i32,
        repos: &[RepoConfig],
    ) -> Result<(MailingListCache, Vec<i32>, bool), String> {
        log::info!("job {}: initializing unified cache", job_id);

        let cache_dir = std::env::var("THREADING_CACHE_BASE_PATH")
            .unwrap_or_else(|_| "./cache".to_string());

        // Enumerate all epochs for this mailing list
        let all_epochs: Vec<i32> = repos.iter()
            .map(|r| r.order)
            .collect();

        // Determine sync type based on checkpoint existence
        // Checkpoints store the last processed commit hash for each epoch
        let last_commits = checkpoint::load_last_indexed_commits(&self.pool, list_id).await?;
        let is_full_sync = last_commits.is_empty(); // No checkpoints = first sync

        let epochs_to_process = if is_full_sync {
            // Full sync: Process all epochs from 0 to latest
            all_epochs.clone()
        } else {
            // Incremental sync: Process last 2 epochs for safety
            // Why 2? public-inbox can append emails to previous epochs after they're "closed"
            // Processing both ensures we catch late-arriving emails in epoch N-1
            let max = all_epochs.iter().max().copied().unwrap_or(0);
            vec![max - 1, max].into_iter().filter(|&e| e >= 0).collect()
        };

        log::info!("job {}: {} sync - {} epochs to process",
            job_id,
            if is_full_sync { "FULL" } else { "INCREMENTAL" },
            epochs_to_process.len()
        );

        // Initialize unified threading cache
        // The cache stores email metadata needed for JWZ threading algorithm:
        // (email_id, message_id, subject, date, references)
        let cache = if is_full_sync {
            // Full sync: Start with empty cache, will be populated during import
            log::info!("job {}: creating empty cache for full sync", job_id);
            MailingListCache::new(list_id)
        } else {
            // Incremental sync: Load existing cache to preserve all historical email data
            // Try disk first (fast), fall back to database (slower but reliable)
            log::info!("job {}: loading existing cache for incremental sync", job_id);
            match MailingListCache::load_from_disk(list_id, &PathBuf::from(&cache_dir)) {
                Ok(cache) => {
                    // Disk cache hit - fastest path
                    log::info!("job {}: loaded cache from disk", job_id);
                    cache
                }
                Err(_) => {
                    // Disk cache miss - reconstruct from database
                    // This can happen if cache was evicted or server restarted
                    log::info!("job {}: cache not on disk, loading from database", job_id);
                    MailingListCache::load_from_database(&self.pool, list_id)
                        .await
                        .map_err(|e| format!("Failed to load cache from database: {}", e))?
                }
            }
        };

        log::info!("job {}: unified cache initialized", job_id);

        Ok((cache, epochs_to_process, is_full_sync))
    }

    async fn load_mailing_list_configuration(&self, list_id: i32)
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
