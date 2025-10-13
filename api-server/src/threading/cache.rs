use std::collections::{HashMap, HashSet};
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Fast in-memory cache for threading operations
/// Stores minimal data needed for JWZ algorithm
/// Covers a 2-epoch window for memory efficiency
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThreadingCache {
    /// Map: message_id -> EmailThreadingInfo
    emails: HashMap<String, EmailThreadingInfo>,
    /// Map: email_id -> Vec<referenced_message_ids>
    references: HashMap<i32, Vec<String>>,
    /// Which epochs this cache covers
    epoch_range: (i32, i32),  // (min_epoch, max_epoch)
    /// Mailing list this cache belongs to
    mailing_list_id: i32,
    /// Cache version (for invalidation)
    version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailThreadingInfo {
    pub email_id: i32,
    pub message_id: String,
    pub subject: String,
    pub in_reply_to: Option<String>,
    pub date: DateTime<Utc>,
    pub series_id: Option<String>,
    pub series_number: Option<i32>,
    pub series_total: Option<i32>,
}

impl ThreadingCache {
    /// Current cache version
    const CACHE_VERSION: u32 = 1;

    /// Create a new empty cache
    pub fn new(mailing_list_id: i32, epoch_range: (i32, i32)) -> Self {
        Self {
            emails: HashMap::new(),
            references: HashMap::new(),
            mailing_list_id,
            epoch_range,
            version: Self::CACHE_VERSION,
        }
    }

    /// Load cache from database for all emails in mailing list
    pub async fn load_from_db(
        pool: &PgPool,
        mailing_list_id: i32,
        epoch_range: (i32, i32),
    ) -> Result<Self, sqlx::Error> {
        log::info!(
            "Loading threading cache from database for list {} (epochs {}-{})",
            mailing_list_id, epoch_range.0, epoch_range.1
        );

        let mut cache = Self::new(mailing_list_id, epoch_range);

        // Load email threading info for the specified epoch range
        let emails = sqlx::query_as::<_, EmailThreadingInfoRow>(
            r#"
            SELECT
                e.id as email_id,
                e.message_id,
                e.subject,
                e.in_reply_to,
                e.date,
                e.series_id,
                e.series_number,
                e.series_total
            FROM emails e
            WHERE e.mailing_list_id = $1
              AND e.epoch >= $2
              AND e.epoch <= $3
            "#
        )
        .bind(mailing_list_id)
        .bind(epoch_range.0)
        .bind(epoch_range.1)
        .fetch_all(pool)
        .await?;

        log::info!("Loaded {} emails from database", emails.len());

        for row in emails {
            let info = EmailThreadingInfo {
                email_id: row.email_id,
                message_id: row.message_id.clone(),
                subject: row.subject,
                in_reply_to: row.in_reply_to,
                date: row.date,
                series_id: row.series_id,
                series_number: row.series_number,
                series_total: row.series_total,
            };
            cache.emails.insert(row.message_id, info);
        }

        // Load references
        let references = sqlx::query_as::<_, EmailReferenceRow>(
            r#"
            SELECT
                er.email_id,
                er.referenced_message_id as reference_message_id
            FROM email_references er
            WHERE er.mailing_list_id = $1
            ORDER BY er.email_id, er.position
            "#
        )
        .bind(mailing_list_id)
        .fetch_all(pool)
        .await?;

        log::info!("Loaded {} references from database", references.len());

        for row in references {
            cache.references
                .entry(row.email_id)
                .or_insert_with(Vec::new)
                .push(row.reference_message_id);
        }

        Ok(cache)
    }

    /// Load cache from disk (bincode format)
    pub fn load_from_disk(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("Loading threading cache from disk: {}", path.display());
        let data = std::fs::read(path)?;
        let cache: Self = bincode::deserialize(&data)?;

        // Validate version
        if cache.version != Self::CACHE_VERSION {
            return Err(format!(
                "Cache version mismatch: expected {}, got {}",
                Self::CACHE_VERSION,
                cache.version
            ).into());
        }

        log::info!(
            "Loaded cache: {} emails, {} reference entries, epochs {}-{}",
            cache.emails.len(),
            cache.references.len(),
            cache.epoch_range.0,
            cache.epoch_range.1
        );

        Ok(cache)
    }

    /// Save cache to disk
    pub fn save_to_disk(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Saving threading cache to disk: {}", path.display());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = bincode::serialize(self)?;
        let data_len = data.len();
        std::fs::write(path, data)?;

        log::info!(
            "Saved cache: {} emails, {} reference entries ({} bytes)",
            self.emails.len(),
            self.references.len(),
            data_len
        );

        Ok(())
    }

    /// Merge newly parsed emails into cache
    pub fn merge_new_emails(&mut self, new_emails: Vec<(i32, String, String, Option<String>, DateTime<Utc>, Option<String>, Option<i32>, Option<i32>)>) {
        for (email_id, message_id, subject, in_reply_to, date, series_id, series_number, series_total) in new_emails {
            let info = EmailThreadingInfo {
                email_id,
                message_id: message_id.clone(),
                subject,
                in_reply_to,
                date,
                series_id,
                series_number,
                series_total,
            };
            self.emails.insert(message_id, info);
        }
    }

    /// Merge references for newly parsed emails
    pub fn merge_new_references(&mut self, new_references: Vec<(i32, Vec<String>)>) {
        for (email_id, refs) in new_references {
            self.references.insert(email_id, refs);
        }
    }

    /// Get email info for threading (O(1) lookup)
    pub fn get_email_info(&self, message_id: &str) -> Option<&EmailThreadingInfo> {
        self.emails.get(message_id)
    }

    /// Get references for an email
    pub fn get_references(&self, email_id: i32) -> Option<&Vec<String>> {
        self.references.get(&email_id)
    }

    /// Check if cache covers the required epochs
    pub fn covers_epochs(&self, required: (i32, i32)) -> bool {
        self.epoch_range.0 <= required.0 && self.epoch_range.1 >= required.1
    }

    /// Update epoch range and prune old data
    pub fn update_epoch_range(&mut self, new_range: (i32, i32)) {
        if self.epoch_range == new_range {
            return;
        }

        log::info!(
            "Updating cache epoch range from {}-{} to {}-{}",
            self.epoch_range.0,
            self.epoch_range.1,
            new_range.0,
            new_range.1
        );

        self.epoch_range = new_range;

        // Note: We don't prune data here because we don't store epoch info per-email
        // in the cache. The cache will be fully rebuilt when needed.
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            email_count: self.emails.len(),
            reference_count: self.references.len(),
            epoch_range: self.epoch_range,
            mailing_list_id: self.mailing_list_id,
            version: self.version,
        }
    }

    /// Get all emails from cache as a HashMap for threading
    /// Returns map of email_id -> (message_id, subject, in_reply_to, date, series_id, series_number, series_total)
    pub fn get_all_email_data(&self) -> HashMap<i32, (String, String, Option<String>, DateTime<Utc>, Option<String>, Option<i32>, Option<i32>)> {
        self.emails
            .values()
            .map(|info| {
                (
                    info.email_id,
                    (
                        info.message_id.clone(),
                        info.subject.clone(),
                        info.in_reply_to.clone(),
                        info.date,
                        info.series_id.clone(),
                        info.series_number,
                        info.series_total,
                    )
                )
            })
            .collect()
    }

    /// Get all references from cache
    /// Returns map of email_id -> Vec<referenced_message_ids>
    pub fn get_all_references(&self) -> HashMap<i32, Vec<String>> {
        self.references.clone()
    }

    /// Check if the cache contains data for a specific email ID
    pub fn contains_email_id(&self, email_id: i32) -> bool {
        self.emails.values().any(|info| info.email_id == email_id)
    }

    /// Get the set of email IDs in the cache
    pub fn get_email_ids(&self) -> HashSet<i32> {
        self.emails.values().map(|info| info.email_id).collect()
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub email_count: usize,
    pub reference_count: usize,
    pub epoch_range: (i32, i32),
    pub mailing_list_id: i32,
    pub version: u32,
}

// Database row types
#[derive(sqlx::FromRow)]
struct EmailThreadingInfoRow {
    email_id: i32,
    message_id: String,
    subject: String,
    in_reply_to: Option<String>,
    date: DateTime<Utc>,
    series_id: Option<String>,
    series_number: Option<i32>,
    series_total: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct EmailReferenceRow {
    email_id: i32,
    reference_message_id: String,
}
