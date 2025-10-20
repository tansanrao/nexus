//! Data structures for bulk database operations.
//!
//! These structures hold prepared data in parallel vectors (columnar format)
//! optimized for PostgreSQL's UNNEST bulk insert operations.

use crate::models::PatchType;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Prepared email data for bulk insertion.
///
/// All vectors must have the same length. Each index represents one email record.
/// Uses columnar format optimized for PostgreSQL UNNEST operations.
#[derive(Default)]
pub struct EmailsData {
    pub message_ids: Vec<String>,
    pub commit_hashes: Vec<String>,
    pub author_ids: Vec<i32>,
    pub subjects: Vec<String>,
    pub normalized_subjects: Vec<String>,
    pub dates: Vec<DateTime<Utc>>,
    pub in_reply_tos: Vec<Option<String>>,
    pub bodies: Vec<String>,
    pub series_ids: Vec<Option<String>>,
    pub series_numbers: Vec<Option<i32>>,
    pub series_totals: Vec<Option<i32>>,
    pub epochs: Vec<i32>,
    pub patch_types: Vec<PatchType>,
    pub is_patch_only: Vec<bool>,
    pub patch_metadata: Vec<Option<Value>>,
}

/// Prepared recipient data for bulk insertion.
///
/// All vectors must have the same length. Each index represents one recipient record.
#[derive(Default)]
pub struct RecipientsData {
    pub list_ids: Vec<i32>,
    pub email_ids: Vec<i32>,
    pub author_ids: Vec<i32>,
    pub recipient_types: Vec<String>,
}

/// Prepared email reference data for bulk insertion.
///
/// All vectors must have the same length. Each index represents one reference record.
/// References are ordered by position to preserve the original email header order.
#[derive(Default, Clone)]
pub struct ReferencesData {
    pub list_ids: Vec<i32>,
    pub email_ids: Vec<i32>,
    pub referenced_message_ids: Vec<String>,
    pub positions: Vec<i32>,
}

/// Data needed to merge newly imported emails into the threading cache.
///
/// This structure contains email metadata and references that will be added
/// to the in-memory threading cache after successful database insertion.
pub struct ChunkCacheData {
    /// Email metadata: (email_id, message_id, subject, in_reply_to, date, series_id, series_number, series_total)
    pub emails: Vec<(
        i32,
        String,
        String,
        Option<String>,
        DateTime<Utc>,
        Option<String>,
        Option<i32>,
        Option<i32>,
    )>,
    /// References: (email_id, Vec<referenced_message_ids>)
    pub references: Vec<(i32, Vec<String>)>,
}
