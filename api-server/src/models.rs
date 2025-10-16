//! Data transfer objects exposed by the API.
//!
//! Every struct in this module derives `JsonSchema` so `rocket_okapi` can describe
//! the payloads accurately in the generated OpenAPI document.

use chrono::{DateTime, Utc};
use rocket_db_pools::sqlx::FromRow;
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Metadata for a mailing list managed by the service.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct MailingList {
    /// Database identifier.
    pub id: i32,
    /// Human-friendly display name.
    pub name: String,
    /// Unique slug used in URLs (e.g. `linux-kernel`).
    pub slug: String,
    /// Optional description sourced from the grokmirror manifest.
    pub description: Option<String>,
    /// Whether sync jobs are allowed to run for this mailing list.
    pub enabled: bool,
    /// Priority applied when enqueuing sync jobs (lower value first).
    pub sync_priority: i32,
    /// When the list record was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of the last successful sync, if any.
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Repository shard backing a mailing list (one per public-inbox epoch).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct MailingListRepository {
    /// Database identifier.
    pub id: i32,
    /// Parent mailing list identifier.
    pub mailing_list_id: i32,
    /// Remote repository URL (https://lore.kernel.org/...).
    pub repo_url: String,
    /// Repository order/epoch. Lower numbers represent older history.
    pub repo_order: i32,
    /// Last commit processed during sync, if any.
    pub last_indexed_commit: Option<String>,
    /// Timestamp of when the shard configuration was added.
    pub created_at: Option<DateTime<Utc>>,
}

/// Mailing list descriptor bundled with all configured repositories.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MailingListWithRepos {
    /// Mailing list metadata.
    #[serde(flatten)]
    pub list: MailingList,
    /// Repository shards belonging to the mailing list.
    pub repos: Vec<MailingListRepository>,
}

/// Thread metadata stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct Thread {
    /// Database identifier.
    pub id: i32,
    /// Mailing list identifier.
    pub mailing_list_id: i32,
    /// RFC 822 message-id of the root email.
    pub root_message_id: String,
    /// Normalized thread subject.
    pub subject: String,
    /// Timestamp of the first email in the thread.
    pub start_date: DateTime<Utc>,
    /// Timestamp of the latest email in the thread.
    pub last_date: DateTime<Utc>,
    /// Total number of emails in the thread.
    pub message_count: Option<i32>,
}

/// Email row enriched with author metadata for API responses.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct EmailWithAuthor {
    /// Database identifier.
    pub id: i32,
    /// Mailing list identifier.
    pub mailing_list_id: i32,
    /// RFC 822 message-id.
    pub message_id: String,
    /// Git commit hash referencing the blob inside the mirror.
    pub git_commit_hash: String,
    /// Author identifier.
    pub author_id: i32,
    /// Email subject line.
    pub subject: String,
    /// Original message timestamp.
    pub date: DateTime<Utc>,
    /// Optional parent message-id (for replies).
    pub in_reply_to: Option<String>,
    /// Message body (may be truncated or sanitized).
    pub body: Option<String>,
    /// Timestamp when the row was inserted.
    pub created_at: Option<DateTime<Utc>>,
    /// Canonical author name, if known.
    pub author_name: Option<String>,
    /// Author email address.
    pub author_email: String,
}

/// Thread details including the threaded list of emails.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThreadDetail {
    /// Thread metadata.
    pub thread: Thread,
    /// Emails that belong to the thread ordered depth-first.
    pub emails: Vec<EmailHierarchy>,
}

/// Email node enriched with depth information for thread rendering.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct EmailHierarchy {
    /// Email identifier.
    pub id: i32,
    /// Mailing list identifier.
    pub mailing_list_id: i32,
    /// RFC 822 message-id.
    pub message_id: String,
    /// Git commit hash referencing the blob inside the mirror.
    pub git_commit_hash: String,
    /// Author identifier.
    pub author_id: i32,
    /// Email subject.
    pub subject: String,
    /// Email timestamp.
    pub date: DateTime<Utc>,
    /// Optional parent message-id.
    pub in_reply_to: Option<String>,
    /// Message body, where available.
    pub body: Option<String>,
    /// Timestamp when the row was inserted.
    pub created_at: Option<DateTime<Utc>>,
    /// Canonical author name, if known.
    pub author_name: Option<String>,
    /// Author email address.
    pub author_email: String,
    /// Depth within the thread tree (root = 0).
    pub depth: i32,
}

/// Aggregated author statistics used in list and detail endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthorWithStats {
    /// Author identifier.
    pub id: i32,
    /// Primary email address.
    pub email: String,
    /// Normalized/canonical author name.
    pub canonical_name: Option<String>,
    /// When the author was first seen in any mailing list.
    pub first_seen: Option<DateTime<Utc>>,
    /// Most recent activity timestamp.
    pub last_seen: Option<DateTime<Utc>>,
    /// Number of emails authored in the target mailing list.
    pub email_count: i64,
    /// Number of threads the author participated in.
    pub thread_count: i64,
    /// Timestamp of the first email authored in the list.
    pub first_email_date: Option<DateTime<Utc>>,
    /// Timestamp of the latest email authored in the list.
    pub last_email_date: Option<DateTime<Utc>>,
    /// All mailing list slugs where the author is active.
    pub mailing_lists: Vec<String>,
    /// Observed name variants sorted by usage count.
    pub name_variations: Vec<String>,
}

/// Summary statistics for a mailing list.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct Stats {
    /// Total number of emails stored for the mailing list.
    pub total_emails: i64,
    /// Total number of threads.
    pub total_threads: i64,
    /// Number of unique authors.
    pub total_authors: i64,
    /// Oldest email timestamp.
    pub date_range_start: Option<DateTime<Utc>>,
    /// Newest email timestamp.
    pub date_range_end: Option<DateTime<Utc>>,
}

/// Thread metadata augmented with the starter author.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct ThreadWithStarter {
    /// Thread identifier.
    pub id: i32,
    /// Mailing list identifier.
    pub mailing_list_id: i32,
    /// RFC 822 message-id of the root email.
    pub root_message_id: String,
    /// Thread subject.
    pub subject: String,
    /// Thread start timestamp.
    pub start_date: DateTime<Utc>,
    /// Most recent activity timestamp.
    pub last_date: DateTime<Utc>,
    /// Total number of emails in the thread.
    pub message_count: Option<i32>,
    /// Author identifier for the thread starter.
    pub starter_id: i32,
    /// Canonical name of the starter, if known.
    pub starter_name: Option<String>,
    /// Email of the thread starter.
    pub starter_email: String,
}

/// Pagination metadata accompanying list responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PageMetadata {
    /// One-based page index.
    pub page: i64,
    /// Page size.
    pub size: i64,
    /// Total number of pages.
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    /// Total number of matching records.
    #[serde(rename = "totalElements")]
    pub total_elements: i64,
}

/// Wrapper for paginated datasets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    /// Page content.
    pub data: Vec<T>,
    /// Associated pagination metadata.
    pub page: PageMetadata,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response and compute pagination totals.
    pub fn new(data: Vec<T>, page: i64, size: i64, total_elements: i64) -> Self {
        let total_pages = if size > 0 {
            (total_elements + size - 1) / size
        } else {
            0
        };

        Self {
            data,
            page: PageMetadata {
                page,
                size,
                total_pages,
                total_elements,
            },
        }
    }
}

/// Generic wrapper used by endpoints that return simple collections.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataResponse<T> {
    /// Response payload.
    pub data: T,
}
