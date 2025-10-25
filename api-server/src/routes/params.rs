//! Query parameter helpers shared by multiple API route handlers.
//!
//! These structs and enums provide strongly-typed parsing for URL query strings
//! while exposing the metadata needed for OpenAPI generation via `rocket_okapi`.
//! The types follow Rocket's `FromForm` conventions and derive `JsonSchema` so
//! generated documentation reflects the available parameters and their defaults.

use rocket::form::{self, FromForm, FromFormField, ValueField};
use rocket_okapi::okapi::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const fn default_page() -> i64 {
    1
}

const fn default_page_size() -> i64 {
    50
}

const fn default_search_page_size() -> i64 {
    25
}

const MAX_PAGE_SIZE: i64 = 100;

fn default_sort_order() -> SortOrder {
    SortOrder::Desc
}

fn default_author_sort_field() -> AuthorSortField {
    AuthorSortField::EmailCount
}

fn default_thread_sort_field() -> ThreadSortField {
    ThreadSortField::LastDate
}

fn default_optional_string() -> Option<String> {
    None
}

/// Common pagination parameters applied to list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    /// One-based page index (defaults to the first page).
    #[field(default = 1)]
    #[serde(default = "default_page")]
    pub page: i64,
    /// Number of items per page (clamped between 1 and 100, default 50).
    #[field(default = 50)]
    #[serde(default = "default_page_size")]
    pub size: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            size: default_page_size(),
        }
    }
}

impl PaginationParams {
    /// Normalized 1-based page index.
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    /// Normalized page size capped at [`MAX_PAGE_SIZE`].
    pub fn size(&self) -> i64 {
        self.size.clamp(1, MAX_PAGE_SIZE)
    }
}

/// Sort direction for list endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Sort ascending.
    Asc,
    /// Sort descending.
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

impl SortOrder {
    /// Render the sort order as a SQL keyword.
    pub fn sql_keyword(self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

impl<'r> FromFormField<'r> for SortOrder {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value.to_ascii_lowercase().as_str() {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            other => Err(form::Error::validation(format!(
                "invalid sort order '{other}'; expected 'asc' or 'desc'"
            ))
            .into()),
        }
    }
}

/// Sort keys supported by the author search endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AuthorSortField {
    CanonicalName,
    Email,
    EmailCount,
    ThreadCount,
    FirstEmailDate,
    LastEmailDate,
}

impl Default for AuthorSortField {
    fn default() -> Self {
        AuthorSortField::EmailCount
    }
}

impl AuthorSortField {
    /// Name of the column used when ordering query results.
    pub fn sql_column(self) -> &'static str {
        match self {
            AuthorSortField::CanonicalName => "canonical_name",
            AuthorSortField::Email => "email",
            AuthorSortField::EmailCount => "email_count",
            AuthorSortField::ThreadCount => "thread_count",
            AuthorSortField::FirstEmailDate => "first_email_date",
            AuthorSortField::LastEmailDate => "last_email_date",
        }
    }
}

impl<'r> FromFormField<'r> for AuthorSortField {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value {
            "canonicalName" => Ok(AuthorSortField::CanonicalName),
            "email" => Ok(AuthorSortField::Email),
            "emailCount" => Ok(AuthorSortField::EmailCount),
            "threadCount" => Ok(AuthorSortField::ThreadCount),
            "firstEmailDate" => Ok(AuthorSortField::FirstEmailDate),
            "lastEmailDate" => Ok(AuthorSortField::LastEmailDate),
            other => {
                Err(form::Error::validation(format!("invalid author sort key '{other}'")).into())
            }
        }
    }
}

/// Query parameters accepted by the author search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSearchParams {
    /// Optional full-text search term matched against author name/email.
    #[serde(default = "default_optional_string")]
    pub q: Option<String>,
    /// Page of results to fetch (defaults to 1).
    #[field(default = 1)]
    #[serde(default = "default_page")]
    pub page: i64,
    /// Page size (defaults to 50, maximum 100).
    #[field(default = 50)]
    #[serde(default = "default_page_size")]
    pub size: i64,
    /// Sort column (defaults to `emailCount`).
    #[field(name = "sortBy", default = AuthorSortField::EmailCount)]
    #[serde(default = "default_author_sort_field")]
    pub sort_by: AuthorSortField,
    /// Sort direction (defaults to `desc`).
    #[field(default = SortOrder::Desc)]
    #[serde(default = "default_sort_order")]
    pub order: SortOrder,
}

impl Default for AuthorSearchParams {
    fn default() -> Self {
        Self {
            q: None,
            page: default_page(),
            size: default_page_size(),
            sort_by: default_author_sort_field(),
            order: default_sort_order(),
        }
    }
}

impl AuthorSearchParams {
    /// Normalized page index.
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    /// Normalized page size.
    pub fn size(&self) -> i64 {
        self.size.clamp(1, MAX_PAGE_SIZE)
    }

    /// SQL column used for ordering.
    pub fn sort_column(&self) -> &'static str {
        self.sort_by.sql_column()
    }

    /// SQL keyword representing the sort direction.
    pub fn sort_order(&self) -> &'static str {
        self.order.sql_keyword()
    }

    /// Lower-cased search term with surrounding whitespace removed.
    pub fn normalized_query(&self) -> Option<String> {
        self.q.as_ref().and_then(|value| {
            let normalized = value.trim().to_lowercase();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        })
    }
}

/// Sorting options for thread listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ThreadSortField {
    StartDate,
    LastDate,
    MessageCount,
}

impl Default for ThreadSortField {
    fn default() -> Self {
        ThreadSortField::LastDate
    }
}

impl ThreadSortField {
    /// SQL column corresponding to the sort key.
    pub fn sql_column(self) -> &'static str {
        match self {
            ThreadSortField::StartDate => "start_date",
            ThreadSortField::LastDate => "last_date",
            ThreadSortField::MessageCount => "message_count",
        }
    }
}

impl<'r> FromFormField<'r> for ThreadSortField {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match field.value {
            "startDate" => Ok(ThreadSortField::StartDate),
            "lastDate" => Ok(ThreadSortField::LastDate),
            "messageCount" => Ok(ThreadSortField::MessageCount),
            other => {
                Err(form::Error::validation(format!("invalid thread sort key '{other}'")).into())
            }
        }
    }
}

/// Query parameters supported by the thread list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListParams {
    /// Page of results to fetch (defaults to 1).
    #[field(default = 1)]
    #[serde(default = "default_page")]
    pub page: i64,
    /// Page size (defaults to 50, maximum 100).
    #[field(default = 50)]
    #[serde(default = "default_page_size")]
    pub size: i64,
    /// Sort column (defaults to `lastDate`).
    #[field(name = "sortBy", default = ThreadSortField::LastDate)]
    #[serde(default = "default_thread_sort_field")]
    pub sort_by: ThreadSortField,
    /// Sort direction (defaults to `desc`).
    #[field(default = SortOrder::Desc)]
    #[serde(default = "default_sort_order")]
    pub order: SortOrder,
}

impl Default for ThreadListParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            size: default_page_size(),
            sort_by: default_thread_sort_field(),
            order: default_sort_order(),
        }
    }
}

impl ThreadListParams {
    /// Normalized page index.
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    /// Normalized page size.
    pub fn size(&self) -> i64 {
        self.size.clamp(1, MAX_PAGE_SIZE)
    }

    /// SQL column used for sorting.
    pub fn sort_column(&self) -> &'static str {
        self.sort_by.sql_column()
    }

    /// SQL keyword representing the sort order.
    pub fn sort_order(&self) -> &'static str {
        self.order.sql_keyword()
    }
}

/// Query parameters for the thread search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, FromForm, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSearchParams {
    /// Free-text search term. If omitted, the endpoint returns an empty result set.
    #[serde(default = "default_optional_string")]
    pub q: Option<String>,
    /// Page of results to fetch (defaults to 1).
    #[field(default = 1)]
    #[serde(default = "default_page")]
    pub page: i64,
    /// Page size (defaults to 25, maximum 100).
    #[field(default = 25)]
    #[serde(default = "default_search_page_size")]
    pub size: i64,
}

impl Default for ThreadSearchParams {
    fn default() -> Self {
        Self {
            q: None,
            page: default_page(),
            size: default_search_page_size(),
        }
    }
}

impl ThreadSearchParams {
    /// Normalized search term (trimmed) with empty strings removed.
    pub fn query(&self) -> Option<&str> {
        self.q
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
    }

    /// Normalized page index.
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    /// Normalized page size.
    pub fn size(&self) -> i64 {
        self.size.clamp(1, MAX_PAGE_SIZE)
    }
}
