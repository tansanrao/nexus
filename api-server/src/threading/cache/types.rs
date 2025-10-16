//! Shared types and data structures for threading cache implementations
//!
//! This module contains common types used across all cache implementations:
//! - Email threading information for caching
//! - Cache statistics
//! - Error types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Email threading information stored in cache
///
/// Contains the minimal set of data needed to perform threading operations
/// using the JWZ algorithm. This is cached to avoid repeated database queries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailThreadingInfo {
    /// Database ID of the email
    pub email_id: i32,

    /// Message-ID from email header (unique identifier for threading)
    pub message_id: String,

    /// Email subject line
    pub subject: String,

    /// In-Reply-To header value (immediate parent reference)
    pub in_reply_to: Option<String>,

    /// Date the email was sent
    pub date: DateTime<Utc>,

    /// Patch series ID (for Linux kernel patch series)
    pub series_id: Option<String>,

    /// Patch number within series (e.g., 2 in "[PATCH 2/5]")
    pub series_number: Option<i32>,

    /// Total patches in series (e.g., 5 in "[PATCH 2/5]")
    pub series_total: Option<i32>,
}

/// Statistics about cache contents
///
/// Used for monitoring cache size and performance
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Which epoch this cache represents
    pub epoch: i32,

    /// Number of emails stored in cache
    pub email_count: usize,

    /// Number of reference entries stored
    pub reference_count: usize,

    /// Estimated memory usage in megabytes
    pub size_estimate_mb: usize,
}

/// Statistics for unified (non-epoch) cache
#[derive(Debug, Clone)]
pub struct UnifiedCacheStats {
    /// Number of emails stored in cache
    pub email_count: usize,

    /// Number of reference entries stored
    pub reference_count: usize,

    /// Estimated memory usage in megabytes
    pub size_estimate_mb: usize,
}

/// Errors that can occur during cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache not found")]
    NotFound,

    #[error("Epoch {0} not found in cache manager")]
    EpochNotFound(i32),

    #[error("Cache version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializeError(String),

    #[error("Deserialization error: {0}")]
    DeserializeError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Database row type for loading email threading info from database
///
/// This is an internal type used for sqlx query mapping
#[derive(sqlx::FromRow)]
pub(crate) struct EmailThreadingInfoRow {
    pub id: i32,
    pub message_id: String,
    pub subject: String,
    pub in_reply_to: Option<String>,
    pub date: DateTime<Utc>,
    pub series_id: Option<String>,
    pub series_number: Option<i32>,
    pub series_total: Option<i32>,
}

impl From<EmailThreadingInfoRow> for EmailThreadingInfo {
    fn from(row: EmailThreadingInfoRow) -> Self {
        EmailThreadingInfo {
            email_id: row.id,
            message_id: row.message_id,
            subject: row.subject,
            in_reply_to: row.in_reply_to,
            date: row.date,
            series_id: row.series_id,
            series_number: row.series_number,
            series_total: row.series_total,
        }
    }
}
