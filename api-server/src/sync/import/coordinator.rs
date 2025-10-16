//! Import coordination for bulk email operations.
//!
//! The BulkImporter coordinates the entire import pipeline:
//! 1. Extract unique authors
//! 2. Insert authors
//! 3. Prepare and insert emails
//! 4. Insert recipients and references in parallel
//! 5. Populate threading cache

use crate::sync::import::{
    data_builder, data_structures::ChunkCacheData, database_operations, stats::ImportStats,
};
use crate::sync::parser::ParsedEmail;
use crate::threading::{EmailThreadingInfo, MailingListCache};
use rocket_db_pools::sqlx::PgPool;
use std::collections::HashMap;

/// Chunk size for streaming imports to avoid overwhelming database connections
const EMAIL_IMPORT_BATCH_SIZE: usize = 25_000;

/// Coordinates bulk import operations for email data.
///
/// Handles the entire import pipeline from parsed emails to database records
/// and threading cache population.
pub struct BulkImporter {
    pool: PgPool,
    mailing_list_id: i32,
}

impl BulkImporter {
    /// Create a new BulkImporter for a specific mailing list.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `mailing_list_id` - ID of the mailing list being imported
    pub fn new(pool: PgPool, mailing_list_id: i32) -> Self {
        Self {
            pool,
            mailing_list_id,
        }
    }

    /// Import a single chunk of emails with enhanced parallel database operations.
    ///
    /// # Process
    /// 1. Extract and insert authors
    /// 2. Prepare and insert emails
    /// 3. Load email and recipient author IDs in parallel
    /// 4. Insert recipients and references in parallel
    /// 5. Extract cache data for threading
    ///
    /// # Optimizations
    /// - Uses up to 6 parallel connections from the bulk_write_db pool
    /// - Parallelizes data loading operations where possible
    ///
    /// # Arguments
    /// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
    ///
    /// # Returns
    /// Tuple of (ImportStats, ChunkCacheData) for cache merging
    async fn import_chunk(
        &self,
        chunk: &[(String, ParsedEmail, i32)],
    ) -> Result<(ImportStats, ChunkCacheData), sqlx::Error> {
        // Phase 1: Prepare and insert authors
        let authors_data = data_builder::extract_unique_authors_from_chunk(chunk);
        let prepared_author_count = authors_data.len();

        let mut author_conn = self.pool.acquire().await?;
        let author_count =
            database_operations::insert_authors_batch(&mut author_conn, authors_data).await?;
        drop(author_conn); // Release connection

        log::trace!(
            "chunk: prepared {} authors, bulk_insert returned {}",
            prepared_author_count,
            author_count
        );

        // Phase 2: Prepare and insert emails
        let emails_data =
            data_builder::build_email_batch_data(&self.pool, self.mailing_list_id, chunk).await?;

        log::trace!(
            "chunk: prepared {} emails for insertion (chunk size: {})",
            emails_data.message_ids.len(),
            chunk.len()
        );

        let mut email_conn = self.pool.acquire().await?;
        let email_count = database_operations::insert_emails_batch(
            &mut email_conn,
            self.mailing_list_id,
            &emails_data,
        )
        .await?;
        drop(email_conn); // Release connection

        // Phase 3: Load email IDs and recipient author IDs in parallel
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
                    "SELECT message_id, id FROM emails WHERE mailing_list_id = $1 AND message_id = ANY($2)",
                )
                .bind(self.mailing_list_id)
                .bind(&message_ids)
                .fetch_all(&self.pool)
                .await
            },
            async {
                if !recipient_emails_vec.is_empty() {
                    sqlx::query_as::<_, (String, i32)>(
                        "SELECT email, id FROM authors WHERE email = ANY($1)",
                    )
                    .bind(&recipient_emails_vec)
                    .fetch_all(&self.pool)
                    .await
                } else {
                    Ok(Vec::new())
                }
            }
        )?;

        let email_id_map: HashMap<String, i32> = email_id_rows.into_iter().collect();
        let recipient_author_map: HashMap<String, i32> =
            recipient_author_rows.into_iter().collect();

        // Phase 4: Prepare and insert recipients and references in parallel
        let recipients_data = data_builder::build_recipient_batch_data(
            self.mailing_list_id,
            chunk,
            &email_id_map,
            &recipient_author_map,
        );
        let references_data =
            data_builder::build_reference_batch_data(self.mailing_list_id, chunk, &email_id_map);

        let mut recipient_conn = self.pool.acquire().await?;
        let mut reference_conn = self.pool.acquire().await?;

        // Clone references_data before moving it to insert_references_batch
        let references_data_clone = references_data.clone();

        let (recipient_count, reference_count) = tokio::try_join!(
            database_operations::insert_recipients_batch(&mut recipient_conn, recipients_data),
            database_operations::insert_references_batch(
                &mut reference_conn,
                references_data_clone
            ),
        )?;

        // Phase 5: Extract cache data
        let cache_data = data_builder::extract_cache_data_from_chunk(chunk, &email_id_map);

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

    /// Import emails in chunks and populate the threading cache.
    ///
    /// This is the main entry point used by the dispatcher. Processes emails
    /// in chunks to avoid connection timeouts and memory issues.
    ///
    /// # Arguments
    /// * `emails` - Slice of (commit_hash, parsed_email, epoch) tuples
    /// * `cache` - Threading cache to populate with imported emails
    ///
    /// # Returns
    /// ImportStats with counts of all inserted records
    pub async fn import_chunk_with_epoch_cache(
        &self,
        emails: &[(String, ParsedEmail, i32)],
        cache: &MailingListCache,
    ) -> Result<ImportStats, sqlx::Error> {
        let total = emails.len();
        log::info!("importing {} emails with epoch cache population", total);

        // Import using existing chunk logic
        let (stats, cache_data) = self.import_chunk(emails).await?;

        // Populate epoch cache with newly imported emails
        log::debug!("populating cache with {} emails", cache_data.emails.len());
        for (
            email_id,
            message_id,
            subject,
            in_reply_to,
            date,
            series_id,
            series_number,
            series_total,
        ) in cache_data.emails
        {
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

    /// Update author activity statistics for the mailing list.
    ///
    /// Calculates aggregate statistics per author including email count,
    /// thread participation, and first/last email dates.
    ///
    /// # Returns
    /// Ok(()) if update succeeds, database error otherwise
    pub async fn update_author_activity(&self) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

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
                   thread_count = EXCLUDED.thread_count"#,
        )
        .bind(self.mailing_list_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}
