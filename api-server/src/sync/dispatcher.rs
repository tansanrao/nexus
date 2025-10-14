use crate::sync::{SyncOrchestrator, queue::JobQueue, git::{MailingListSyncConfig, RepoConfig}};
use crate::sync::bulk_import::BulkImporter;
use crate::sync::parser::ParsedEmail;
use crate::threading::{MailingListCache, build_threads};
use crate::threading::container::ThreadInfo;
use rocket_db_pools::sqlx::{PgPool, Acquire};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Dispatcher orchestrates entire sync lifecycle
/// Renamed from SyncWorker to better reflect its role
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
            if let Err(e) = self.process_job(job).await {
                log::error!("dispatcher: job processing failed: {}", e);
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

        log::info!("job {}: processing mailing list '{}' with {} repos (epochs)",
            job_id, slug, repos.len());

        // Create sync configuration
        let git_config = MailingListSyncConfig::new(list_id, slug.clone(), repos.clone());

        // Phase 1: Initialize unified cache
        log::info!("job {}: initializing unified cache", job_id);

        let cache_dir = std::env::var("THREADING_CACHE_BASE_PATH")
            .unwrap_or_else(|_| "./cache".to_string());

        // Enumerate epochs
        let all_epochs: Vec<i32> = repos.iter()
            .map(|r| r.order)
            .collect();

        // Determine sync type
        let last_commits = self.load_last_indexed_commits(list_id).await?;
        let is_full_sync = last_commits.is_empty();

        let epochs_to_process = if is_full_sync {
            all_epochs.clone()
        } else {
            // Incremental: last 2 epochs
            let max = all_epochs.iter().max().copied().unwrap_or(0);
            vec![max - 1, max].into_iter().filter(|&e| e >= 0).collect()
        };

        log::info!("job {}: {} sync - {} epochs to process",
            job_id,
            if is_full_sync { "FULL" } else { "INCREMENTAL" },
            epochs_to_process.len()
        );

        // Initialize unified cache (load from disk/db for incremental, empty for full)
        let cache = if is_full_sync {
            log::info!("job {}: creating empty cache for full sync", job_id);
            MailingListCache::new(list_id)
        } else {
            log::info!("job {}: loading existing cache for incremental sync", job_id);
            match MailingListCache::load_from_disk(list_id, &PathBuf::from(&cache_dir)) {
                Ok(cache) => {
                    log::info!("job {}: loaded cache from disk", job_id);
                    cache
                }
                Err(_) => {
                    log::info!("job {}: cache not on disk, loading from database", job_id);
                    MailingListCache::load_from_db(&self.pool, list_id)
                        .await
                        .map_err(|e| {
                            let error_msg = format!("Failed to load cache from database: {}", e);
                            let _ = self.queue.fail_job(job_id, error_msg.clone());
                            error_msg
                        })?
                }
            }
        };

        log::info!("job {}: unified cache initialized", job_id);

        // Phase 2: Sequential parsing & import with cache population
        log::info!("job {}: starting sequential parsing & import phase", job_id);
        self.queue.update_phase(job_id, "parsing").await
            .map_err(|e| {
                let _ = self.queue.fail_job(job_id, format!("Failed to update phase: {}", e));
                format!("Failed to update phase: {}", e)
            })?;

        let orchestrator = SyncOrchestrator::new(git_config, self.pool.clone(), list_id);

        let mut total_emails_imported = 0;
        let mut epoch_checkpoints = HashMap::new();

        for &epoch in &epochs_to_process {
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
                .map_err(|e| {
                    let error_msg = format!("Failed to get commits for epoch {}: {}", epoch, e);
                    let _ = self.queue.fail_job(job_id, error_msg.clone());
                    error_msg
                })?;

            if commits.is_empty() {
                log::info!("job {}: epoch {} - no new commits", job_id, epoch);
                continue;
            }

            log::info!("job {}: epoch {} - {} commits", job_id, epoch, commits.len());

            // Parse emails (Rayon parallel)
            let parsed = orchestrator.parse_all_parallel(commits.clone()).await?;
            log::info!("job {}: epoch {} - parsed {} emails", job_id, epoch, parsed.len());

            // Import and populate unified cache
            let emails_imported = self.import_and_populate_cache(
                job_id,
                list_id,
                parsed,
                epoch,
                &cache,
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

        // Check if job was cancelled before threading
        if self.queue.is_job_cancelled(job_id).await.unwrap_or(false) {
            log::warn!("job {}: cancelled by user before threading, stopping", job_id);
            return Err("Job cancelled by user".to_string());
        }

        // Phase 3: Single-pass threading
        log::info!("job {}: starting threading phase", job_id);
        self.queue.update_phase(job_id, "threading").await
            .map_err(|e| {
                let _ = self.queue.fail_job(job_id, format!("Failed to update phase: {}", e));
                format!("Failed to update phase: {}", e)
            })?;

        let (total_threads, total_memberships) = self.thread_unified_cache(
            list_id,
            &cache,
        ).await?;

        log::info!("job {}: threading complete - {} threads, {} memberships",
            job_id, total_threads, total_memberships);

        // Phase 4: Save unified cache to disk
        log::info!("job {}: saving unified cache to disk", job_id);
        let _ = cache.save_to_disk(&PathBuf::from(&cache_dir))
            .map_err(|e| {
                log::warn!("job {}: failed to save cache (non-fatal): {}", job_id, e);
                // Don't fail the job for cache save errors
            });

        // Phase 5: Update author activity
        log::info!("job {}: updating author activity", job_id);
        let importer = BulkImporter::new(self.pool.clone(), list_id);
        importer.update_author_activity().await
            .map_err(|e| format!("Failed to update author activity: {}", e))?;

        // Phase 6: Save checkpoints
        self.save_last_indexed_commits(list_id, &epoch_checkpoints).await?;
        self.save_last_threaded_at(list_id).await?;

        // Complete job
        self.queue.complete_job(job_id).await
            .map_err(|e| format!("Failed to mark job complete: {}", e))?;

        log::info!("job {}: complete - {} emails, {} threads", job_id, total_emails_imported, total_threads);
        Ok(())
    }

    /// Import emails and populate unified cache concurrently
    /// Processes emails in chunks of 25,000 to avoid overwhelming the database
    async fn import_and_populate_cache(
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

    /// Thread unified cache - simplified single-pass threading
    ///
    /// Runs JWZ algorithm on the complete unified cache dataset.
    async fn thread_unified_cache(
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

        let threads_to_create = build_threads(all_email_data, all_references);

        log::info!("JWZ complete: {} threads identified", threads_to_create.len());

        // Step 3: Bulk insert threads and memberships
        log::info!("Bulk inserting {} threads to database", threads_to_create.len());

        let (thread_count, membership_count) = self.bulk_insert_threads(
            mailing_list_id,
            threads_to_create,
        ).await?;

        log::info!("Threading complete: {} threads, {} memberships inserted",
            thread_count, membership_count);

        Ok((thread_count, membership_count))
    }

    /// Bulk insert threads and memberships with change detection
    async fn bulk_insert_threads(
        &self,
        mailing_list_id: i32,
        threads_to_create: Vec<ThreadInfo>,
    ) -> Result<(usize, usize), String> {
        use sha2::{Sha256, Digest};

        if threads_to_create.is_empty() {
            return Ok((0, 0));
        }

        let mut conn = self.pool.acquire().await
            .map_err(|e| format!("Failed to acquire connection: {}", e))?;

        let mut tx = conn.begin().await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Prepare thread data
        log::debug!("Preparing {} threads for bulk insert", threads_to_create.len());

        let mut prepared_threads = Vec::new();

        for thread_info in threads_to_create {
            // Build membership map
            let mut membership_map = HashMap::new();
            for (email_id, depth) in thread_info.emails {
                membership_map.entry(email_id).or_insert(depth);
            }

            // Compute membership hash
            let mut sorted_email_ids: Vec<i32> = membership_map.keys().copied().collect();
            sorted_email_ids.sort_unstable();
            let mut hasher = Sha256::new();
            for email_id in sorted_email_ids {
                hasher.update(email_id.to_le_bytes());
            }
            let membership_hash = hasher.finalize().to_vec();

            // Compute thread statistics
            let message_count = membership_map.len() as i32;
            // Use the dates already computed by JWZ
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

        let thread_count = prepared_threads.len();

        // Bulk check existing threads
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

        // Filter out unchanged threads
        let mut threads_to_upsert = Vec::new();
        let mut thread_id_map: HashMap<String, i32> = HashMap::new();
        let mut skipped_count = 0;

        for thread in prepared_threads {
            let (ref root_msg_id, .., ref hash, _) = thread;
            if let Some((existing_id, Some(existing_hash))) = existing_map.get(root_msg_id) {
                if existing_hash == hash {
                    // Thread unchanged, skip
                    skipped_count += 1;
                    continue;
                }
                // Thread exists but changed, will update
                thread_id_map.insert(root_msg_id.clone(), *existing_id);
            }
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

        // Bulk insert/update threads
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

        // Bulk insert memberships
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

    async fn load_last_indexed_commits(&self, list_id: i32)
        -> Result<HashMap<i32, String>, String> {
        let rows: Vec<(i32, Option<String>)> = sqlx::query_as(
            r#"SELECT repo_order, last_indexed_commit
               FROM mailing_list_repositories
               WHERE mailing_list_id = $1"#
        )
        .bind(list_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load last indexed commits: {}", e))?;

        let mut map = HashMap::new();
        for (repo_order, last_commit) in rows {
            if let Some(commit_hash) = last_commit {
                map.insert(repo_order, commit_hash);
            }
        }

        Ok(map)
    }

    async fn save_last_indexed_commits(&self, list_id: i32, commits: &HashMap<i32, String>) -> Result<(), String> {
        for (repo_order, commit_hash) in commits {
            sqlx::query(
                r#"UPDATE mailing_list_repositories
                   SET last_indexed_commit = $1
                   WHERE mailing_list_id = $2 AND repo_order = $3"#
            )
            .bind(commit_hash)
            .bind(list_id)
            .bind(repo_order)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to save last indexed commit for repo {}: {}", repo_order, e))?;
        }

        Ok(())
    }

    async fn save_last_threaded_at(&self, list_id: i32) -> Result<(), String> {
        let timestamp = chrono::Utc::now();
        sqlx::query(
            r#"UPDATE mailing_lists
               SET last_threaded_at = $1
               WHERE id = $2"#
        )
        .bind(timestamp)
        .bind(list_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save last threaded timestamp: {}", e))?;

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
