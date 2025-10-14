//! Caching system for threading operations
//!
//! This module implements caching for email threading data:
//!
//! - **MailingListCache**: Unified thread-safe cache for entire mailing list using DashMap
//! - **EpochCache**: Legacy per-epoch cache (deprecated, kept for compatibility)
//! - **EpochCacheManager**: Legacy manager (deprecated, kept for compatibility)
//!
//! The unified cache supports concurrent reads/writes during import and provides
//! snapshots for the threading phase.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::EmailData;

/// Email threading information stored in cache
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

/// Cache for a single epoch
/// Uses DashMap for thread-safe concurrent access during import
#[derive(Clone)]
pub struct EpochCache {
    epoch: i32,
    mailing_list_id: i32,

    // DashMap allows concurrent reads/writes during population
    emails: Arc<DashMap<String, EmailThreadingInfo>>,
    references: Arc<DashMap<i32, Vec<String>>>,

    // Metadata
    version: u32,
}

/// Serializable version for disk storage
/// We convert DashMap to HashMap for serialization
#[derive(Serialize, Deserialize)]
struct StoredEpochCache {
    epoch: i32,
    mailing_list_id: i32,
    emails: HashMap<String, EmailThreadingInfo>,
    references: HashMap<i32, Vec<String>>,
    version: u32,
}

impl EpochCache {
    /// Current cache version
    const CACHE_VERSION: u32 = 1;

    /// Create new empty cache for an epoch
    pub fn new(mailing_list_id: i32, epoch: i32) -> Self {
        Self {
            epoch,
            mailing_list_id,
            emails: Arc::new(DashMap::new()),
            references: Arc::new(DashMap::new()),
            version: Self::CACHE_VERSION,
        }
    }

    /// Load cache from disk (bincode serialized)
    /// Path: cache/{mailing_list_id}_epoch_{epoch}_v1.bin
    pub fn load_from_disk(
        mailing_list_id: i32,
        epoch: i32,
        cache_dir: &Path
    ) -> Result<Self, CacheError> {
        let path = cache_dir.join(format!("{}_epoch_{}_v1.bin", mailing_list_id, epoch));

        if !path.exists() {
            return Err(CacheError::NotFound);
        }

        let data = std::fs::read(&path)
            .map_err(|e| CacheError::IoError(e.to_string()))?;
        let stored: StoredEpochCache = bincode::deserialize(&data)
            .map_err(|e| CacheError::DeserializeError(e.to_string()))?;

        // Validate version
        if stored.version != Self::CACHE_VERSION {
            return Err(CacheError::VersionMismatch {
                expected: Self::CACHE_VERSION,
                found: stored.version,
            });
        }

        // Convert HashMap to DashMap
        let emails = Arc::new(DashMap::new());
        for (k, v) in stored.emails {
            emails.insert(k, v);
        }

        let references = Arc::new(DashMap::new());
        for (k, v) in stored.references {
            references.insert(k, v);
        }

        log::info!("Loaded epoch {} cache from disk: {} emails, {} references",
            epoch, emails.len(), references.len());

        Ok(Self {
            epoch,
            mailing_list_id,
            emails,
            references,
            version: stored.version,
        })
    }

    /// Load cache from database for specific epoch
    pub async fn load_from_db(
        pool: &PgPool,
        mailing_list_id: i32,
        epoch: i32,
    ) -> Result<Self, sqlx::Error> {
        log::info!("Loading epoch {} cache from database", epoch);

        let cache = Self::new(mailing_list_id, epoch);

        // Load emails for this epoch only
        let email_rows: Vec<EmailThreadingInfoRow> = sqlx::query_as(
            r#"SELECT id, message_id, subject, in_reply_to, date,
                      series_id, series_number, series_total
               FROM emails
               WHERE mailing_list_id = $1 AND epoch = $2"#
        )
        .bind(mailing_list_id)
        .bind(epoch)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} emails for epoch {} from database", email_rows.len(), epoch);

        for row in email_rows {
            let info = EmailThreadingInfo {
                email_id: row.id,
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
        let ref_rows: Vec<(i32, String)> = sqlx::query_as(
            r#"SELECT er.email_id, er.referenced_message_id
               FROM email_references er
               JOIN emails e ON er.email_id = e.id
               WHERE er.mailing_list_id = $1 AND e.epoch = $2
               ORDER BY er.email_id, er.position"#
        )
        .bind(mailing_list_id)
        .bind(epoch)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} references for epoch {}", ref_rows.len(), epoch);

        for (email_id, ref_msg_id) in ref_rows {
            cache.references.entry(email_id)
                .or_insert_with(Vec::new)
                .push(ref_msg_id);
        }

        Ok(cache)
    }

    /// Save cache to disk
    pub fn save_to_disk(&self, cache_dir: &Path) -> Result<(), CacheError> {
        let path = cache_dir.join(format!("{}_epoch_{}_v1.bin",
            self.mailing_list_id, self.epoch));

        log::debug!("Saving epoch {} cache to disk: {}", self.epoch, path.display());

        // Convert DashMap to HashMap for serialization
        let stored = StoredEpochCache {
            epoch: self.epoch,
            mailing_list_id: self.mailing_list_id,
            emails: self.emails.iter().map(|e| (e.key().clone(), e.value().clone())).collect(),
            references: self.references.iter().map(|r| (*r.key(), r.value().clone())).collect(),
            version: self.version,
        };

        std::fs::create_dir_all(cache_dir)
            .map_err(|e| CacheError::IoError(e.to_string()))?;
        let data = bincode::serialize(&stored)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;
        std::fs::write(&path, &data)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        log::info!("Saved epoch {} cache: {} emails, {} references ({} bytes)",
            self.epoch, self.emails.len(), self.references.len(), data.len());

        Ok(())
    }

    /// Insert email during import phase (thread-safe)
    pub fn insert_email(&self, message_id: String, info: EmailThreadingInfo) {
        self.emails.insert(message_id, info);
    }

    /// Insert references during import phase (thread-safe)
    pub fn insert_references(&self, email_id: i32, refs: Vec<String>) {
        self.references.insert(email_id, refs);
    }

    /// Get all data for threading (creates snapshot)
    /// Returns (email_data, references) suitable for JWZ algorithm
    pub fn get_all_for_threading(&self) -> (HashMap<i32, EmailData>, HashMap<i32, Vec<String>>) {
        let emails = self.emails.iter()
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
                    }
                )
            })
            .collect();

        let references = self.references.iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        (emails, references)
    }

    /// Get statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            epoch: self.epoch,
            email_count: self.emails.len(),
            reference_count: self.references.len(),
            size_estimate_mb: self.estimate_size_mb(),
        }
    }

    fn estimate_size_mb(&self) -> usize {
        // Rough estimate: ~1KB per email entry
        (self.emails.len() * 1024) / (1024 * 1024)
    }
}

/// Manages all per-epoch caches for a mailing list
pub struct EpochCacheManager {
    mailing_list_id: i32,
    cache_dir: PathBuf,

    // Concurrent access to epoch caches
    caches: Arc<DashMap<i32, Arc<EpochCache>>>,
}

impl EpochCacheManager {
    /// Initialize cache manager for a mailing list
    pub fn new(mailing_list_id: i32, cache_dir: PathBuf) -> Self {
        Self {
            mailing_list_id,
            cache_dir,
            caches: Arc::new(DashMap::new()),
        }
    }

    /// Initialize caches for all epochs
    /// For full sync: Create empty caches
    /// For incremental: Load from disk or DB
    pub async fn initialize(
        &self,
        pool: &PgPool,
        epochs: &[i32],
        is_full_sync: bool,
    ) -> Result<(), CacheError> {
        log::info!("Initializing caches for {} epochs (full_sync: {})",
            epochs.len(), is_full_sync);

        for &epoch in epochs {
            let cache = if is_full_sync {
                // Full sync: Start with empty cache
                log::debug!("Creating empty cache for epoch {}", epoch);
                Arc::new(EpochCache::new(self.mailing_list_id, epoch))
            } else {
                // Incremental: Try disk first, then DB
                log::debug!("Loading cache for epoch {} (disk → DB fallback)", epoch);

                match EpochCache::load_from_disk(self.mailing_list_id, epoch, &self.cache_dir) {
                    Ok(cache) => {
                        log::debug!("Loaded epoch {} cache from disk", epoch);
                        Arc::new(cache)
                    }
                    Err(CacheError::NotFound) => {
                        log::debug!("Disk cache not found for epoch {}, loading from DB", epoch);
                        Arc::new(EpochCache::load_from_db(pool, self.mailing_list_id, epoch)
                            .await
                            .map_err(|e| CacheError::DatabaseError(e.to_string()))?)
                    }
                    Err(e) => return Err(e),
                }
            };

            self.caches.insert(epoch, cache);
        }

        log::info!("Cache initialization complete: {} caches loaded", self.caches.len());
        Ok(())
    }

    /// Get cache for a specific epoch (for reading/writing during import)
    pub fn get_cache(&self, epoch: i32) -> Option<Arc<EpochCache>> {
        self.caches.get(&epoch).map(|entry| entry.value().clone())
    }

    /// Save all caches to disk after sync completes
    pub async fn save_all(&self) -> Result<(), CacheError> {
        log::info!("Saving {} caches to disk", self.caches.len());

        for entry in self.caches.iter() {
            let epoch = *entry.key();
            let cache = entry.value();

            log::debug!("Saving cache for epoch {}", epoch);
            cache.save_to_disk(&self.cache_dir)?;
        }

        log::info!("All caches saved successfully");
        Ok(())
    }

    /// Get total memory usage estimate
    pub fn total_memory_mb(&self) -> usize {
        self.caches.iter()
            .map(|entry| entry.value().estimate_size_mb())
            .sum()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub epoch: i32,
    pub email_count: usize,
    pub reference_count: usize,
    pub size_estimate_mb: usize,
}

/// Unified cache for entire mailing list
/// Replaces per-epoch caching with a single unified cache for simplified architecture
pub struct MailingListCache {
    mailing_list_id: i32,

    // DashMap allows concurrent reads/writes during population
    emails: Arc<DashMap<String, EmailThreadingInfo>>,
    references: Arc<DashMap<i32, Vec<String>>>,

    // Metadata
    version: u32,
}

/// Serializable version for disk storage (unified cache)
#[derive(Serialize, Deserialize)]
struct StoredMailingListCache {
    mailing_list_id: i32,
    emails: HashMap<String, EmailThreadingInfo>,
    references: HashMap<i32, Vec<String>>,
    version: u32,
}

impl MailingListCache {
    /// Current cache version
    const CACHE_VERSION: u32 = 1;

    /// Create new empty cache
    pub fn new(mailing_list_id: i32) -> Self {
        Self {
            mailing_list_id,
            emails: Arc::new(DashMap::new()),
            references: Arc::new(DashMap::new()),
            version: Self::CACHE_VERSION,
        }
    }

    /// Load cache from disk (bincode serialized)
    /// Path: cache/{mailing_list_id}_unified_v1.bin
    pub fn load_from_disk(
        mailing_list_id: i32,
        cache_dir: &Path,
    ) -> Result<Self, CacheError> {
        let path = cache_dir.join(format!("{}_unified_v1.bin", mailing_list_id));

        if !path.exists() {
            return Err(CacheError::NotFound);
        }

        let data = std::fs::read(&path)
            .map_err(|e| CacheError::IoError(e.to_string()))?;
        let stored: StoredMailingListCache = bincode::deserialize(&data)
            .map_err(|e| CacheError::DeserializeError(e.to_string()))?;

        // Validate version
        if stored.version != Self::CACHE_VERSION {
            return Err(CacheError::VersionMismatch {
                expected: Self::CACHE_VERSION,
                found: stored.version,
            });
        }

        // Convert HashMap to DashMap
        let emails = Arc::new(DashMap::new());
        for (k, v) in stored.emails {
            emails.insert(k, v);
        }

        let references = Arc::new(DashMap::new());
        for (k, v) in stored.references {
            references.insert(k, v);
        }

        log::info!("Loaded unified cache from disk: {} emails, {} references",
            emails.len(), references.len());

        Ok(Self {
            mailing_list_id,
            emails,
            references,
            version: stored.version,
        })
    }

    /// Load cache from database (all emails for mailing list)
    pub async fn load_from_db(
        pool: &PgPool,
        mailing_list_id: i32,
    ) -> Result<Self, sqlx::Error> {
        log::info!("Loading unified cache from database for mailing list {}", mailing_list_id);

        let cache = Self::new(mailing_list_id);

        // Load all emails for this mailing list
        let email_rows: Vec<EmailThreadingInfoRow> = sqlx::query_as(
            r#"SELECT id, message_id, subject, in_reply_to, date,
                      series_id, series_number, series_total
               FROM emails
               WHERE mailing_list_id = $1"#
        )
        .bind(mailing_list_id)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} emails from database", email_rows.len());

        for row in email_rows {
            let info = EmailThreadingInfo {
                email_id: row.id,
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

        // Load all references
        let ref_rows: Vec<(i32, String)> = sqlx::query_as(
            r#"SELECT email_id, referenced_message_id
               FROM email_references
               WHERE mailing_list_id = $1
               ORDER BY email_id, position"#
        )
        .bind(mailing_list_id)
        .fetch_all(pool)
        .await?;

        log::debug!("Loaded {} references", ref_rows.len());

        for (email_id, ref_msg_id) in ref_rows {
            cache.references.entry(email_id)
                .or_insert_with(Vec::new)
                .push(ref_msg_id);
        }

        log::info!("Unified cache loaded: {} emails, {} references",
            cache.emails.len(), cache.references.len());

        Ok(cache)
    }

    /// Save cache to disk
    pub fn save_to_disk(&self, cache_dir: &Path) -> Result<(), CacheError> {
        let path = cache_dir.join(format!("{}_unified_v1.bin", self.mailing_list_id));

        log::debug!("Saving unified cache to disk: {}", path.display());

        // Convert DashMap to HashMap for serialization
        let stored = StoredMailingListCache {
            mailing_list_id: self.mailing_list_id,
            emails: self.emails.iter().map(|e| (e.key().clone(), e.value().clone())).collect(),
            references: self.references.iter().map(|r| (*r.key(), r.value().clone())).collect(),
            version: self.version,
        };

        std::fs::create_dir_all(cache_dir)
            .map_err(|e| CacheError::IoError(e.to_string()))?;
        let data = bincode::serialize(&stored)
            .map_err(|e| CacheError::SerializeError(e.to_string()))?;
        std::fs::write(&path, &data)
            .map_err(|e| CacheError::IoError(e.to_string()))?;

        log::info!("Saved unified cache: {} emails, {} references ({} bytes)",
            self.emails.len(), self.references.len(), data.len());

        Ok(())
    }

    /// Insert email during import phase (thread-safe)
    pub fn insert_email(&self, message_id: String, info: EmailThreadingInfo) {
        self.emails.insert(message_id, info);
    }

    /// Insert references during import phase (thread-safe)
    pub fn insert_references(&self, email_id: i32, refs: Vec<String>) {
        self.references.insert(email_id, refs);
    }

    /// Get all data for threading (creates snapshot)
    /// Returns (email_data, references) suitable for JWZ algorithm
    pub fn get_all_for_threading(&self) -> (HashMap<i32, EmailData>, HashMap<i32, Vec<String>>) {
        let emails = self.emails.iter()
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
                    }
                )
            })
            .collect();

        let references = self.references.iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        (emails, references)
    }

    /// Get statistics
    pub fn stats(&self) -> UnifiedCacheStats {
        UnifiedCacheStats {
            email_count: self.emails.len(),
            reference_count: self.references.len(),
            size_estimate_mb: self.estimate_size_mb(),
        }
    }

    fn estimate_size_mb(&self) -> usize {
        // Rough estimate: ~1KB per email entry
        (self.emails.len() * 1024) / (1024 * 1024)
    }
}

/// Cache statistics for unified cache
#[derive(Debug, Clone)]
pub struct UnifiedCacheStats {
    pub email_count: usize,
    pub reference_count: usize,
    pub size_estimate_mb: usize,
}

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache not found")]
    NotFound,

    #[error("Epoch {0} not found in cache manager")]
    EpochNotFound(i32),

    #[error("Cache version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        expected: u32,
        found: u32,
    },

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializeError(String),

    #[error("Deserialization error: {0}")]
    DeserializeError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Database row types for loading
#[derive(sqlx::FromRow)]
struct EmailThreadingInfoRow {
    id: i32,
    message_id: String,
    subject: String,
    in_reply_to: Option<String>,
    date: DateTime<Utc>,
    series_id: Option<String>,
    series_number: Option<i32>,
    series_total: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_cache_new() {
        let cache = EpochCache::new(1, 5);
        let stats = cache.stats();

        assert_eq!(stats.epoch, 5);
        assert_eq!(stats.email_count, 0);
        assert_eq!(stats.reference_count, 0);
    }

    #[test]
    fn test_epoch_cache_insert() {
        let cache = EpochCache::new(1, 5);

        let info = EmailThreadingInfo {
            email_id: 100,
            message_id: "test@example.com".to_string(),
            subject: "Test".to_string(),
            in_reply_to: None,
            date: Utc::now(),
            series_id: None,
            series_number: None,
            series_total: None,
        };

        cache.insert_email("test@example.com".to_string(), info);
        cache.insert_references(100, vec!["ref1@example.com".to_string()]);

        let stats = cache.stats();
        assert_eq!(stats.email_count, 1);
        assert_eq!(stats.reference_count, 1);
    }

    #[test]
    fn test_unified_cache() {
        let cache = MailingListCache::new(1);

        let info = EmailThreadingInfo {
            email_id: 200,
            message_id: "unified@example.com".to_string(),
            subject: "Unified Test".to_string(),
            in_reply_to: None,
            date: Utc::now(),
            series_id: None,
            series_number: None,
            series_total: None,
        };

        cache.insert_email("unified@example.com".to_string(), info);
        cache.insert_references(200, vec!["ref2@example.com".to_string()]);

        let stats = cache.stats();
        assert_eq!(stats.email_count, 1);
        assert_eq!(stats.reference_count, 1);
    }
}
