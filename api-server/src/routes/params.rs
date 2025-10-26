//! Query parameter helpers shared by multiple API route handlers.
//!
//! These structs and enums provide strongly-typed parsing for URL query strings
//! while exposing the metadata needed for OpenAPI generation via `rocket_okapi`.
//! The types follow Rocket's `FromForm` conventions and derive `JsonSchema` so
//! generated documentation reflects the available parameters and their defaults.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use rocket::form::{self, FromFormField, ValueField};
use rocket_okapi::okapi::schemars::{self, JsonSchema};
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

/// Wrapper for parsing ISO-8601 dates from query parameters.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct DateParam(pub NaiveDate);

impl<'r> FromFormField<'r> for DateParam {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        let trimmed = field.value.trim();
        match NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            Ok(date) => Ok(DateParam(date)),
            Err(_) => Err(form::Error::validation(format!(
                "invalid date '{}', expected YYYY-MM-DD",
                field.value
            )))?,
        }
    }
}

/// Common pagination parameters applied to list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rocket::form::FromForm)]
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
#[derive(Debug, Clone, Serialize, Deserialize, rocket::form::FromForm, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, rocket::form::FromForm, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, rocket::form::FromForm)]
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
    /// Optional inclusive lower bound (UTC date) for thread last activity.
    #[field(name = "startDate")]
    #[serde(default)]
    pub start_date: Option<DateParam>,
    /// Optional inclusive upper bound (UTC date) for thread start date.
    #[field(name = "endDate")]
    #[serde(default)]
    pub end_date: Option<DateParam>,
    /// Optional semantic/lexical mixing ratio for hybrid Meilisearch queries.
    #[field(name = "semanticRatio")]
    #[serde(default)]
    pub semantic_ratio: Option<f32>,
}

impl Default for ThreadSearchParams {
    fn default() -> Self {
        Self {
            q: None,
            page: default_page(),
            size: default_search_page_size(),
            start_date: None,
            end_date: None,
            semantic_ratio: None,
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

    /// Inclusive lower bound converted to UTC midnight.
    pub fn start_date_utc(&self) -> Option<DateTime<Utc>> {
        self.start_date.as_ref().and_then(|param| {
            param
                .0
                .and_hms_opt(0, 0, 0)
                .map(|naive| Utc.from_utc_datetime(&naive))
        })
    }

    /// Inclusive upper bound converted to UTC end-of-day.
    pub fn end_date_utc(&self) -> Option<DateTime<Utc>> {
        self.end_date.as_ref().and_then(|param| {
            param
                .0
                .and_hms_milli_opt(23, 59, 59, 999)
                .map(|naive| Utc.from_utc_datetime(&naive))
        })
    }

    /// Optional semantic ratio clamped between 0.0 and 1.0.
    pub fn semantic_ratio(&self) -> Option<f32> {
        self.semantic_ratio
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 1.0))
    }
}

impl JsonSchema for ThreadSearchParams {
    fn schema_name() -> String {
        "ThreadSearchParams".to_string()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        #[derive(Serialize, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        struct ThreadSearchParamsDoc {
            #[serde(default = "default_optional_string")]
            q: Option<String>,
            #[serde(default = "default_page")]
            page: i64,
            #[serde(default = "default_search_page_size")]
            size: i64,
            #[serde(default)]
            start_date: Option<String>,
            #[serde(default)]
            end_date: Option<String>,
            #[serde(default)]
            semantic_ratio: Option<f32>,
        }

        ThreadSearchParamsDoc::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use rocket::form::Form;

    #[test]
    fn parses_thread_search_query() {
        let parsed: ThreadSearchParams = Form::parse("q=test&page=2&size=10").unwrap();
        assert_eq!(parsed.q.as_deref(), Some("test"));
        assert_eq!(parsed.page(), 2);
        assert_eq!(parsed.size(), 10);
        assert!(parsed.start_date.is_none());
        assert!(parsed.end_date.is_none());
        assert!(parsed.semantic_ratio.is_none());

        let parsed_default: ThreadSearchParams = Form::parse("").unwrap();
        assert_eq!(parsed_default.q, None);
        assert_eq!(parsed_default.page(), 1);
        assert_eq!(parsed_default.size(), 25);
        assert!(parsed_default.start_date.is_none());
        assert!(parsed_default.end_date.is_none());
    }

    #[test]
    fn parses_thread_search_date_filters() {
        let parsed: ThreadSearchParams =
            Form::parse("startDate=2025-01-01&endDate=2025-02-15").unwrap();

        let start = parsed.start_date_utc().unwrap();
        let end = parsed.end_date_utc().unwrap();

        assert_eq!(
            start.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        );
        assert_eq!(start.hour(), 0);
        assert_eq!(
            end.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2025, 2, 15).unwrap()
        );
        assert_eq!(end.hour(), 23);
        assert_eq!(end.minute(), 59);
    }

    #[test]
    fn clamps_semantic_ratio() {
        let parsed: ThreadSearchParams = Form::parse("semanticRatio=1.5").unwrap();
        assert_eq!(parsed.semantic_ratio(), Some(1.0));

        let parsed_zero: ThreadSearchParams = Form::parse("semanticRatio=-0.5").unwrap();
        assert_eq!(parsed_zero.semantic_ratio(), Some(0.0));
    }
}
