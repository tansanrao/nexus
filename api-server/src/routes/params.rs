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
    25
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
    #[field(name = "pageSize", default = 25)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    pub page_size: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

impl PaginationParams {
    /// Normalized 1-based page index.
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    /// Normalized page size capped at [`MAX_PAGE_SIZE`].
    pub fn page_size(&self) -> i64 {
        self.page_size.clamp(1, MAX_PAGE_SIZE)
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
    /// Optional mailing list filters.
    #[field(name = "mailingList")]
    #[serde(default)]
    pub mailing_lists: Vec<String>,
}

impl Default for AuthorSearchParams {
    fn default() -> Self {
        Self {
            q: None,
            page: default_page(),
            size: default_page_size(),
            sort_by: default_author_sort_field(),
            order: default_sort_order(),
            mailing_lists: Vec::new(),
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

    /// Original trimmed search term preserving case.
    pub fn raw_query(&self) -> Option<String> {
        self.q
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    }

    /// Normalized mailing list filters (trimmed, deduplicated).
    pub fn mailing_lists(&self) -> Vec<String> {
        let mut lists: Vec<String> = self
            .mailing_lists
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect();
        lists.sort();
        lists.dedup();
        lists
    }
}

/// Query parameters supported by the thread list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, rocket::form::FromForm, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThreadListParams {
    #[field(default = 1)]
    #[serde(default = "default_page")]
    page: i64,
    #[field(name = "pageSize", default = 50)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i64,
    #[field(name = "sort")]
    #[serde(default)]
    sort: Vec<String>,
}

impl Default for ThreadListParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            sort: vec!["lastActivity:desc".to_string()],
        }
    }
}

impl ThreadListParams {
    pub fn page(&self) -> i64 {
        self.page.max(1)
    }

    pub fn page_size(&self) -> i64 {
        self.page_size.clamp(1, MAX_PAGE_SIZE)
    }

    pub fn sort(&self) -> Vec<String> {
        if self.sort.is_empty() {
            vec!["lastActivity:desc".to_string()]
        } else {
            self.sort.clone()
        }
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
    /// Optional filter limiting results to threads containing patches.
    #[field(name = "hasPatches")]
    #[serde(default)]
    pub has_patches: Option<bool>,
    /// Optional filter limiting results to threads started by the given author id.
    #[field(name = "starterId")]
    #[serde(default)]
    pub starter_id: Option<i32>,
    /// Optional filter matching threads with at least one of the specified participant ids.
    #[field(name = "participantId")]
    #[serde(default)]
    pub participant_ids: Vec<i32>,
    /// Optional filter limiting results to a series identifier.
    #[field(name = "seriesId")]
    #[serde(default = "default_optional_string")]
    pub series_id: Option<String>,
    /// Optional sort descriptors (field:direction).
    #[field(name = "sort")]
    #[serde(default)]
    pub sort: Vec<String>,
    /// Optional list of mailing lists (for global search endpoint).
    #[field(name = "mailingList")]
    #[serde(default)]
    pub mailing_lists: Vec<String>,
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
            has_patches: None,
            starter_id: None,
            participant_ids: Vec::new(),
            series_id: None,
            sort: Vec::new(),
            mailing_lists: Vec::new(),
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

    /// Optional patch flag filter.
    pub fn has_patches(&self) -> Option<bool> {
        self.has_patches
    }

    /// Optional starter id filter (positive integers only).
    pub fn starter_id(&self) -> Option<i32> {
        self.starter_id.filter(|id| *id > 0)
    }

    /// Deduplicated participant ids (positive integers).
    pub fn participant_ids(&self) -> Vec<i32> {
        let mut ids: Vec<i32> = self
            .participant_ids
            .iter()
            .copied()
            .filter(|id| *id > 0)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Normalized series identifier.
    pub fn series_id(&self) -> Option<String> {
        self.series_id
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    }

    /// Normalized sort expressions.
    pub fn sort_fields(&self) -> Vec<String> {
        self.sort
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect()
    }

    /// Normalized mailing list filters (lowercase, deduplicated).
    pub fn mailing_lists(&self) -> Vec<String> {
        let mut lists: Vec<String> = self
            .mailing_lists
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect();
        lists.sort();
        lists.dedup();
        lists
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
            #[serde(default)]
            has_patches: Option<bool>,
            #[serde(default)]
            starter_id: Option<i32>,
            #[serde(default)]
            participant_id: Vec<i32>,
            #[serde(default = "default_optional_string")]
            series_id: Option<String>,
            #[serde(default)]
            sort: Vec<String>,
            #[serde(default)]
            mailing_list: Vec<String>,
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
        assert_eq!(parsed.has_patches(), None);
        assert_eq!(parsed.starter_id(), None);
        assert!(parsed.participant_ids().is_empty());
        assert_eq!(parsed.series_id(), None);
        assert!(parsed.sort_fields().is_empty());
        assert!(parsed.mailing_lists().is_empty());

        let parsed_default: ThreadSearchParams = Form::parse("").unwrap();
        assert_eq!(parsed_default.q, None);
        assert_eq!(parsed_default.page(), 1);
        assert_eq!(parsed_default.size(), 25);
        assert!(parsed_default.start_date.is_none());
        assert!(parsed_default.end_date.is_none());
        assert!(parsed_default.sort_fields().is_empty());
        assert!(parsed_default.mailing_lists().is_empty());
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

    #[test]
    fn parses_thread_search_filters_and_sort() {
        let parsed: ThreadSearchParams = Form::parse(
            "hasPatches=true&starterId=42&participantId=10&participantId=42&participantId=-1&seriesId= abc123 &sort=lastActivity:desc&sort=messageCount:asc&mailingList=linux-kernel&mailingList=netdev",
        )
        .unwrap();

        assert_eq!(parsed.has_patches(), Some(true));
        assert_eq!(parsed.starter_id(), Some(42));
        assert_eq!(parsed.participant_ids(), vec![10, 42]);
        assert_eq!(parsed.series_id().as_deref(), Some("abc123"));
        assert_eq!(
            parsed.sort_fields(),
            vec![
                "lastActivity:desc".to_string(),
                "messageCount:asc".to_string()
            ]
        );
        assert_eq!(
            parsed.mailing_lists(),
            vec!["linux-kernel".to_string(), "netdev".to_string()]
        );
    }

    #[test]
    fn author_search_mailing_lists_dedup() {
        let parsed: AuthorSearchParams =
            Form::parse("mailingList=linux-kernel&mailingList= netdev &mailingList=linux-kernel")
                .unwrap();

        assert_eq!(
            parsed.mailing_lists(),
            vec!["linux-kernel".to_string(), "netdev".to_string()]
        );
    }
}
