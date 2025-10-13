use crate::sync::parser::ParsedEmail;
use crate::threading::{build_threads, extract_patch_series_info, EmailData as ThreadEmailData, ThreadingCache};
use rocket_db_pools::sqlx::{PgPool, Postgres};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use chrono::{DateTime, Utc};

const STREAM_CHUNK_SIZE: usize = 25_000;

pub struct BulkImporter {
    pool: PgPool,
    mailing_list_id: i32,
}

#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub authors: usize,
    pub emails: usize,
    pub recipients: usize,
    pub references: usize,
    pub threads: usize,
    pub thread_memberships: usize,
}

impl ImportStats {
    pub fn merge(&mut self, other: ImportStats) {
        self.authors += other.authors;
        self.emails += other.emails;
        self.recipients += other.recipients;
        self.references += other.references;
        self.threads += other.threads;
        self.thread_memberships += other.thread_memberships;
    }
}

// Data structures for bulk operations
#[derive(Default)]
struct EmailsData {
    message_ids: Vec<String>,
    commit_hashes: Vec<String>,
    author_ids: Vec<i32>,
    subjects: Vec<String>,
    normalized_subjects: Vec<String>,
    dates: Vec<DateTime<Utc>>,
    in_reply_tos: Vec<Option<String>>,
    bodies: Vec<String>,
    series_ids: Vec<Option<String>>,
    series_numbers: Vec<Option<i32>>,
    series_totals: Vec<Option<i32>>,
    epochs: Vec<i32>,
}

#[derive(Default)]
struct RecipientsData {
    list_ids: Vec<i32>,
    email_ids: Vec<i32>,
    author_ids: Vec<i32>,
    recipient_types: Vec<String>,
}

#[derive(Default, Clone)]
struct ReferencesData {
    list_ids: Vec<i32>,
    email_ids: Vec<i32>,
    referenced_message_ids: Vec<String>,
    positions: Vec<i32>,
}

/// Data needed to merge newly imported emails into the threading cache
struct ChunkCacheData {
    /// Email data: (email_id, message_id, subject, in_reply_to, date, series_id, series_number, series_total)
    emails: Vec<(i32, String, String, Option<String>, DateTime<Utc>, Option<String>, Option<i32>, Option<i32>)>,
    /// References: (email_id, Vec<referenced_message_ids>)
    references: Vec<(i32, Vec<String>)>,
}

impl BulkImporter {
    pub fn new(pool: PgPool, mailing_list_id: i32) -> Self {
        Self { pool, mailing_list_id }
    }

    /// Import and thread emails for a specific epoch with 2-epoch cache window
    /// This is the core of the optimized sync workflow:
    /// 1. Load/build cache for 2-epoch window FIRST
    /// 2. Import emails to database (streaming in chunks) with epoch tags
    /// 3. As each chunk is imported, merge new emails into cache
    /// 4. Thread using the populated cache
    /// 5. Save cache to disk for next iteration
    pub async fn import_and_thread_epoch(
        &self,
        emails: Vec<(String, ParsedEmail, i32)>, // (commit, email, epoch)
        cache_epoch_range: (i32, i32),
    ) -> Result<ImportStats, sqlx::Error> {
        let total = emails.len();
        let num_chunks = (total + STREAM_CHUNK_SIZE - 1) / STREAM_CHUNK_SIZE;

        log::info!(
            "import+thread: {} emails in {} chunks, cache window: epochs {}-{}",
            total, num_chunks, cache_epoch_range.0, cache_epoch_range.1
        );

        // Step 1: Load or build cache for 2-epoch window BEFORE importing
        log::info!("loading threading cache for list {} (epochs {}-{})",
            self.mailing_list_id, cache_epoch_range.0, cache_epoch_range.1);

        let cache_base_path = std::env::var("THREADING_CACHE_BASE_PATH")
            .unwrap_or_else(|_| "./cache".to_string());
        let cache_path = format!("{}/{}_threading_v1.bin", cache_base_path, self.mailing_list_id);

        // Try to load cache from disk (synchronously, without await)
        let cache_load_result = if Path::new(&cache_path).exists() {
            ThreadingCache::load_from_disk(Path::new(&cache_path)).ok()
        } else {
            None
        };

        // Now handle the result with awaits (after the non-Send type is dropped)
        let mut cache = match cache_load_result {
            Some(c) if c.covers_epochs(cache_epoch_range) => {
                log::info!("loaded threading cache from disk (fast path)");
                c
            }
            Some(_) => {
                log::info!("cache exists but doesn't cover required epochs, rebuilding from database");
                ThreadingCache::load_from_db(&self.pool, self.mailing_list_id, cache_epoch_range).await?
            }
            None => {
                log::info!("no cache found or failed to load, building from database");
                ThreadingCache::load_from_db(&self.pool, self.mailing_list_id, cache_epoch_range).await?
            }
        };

        let cache_stats = cache.stats();
        log::info!(
            "cache ready: {} emails, {} references from epochs {}-{}",
            cache_stats.email_count,
            cache_stats.reference_count,
            cache_stats.epoch_range.0,
            cache_stats.epoch_range.1
        );

        let mut cumulative_stats = ImportStats::default();

        // Step 2: Import emails in chunks and merge into cache
        for (chunk_idx, chunk) in emails.chunks(STREAM_CHUNK_SIZE).enumerate() {
            log::debug!("importing chunk {}/{}", chunk_idx + 1, num_chunks);
            let (stats, cache_data) = self.import_chunk(chunk).await?;
            cumulative_stats.merge(stats);

            // Merge newly imported emails into cache
            if !cache_data.emails.is_empty() {
                log::trace!("merging {} emails from chunk {} into cache", cache_data.emails.len(), chunk_idx + 1);
                cache.merge_new_emails(cache_data.emails);
                cache.merge_new_references(cache_data.references);
            }
        }

        // Step 3: Build threads with the populated cache
        log::info!("threading with cache for epochs {}-{}", cache_epoch_range.0, cache_epoch_range.1);
        let (thread_count, membership_count) = self
            .build_threads_with_cache(cache, &cache_path)
            .await?;

        cumulative_stats.threads = thread_count;
        cumulative_stats.thread_memberships = membership_count;

        // Step 4: Update author activity stats
        log::info!("updating author activity");
        self.update_author_activity().await?;

        Ok(cumulative_stats)
    }

    /// Import single chunk with 4 parallel database connections
    /// Returns ImportStats and ChunkCacheData for merging into threading cache
    async fn import_chunk(&self, chunk: &[(String, ParsedEmail, i32)])
        -> Result<(ImportStats, ChunkCacheData), sqlx::Error> {

        // Prepare data structures (in-memory, fast)
        let authors_data = self.prepare_authors(chunk);
        let prepared_author_count = authors_data.len();

        // First insert authors so we have their IDs
        let mut conn1 = self.pool.acquire().await?;
        let author_count = self.bulk_insert_authors(&mut conn1, authors_data).await?;
        drop(conn1); // Release connection

        log::trace!("chunk: prepared {} authors, bulk_insert returned {}", prepared_author_count, author_count);

        // Load author IDs and prepare emails data (now includes epochs)
        let emails_data = self.prepare_emails(chunk).await?;

        log::trace!("chunk: prepared {} emails for insertion (chunk size: {})", emails_data.message_ids.len(), chunk.len());

        // Insert emails to get their IDs
        let mut conn2 = self.pool.acquire().await?;
        let email_count = self.bulk_insert_emails(&mut conn2, &emails_data).await?;
        drop(conn2); // Release connection

        // Load email IDs for recipients and references
        let email_id_map = self.load_email_ids(chunk).await?;
        let recipients_data = self.prepare_recipients(chunk, &email_id_map).await?;
        let references_data = self.prepare_references(chunk, &email_id_map);

        // Import recipients and references in parallel
        let mut conn3 = self.pool.acquire().await?;
        let mut conn4 = self.pool.acquire().await?;

        // Clone references_data before moving it to bulk_insert_references
        let references_data_clone = references_data.clone();

        let (recipient_count, reference_count) = tokio::try_join!(
            self.bulk_insert_recipients(&mut conn3, recipients_data),
            self.bulk_insert_references(&mut conn4, references_data_clone),
        )?;

        // Prepare cache data for merging
        let mut cache_emails = Vec::new();
        let mut cache_refs_map: HashMap<i32, Vec<String>> = HashMap::new();
        let mut cache_miss_count = 0;

        // Build email data for cache
        for (_, email, _) in chunk {
            if let Some(&email_id) = email_id_map.get(&email.message_id) {
                let series_info = extract_patch_series_info(&email.subject);
                let (series_id, series_number, series_total) = if let Some((sid, snum, stot)) = series_info {
                    (Some(sid), Some(snum), Some(stot))
                } else {
                    (None, None, None)
                };

                cache_emails.push((
                    email_id,
                    email.message_id.clone(),
                    email.subject.clone(),
                    email.in_reply_to.clone(),
                    email.date,
                    series_id,
                    series_number,
                    series_total,
                ));

                // Build references map for this email
                let mut refs = Vec::new();
                for ref_msg_id in &email.references {
                    refs.push(ref_msg_id.clone());
                }
                if !refs.is_empty() {
                    cache_refs_map.insert(email_id, refs);
                }
            } else {
                // DIAGNOSTIC: Email's ID not found in email_id_map
                cache_miss_count += 1;
                if cache_miss_count <= 5 {
                    log::warn!(
                        "Email {} not found in email_id_map (not inserted to DB?)",
                        &email.message_id
                    );
                }
            }
        }

        if cache_miss_count > 0 {
            log::warn!(
                "import_chunk: {} emails not added to cache (not in email_id_map), chunk size: {}",
                cache_miss_count,
                chunk.len()
            );
        }

        let cache_data = ChunkCacheData {
            emails: cache_emails,
            references: cache_refs_map.into_iter().collect(),
        };

        let stats = ImportStats {
            authors: author_count,
            emails: email_count,
            recipients: recipient_count,
            references: reference_count,
            threads: 0,
            thread_memberships: 0,
        };

        Ok((stats, cache_data))
    }

    /// Prepare author data from chunk
    fn prepare_authors(&self, chunk: &[(String, ParsedEmail, i32)]) -> HashMap<String, String> {
        let mut authors = HashMap::new();

        for (_, email, _) in chunk {
            // Add sender (pick first name if multiple are seen)
            authors.entry(email.author_email.clone())
                .or_insert_with(|| email.author_name.clone());

            // Add recipients
            for (name, addr) in &email.to_addrs {
                authors.entry(addr.clone()).or_insert_with(|| name.clone());
            }
            for (name, addr) in &email.cc_addrs {
                authors.entry(addr.clone()).or_insert_with(|| name.clone());
            }
        }

        authors
    }

    /// Bulk insert authors using UNNEST
    async fn bulk_insert_authors(
        &self,
        conn: &mut sqlx::pool::PoolConnection<Postgres>,
        authors: HashMap<String, String>,
    ) -> Result<usize, sqlx::Error> {

        if authors.is_empty() {
            return Ok(0);
        }

        let mut emails = Vec::new();
        let mut names = Vec::new();

        for (email, name) in authors {
            emails.push(email);
            names.push(if name.is_empty() { None } else { Some(name) });
        }

        let count = emails.len();

        sqlx::query(
            r#"INSERT INTO authors (email, canonical_name, first_seen, last_seen)
               SELECT email, name, NOW(), NOW()
               FROM UNNEST($1::text[], $2::text[]) AS t(email, name)
               ON CONFLICT (email) DO UPDATE
               SET last_seen = NOW(),
                   canonical_name = COALESCE(EXCLUDED.canonical_name, authors.canonical_name)"#
        )
        .bind(&emails)
        .bind(&names)
        .execute(&mut **conn)
        .await?;

        log::trace!("bulk inserted {} authors", count);
        Ok(count)
    }

    /// Prepare emails data from chunk (now includes epochs)
    async fn prepare_emails(&self, chunk: &[(String, ParsedEmail, i32)]) -> Result<EmailsData, sqlx::Error> {
        // Load author IDs first
        let unique_emails: Vec<String> = chunk.iter()
            .map(|(_, e, _)| e.author_email.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let author_rows: Vec<(String, i32)> = sqlx::query_as(
            "SELECT email, id FROM authors WHERE email = ANY($1)"
        )
        .bind(&unique_emails)
        .fetch_all(&self.pool)
        .await?;

        let author_map: HashMap<String, i32> = author_rows.into_iter().collect();

        // Diagnostic: log if we have missing authors
        let unique_email_count = unique_emails.len();
        let found_author_count = author_map.len();
        if unique_email_count != found_author_count {
            log::warn!(
                "Author lookup mismatch: expected {} unique authors, found {} (missing: {})",
                unique_email_count,
                found_author_count,
                unique_email_count - found_author_count
            );
        }

        // Build parallel vectors for UNNEST
        let mut data = EmailsData::default();
        let mut skipped_count = 0;

        for (commit_hash, email, epoch) in chunk {
            if let Some(&author_id) = author_map.get(&email.author_email) {
                let series_info = extract_patch_series_info(&email.subject);

                data.message_ids.push(email.message_id.clone());
                data.commit_hashes.push(commit_hash.clone());
                data.author_ids.push(author_id);
                data.subjects.push(email.subject.clone());
                data.normalized_subjects.push(email.normalized_subject.clone());
                data.dates.push(email.date);
                data.in_reply_tos.push(email.in_reply_to.clone());
                data.bodies.push(email.body.clone());

                if let Some((series_id, series_num, series_total)) = series_info {
                    data.series_ids.push(Some(series_id));
                    data.series_numbers.push(Some(series_num));
                    data.series_totals.push(Some(series_total));
                } else {
                    data.series_ids.push(None);
                    data.series_numbers.push(None);
                    data.series_totals.push(None);
                }

                // Store epoch for this email
                data.epochs.push(*epoch);
            } else {
                // DIAGNOSTIC: Email skipped due to missing author
                skipped_count += 1;
                if skipped_count <= 5 {
                    log::warn!(
                        "Skipping email {} (commit {}) - author '{}' not found in author_map",
                        &email.message_id,
                        &commit_hash[..8],
                        &email.author_email
                    );
                }
            }
        }

        if skipped_count > 0 {
            log::warn!(
                "prepare_emails: skipped {} emails due to missing author IDs (chunk size: {})",
                skipped_count,
                chunk.len()
            );
        }

        Ok(data)
    }

    /// Bulk insert emails using UNNEST (all 13 fields as arrays, including epoch)
    async fn bulk_insert_emails(
        &self,
        conn: &mut sqlx::pool::PoolConnection<Postgres>,
        data: &EmailsData,
    ) -> Result<usize, sqlx::Error> {

        if data.message_ids.is_empty() {
            return Ok(0);
        }

        let count = data.message_ids.len();
        let list_ids = vec![self.mailing_list_id; count];

        let result = sqlx::query(
            r#"INSERT INTO emails (
                mailing_list_id, message_id, git_commit_hash, author_id,
                subject, normalized_subject, date, in_reply_to, body,
                series_id, series_number, series_total, epoch
               )
               SELECT * FROM UNNEST(
                   $1::int[], $2::text[], $3::text[], $4::int[],
                   $5::text[], $6::text[], $7::timestamptz[], $8::text[], $9::text[],
                   $10::text[], $11::int[], $12::int[], $13::int[]
               )
               ON CONFLICT (mailing_list_id, message_id) DO NOTHING"#
        )
        .bind(&list_ids)
        .bind(&data.message_ids)
        .bind(&data.commit_hashes)
        .bind(&data.author_ids)
        .bind(&data.subjects)
        .bind(&data.normalized_subjects)
        .bind(&data.dates)
        .bind(&data.in_reply_tos)
        .bind(&data.bodies)
        .bind(&data.series_ids)
        .bind(&data.series_numbers)
        .bind(&data.series_totals)
        .bind(&data.epochs)
        .execute(&mut **conn)
        .await?;

        let rows_affected = result.rows_affected() as usize;
        if rows_affected < count {
            log::debug!(
                "bulk_insert_emails: tried to insert {} emails, but only {} were inserted ({} skipped due to conflicts)",
                count, rows_affected, count - rows_affected
            );
        }

        log::trace!("bulk inserted {} emails", rows_affected);
        Ok(rows_affected)
    }

    /// Load email IDs after insertion
    async fn load_email_ids(&self, chunk: &[(String, ParsedEmail, i32)]) -> Result<HashMap<String, i32>, sqlx::Error> {
        let message_ids: Vec<String> = chunk.iter()
            .map(|(_, e, _)| e.message_id.clone())
            .collect();

        let expected_count = message_ids.len();

        let rows: Vec<(String, i32)> = sqlx::query_as(
            "SELECT message_id, id FROM emails WHERE mailing_list_id = $1 AND message_id = ANY($2)"
        )
        .bind(self.mailing_list_id)
        .bind(&message_ids)
        .fetch_all(&self.pool)
        .await?;

        let found_count = rows.len();
        if found_count < expected_count {
            log::debug!(
                "load_email_ids: expected {} emails but only found {} in database (missing: {})",
                expected_count, found_count, expected_count - found_count
            );
        }

        Ok(rows.into_iter().collect())
    }

    /// Prepare recipients data from chunk
    async fn prepare_recipients(
        &self,
        chunk: &[(String, ParsedEmail, i32)],
        email_id_map: &HashMap<String, i32>,
    ) -> Result<RecipientsData, sqlx::Error> {
        // Collect all recipient emails to load their author IDs
        let mut recipient_emails = HashSet::new();
        for (_, email, _) in chunk {
            for (_, addr) in &email.to_addrs {
                recipient_emails.insert(addr.clone());
            }
            for (_, addr) in &email.cc_addrs {
                recipient_emails.insert(addr.clone());
            }
        }

        let recipient_emails_vec: Vec<String> = recipient_emails.into_iter().collect();
        let author_rows: Vec<(String, i32)> = if !recipient_emails_vec.is_empty() {
            sqlx::query_as(
                "SELECT email, id FROM authors WHERE email = ANY($1)"
            )
            .bind(&recipient_emails_vec)
            .fetch_all(&self.pool)
            .await?
        } else {
            Vec::new()
        };

        let author_map: HashMap<String, i32> = author_rows.into_iter().collect();

        // Use a HashSet to deduplicate (email_id, author_id, recipient_type) tuples
        let mut recipient_set: HashSet<(i32, i32, String)> = HashSet::new();

        for (_, email, _) in chunk {
            if let Some(&email_id) = email_id_map.get(&email.message_id) {
                // Collect To recipients
                for (_, addr) in &email.to_addrs {
                    if let Some(&author_id) = author_map.get(addr) {
                        recipient_set.insert((email_id, author_id, "to".to_string()));
                    }
                }

                // Collect Cc recipients
                for (_, addr) in &email.cc_addrs {
                    if let Some(&author_id) = author_map.get(addr) {
                        recipient_set.insert((email_id, author_id, "cc".to_string()));
                    }
                }
            }
        }

        // Build parallel vectors
        let mut data = RecipientsData::default();
        for (email_id, author_id, recipient_type) in recipient_set {
            data.list_ids.push(self.mailing_list_id);
            data.email_ids.push(email_id);
            data.author_ids.push(author_id);
            data.recipient_types.push(recipient_type);
        }

        Ok(data)
    }

    /// Bulk insert recipients using UNNEST
    async fn bulk_insert_recipients(
        &self,
        conn: &mut sqlx::pool::PoolConnection<Postgres>,
        data: RecipientsData,
    ) -> Result<usize, sqlx::Error> {

        if data.email_ids.is_empty() {
            return Ok(0);
        }

        let count = data.email_ids.len();

        sqlx::query(
            r#"INSERT INTO email_recipients (mailing_list_id, email_id, author_id, recipient_type)
               SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::text[])"#
        )
        .bind(&data.list_ids)
        .bind(&data.email_ids)
        .bind(&data.author_ids)
        .bind(&data.recipient_types)
        .execute(&mut **conn)
        .await?;

        log::trace!("bulk inserted {} recipients", count);
        Ok(count)
    }

    /// Prepare references data from chunk
    fn prepare_references(
        &self,
        chunk: &[(String, ParsedEmail, i32)],
        email_id_map: &HashMap<String, i32>,
    ) -> ReferencesData {
        // Use a HashMap to deduplicate (email_id, referenced_message_id) pairs
        // Keep the first occurrence's position
        let mut reference_map: HashMap<(i32, String), i32> = HashMap::new();

        for (_, email, _) in chunk {
            if let Some(&email_id) = email_id_map.get(&email.message_id) {
                // Collect references with their position to preserve order
                for (position, ref_msg_id) in email.references.iter().enumerate() {
                    let key = (email_id, ref_msg_id.clone());
                    reference_map.entry(key).or_insert(position as i32);
                }
            }
        }

        // Build parallel vectors
        let mut data = ReferencesData::default();
        for ((email_id, ref_msg_id), position) in reference_map {
            data.list_ids.push(self.mailing_list_id);
            data.email_ids.push(email_id);
            data.referenced_message_ids.push(ref_msg_id);
            data.positions.push(position);
        }

        data
    }

    /// Bulk insert references using UNNEST
    async fn bulk_insert_references(
        &self,
        conn: &mut sqlx::pool::PoolConnection<Postgres>,
        data: ReferencesData,
    ) -> Result<usize, sqlx::Error> {

        if data.email_ids.is_empty() {
            return Ok(0);
        }

        let count = data.email_ids.len();

        sqlx::query(
            r#"INSERT INTO email_references (mailing_list_id, email_id, referenced_message_id, position)
               SELECT * FROM UNNEST($1::int[], $2::int[], $3::text[], $4::int[])
               ON CONFLICT (mailing_list_id, email_id, referenced_message_id) DO NOTHING"#
        )
        .bind(&data.list_ids)
        .bind(&data.email_ids)
        .bind(&data.referenced_message_ids)
        .bind(&data.positions)
        .execute(&mut **conn)
        .await?;

        log::trace!("bulk inserted {} references", count);
        Ok(count)
    }

    /// Build threads with cache for 2-epoch window
    /// Implements the optimized threading path using an in-memory cache
    ///
    /// This method uses a pre-populated cache that already contains:
    /// 1. All emails from the cache epoch range that were in the database before sync
    /// 2. All newly imported emails merged during the import phase
    ///
    /// The cache provides:
    /// - Fast loading from disk (bincode) instead of database queries
    /// - Bounded memory usage (only 2 epochs worth of data in cache)
    /// - Persistence across sync runs
    async fn build_threads_with_cache(
        &self,
        cache: ThreadingCache,
        cache_path: &str,
    ) -> Result<(usize, usize), sqlx::Error> {
        let cache_stats = cache.stats();
        let cache_epoch_range = cache_stats.epoch_range;

        log::info!(
            "threading with populated cache: {} emails, {} references from epochs {}-{}",
            cache_stats.email_count,
            cache_stats.reference_count,
            cache_epoch_range.0,
            cache_epoch_range.1
        );

        // Step 1: Get cached data for threading
        let cached_email_data = cache.get_all_email_data();
        let cached_references = cache.get_all_references();

        // Step 2: Load emails OUTSIDE cache epoch range from database
        // These are needed for complete threading (old emails from other epochs that might be referenced)
        log::info!("loading emails outside cache epoch range from database");

        let mut tx = self.pool.begin().await?;

        let db_only_emails: Vec<(i32, String, String, Option<String>, DateTime<Utc>, Option<String>, Option<i32>, Option<i32>)> =
            sqlx::query_as(
                r#"SELECT e.id, e.message_id, e.subject, e.in_reply_to, e.date,
                          e.series_id, e.series_number, e.series_total
                   FROM emails e
                   WHERE e.mailing_list_id = $1
                   AND (e.epoch < $2 OR e.epoch > $3)
                   ORDER BY e.date"#
            )
            .bind(self.mailing_list_id)
            .bind(cache_epoch_range.0)
            .bind(cache_epoch_range.1)
            .fetch_all(&mut *tx)
            .await?;

        let db_only_email_count = db_only_emails.len();
        log::info!("loaded {} emails from database (outside cache)", db_only_email_count);

        // Load references for emails outside cache
        let db_email_ids: Vec<i32> = db_only_emails.iter().map(|(id, ..)| *id).collect();
        let db_only_refs: Vec<(i32, String)> = if !db_email_ids.is_empty() {
            sqlx::query_as(
                r#"SELECT email_id, referenced_message_id
                   FROM email_references
                   WHERE mailing_list_id = $1 AND email_id = ANY($2)
                   ORDER BY email_id, position"#
            )
            .bind(self.mailing_list_id)
            .bind(&db_email_ids)
            .fetch_all(&mut *tx)
            .await?
        } else {
            Vec::new()
        };

        // Step 3: Merge cached and DB data
        let mut all_email_data = HashMap::new();
        let mut all_references = HashMap::new();

        // Add cached data (includes newly imported emails)
        for (email_id, (msg_id, subject, in_reply_to, date, series_id, series_num, series_total)) in cached_email_data {
            all_email_data.insert(
                email_id,
                ThreadEmailData {
                    id: email_id,
                    message_id: msg_id,
                    subject,
                    in_reply_to,
                    date,
                    series_id,
                    series_number: series_num,
                    series_total,
                }
            );
        }
        all_references.extend(cached_references);

        // Add DB data (emails from outside cache epoch range)
        for (id, message_id, subject, in_reply_to, date, series_id, series_number, series_total) in db_only_emails {
            all_email_data.insert(
                id,
                ThreadEmailData {
                    id,
                    message_id,
                    subject,
                    in_reply_to,
                    date,
                    series_id,
                    series_number,
                    series_total,
                },
            );
        }

        // Group DB references
        for (email_id, ref_msg_id) in db_only_refs {
            all_references
                .entry(email_id)
                .or_insert_with(Vec::new)
                .push(ref_msg_id);
        }

        log::info!(
            "threading with merged data: {} total emails ({} from cache + newly imported, {} from DB outside cache)",
            all_email_data.len(),
            cache_stats.email_count,
            db_only_email_count
        );

        // Step 4: Run JWZ threading algorithm
        log::debug!("running JWZ algorithm on {} emails", all_email_data.len());

        let threads_to_create = tokio::task::spawn_blocking(move || {
            build_threads(all_email_data, all_references)
        })
        .await
        .map_err(|e| sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Threading task panicked: {}", e)
        )))?;

        log::debug!("threading complete: {} threads", threads_to_create.len());

        // Step 6: Insert threads and memberships
        let thread_count = threads_to_create.len();
        let mut membership_count = 0;

        for thread_info in threads_to_create {
            // Bulk insert memberships
            let mut membership_map: HashMap<i32, i32> = HashMap::new();
            for (email_id, depth) in thread_info.emails {
                membership_map.entry(email_id).or_insert(depth);
            }

            // Compute membership hash: SHA256 of sorted email IDs
            let membership_hash = {
                let mut sorted_email_ids: Vec<i32> = membership_map.keys().copied().collect();
                sorted_email_ids.sort_unstable();
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                for email_id in sorted_email_ids {
                    hasher.update(email_id.to_le_bytes());
                }
                hasher.finalize().to_vec()
            };

            // Check if thread exists with the same membership_hash
            let existing_hash: Option<(i32, Option<Vec<u8>>)> = sqlx::query_as(
                "SELECT id, membership_hash FROM threads WHERE mailing_list_id = $1 AND root_message_id = $2"
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .fetch_optional(&mut *tx)
            .await?;

            let should_update = match existing_hash {
                Some((_, Some(ref existing))) if existing == &membership_hash => {
                    log::trace!("thread {} unchanged, skipping update", &thread_info.root_message_id);
                    false
                }
                _ => true
            };

            if !should_update {
                continue;
            }

            // Insert thread with membership_hash
            sqlx::query(
                r#"INSERT INTO threads (mailing_list_id, root_message_id, subject, start_date, last_date, message_count, membership_hash)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   ON CONFLICT (mailing_list_id, root_message_id) DO NOTHING"#
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .bind(&thread_info.subject)
            .bind(thread_info.start_date)
            .bind(thread_info.start_date)
            .bind(membership_map.len() as i32)
            .bind(&membership_hash)
            .execute(&mut *tx)
            .await?;

            // Get thread ID
            let thread_row: (i32,) = sqlx::query_as(
                "SELECT id FROM threads WHERE mailing_list_id = $1 AND root_message_id = $2"
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .fetch_one(&mut *tx)
            .await?;

            let thread_id = thread_row.0;

            let membership_count_for_thread = membership_map.len();
            if membership_count_for_thread > 0 {
                let mut mailing_list_ids = Vec::new();
                let mut thread_ids = Vec::new();
                let mut email_ids = Vec::new();
                let mut depths = Vec::new();

                for (email_id, depth) in membership_map {
                    mailing_list_ids.push(self.mailing_list_id);
                    thread_ids.push(thread_id);
                    email_ids.push(email_id);
                    depths.push(depth);
                }

                sqlx::query(
                    r#"INSERT INTO thread_memberships (mailing_list_id, thread_id, email_id, depth)
                       SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::int[])
                       ON CONFLICT (mailing_list_id, thread_id, email_id) DO NOTHING"#
                )
                .bind(&mailing_list_ids)
                .bind(&thread_ids)
                .bind(&email_ids)
                .bind(&depths)
                .execute(&mut *tx)
                .await?;

                membership_count += membership_count_for_thread;
            }

            // Update thread statistics
            sqlx::query(
                r#"UPDATE threads SET
                    message_count = (SELECT COUNT(*) FROM thread_memberships
                                    WHERE mailing_list_id = $1 AND thread_id = $2),
                    start_date = (
                        SELECT MIN(e.date) FROM emails e
                        JOIN thread_memberships tm ON tm.email_id = e.id
                        WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
                    ),
                    last_date = (
                        SELECT MAX(e.date) FROM emails e
                        JOIN thread_memberships tm ON tm.email_id = e.id
                        WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
                    ),
                    membership_hash = $3
                WHERE mailing_list_id = $1 AND id = $2"#
            )
            .bind(self.mailing_list_id)
            .bind(thread_id)
            .bind(&membership_hash)
            .execute(&mut *tx)
            .await?;
        }

        // Mark all emails as threaded
        sqlx::query(
            r#"UPDATE emails SET threaded_at = NOW()
               WHERE mailing_list_id = $1"#
        )
        .bind(self.mailing_list_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Step 7: Save updated cache to disk for next iteration
        log::info!("saving threading cache to disk");
        cache.save_to_disk(Path::new(&cache_path))
            .map_err(|e| sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save cache: {}", e)
            )))?;

        Ok((thread_count, membership_count))
    }

    /// Build threads for all emails in the mailing list
    async fn build_threads_full(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> Result<(usize, usize), sqlx::Error> {
        // Load all emails
        let email_rows: Vec<(
            i32,
            String,
            String,
            Option<String>,
            DateTime<Utc>,
            Option<String>,
            Option<i32>,
            Option<i32>,
        )> = sqlx::query_as(
            r#"SELECT e.id, e.message_id, e.subject, e.in_reply_to, e.date,
                      e.series_id, e.series_number, e.series_total
               FROM emails e
               WHERE e.mailing_list_id = $1
               ORDER BY e.date"#
        )
        .bind(self.mailing_list_id)
        .fetch_all(&mut **tx)
        .await?;

        let email_ids: Vec<i32> = email_rows.iter().map(|(id, ..)| *id).collect();

        self.build_threads_for_emails(tx, &email_ids).await
    }

    /// Build threads for a specific set of emails
    async fn build_threads_for_emails(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        email_ids: &[i32],
    ) -> Result<(usize, usize), sqlx::Error> {
        // Load email data
        let email_rows: Vec<(
            i32,
            String,
            String,
            Option<String>,
            DateTime<Utc>,
            Option<String>,
            Option<i32>,
            Option<i32>,
        )> = sqlx::query_as(
            r#"SELECT e.id, e.message_id, e.subject, e.in_reply_to, e.date,
                      e.series_id, e.series_number, e.series_total
               FROM emails e
               WHERE e.mailing_list_id = $1 AND e.id = ANY($2)
               ORDER BY e.date"#
        )
        .bind(self.mailing_list_id)
        .bind(email_ids)
        .fetch_all(&mut **tx)
        .await?;

        let mut email_data: HashMap<i32, ThreadEmailData> = HashMap::new();
        for (id, message_id, subject, in_reply_to, date, series_id, series_number, series_total) in email_rows {
            email_data.insert(
                id,
                ThreadEmailData {
                    id,
                    message_id,
                    subject,
                    in_reply_to,
                    date,
                    series_id,
                    series_number,
                    series_total,
                },
            );
        }

        // Load references for these emails
        let ref_rows: Vec<(i32, String)> = sqlx::query_as(
            r#"SELECT email_id, referenced_message_id
               FROM email_references
               WHERE mailing_list_id = $1 AND email_id = ANY($2)
               ORDER BY email_id, position"#
        )
        .bind(self.mailing_list_id)
        .bind(email_ids)
        .fetch_all(&mut **tx)
        .await?;

        let mut email_references: HashMap<i32, Vec<String>> = HashMap::new();
        for (email_id, ref_msg_id) in ref_rows {
            email_references
                .entry(email_id)
                .or_insert_with(Vec::new)
                .push(ref_msg_id);
        }

        log::debug!("running JWZ on {} emails", email_data.len());

        // Run JWZ algorithm
        let threads_to_create = tokio::task::spawn_blocking(move || {
            build_threads(email_data, email_references)
        })
        .await
        .map_err(|e| sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Threading task panicked: {}", e)
        )))?;

        log::debug!("threading complete: {} threads", threads_to_create.len());

        // Insert threads and memberships
        let thread_count = threads_to_create.len();
        let mut membership_count = 0;

        for thread_info in threads_to_create {
            // Bulk insert memberships
            let mut membership_map: HashMap<i32, i32> = HashMap::new();
            for (email_id, depth) in thread_info.emails {
                membership_map.entry(email_id).or_insert(depth);
            }

            // Compute membership hash: SHA256 of sorted email IDs
            // This allows us to skip updates if membership hasn't actually changed
            let membership_hash = {
                let mut sorted_email_ids: Vec<i32> = membership_map.keys().copied().collect();
                sorted_email_ids.sort_unstable();
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                for email_id in sorted_email_ids {
                    hasher.update(email_id.to_le_bytes());
                }
                hasher.finalize().to_vec()
            };

            // Check if thread exists with the same membership_hash
            let existing_hash: Option<(i32, Option<Vec<u8>>)> = sqlx::query_as(
                "SELECT id, membership_hash FROM threads WHERE mailing_list_id = $1 AND root_message_id = $2"
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .fetch_optional(&mut **tx)
            .await?;

            let should_update = match existing_hash {
                Some((_, Some(ref existing))) if existing == &membership_hash => {
                    // Hash matches - no structural change, skip update
                    log::trace!("thread {} unchanged, skipping update", &thread_info.root_message_id);
                    false
                }
                _ => true
            };

            if !should_update {
                // Thread exists and hasn't changed, skip
                continue;
            }

            // Insert thread with membership_hash
            sqlx::query(
                r#"INSERT INTO threads (mailing_list_id, root_message_id, subject, start_date, last_date, message_count, membership_hash)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   ON CONFLICT (mailing_list_id, root_message_id) DO NOTHING"#
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .bind(&thread_info.subject)
            .bind(thread_info.start_date)
            .bind(thread_info.start_date)
            .bind(membership_map.len() as i32)
            .bind(&membership_hash)
            .execute(&mut **tx)
            .await?;

            // Get thread ID
            let thread_row: (i32,) = sqlx::query_as(
                "SELECT id FROM threads WHERE mailing_list_id = $1 AND root_message_id = $2"
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .fetch_one(&mut **tx)
            .await?;

            let thread_id = thread_row.0;

            let membership_count_for_thread = membership_map.len();
            if membership_count_for_thread > 0 {
                let mut mailing_list_ids = Vec::new();
                let mut thread_ids = Vec::new();
                let mut email_ids = Vec::new();
                let mut depths = Vec::new();

                for (email_id, depth) in membership_map {
                    mailing_list_ids.push(self.mailing_list_id);
                    thread_ids.push(thread_id);
                    email_ids.push(email_id);
                    depths.push(depth);
                }

                sqlx::query(
                    r#"INSERT INTO thread_memberships (mailing_list_id, thread_id, email_id, depth)
                       SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::int[])
                       ON CONFLICT (mailing_list_id, thread_id, email_id) DO NOTHING"#
                )
                .bind(&mailing_list_ids)
                .bind(&thread_ids)
                .bind(&email_ids)
                .bind(&depths)
                .execute(&mut **tx)
                .await?;

                membership_count += membership_count_for_thread;
            }

            // Update thread statistics including membership_hash
            sqlx::query(
                r#"UPDATE threads SET
                    message_count = (SELECT COUNT(*) FROM thread_memberships
                                    WHERE mailing_list_id = $1 AND thread_id = $2),
                    start_date = (
                        SELECT MIN(e.date) FROM emails e
                        JOIN thread_memberships tm ON tm.email_id = e.id
                        WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
                    ),
                    last_date = (
                        SELECT MAX(e.date) FROM emails e
                        JOIN thread_memberships tm ON tm.email_id = e.id
                        WHERE tm.mailing_list_id = $1 AND tm.thread_id = $2
                    ),
                    membership_hash = $3
                WHERE mailing_list_id = $1 AND id = $2"#
            )
            .bind(self.mailing_list_id)
            .bind(thread_id)
            .bind(&membership_hash)
            .execute(&mut **tx)
            .await?;
        }

        Ok((thread_count, membership_count))
    }

    /// Update author activity stats
    async fn update_author_activity(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Calculate stats per author for this mailing list
        sqlx::query(
            r#"INSERT INTO author_mailing_list_activity (author_id, mailing_list_id, first_email_date, last_email_date, email_count, thread_count)
               SELECT
                   e.author_id,
                   e.mailing_list_id,
                   MIN(e.date) as first_email_date,
                   MAX(e.date) as last_email_date,
                   COUNT(DISTINCT e.id) as email_count,
                   COUNT(DISTINCT tm.thread_id) as thread_count
               FROM emails e
               LEFT JOIN thread_memberships tm ON e.id = tm.email_id AND e.mailing_list_id = tm.mailing_list_id
               WHERE e.mailing_list_id = $1
               GROUP BY e.author_id, e.mailing_list_id
               ON CONFLICT (author_id, mailing_list_id) DO UPDATE
               SET first_email_date = EXCLUDED.first_email_date,
                   last_email_date = EXCLUDED.last_email_date,
                   email_count = EXCLUDED.email_count,
                   thread_count = EXCLUDED.thread_count"#
        )
        .bind(self.mailing_list_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
