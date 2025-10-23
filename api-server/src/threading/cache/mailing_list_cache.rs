//! Unified cache for entire mailing list
//!
//! This is the current, active caching strategy that replaces per-epoch caching
//! with a single unified cache for the entire mailing list. This simplified
//! architecture provides better performance and easier maintenance.
//!
//! ## Design
//!
//! - Uses DashMap for thread-safe concurrent access during import
//! - Stores all emails and references for a mailing list in memory
//! - Provides snapshot capability for threading operations
//! - Supports disk persistence via bincode serialization

use super::{CacheError, EmailThreadingInfo, UnifiedCacheStats};
use crate::threading::container::EmailData;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Unified cache for an entire mailing list
///
/// Replaces per-epoch caching with a simpler, unified approach. All emails
/// for a mailing list are cached together, providing a complete view for
/// threading operations.
pub struct MailingListCache {
    /// ID of the mailing list this cache represents
    mailing_list_id: i32,

    /// Email data keyed by message_id
    /// DashMap allows concurrent reads/writes during population phase
    email_map: Arc<DashMap<String, EmailThreadingInfo>>,

    /// References for each email, keyed by email_id
    /// Maps email_id → Vec<referenced_message_ids> in order
    reference_map: Arc<DashMap<i32, Vec<String>>>,

    /// Cache format version for compatibility checking
    version: u32,
}

/// Serializable format for disk storage
///
/// We convert DashMap to HashMap for serialization since DashMap
/// doesn't implement Serialize directly
#[derive(Serialize, Deserialize)]
struct StoredMailingListCache {
    mailing_list_id: i32,
    emails: HashMap<String, EmailThreadingInfo>,
    references: HashMap<i32, Vec<String>>,
    version: u32,
}

impl MailingListCache {
    /// Current cache format version
    const CACHE_VERSION: u32 = 1;

    /// Create new empty cache for a mailing list
    pub fn new(mailing_list_id: i32) -> Self {
        Self {
            mailing_list_id,
            email_map: Arc::new(DashMap::new()),
            reference_map: Arc::new(DashMap::new()),
            version: Self::CACHE_VERSION,
        }
    }

    /// Load cache from disk (bincode serialized)
    ///
    /// Path format: `{cache_dir}/{mailing_list_id}_unified_v1.bin`
    ///
    /// ## Errors
    ///
    /// Returns `CacheError::NotFound` if cache file doesn't exist
    /// Returns `CacheError::VersionMismatch` if cache version is incompatible
    pub fn load_from_disk(mailing_list_id: i32, cache_dir: &Path) -> Result<Self, CacheError> {
        let cache_file_path = cache_dir.join(format!("{}_unified_v1.bin", mailing_list_id));

        if !cache_file_path.exists() {
            return Err(CacheError::NotFound);
        }

        // Read and deserialize cache file
        let serialized_data =
            std::fs::read(&cache_file_path).map_err(|e| CacheError::IoError(e.to_string()))?;

        let stored: StoredMailingListCache = bincode::deserialize(&serialized_data)
            .map_err(|e| CacheError::DeserializeError(e.to_string()))?;

        // Validate cache version compatibility
        if stored.version != Self::CACHE_VERSION {
            return Err(CacheError::VersionMismatch {
                expected: Self::CACHE_VERSION,
                found: stored.version,
            });
        }

        // Convert HashMap to DashMap for concurrent access
        let email_map = Arc::new(DashMap::new());
        for (message_id, email_info) in stored.emails {
            email_map.insert(message_id, email_info);
        }

        let reference_map = Arc::new(DashMap::new());
        for (email_id, refs) in stored.references {
            reference_map.insert(email_id, refs);
        }

        log::info!(
            "Loaded unified cache from disk: {} emails, {} reference entries",
            email_map.len(),
            reference_map.len()
        );

        Ok(Self {
            mailing_list_id,
            email_map,
            reference_map,
            version: stored.version,
        })
    }

    /// Load cache from database
    ///
    /// Loads all emails and references for this mailing list from the database.
    /// This is used when no disk cache exists or during full sync operations.
    pub async fn load_from_database(
        pool: &PgPool,
        mailing_list_id: i32,
    ) -> Result<Self, sqlx::Error> {
        use super::types::EmailThreadingInfoRow;

        log::info!(
            "Loading unified cache from database for mailing list {}",
            mailing_list_id
        );

        let cache = Self::new(mailing_list_id);

        // Load all emails for this mailing list
        let email_rows: Vec<EmailThreadingInfoRow> = sqlx::query_as(
            r#"SELECT id, message_id, subject, in_reply_to, date,
                      series_id, series_number, series_total
               FROM emails
               WHERE mailing_list_id = $1"#,
        )
        .bind(mailing_list_id)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} emails from database", email_rows.len());

        // Populate email map
        for row in email_rows {
            let info = EmailThreadingInfo::from(row);
            cache.email_map.insert(info.message_id.clone(), info);
        }

        // Load all references for this mailing list
        let reference_rows: Vec<(i32, String)> = sqlx::query_as(
            r#"SELECT email_id, referenced_message_id
               FROM email_references
               WHERE mailing_list_id = $1
               ORDER BY email_id, position"#,
        )
        .bind(mailing_list_id)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} reference entries", reference_rows.len());

        // Build reference map
        for (email_id, referenced_message_id) in reference_rows {
            cache
                .reference_map
                .entry(email_id)
                .or_insert_with(Vec::new)
                .push(referenced_message_id);
        }

        log::info!(
            "Unified cache loaded: {} emails, {} reference entries",
            cache.email_map.len(),
            cache.reference_map.len()
        );

        Ok(cache)
    }

    /// Save cache to disk
    ///
    /// Serializes the cache to disk using bincode format for fast loading.
    pub fn save_to_disk(&self, cache_dir: &Path) -> Result<(), CacheError> {
        let cache_file_path = cache_dir.join(format!("{}_unified_v1.bin", self.mailing_list_id));

        log::debug!(
            "Saving unified cache to disk: {}",
            cache_file_path.display()
        );

        // Convert DashMap to HashMap for serialization
        let stored = StoredMailingListCache {
            mailing_list_id: self.mailing_list_id,
            emails: self
                .email_map
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            references: self
                .reference_map
                .iter()
                .map(|entry| (*entry.key(), entry.value().clone()))
                .collect(),
            version: self.version,
        };

        // Ensure cache directory exists
        std::fs::create_dir_all(cache_dir).map_err(|e| CacheError::IoError(e.to_string()))?;

        // Serialize and write to disk
        let serialized_data =
            bincode::serialize(&stored).map_err(|e| CacheError::SerializeError(e.to_string()))?;

        std::fs::write(&cache_file_path, &serialized_data)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        log::info!(
            "Saved unified cache: {} emails, {} references ({} bytes)",
            self.email_map.len(),
            self.reference_map.len(),
            serialized_data.len()
        );

        Ok(())
    }

    /// Insert email during import phase (thread-safe)
    ///
    /// This can be called concurrently from multiple threads during the
    /// import phase thanks to DashMap's concurrent access support.
    pub fn insert_email(&self, message_id: String, email_info: EmailThreadingInfo) {
        self.email_map.insert(message_id, email_info);
    }

    /// Insert references during import phase (thread-safe)
    ///
    /// This can be called concurrently from multiple threads during the
    /// import phase thanks to DashMap's concurrent access support.
    pub fn insert_references(&self, email_id: i32, references: Vec<String>) {
        self.reference_map.insert(email_id, references);
    }

    /// Get all data for threading (creates snapshot)
    ///
    /// Creates a point-in-time snapshot of the cache data suitable for
    /// the JWZ threading algorithm. Returns owned data that can be moved
    /// to worker threads.
    ///
    /// ## Returns
    ///
    /// Tuple of (email_data_map, reference_map) where:
    /// - email_data_map: email_id → EmailData
    /// - reference_map: email_id → Vec<referenced_message_ids>
    pub fn get_all_for_threading(&self) -> (HashMap<i32, EmailData>, HashMap<i32, Vec<String>>) {
        // Convert cached email info to EmailData format needed by JWZ algorithm
        let email_data_map = self
            .email_map
            .iter()
            .map(|entry| {
                let info = entry.value();
                (
                    info.email_id,
                    EmailData {
                        id: info.email_id,
                        message_id: info.message_id.clone(),
                        subject: info.subject.clone(),
                        in_reply_to: info.in_reply_to.clone(),
                        date: info.date,
                        series_id: info.series_id.clone(),
                        series_number: info.series_number,
                        series_total: info.series_total,
                    },
                )
            })
            .collect();

        // Create owned copy of reference data
        let reference_map = self
            .reference_map
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        (email_data_map, reference_map)
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> UnifiedCacheStats {
        UnifiedCacheStats {
            email_count: self.email_map.len(),
            reference_count: self.reference_map.len(),
            size_estimate_mb: self.estimate_memory_usage_mb(),
        }
    }

    /// Estimate memory usage in megabytes
    ///
    /// Rough estimation: assumes ~1KB per email entry
    fn estimate_memory_usage_mb(&self) -> usize {
        (self.email_map.len() * 1024) / (1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_new_cache() {
        let cache = MailingListCache::new(1);
        let stats = cache.get_stats();

        assert_eq!(stats.email_count, 0);
        assert_eq!(stats.reference_count, 0);
    }

    #[test]
    fn test_insert_email() {
        let cache = MailingListCache::new(1);

        let email_info = EmailThreadingInfo {
            email_id: 100,
            message_id: "test@example.com".to_string(),
            subject: "Test Email".to_string(),
            in_reply_to: None,
            date: Utc::now(),
            series_id: None,
            series_number: None,
            series_total: None,
        };

        cache.insert_email("test@example.com".to_string(), email_info);

        let stats = cache.get_stats();
        assert_eq!(stats.email_count, 1);
    }

    #[test]
    fn test_insert_references() {
        let cache = MailingListCache::new(1);

        cache.insert_references(100, vec!["ref1@example.com".to_string()]);

        let stats = cache.get_stats();
        assert_eq!(stats.reference_count, 1);
    }
}
