use rocket::serde::json::Json;
use rocket::get;
use rocket_db_pools::{sqlx, Connection};
use std::collections::HashMap;
use rocket_okapi::openapi;

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{AuthorWithStats, EmailWithAuthor, ThreadWithStarter, Thread, PaginatedResponse};

#[openapi(tag = "Authors")]
#[get("/<slug>/authors?<q>&<page>&<size>&<sortBy>&<order>")]
pub async fn search_authors(
    slug: String,
    mut db: Connection<NexusDb>,
    q: Option<String>,
    page: Option<i64>,
    size: Option<i64>,
    #[allow(non_snake_case)]
    sortBy: Option<String>,
    order: Option<String>,
) -> Result<Json<PaginatedResponse<AuthorWithStats>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let size = size.unwrap_or(50).min(100);
    let offset = (page - 1) * size;

    // Parse sort parameters with defaults
    let sort_field = match sortBy.as_deref() {
        Some("canonicalName") => "canonical_name",
        Some("email") => "email",
        Some("emailCount") => "email_count",
        Some("threadCount") => "thread_count",
        Some("firstEmailDate") => "first_email_date",
        Some("lastEmailDate") => "last_email_date",
        _ => "email_count", // default: sort by most active
    };

    let sort_order = match order.as_deref() {
        Some("asc") => "ASC",
        Some("desc") => "DESC",
        _ => "DESC", // default
    };

    // Base query that gets authors active in this mailing list
    let base_query = r#"
        SELECT
            a.id, a.email, a.canonical_name, a.first_seen, a.last_seen,
            COALESCE(act.email_count, 0) as email_count,
            COALESCE(act.thread_count, 0) as thread_count,
            act.first_email_date,
            act.last_email_date
        FROM authors a
        INNER JOIN author_mailing_list_activity act ON a.id = act.author_id
        WHERE act.mailing_list_id = $1
    "#;

    #[derive(Debug, sqlx::FromRow)]
    struct AuthorRow {
        id: i32,
        email: String,
        canonical_name: Option<String>,
        first_seen: Option<chrono::DateTime<chrono::Utc>>,
        last_seen: Option<chrono::DateTime<chrono::Utc>>,
        email_count: i64,
        thread_count: i64,
        first_email_date: Option<chrono::DateTime<chrono::Utc>>,
        last_email_date: Option<chrono::DateTime<chrono::Utc>>,
    }

    // Get total count
    let total_elements: i64 = if let Some(ref search_term) = q {
        let search_pattern = format!("%{}%", search_term.to_lowercase());
        let count_query = r#"
            SELECT COUNT(*)
            FROM authors a
            INNER JOIN author_mailing_list_activity act ON a.id = act.author_id
            WHERE act.mailing_list_id = $1
              AND (LOWER(a.email) LIKE $2 OR LOWER(a.canonical_name) LIKE $2)
        "#;

        let total: (i64,) = sqlx::query_as(count_query)
            .bind(mailing_list_id)
            .bind(&search_pattern)
            .fetch_one(&mut **db)
            .await?;
        total.0
    } else {
        let count_query = r#"
            SELECT COUNT(*)
            FROM authors a
            INNER JOIN author_mailing_list_activity act ON a.id = act.author_id
            WHERE act.mailing_list_id = $1
        "#;

        let total: (i64,) = sqlx::query_as(count_query)
            .bind(mailing_list_id)
            .fetch_one(&mut **db)
            .await?;
        total.0
    };

    let author_rows: Vec<AuthorRow> = if let Some(search_term) = q {
        let search_pattern = format!("%{}%", search_term.to_lowercase());
        let query = format!(
            r#"
            {}
            AND (LOWER(a.email) LIKE $2 OR LOWER(a.canonical_name) LIKE $2)
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            base_query, sort_field, sort_order
        );

        sqlx::query_as::<_, AuthorRow>(&query)
            .bind(mailing_list_id)
            .bind(&search_pattern)
            .bind(size)
            .bind(offset)
            .fetch_all(&mut **db)
            .await?
    } else {
        let query = format!(
            r#"
            {}
            ORDER BY {} {}
            LIMIT $2 OFFSET $3
            "#,
            base_query, sort_field, sort_order
        );

        sqlx::query_as::<_, AuthorRow>(&query)
            .bind(mailing_list_id)
            .bind(size)
            .bind(offset)
            .fetch_all(&mut **db)
            .await?
    };

    // Batch fetch mailing lists and name variations for all authors to avoid N+1 queries
    let author_ids: Vec<i32> = author_rows.iter().map(|r| r.id).collect();

    // Fetch all mailing list slugs for all authors in one query
    let mailing_lists_data: Vec<(i32, String)> = if !author_ids.is_empty() {
        sqlx::query_as(
            r#"SELECT act.author_id, ml.slug
               FROM author_mailing_list_activity act
               JOIN mailing_lists ml ON act.mailing_list_id = ml.id
               WHERE act.author_id = ANY($1)"#
        )
        .bind(&author_ids)
        .fetch_all(&mut **db)
        .await?
    } else {
        Vec::new()
    };

    // Fetch all name variations for all authors in one query
    let name_variations_data: Vec<(i32, String)> = if !author_ids.is_empty() {
        sqlx::query_as(
            "SELECT author_id, name FROM author_name_aliases
             WHERE author_id = ANY($1) ORDER BY author_id, usage_count DESC"
        )
        .bind(&author_ids)
        .fetch_all(&mut **db)
        .await?
    } else {
        Vec::new()
    };

    // Build lookup maps
    let mut mailing_lists_map: HashMap<i32, Vec<String>> = HashMap::new();
    for (author_id, slug) in mailing_lists_data {
        mailing_lists_map.entry(author_id).or_insert_with(Vec::new).push(slug);
    }

    let mut name_variations_map: HashMap<i32, Vec<String>> = HashMap::new();
    for (author_id, name) in name_variations_data {
        name_variations_map.entry(author_id).or_insert_with(Vec::new).push(name);
    }

    // Build final author list with enriched data
    let authors: Vec<AuthorWithStats> = author_rows.into_iter().map(|row| {
        AuthorWithStats {
            id: row.id,
            email: row.email,
            canonical_name: row.canonical_name,
            first_seen: row.first_seen,
            last_seen: row.last_seen,
            email_count: row.email_count,
            thread_count: row.thread_count,
            first_email_date: row.first_email_date,
            last_email_date: row.last_email_date,
            mailing_lists: mailing_lists_map.get(&row.id).cloned().unwrap_or_default(),
            name_variations: name_variations_map.get(&row.id).cloned().unwrap_or_default(),
        }
    }).collect();

    Ok(Json(PaginatedResponse::new(authors, page, size, total_elements)))
}

#[openapi(tag = "Authors")]
#[get("/<slug>/authors/<author_id>")]
pub async fn get_author(
    slug: String,
    mut db: Connection<NexusDb>,
    author_id: i32,
) -> Result<Json<AuthorWithStats>, ApiError> {
    // Get mailing list ID from slug (for context, but we fetch global author)
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    #[derive(Debug, sqlx::FromRow)]
    struct AuthorRow {
        id: i32,
        email: String,
        canonical_name: Option<String>,
        first_seen: Option<chrono::DateTime<chrono::Utc>>,
        last_seen: Option<chrono::DateTime<chrono::Utc>>,
        email_count: i64,
        thread_count: i64,
        first_email_date: Option<chrono::DateTime<chrono::Utc>>,
        last_email_date: Option<chrono::DateTime<chrono::Utc>>,
    }

    // Get author info with stats from this specific mailing list
    let author_row = sqlx::query_as::<_, AuthorRow>(
        r#"
        SELECT
            a.id, a.email, a.canonical_name, a.first_seen, a.last_seen,
            COALESCE(act.email_count, 0) as email_count,
            COALESCE(act.thread_count, 0) as thread_count,
            act.first_email_date,
            act.last_email_date
        FROM authors a
        LEFT JOIN author_mailing_list_activity act ON a.id = act.author_id AND act.mailing_list_id = $1
        WHERE a.id = $2
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;

    // Get mailing lists this author participates in
    let mailing_list_slugs: Vec<(String,)> = sqlx::query_as(
        r#"SELECT ml.slug FROM author_mailing_list_activity act
           JOIN mailing_lists ml ON act.mailing_list_id = ml.id
           WHERE act.author_id = $1"#
    )
    .bind(author_id)
    .fetch_all(&mut **db)
    .await?;

    // Get name variations
    let name_variations: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM author_name_aliases WHERE author_id = $1 ORDER BY usage_count DESC"
    )
    .bind(author_id)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(AuthorWithStats {
        id: author_row.id,
        email: author_row.email,
        canonical_name: author_row.canonical_name,
        first_seen: author_row.first_seen,
        last_seen: author_row.last_seen,
        email_count: author_row.email_count,
        thread_count: author_row.thread_count,
        first_email_date: author_row.first_email_date,
        last_email_date: author_row.last_email_date,
        mailing_lists: mailing_list_slugs.into_iter().map(|(s,)| s).collect(),
        name_variations: name_variations.into_iter().map(|(n,)| n).collect(),
    }))
}

#[openapi(tag = "Authors")]
#[get("/<slug>/authors/<author_id>/emails?<page>&<size>")]
pub async fn get_author_emails(
    slug: String,
    mut db: Connection<NexusDb>,
    author_id: i32,
    page: Option<i64>,
    size: Option<i64>,
) -> Result<Json<PaginatedResponse<EmailWithAuthor>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let size = size.unwrap_or(50).min(100);
    let offset = (page - 1) * size;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM emails WHERE mailing_list_id = $1 AND author_id = $2"
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;
    let total_elements = total.0;

    let emails = sqlx::query_as::<_, EmailWithAuthor>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name as author_name, a.email as author_email
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        WHERE e.mailing_list_id = $1 AND e.author_id = $2
        ORDER BY e.date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .bind(size)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(PaginatedResponse::new(emails, page, size, total_elements)))
}

#[openapi(tag = "Authors")]
#[get("/<slug>/authors/<author_id>/threads-started?<page>&<size>")]
pub async fn get_author_threads_started(
    slug: String,
    mut db: Connection<NexusDb>,
    author_id: i32,
    page: Option<i64>,
    size: Option<i64>,
) -> Result<Json<PaginatedResponse<ThreadWithStarter>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let size = size.unwrap_or(50).min(100);
    let offset = (page - 1) * size;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*)
           FROM threads t
           JOIN emails e ON t.root_message_id = e.message_id AND e.mailing_list_id = $1
           WHERE t.mailing_list_id = $1 AND e.author_id = $2"#
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;
    let total_elements = total.0;

    // Get threads where this author sent the root (first) message
    let threads = sqlx::query_as::<_, ThreadWithStarter>(
        r#"
        SELECT
            t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
            CAST(t.message_count AS INTEGER) as message_count,
            a.id as starter_id, a.canonical_name as starter_name, a.email as starter_email
        FROM threads t
        JOIN emails e ON t.root_message_id = e.message_id AND e.mailing_list_id = $1
        JOIN authors a ON e.author_id = a.id
        WHERE t.mailing_list_id = $1 AND e.author_id = $2
        ORDER BY t.start_date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .bind(size)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(PaginatedResponse::new(threads, page, size, total_elements)))
}

#[openapi(tag = "Authors")]
#[get("/<slug>/authors/<author_id>/threads-participated?<page>&<size>")]
pub async fn get_author_threads_participated(
    slug: String,
    mut db: Connection<NexusDb>,
    author_id: i32,
    page: Option<i64>,
    size: Option<i64>,
) -> Result<Json<PaginatedResponse<Thread>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let size = size.unwrap_or(50).min(100);
    let offset = (page - 1) * size;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT t.id)
           FROM threads t
           JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
           JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
           WHERE t.mailing_list_id = $1 AND e.author_id = $2"#
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .fetch_one(&mut **db)
    .await?;
    let total_elements = total.0;

    // Get all threads where this author participated (sent any message)
    let threads = sqlx::query_as::<_, Thread>(
        r#"
        SELECT DISTINCT
            t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
            CAST(t.message_count AS INTEGER) as message_count
        FROM threads t
        JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
        JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
        WHERE t.mailing_list_id = $1 AND e.author_id = $2
        ORDER BY t.last_date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(mailing_list_id)
    .bind(author_id)
    .bind(size)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(PaginatedResponse::new(threads, page, size, total_elements)))
}
