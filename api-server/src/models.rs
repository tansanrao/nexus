//! Data transfer objects exposed by the API.
//!
//! Every struct in this module derives `JsonSchema` so `rocket_okapi` can describe
//! the payloads accurately in the generated OpenAPI document.

use chrono::{DateTime, Utc};
use rocket_db_pools::sqlx::postgres::{PgRow, PgTypeInfo};
use rocket_db_pools::sqlx::{self, FromRow, Row, Type, types::Json};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

/// Classification of an email's patch content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Type)]
#[sqlx(type_name = "patch_type", rename_all = "snake_case")]
pub enum PatchType {
    /// No git patch content detected.
    None,
    /// Inline diff detected within the email body.
    Inline,
    /// Patch provided via attachment (text/x-patch, text/x-diff, etc.).
    Attachment,
}

impl sqlx::postgres::PgHasArrayType for PatchType {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_patch_type")
    }
}

/// Inclusive range (0-based line numbers) marking a logical patch section.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatchSection {
    /// First line (0-based index) belonging to the section.
    pub start_line: usize,
    /// Last line (0-based index) belonging to the section.
    pub end_line: usize,
}

/// Aggregated metadata about inline git patches inside an email.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatchMetadata {
    /// Inline diff chunks detected in the body.
    pub diff_sections: Vec<PatchSection>,
    /// Optional section covering the diffstat block (between `---` separator and the first diff).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffstat_section: Option<PatchSection>,
    /// Sections covering trailers (Signed-off-by, Acked-by, etc.) and optional git footers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trailer_sections: Vec<PatchSection>,
    /// Position of the RFC 822 style `---` separator, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator_line: Option<usize>,
    /// Total number of trailer lines detected (Signed-off-by, Reviewed-by, ...).
    pub trailer_count: usize,
}

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Patch classification for this email.
    pub patch_type: PatchType,
    /// Whether the body is entirely commit message + diff content.
    pub is_patch_only: bool,
    /// Inline patch metadata (diff sections, trailers, diffstat).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_metadata: Option<PatchMetadata>,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Patch classification for this email.
    pub patch_type: PatchType,
    /// Whether the body is entirely commit message + diff content.
    pub is_patch_only: bool,
    /// Inline patch metadata (diff sections, trailers, diffstat).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_metadata: Option<PatchMetadata>,
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

impl<'r> FromRow<'r, PgRow> for EmailWithAuthor {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let patch_metadata: Option<Json<PatchMetadata>> = row.try_get("patch_metadata")?;
        Ok(Self {
            id: row.try_get("id")?,
            mailing_list_id: row.try_get("mailing_list_id")?,
            message_id: row.try_get("message_id")?,
            git_commit_hash: row.try_get("git_commit_hash")?,
            author_id: row.try_get("author_id")?,
            subject: row.try_get("subject")?,
            date: row.try_get("date")?,
            in_reply_to: row.try_get("in_reply_to")?,
            body: row.try_get("body")?,
            created_at: row.try_get("created_at")?,
            author_name: row.try_get("author_name")?,
            author_email: row.try_get("author_email")?,
            patch_type: row.try_get("patch_type")?,
            is_patch_only: row.try_get("is_patch_only")?,
            patch_metadata: patch_metadata.map(|json| json.0),
        })
    }
}

impl<'r> FromRow<'r, PgRow> for EmailHierarchy {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let patch_metadata: Option<Json<PatchMetadata>> = row.try_get("patch_metadata")?;
        Ok(Self {
            id: row.try_get("id")?,
            mailing_list_id: row.try_get("mailing_list_id")?,
            message_id: row.try_get("message_id")?,
            git_commit_hash: row.try_get("git_commit_hash")?,
            author_id: row.try_get("author_id")?,
            subject: row.try_get("subject")?,
            date: row.try_get("date")?,
            in_reply_to: row.try_get("in_reply_to")?,
            body: row.try_get("body")?,
            created_at: row.try_get("created_at")?,
            author_name: row.try_get("author_name")?,
            author_email: row.try_get("author_email")?,
            depth: row.try_get("depth")?,
            patch_type: row.try_get("patch_type")?,
            is_patch_only: row.try_get("is_patch_only")?,
            patch_metadata: patch_metadata.map(|json| json.0),
        })
    }
}

/// Summary statistics for a single mailing list.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, JsonSchema)]
pub struct MailingListStats {
    /// Total number of emails stored for the mailing list.
    #[serde(rename = "emailCount")]
    pub total_emails: i64,
    /// Total number of threads.
    #[serde(rename = "threadCount")]
    pub total_threads: i64,
    /// Number of unique authors.
    #[serde(rename = "authorCount")]
    pub total_authors: i64,
    /// Oldest email timestamp.
    #[serde(rename = "dateRangeStart")]
    pub date_range_start: Option<DateTime<Utc>>,
    /// Newest email timestamp.
    #[serde(rename = "dateRangeEnd")]
    pub date_range_end: Option<DateTime<Utc>>,
}

/// Aggregate mailing list statistics across the deployment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListAggregateStats {
    #[serde(rename = "totalLists")]
    pub total_lists: i64,
    #[serde(rename = "totalEmails")]
    pub total_emails: i64,
    #[serde(rename = "totalThreads")]
    pub total_threads: i64,
    #[serde(rename = "totalAuthors")]
    pub total_authors: i64,
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

/// Search hit metadata for thread queries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThreadSearchHit {
    /// Thread metadata and starter information.
    pub thread: ThreadWithStarter,
    /// Lexical score (0..1) when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical_score: Option<f32>,
}

impl ThreadSearchHit {
    pub fn from_thread(thread: ThreadWithStarter) -> Self {
        Self {
            thread,
            lexical_score: None,
        }
    }
}

/// Response envelope for thread search queries.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSearchResponse {
    /// Original query string trimmed.
    pub query: String,
    /// One-based page index.
    pub page: i64,
    /// Page size used for the request.
    pub size: i64,
    /// Total number of matching threads (best effort for hybrid).
    pub total: i64,
    /// Ranked search hits.
    pub results: Vec<ThreadSearchHit>,
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

/// Generic wrapper used by endpoints that return simple collections.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataResponse<T> {
    /// Response payload.
    pub data: T,
}

/// Direction applied to a sort field.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Sort metadata returned in the response envelope.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SortDescriptor {
    pub field: String,
    pub direction: SortDirection,
}

/// Pagination metadata attached to list responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaginationMeta {
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "totalItems")]
    pub total_items: i64,
}

impl PaginationMeta {
    pub fn new(page: i64, page_size: i64, total_items: i64) -> Self {
        let total_pages = if page_size > 0 {
            (total_items + page_size - 1) / page_size
        } else {
            0
        };

        Self {
            page,
            page_size,
            total_pages,
            total_items,
        }
    }
}

/// Standard response envelope for the public and admin APIs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort: Vec<SortDescriptor>,
    #[serde(rename = "listId", skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<JsonMap<String, JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<JsonMap<String, JsonValue>>,
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            pagination: None,
            sort: Vec::new(),
            list_id: None,
            filters: None,
            extra: None,
        }
    }
}

impl ResponseMeta {
    pub fn with_pagination(mut self, page: PaginationMeta) -> Self {
        self.pagination = Some(page);
        self
    }

    pub fn with_sort(mut self, sort: Vec<SortDescriptor>) -> Self {
        self.sort = sort;
        self
    }

    pub fn with_list_id(mut self, slug: impl Into<String>) -> Self {
        self.list_id = Some(slug.into());
        self
    }

    pub fn with_filters(mut self, filters: JsonMap<String, JsonValue>) -> Self {
        self.filters = Some(filters);
        self
    }

    pub fn with_extra(mut self, extra: JsonMap<String, JsonValue>) -> Self {
        self.extra = Some(extra);
        self
    }
}

/// Root response payload returned by REST endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(default)]
    pub meta: ResponseMeta,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self::with_meta(data, ResponseMeta::default())
    }

    pub fn with_meta(data: T, meta: ResponseMeta) -> Self {
        Self { data, meta }
    }
}
