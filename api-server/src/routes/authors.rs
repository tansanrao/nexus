use rocket::serde::json::Json;
use rocket::get;
use rocket_db_pools::Connection;

use crate::db::LinuxKbDb;
use crate::error::ApiError;
use crate::models::{Author, AuthorWithStats, EmailWithAuthor, ThreadWithStarter, Thread};

#[get("/<slug>/authors?<search>&<page>&<limit>&<sort_by>&<order>")]
pub async fn search_authors(
    slug: String,
    mut db: Connection<LinuxKbDb>,
    search: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<Json<Vec<AuthorWithStats>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    // Parse sort parameters with defaults
    let sort_field = match sort_by.as_deref() {
        Some("canonical_name") => "canonical_name",
        Some("email") => "email",
        Some("email_count") => "email_count",
        Some("thread_count") => "thread_count",
        Some("first_email_date") => "first_email_date",
        Some("last_email_date") => "last_email_date",
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

    let author_rows: Vec<AuthorRow> = if let Some(search_term) = search {
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
            .bind(limit)
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
            .bind(limit)
            .bind(offset)
            .fetch_all(&mut **db)
            .await?
    };

    // Enrich with mailing lists and name variations
    let mut authors: Vec<AuthorWithStats> = Vec::new();
    for row in author_rows {
        // Get mailing lists this author participates in
        let mailing_list_slugs: Vec<(String,)> = sqlx::query_as(
            r#"SELECT ml.slug FROM author_mailing_list_activity act
               JOIN mailing_lists ml ON act.mailing_list_id = ml.id
               WHERE act.author_id = $1"#
        )
        .bind(row.id)
        .fetch_all(&mut **db)
        .await?;

        // Get name variations
        let name_variations: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT name FROM author_name_aliases WHERE author_id = $1 ORDER BY usage_count DESC"
        )
        .bind(row.id)
        .fetch_all(&mut **db)
        .await?;

        authors.push(AuthorWithStats {
            id: row.id,
            email: row.email,
            canonical_name: row.canonical_name,
            first_seen: row.first_seen,
            last_seen: row.last_seen,
            email_count: row.email_count,
            thread_count: row.thread_count,
            first_email_date: row.first_email_date,
            last_email_date: row.last_email_date,
            mailing_lists: mailing_list_slugs.into_iter().map(|(s,)| s).collect(),
            name_variations: name_variations.into_iter().map(|(n,)| n).collect(),
        });
    }

    Ok(Json(authors))
}

#[get("/<slug>/authors/<author_id>")]
pub async fn get_author(
    slug: String,
    mut db: Connection<LinuxKbDb>,
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
        "SELECT DISTINCT name FROM author_name_aliases WHERE author_id = $1 ORDER BY usage_count DESC"
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

#[get("/<slug>/authors/<author_id>/emails?<page>&<limit>")]
pub async fn get_author_emails(
    slug: String,
    mut db: Connection<LinuxKbDb>,
    author_id: i32,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<Vec<EmailWithAuthor>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

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
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(emails))
}

#[get("/<slug>/authors/<author_id>/threads-started?<page>&<limit>")]
pub async fn get_author_threads_started(
    slug: String,
    mut db: Connection<LinuxKbDb>,
    author_id: i32,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<Vec<ThreadWithStarter>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

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
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(threads))
}

#[get("/<slug>/authors/<author_id>/threads-participated?<page>&<limit>")]
pub async fn get_author_threads_participated(
    slug: String,
    mut db: Connection<LinuxKbDb>,
    author_id: i32,
    page: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<Vec<Thread>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

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
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut **db)
    .await?;

    Ok(Json(threads))
}
