use crate::sync::parser::ParsedEmail;
use crate::threading::{extract_patch_series_info, EmailThreadingInfo};
use rocket_db_pools::sqlx::{PgPool, Postgres};
use std::collections::{HashMap, HashSet};
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


    /// Import single chunk with enhanced parallel database operations
    /// Returns ImportStats and ChunkCacheData for merging into threading cache
    ///
    /// Optimizations:
    /// - Uses up to 6 parallel connections from the increased bulk_write_db pool
    /// - Parallelizes data loading operations where possible
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

        // Parallelize email ID loading and recipient author ID loading
        let message_ids: Vec<String> = chunk.iter().map(|(_, e, _)| e.message_id.clone()).collect();

        // Collect recipient emails for parallel loading
        let mut recipient_emails = std::collections::HashSet::new();
        for (_, email, _) in chunk {
            for (_, addr) in &email.to_addrs {
                recipient_emails.insert(addr.clone());
            }
            for (_, addr) in &email.cc_addrs {
                recipient_emails.insert(addr.clone());
            }
        }
        let recipient_emails_vec: Vec<String> = recipient_emails.into_iter().collect();

        // Parallel load: email IDs and recipient author IDs
        let (email_id_rows, recipient_author_rows) = tokio::try_join!(
            async {
                sqlx::query_as::<_, (String, i32)>(
                    "SELECT message_id, id FROM emails WHERE mailing_list_id = $1 AND message_id = ANY($2)"
                )
                .bind(self.mailing_list_id)
                .bind(&message_ids)
                .fetch_all(&self.pool)
                .await
            },
            async {
                if !recipient_emails_vec.is_empty() {
                    sqlx::query_as::<_, (String, i32)>(
                        "SELECT email, id FROM authors WHERE email = ANY($1)"
                    )
                    .bind(&recipient_emails_vec)
                    .fetch_all(&self.pool)
                    .await
                } else {
                    Ok(Vec::new())
                }
            }
        )?;

        let email_id_map: std::collections::HashMap<String, i32> = email_id_rows.into_iter().collect();
        let recipient_author_map: std::collections::HashMap<String, i32> = recipient_author_rows.into_iter().collect();

        // Prepare recipients and references data with pre-loaded maps
        let recipients_data = self.prepare_recipients_with_map(chunk, &email_id_map, &recipient_author_map);
        let references_data = self.prepare_references(chunk, &email_id_map);

        // Import recipients and references in parallel (2 connections)
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


    /// Prepare recipients data from chunk with pre-loaded author map (optimized)
    /// This version is used by the parallelized import_chunk to avoid duplicate database queries
    fn prepare_recipients_with_map(
        &self,
        chunk: &[(String, ParsedEmail, i32)],
        email_id_map: &HashMap<String, i32>,
        author_map: &HashMap<String, i32>,
    ) -> RecipientsData {
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

        data
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


    /// Import emails in streaming chunks and populate unified cache
    /// This is used by the dispatcher for threading
    pub async fn import_chunk_with_epoch_cache(
        &self,
        emails: &[(String, ParsedEmail, i32)],
        cache: &crate::threading::MailingListCache,
    ) -> Result<ImportStats, sqlx::Error> {
        let total = emails.len();
        log::info!("importing {} emails with epoch cache population", total);

        // Import using existing chunk logic
        let (stats, cache_data) = self.import_chunk(emails).await?;

        // Populate epoch cache with newly imported emails
        log::debug!("populating cache with {} emails", cache_data.emails.len());
        for (email_id, message_id, subject, in_reply_to, date, series_id, series_number, series_total) in cache_data.emails {
            cache.insert_email(
                message_id.clone(),
                EmailThreadingInfo {
                    email_id,
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

        // Populate references
        for (email_id, refs) in cache_data.references {
            cache.insert_references(email_id, refs);
        }

        log::debug!("cache population complete for {} emails", stats.emails);
        Ok(stats)
    }

    /// Update author activity stats
    pub async fn update_author_activity(&self) -> Result<(), sqlx::Error> {
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
