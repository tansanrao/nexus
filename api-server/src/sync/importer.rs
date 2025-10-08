use crate::sync::parser::ParsedEmail;
use crate::threading::{build_threads, extract_patch_series_info, EmailData as ThreadEmailData};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{HashMap, HashSet};

const BATCH_SIZE: usize = 1000;

pub struct Importer {
    pool: PgPool,
    mailing_list_id: i32,
}

impl Importer {
    pub fn new(pool: PgPool, mailing_list_id: i32) -> Self {
        Self { pool, mailing_list_id }
    }

    /// Import all parsed emails into the database for a specific mailing list
    /// Uses multiple smaller transactions to reduce lock contention and allow incremental progress
    pub async fn import_emails(
        &self,
        emails_with_commits: Vec<(String, ParsedEmail)>,
    ) -> Result<ImportStats, sqlx::Error> {
        log::info!(
            "Starting database import for {} emails into mailing list {}",
            emails_with_commits.len(),
            self.mailing_list_id
        );

        // Phase 1: Authors (separate transaction)
        log::info!("Phase 1: Importing authors...");
        let author_cache = {
            let mut tx = self.pool.begin().await?;
            let cache = self.insert_authors(&mut tx, &emails_with_commits).await?;
            tx.commit().await?;
            log::info!("Committed {} unique authors", cache.len());
            cache
        };

        // Phase 2: Emails (separate transaction)
        log::info!("Phase 2: Importing emails...");
        let email_id_map = {
            let mut tx = self.pool.begin().await?;
            let map = self
                .insert_emails(&mut tx, &emails_with_commits, &author_cache)
                .await?;
            tx.commit().await?;
            log::info!("Committed {} emails", map.len());
            map
        };

        // Phase 3: Recipients and References (separate transaction, can be parallel in future)
        log::info!("Phase 3: Importing recipients and references...");
        let (recipient_count, reference_count) = {
            let mut tx = self.pool.begin().await?;

            log::info!("Importing email recipients...");
            let recipients = self
                .insert_recipients(&mut tx, &emails_with_commits, &author_cache, &email_id_map)
                .await?;

            log::info!("Importing email references...");
            let references = self
                .insert_references(&mut tx, &emails_with_commits, &email_id_map)
                .await?;

            tx.commit().await?;
            log::info!("Committed {} recipients and {} references", recipients, references);
            (recipients, references)
        };

        // Phase 4: Threads and Memberships (separate transaction)
        log::info!("Phase 4: Building email threads...");
        let (thread_count, membership_count) = {
            let mut tx = self.pool.begin().await?;
            let result = self.build_threads(&mut tx, &email_id_map).await?;
            tx.commit().await?;
            log::info!("Committed {} threads with {} memberships", result.0, result.1);
            result
        };

        // Phase 5: Author activity stats (separate transaction)
        log::info!("Phase 5: Updating author activity stats...");
        {
            let mut tx = self.pool.begin().await?;
            self.update_author_activity(&mut tx).await?;
            tx.commit().await?;
            log::info!("Committed author activity stats");
        }

        log::info!("Database import completed successfully");

        Ok(ImportStats {
            authors: author_cache.len(),
            emails: email_id_map.len(),
            recipients: recipient_count,
            references: reference_count,
            threads: thread_count,
            thread_memberships: membership_count,
        })
    }

    /// Insert unique authors globally and return a mapping of email -> author_id
    /// Also tracks name variations and per-list activity
    async fn insert_authors(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        emails: &[(String, ParsedEmail)],
    ) -> Result<HashMap<String, i32>, sqlx::Error> {
        let mut unique_authors: HashMap<String, Vec<String>> = HashMap::new();

        // Collect unique authors with all name variations
        for (_, email) in emails {
            unique_authors
                .entry(email.author_email.clone())
                .or_insert_with(Vec::new)
                .push(email.author_name.clone());

            // Also collect recipients
            for (name, addr) in &email.to_addrs {
                unique_authors
                    .entry(addr.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
            for (name, addr) in &email.cc_addrs {
                unique_authors
                    .entry(addr.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
        }

        // Batch insert authors globally (not partitioned)
        let chunks: Vec<_> = unique_authors.iter().collect::<Vec<_>>().chunks(BATCH_SIZE).map(|c| c.to_vec()).collect();
        for (idx, chunk) in chunks.iter().enumerate() {
            for (email, names) in chunk {
                // Pick the most common name or first non-empty name as canonical
                let canonical_name = names.iter()
                    .filter(|n| !n.is_empty())
                    .max_by_key(|n| names.iter().filter(|x| x == n).count())
                    .cloned()
                    .or_else(|| names.first().cloned())
                    .unwrap_or_default();

                // Upsert author globally
                sqlx::query(
                    r#"INSERT INTO authors (email, canonical_name, first_seen, last_seen)
                       VALUES ($1, $2, NOW(), NOW())
                       ON CONFLICT (email) DO UPDATE
                       SET last_seen = NOW(),
                           canonical_name = COALESCE(EXCLUDED.canonical_name, authors.canonical_name)"#
                )
                .bind(email.as_str())
                .bind(if canonical_name.is_empty() { None } else { Some(canonical_name.as_str()) })
                .execute(&mut **tx)
                .await?;
            }
            if (idx + 1) % 10 == 0 || idx == chunks.len() - 1 {
                log::debug!("Inserted author batch {}/{}", idx + 1, chunks.len());
            }
        }

        // Load all author IDs
        let author_rows: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, email FROM authors"
        )
            .fetch_all(&mut **tx)
            .await?;

        let mut cache = HashMap::new();
        for (id, email) in author_rows {
            cache.insert(email, id);
        }

        // Insert name aliases for each author using bulk UNNEST
        // Use a HashMap to deduplicate (author_id, name) pairs before inserting
        let mut alias_map: HashMap<(i32, String), usize> = HashMap::new();

        for (email, names) in unique_authors.iter() {
            if let Some(&author_id) = cache.get(email) {
                for name in names.iter().filter(|n| !n.is_empty()) {
                    let key = (author_id, name.clone());
                    *alias_map.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Bulk insert all unique aliases at once
        if !alias_map.is_empty() {
            let mut alias_author_ids = Vec::new();
            let mut alias_names = Vec::new();
            let mut usage_counts = Vec::new();

            for ((author_id, name), count) in alias_map.iter() {
                alias_author_ids.push(*author_id);
                alias_names.push(name.clone());
                usage_counts.push(*count as i32);
            }

            sqlx::query(
                r#"INSERT INTO author_name_aliases (author_id, name, usage_count, first_seen, last_seen)
                   SELECT * FROM UNNEST($1::int[], $2::text[], $3::int[]) AS t(author_id, name, usage_count)
                   CROSS JOIN (SELECT NOW() as first_seen, NOW() as last_seen) AS defaults
                   ON CONFLICT (author_id, name) DO UPDATE
                   SET usage_count = author_name_aliases.usage_count + EXCLUDED.usage_count,
                       last_seen = NOW()"#
            )
            .bind(&alias_author_ids)
            .bind(&alias_names)
            .bind(&usage_counts)
            .execute(&mut **tx)
            .await?;

            log::debug!("Bulk inserted {} unique author name aliases", alias_author_ids.len());
        }

        Ok(cache)
    }

    /// Insert emails and return a mapping of message_id -> email_id
    async fn insert_emails(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        emails: &[(String, ParsedEmail)],
        author_cache: &HashMap<String, i32>,
    ) -> Result<HashMap<String, i32>, sqlx::Error> {
        // Batch insert emails
        let total_chunks = (emails.len() + BATCH_SIZE - 1) / BATCH_SIZE;
        for (idx, chunk) in emails.chunks(BATCH_SIZE).enumerate() {
            for (commit_hash, email) in chunk {
                if let Some(&author_id) = author_cache.get(&email.author_email) {
                    // Extract patch series info if present
                    let series_info = extract_patch_series_info(&email.subject);
                    let (series_id, series_num, series_total) = series_info
                        .as_ref()
                        .map(|(id, num, total)| (Some(id.clone()), Some(*num), Some(*total)))
                        .unwrap_or((None, None, None));

                    sqlx::query(
                        r#"INSERT INTO emails
                           (mailing_list_id, message_id, git_commit_hash, author_id, subject, normalized_subject, date, in_reply_to, body, series_id, series_number, series_total)
                           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                           ON CONFLICT (mailing_list_id, message_id) DO NOTHING"#
                    )
                    .bind(self.mailing_list_id)
                    .bind(&email.message_id)
                    .bind(commit_hash)
                    .bind(author_id)
                    .bind(&email.subject)
                    .bind(&email.normalized_subject)
                    .bind(email.date)
                    .bind(&email.in_reply_to)
                    .bind(&email.body)
                    .bind(&series_id)
                    .bind(&series_num)
                    .bind(&series_total)
                    .execute(&mut **tx)
                    .await?;
                }
            }
            if (idx + 1) % 10 == 0 || idx == total_chunks - 1 {
                log::debug!("Inserted email batch {}/{}", idx + 1, total_chunks);
            }
        }

        // Load email IDs
        let rows: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, message_id FROM emails WHERE mailing_list_id = $1"
        )
            .bind(self.mailing_list_id)
            .fetch_all(&mut **tx)
            .await?;

        let mut map = HashMap::new();
        for (id, message_id) in rows {
            map.insert(message_id, id);
        }

        Ok(map)
    }

    /// Insert email recipients using bulk UNNEST
    async fn insert_recipients(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        emails: &[(String, ParsedEmail)],
        author_cache: &HashMap<String, i32>,
        email_id_map: &HashMap<String, i32>,
    ) -> Result<usize, sqlx::Error> {
        // Use a HashSet to deduplicate (email_id, author_id, recipient_type) tuples
        let mut recipient_set: HashSet<(i32, i32, String)> = HashSet::new();

        for (_, email) in emails {
            if let Some(&email_id) = email_id_map.get(&email.message_id) {
                // Collect To recipients
                for (_, addr) in &email.to_addrs {
                    if let Some(&author_id) = author_cache.get(addr) {
                        recipient_set.insert((email_id, author_id, "to".to_string()));
                    }
                }

                // Collect Cc recipients
                for (_, addr) in &email.cc_addrs {
                    if let Some(&author_id) = author_cache.get(addr) {
                        recipient_set.insert((email_id, author_id, "cc".to_string()));
                    }
                }
            }
        }

        let count = recipient_set.len();

        // Bulk insert all unique recipients at once
        if count > 0 {
            let mut mailing_list_ids = Vec::new();
            let mut recipient_email_ids = Vec::new();
            let mut recipient_author_ids = Vec::new();
            let mut recipient_types = Vec::new();

            for (email_id, author_id, recipient_type) in recipient_set {
                mailing_list_ids.push(self.mailing_list_id);
                recipient_email_ids.push(email_id);
                recipient_author_ids.push(author_id);
                recipient_types.push(recipient_type);
            }

            sqlx::query(
                r#"INSERT INTO email_recipients (mailing_list_id, email_id, author_id, recipient_type)
                   SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::text[])"#
            )
            .bind(&mailing_list_ids)
            .bind(&recipient_email_ids)
            .bind(&recipient_author_ids)
            .bind(&recipient_types)
            .execute(&mut **tx)
            .await?;

            log::debug!("Bulk inserted {} unique email recipients", count);
        }

        Ok(count)
    }

    /// Insert email references using bulk UNNEST
    async fn insert_references(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        emails: &[(String, ParsedEmail)],
        email_id_map: &HashMap<String, i32>,
    ) -> Result<usize, sqlx::Error> {
        // Use a HashMap to deduplicate (email_id, referenced_message_id) pairs
        // Keep the first occurrence's position
        let mut reference_map: HashMap<(i32, String), i32> = HashMap::new();

        for (_, email) in emails {
            if let Some(&email_id) = email_id_map.get(&email.message_id) {
                // Collect references with their position to preserve order
                for (position, ref_msg_id) in email.references.iter().enumerate() {
                    let key = (email_id, ref_msg_id.clone());
                    reference_map.entry(key).or_insert(position as i32);
                }
            }
        }

        let count = reference_map.len();

        // Bulk insert all unique references at once
        if count > 0 {
            let mut mailing_list_ids = Vec::new();
            let mut reference_email_ids = Vec::new();
            let mut referenced_message_ids = Vec::new();
            let mut positions = Vec::new();

            for ((email_id, ref_msg_id), position) in reference_map {
                mailing_list_ids.push(self.mailing_list_id);
                reference_email_ids.push(email_id);
                referenced_message_ids.push(ref_msg_id);
                positions.push(position);
            }

            sqlx::query(
                r#"INSERT INTO email_references (mailing_list_id, email_id, referenced_message_id, position)
                   SELECT * FROM UNNEST($1::int[], $2::int[], $3::text[], $4::int[])
                   ON CONFLICT (mailing_list_id, email_id, referenced_message_id) DO NOTHING"#
            )
            .bind(&mailing_list_ids)
            .bind(&reference_email_ids)
            .bind(&referenced_message_ids)
            .bind(&positions)
            .execute(&mut **tx)
            .await?;

            log::debug!("Bulk inserted {} unique email references", count);
        }

        Ok(count)
    }

    /// Build thread relationships using JWZ threading algorithm
    ///
    /// This uses our modular threading algorithm from the crate::threading module,
    /// which implements the exact JWZ algorithm from egol/mailing-list-parser.
    async fn build_threads(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        _email_id_map: &HashMap<String, i32>,
    ) -> Result<(usize, usize), sqlx::Error> {
        // Step 1: Load all emails with their metadata
        let email_rows: Vec<(
            i32,
            String,
            String,
            Option<String>,
            chrono::DateTime<chrono::Utc>,
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

        // Build email data map
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

        // Step 2: Load all references (ordered by position to preserve References chain)
        let ref_rows: Vec<(i32, String)> = sqlx::query_as(
            "SELECT email_id, referenced_message_id FROM email_references WHERE mailing_list_id = $1 ORDER BY email_id, position"
        )
        .bind(self.mailing_list_id)
        .fetch_all(&mut **tx)
        .await?;

        let mut email_references: HashMap<i32, Vec<String>> = HashMap::new();
        for (email_id, ref_msg_id) in ref_rows {
            email_references
                .entry(email_id)
                .or_insert_with(Vec::new)
                .push(ref_msg_id);
        }

        log::info!("Running JWZ threading algorithm on {} emails", email_data.len());

        // Step 3: Use our modular threading algorithm in a blocking task
        // This is CPU-intensive, so we move it off the async runtime to avoid blocking
        let threads_to_create = tokio::task::spawn_blocking(move || {
            build_threads(email_data, email_references)
        })
        .await
        .map_err(|e| sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Threading task panicked: {}", e)
        )))?;

        log::info!("Threading complete, created {} threads", threads_to_create.len());

        // Step 4: Insert threads and memberships using bulk operations
        let thread_count = threads_to_create.len();
        let mut membership_count = 0;

        for thread_info in threads_to_create {
            // Insert thread
            sqlx::query(
                r#"INSERT INTO threads (mailing_list_id, root_message_id, subject, start_date, last_date, message_count)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (mailing_list_id, root_message_id) DO NOTHING"#
            )
            .bind(self.mailing_list_id)
            .bind(&thread_info.root_message_id)
            .bind(&thread_info.subject)
            .bind(thread_info.start_date)
            .bind(thread_info.start_date)
            .bind(thread_info.emails.len() as i32)
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

            // Bulk insert memberships for this thread
            // Deduplicate in case the same email appears multiple times (shouldn't happen but be defensive)
            let mut membership_map: HashMap<i32, i32> = HashMap::new();
            for (email_id, depth) in thread_info.emails {
                membership_map.entry(email_id).or_insert(depth);
            }

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
                    )
                WHERE mailing_list_id = $1 AND id = $2"#
            )
            .bind(self.mailing_list_id)
            .bind(thread_id)
            .execute(&mut **tx)
            .await?;
        }

        Ok((thread_count, membership_count))
    }

    /// Update author_mailing_list_activity table with stats for this mailing list
    async fn update_author_activity(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<(), sqlx::Error> {
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ImportStats {
    pub authors: usize,
    pub emails: usize,
    pub recipients: usize,
    pub references: usize,
    pub threads: usize,
    pub thread_memberships: usize,
}
