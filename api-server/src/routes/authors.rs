//! Author endpoints providing global and list-scoped views.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{
    ApiResponse, AuthorWithStats, EmailWithAuthor, PaginationMeta, ResponseMeta, SortDescriptor,
    SortDirection, ThreadWithStarter,
};
use crate::routes::{
    helpers::resolve_mailing_list_id,
    params::{PaginationParams, ThreadListParams},
};
use chrono::{DateTime, Utc};
use rocket::get;
use rocket::serde::json::Json;
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::openapi;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use sqlx::{FromRow, QueryBuilder};

fn parse_thread_sorts(values: &[String]) -> (Vec<String>, Vec<SortDescriptor>) {
    let mut clauses = Vec::new();
    let mut descriptors = Vec::new();

    for value in values {
        let mut parts = value.splitn(2, ':');
        let field = parts.next().unwrap_or_default().trim();
        if field.is_empty() {
            continue;
        }
        let direction = parts.next().unwrap_or("desc").trim();
        let (column, api_field) = match field {
            "startDate" => ("start_date", "startDate"),
            "lastActivity" => ("last_date", "lastActivity"),
            "messageCount" => ("message_count", "messageCount"),
            _ => continue,
        };

        let dir = if direction.eq_ignore_ascii_case("asc") {
            SortDirection::Asc
        } else {
            SortDirection::Desc
        };

        let sql_dir = match dir {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        clauses.push(format!("{column} {sql_dir}"));
        descriptors.push(SortDescriptor {
            field: api_field.to_string(),
            direction: dir,
        });
    }

    if clauses.is_empty() {
        clauses.push("last_date DESC".to_string());
        descriptors.push(SortDescriptor {
            field: "lastActivity".to_string(),
            direction: SortDirection::Desc,
        });
    }

    (clauses, descriptors)
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    25
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, rocket::form::FromForm)]
#[serde(rename_all = "camelCase")]
pub struct AuthorListParams {
    #[field(default = 1)]
    #[serde(default = "default_page")]
    page: i64,
    #[field(name = "pageSize", default = 25)]
    #[serde(default = "default_page_size", rename = "pageSize")]
    page_size: i64,
    #[field(name = "sort")]
    #[serde(default)]
    sort: Vec<String>,
    #[field(name = "q")]
    #[serde(default)]
    q: Option<String>,
    #[field(name = "listSlug")]
    #[serde(default)]
    list_slug: Option<String>,
}

impl Default for AuthorListParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
            sort: vec!["lastSeen:desc".to_string()],
            q: None,
            list_slug: None,
        }
    }
}

impl AuthorListParams {
    fn page(&self) -> i64 {
        self.page.max(1)
    }

    fn page_size(&self) -> i64 {
        self.page_size.clamp(1, 100)
    }

    fn sort(&self) -> Vec<String> {
        if self.sort.is_empty() {
            vec!["lastSeen:desc".to_string()]
        } else {
            self.sort.clone()
        }
    }

    fn normalized_query(&self) -> Option<String> {
        self.q
            .as_ref()
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
            .map(|q| q.to_lowercase())
    }

    fn raw_query(&self) -> Option<String> {
        self.q
            .as_ref()
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
            .map(|q| q.to_string())
    }

    fn list_slug(&self) -> Option<String> {
        self.list_slug
            .as_ref()
            .map(|slug| slug.trim())
            .filter(|slug| !slug.is_empty())
            .map(|slug| slug.to_string())
    }
}

#[derive(Debug, FromRow)]
struct DbAuthorRow {
    id: i32,
    email: String,
    canonical_name: Option<String>,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
    email_count: i64,
    thread_count: i64,
    first_email_date: Option<DateTime<Utc>>,
    last_email_date: Option<DateTime<Utc>>,
    mailing_lists: Vec<String>,
    name_variations: Vec<String>,
}

impl From<DbAuthorRow> for AuthorWithStats {
    fn from(row: DbAuthorRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            canonical_name: row.canonical_name,
            first_seen: row.first_seen,
            last_seen: row.last_seen,
            email_count: row.email_count,
            thread_count: row.thread_count,
            first_email_date: row.first_email_date,
            last_email_date: row.last_email_date,
            mailing_lists: row.mailing_lists,
            name_variations: row.name_variations,
        }
    }
}

fn parse_author_sorts(values: &[String]) -> (Vec<String>, Vec<SortDescriptor>) {
    let mut clauses = Vec::new();
    let mut descriptors = Vec::new();

    for value in values {
        let mut parts = value.splitn(2, ':');
        let field = parts.next().unwrap_or_default().trim();
        if field.is_empty() {
            continue;
        }
        let direction = parts.next().unwrap_or("desc").trim();
        let (column, api_field) = match field {
            "lastSeen" => ("last_seen", "lastSeen"),
            "firstSeen" => ("first_seen", "firstSeen"),
            "email" => ("LOWER(a.email)", "email"),
            "canonicalName" => ("LOWER(a.canonical_name)", "canonicalName"),
            "activity" => ("email_count", "activity"),
            "threadCount" => ("thread_count", "threadCount"),
            _ => continue,
        };

        let dir = if direction.eq_ignore_ascii_case("asc") {
            SortDirection::Asc
        } else {
            SortDirection::Desc
        };

        let sql_dir = match dir {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        clauses.push(format!("{column} {sql_dir}"));
        descriptors.push(SortDescriptor {
            field: api_field.to_string(),
            direction: dir,
        });
    }

    if clauses.is_empty() {
        clauses.push("last_seen DESC".to_string());
        descriptors.push(SortDescriptor {
            field: "lastSeen".to_string(),
            direction: SortDirection::Desc,
        });
    }

    (clauses, descriptors)
}

fn apply_author_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    list_slug: Option<&'a str>,
    normalized_query: Option<&'a str>,
) {
    let mut has_where = false;

    if let Some(slug) = list_slug {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ml.slug = ");
        builder.push_bind(slug);
        has_where = true;
    }

    if let Some(query) = normalized_query {
        let pattern = format!("%{}%", query);
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(");
        builder.push("LOWER(a.email) LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR LOWER(a.canonical_name) LIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }
}

#[openapi(tag = "Authors")]
#[get("/authors?<params..>")]
pub async fn list_authors(
    mut db: Connection<NexusDb>,
    params: Option<AuthorListParams>,
) -> Result<Json<ApiResponse<Vec<AuthorWithStats>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let list_slug = params.list_slug();
    let normalized_query = params.normalized_query();
    let (order_clauses, sort_meta) = parse_author_sorts(&params.sort());
    let order_sql = order_clauses.join(", ");

    let mut count_builder = QueryBuilder::new("SELECT COUNT(DISTINCT a.id) FROM authors a");
    count_builder.push(" LEFT JOIN author_mailing_list_activity act ON act.author_id = a.id");
    count_builder.push(" LEFT JOIN mailing_lists ml ON ml.id = act.mailing_list_id");
    apply_author_filters(
        &mut count_builder,
        list_slug.as_deref(),
        normalized_query.as_deref(),
    );

    let total = count_builder
        .build_query_scalar::<i64>()
        .fetch_one(&mut **db)
        .await?;

    let mut data_builder = QueryBuilder::new(
        "SELECT \
            a.id, a.email, a.canonical_name, a.first_seen, a.last_seen, \
            COALESCE(SUM(act.email_count), 0) AS email_count, \
            COALESCE(SUM(act.thread_count), 0) AS thread_count, \
            MIN(act.first_email_date) AS first_email_date, \
            MAX(act.last_email_date) AS last_email_date, \
            COALESCE(ARRAY_REMOVE(ARRAY_AGG(DISTINCT ml.slug), NULL), ARRAY[]::text[]) AS mailing_lists, \
            COALESCE((SELECT ARRAY_AGG(alias.name ORDER BY alias.usage_count DESC) FROM author_name_aliases alias WHERE alias.author_id = a.id), ARRAY[]::text[]) AS name_variations \
        FROM authors a \
        LEFT JOIN author_mailing_list_activity act ON act.author_id = a.id \
        LEFT JOIN mailing_lists ml ON ml.id = act.mailing_list_id",
    );

    apply_author_filters(
        &mut data_builder,
        list_slug.as_deref(),
        normalized_query.as_deref(),
    );
    data_builder.push(" GROUP BY a.id");
    data_builder.push(" ORDER BY ");
    data_builder.push(order_sql);
    data_builder.push(" LIMIT ");
    data_builder.push_bind(page_size);
    data_builder.push(" OFFSET ");
    data_builder.push_bind(offset);

    let rows: Vec<DbAuthorRow> = data_builder.build_query_as().fetch_all(&mut **db).await?;

    let authors: Vec<AuthorWithStats> = rows.into_iter().map(AuthorWithStats::from).collect();

    let mut meta = ResponseMeta::default()
        .with_pagination(PaginationMeta::new(page, page_size, total))
        .with_sort(sort_meta);

    let mut filters = JsonMap::new();
    if let Some(slug) = list_slug {
        filters.insert("listSlug".to_string(), JsonValue::String(slug));
    }
    if let Some(raw_query) = params.raw_query() {
        filters.insert("q".to_string(), JsonValue::String(raw_query));
    }
    if !filters.is_empty() {
        meta = meta.with_filters(filters);
    }

    Ok(Json(ApiResponse::with_meta(authors, meta)))
}

#[openapi(tag = "Authors")]
#[get("/authors/<author_id>")]
pub async fn get_author(
    author_id: i32,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<AuthorWithStats>>, ApiError> {
    let mut builder = QueryBuilder::new(
        "SELECT \
            a.id, a.email, a.canonical_name, a.first_seen, a.last_seen, \
            COALESCE(SUM(act.email_count), 0) AS email_count, \
            COALESCE(SUM(act.thread_count), 0) AS thread_count, \
            MIN(act.first_email_date) AS first_email_date, \
            MAX(act.last_email_date) AS last_email_date, \
            COALESCE(ARRAY_REMOVE(ARRAY_AGG(DISTINCT ml.slug), NULL), ARRAY[]::text[]) AS mailing_lists, \
            COALESCE((SELECT ARRAY_AGG(alias.name ORDER BY alias.usage_count DESC) FROM author_name_aliases alias WHERE alias.author_id = a.id), ARRAY[]::text[]) AS name_variations \
        FROM authors a \
        LEFT JOIN author_mailing_list_activity act ON act.author_id = a.id \
        LEFT JOIN mailing_lists ml ON ml.id = act.mailing_list_id",
    );
    builder.push(" WHERE a.id = ");
    builder.push_bind(author_id);
    builder.push(" GROUP BY a.id");

    let row: Option<DbAuthorRow> = builder.build_query_as().fetch_optional(&mut **db).await?;

    let row = match row {
        Some(row) => row,
        None => return Err(ApiError::NotFound(format!("Author {author_id} not found"))),
    };

    Ok(Json(ApiResponse::new(AuthorWithStats::from(row))))
}

#[openapi(tag = "Authors")]
#[get("/authors/<author_id>/lists/<slug>/emails?<params..>")]
pub async fn get_author_emails(
    author_id: i32,
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<PaginationParams>,
) -> Result<Json<ApiResponse<Vec<EmailWithAuthor>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;

    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let total: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM emails WHERE mailing_list_id = $1 AND author_id = $2")
            .bind(mailing_list_id)
            .bind(author_id)
            .fetch_one(&mut **db)
            .await?;

    let emails = sqlx::query_as::<_, EmailWithAuthor>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name AS author_name, a.email AS author_email,
            e.patch_type, e.is_patch_only, e.patch_metadata
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        WHERE e.mailing_list_id = $1 AND e.author_id = $2
        ORDER BY e.date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    let meta = ResponseMeta::default()
        .with_list_id(slug)
        .with_pagination(PaginationMeta::new(page, page_size, total.0));

    Ok(Json(ApiResponse::with_meta(emails, meta)))
}

#[openapi(tag = "Authors")]
#[get("/authors/<author_id>/lists/<slug>/threads-started?<params..>")]
pub async fn get_author_threads_started(
    author_id: i32,
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadListParams>,
) -> Result<Json<ApiResponse<Vec<ThreadWithStarter>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;
    let sort_values = params.sort();
    let (order_clauses, sort_meta) = parse_thread_sorts(&sort_values);
    let order_sql = order_clauses.join(", ");

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM threads WHERE mailing_list_id = $1 AND root_author_id = $2",
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;

    let query = format!(
        r#"
        SELECT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
               CAST(t.message_count AS INTEGER) AS message_count,
               a.id AS starter_id,
               a.canonical_name AS starter_name,
               a.email AS starter_email
        FROM threads t
        JOIN authors a ON a.id = $2
        WHERE t.mailing_list_id = $1 AND t.root_author_id = $2
        ORDER BY {order_sql}
        LIMIT $3 OFFSET $4
        "#
    );

    let threads = sqlx::query_as::<_, ThreadWithStarter>(&query)
        .bind(mailing_list_id)
        .bind(author_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    let meta = ResponseMeta::default()
        .with_list_id(slug)
        .with_sort(sort_meta)
        .with_pagination(PaginationMeta::new(page, page_size, total.0));

    Ok(Json(ApiResponse::with_meta(threads, meta)))
}

#[openapi(tag = "Authors")]
#[get("/authors/<author_id>/lists/<slug>/threads-participated?<params..>")]
pub async fn get_author_threads_participated(
    author_id: i32,
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadListParams>,
) -> Result<Json<ApiResponse<Vec<ThreadWithStarter>>>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let page_size = params.page_size();
    let offset = (page - 1) * page_size;
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;
    let sort_values = params.sort();
    let (order_clauses, sort_meta) = parse_thread_sorts(&sort_values);
    let order_sql = order_clauses.join(", ");

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM thread_memberships tm
        JOIN threads t ON t.id = tm.thread_id AND t.mailing_list_id = tm.mailing_list_id
        WHERE tm.mailing_list_id = $1 AND tm.author_id = $2
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;

    let query = format!(
        r#"
        SELECT DISTINCT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
               CAST(t.message_count AS INTEGER) AS message_count,
               sa.id AS starter_id,
               sa.canonical_name AS starter_name,
               sa.email AS starter_email
        FROM thread_memberships tm
        JOIN threads t ON t.id = tm.thread_id AND t.mailing_list_id = tm.mailing_list_id
        JOIN emails e ON e.message_id = t.root_message_id AND e.mailing_list_id = t.mailing_list_id
        JOIN authors sa ON sa.id = e.author_id
        WHERE tm.mailing_list_id = $1 AND tm.author_id = $2
        ORDER BY {order_sql}
        LIMIT $3 OFFSET $4
        "#
    );

    let threads = sqlx::query_as::<_, ThreadWithStarter>(&query)
        .bind(mailing_list_id)
        .bind(author_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    let meta = ResponseMeta::default()
        .with_list_id(slug)
        .with_sort(sort_meta)
        .with_pagination(PaginationMeta::new(page, page_size, total.0));

    Ok(Json(ApiResponse::with_meta(threads, meta)))
}
