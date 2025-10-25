//! Thread-focused REST endpoints.
//!
//! Provides listing, detail retrieval, and search capabilities for threads
//! within a specific mailing list. Query parameters are parsed via helpers in
//! [`crate::routes::params`] to keep OpenAPI metadata in sync with Rocket.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{
    EmailHierarchy, PaginatedResponse, Thread, ThreadDetail, ThreadSearchHit, ThreadSearchResponse,
    ThreadWithStarter,
};
use crate::routes::{
    helpers::resolve_mailing_list_id,
    params::{ThreadListParams, ThreadSearchParams},
};
use chrono::{DateTime, Utc};
use rocket::{get, serde::json::Json};
use rocket_db_pools::sqlx::Row;
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;
use std::collections::HashMap;

const THREAD_RECENCY_WEIGHT: f64 = 0.35;
const THREAD_RECENCY_HALF_LIFE_SECONDS: f64 = 31.0 * 24.0 * 3600.0;

/// List threads in a mailing list with pagination and sorting.
#[openapi(tag = "Threads")]
#[get("/<slug>/threads?<params..>")]
pub async fn list_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadListParams>,
) -> Result<Json<PaginatedResponse<ThreadWithStarter>>, ApiError> {
    let params = params.unwrap_or_default();
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let page = params.page();
    let size = params.size();
    let offset = (page - 1) * size;
    let sort_column = params.sort_column();
    let sort_order = params.sort_order();

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threads WHERE mailing_list_id = $1")
        .bind(mailing_list_id)
        .fetch_one(&mut **db)
        .await?;
    let total_elements = total.0;

    let query = format!(
        r#"
        SELECT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
               CAST(t.message_count AS INTEGER) as message_count,
               e.author_id as starter_id,
               a.canonical_name as starter_name,
               a.email as starter_email
        FROM threads t
        JOIN emails e ON t.root_message_id = e.message_id AND t.mailing_list_id = e.mailing_list_id
        JOIN authors a ON e.author_id = a.id
        WHERE t.mailing_list_id = $1
        ORDER BY {} {}
        LIMIT $2 OFFSET $3
        "#,
        sort_column, sort_order
    );

    let threads = sqlx::query_as::<_, ThreadWithStarter>(&query)
        .bind(mailing_list_id)
        .bind(size)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    Ok(Json(PaginatedResponse::new(
        threads,
        page,
        size,
        total_elements,
    )))
}

/// Retrieve thread metadata and the threaded email hierarchy.
#[openapi(tag = "Threads")]
#[get("/<slug>/threads/<thread_id>")]
pub async fn get_thread(
    slug: String,
    mut db: Connection<NexusDb>,
    thread_id: i32,
) -> Result<Json<ThreadDetail>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let thread = sqlx::query_as::<_, Thread>(
        r#"
        SELECT id, mailing_list_id, root_message_id, subject, start_date, last_date,
               CAST(message_count AS INTEGER) as message_count
        FROM threads
        WHERE mailing_list_id = $1 AND id = $2
        "#,
    )
    .bind(mailing_list_id)
    .bind(thread_id)
    .fetch_one(&mut **db)
    .await?;

    let mut emails = sqlx::query_as::<_, EmailHierarchy>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name as author_name, a.email as author_email,
            CAST(COALESCE(tm.depth, 0) AS INTEGER) as depth,
            e.patch_type, e.is_patch_only, e.patch_metadata
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        JOIN thread_memberships tm ON e.id = tm.email_id AND tm.mailing_list_id = $1
        WHERE tm.thread_id = $2 AND tm.mailing_list_id = $1
        "#,
    )
    .bind(mailing_list_id)
    .bind(thread_id)
    .fetch_all(&mut **db)
    .await?;

    emails = sort_emails_by_thread_order(emails);

    Ok(Json(ThreadDetail { thread, emails }))
}

/// Search threads inside a mailing list using lexical ranking.
#[openapi(tag = "Threads")]
#[get("/<slug>/threads/search?<params..>")]
pub async fn search_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadSearchParams>,
) -> Result<Json<ThreadSearchResponse>, ApiError> {
    let params = params.unwrap_or_default();
    let page = params.page();
    let size = params.size();
    let query = match params.query() {
        Some(value) => value.to_string(),
        None => {
            return Ok(Json(ThreadSearchResponse {
                query: String::new(),
                page,
                size,
                total: 0,
                results: Vec::new(),
            }));
        }
    };

    let start_bound = params.start_date_utc();
    let end_bound = params.end_date_utc();
    if let (Some(start), Some(end)) = (start_bound.as_ref(), end_bound.as_ref()) {
        if start > end {
            return Err(ApiError::BadRequest(
                "startDate must be on or before endDate".to_string(),
            ));
        }
    }

    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;
    let offset = (page - 1) * size;
    let (hits, total) = lexical_search(
        &mut **db,
        mailing_list_id,
        &query,
        size,
        offset,
        start_bound,
        end_bound,
    )
    .await
    .map_err(ApiError::from)?;

    Ok(Json(ThreadSearchResponse {
        query,
        page,
        size,
        total,
        results: hits,
    }))
}

async fn lexical_search(
    conn: &mut sqlx::PgConnection,
    mailing_list_id: i32,
    query: &str,
    limit: i64,
    offset: i64,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<(Vec<ThreadSearchHit>, i64), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH query AS (
            SELECT websearch_to_tsquery('english', $2) AS tsq,
                   $8::double precision AS half_life_seconds
        ),
        filtered_threads AS (
            SELECT t.*
            FROM threads t
            WHERE t.mailing_list_id = $1
              AND ($5::timestamptz IS NULL OR t.last_date >= $5::timestamptz)
              AND ($6::timestamptz IS NULL OR t.start_date <= $6::timestamptz)
        ),
        thread_docs AS (
            SELECT
                t.id,
                t.mailing_list_id,
                t.last_date,
                setweight(to_tsvector('english', COALESCE(t.subject, '')), 'A') ||
                setweight(to_tsvector('english', COALESCE(starter.search_body, '')), 'B') ||
                setweight(to_tsvector('english', COALESCE(rest.tail_text, '')), 'D') AS search_vector
            FROM filtered_threads t
            LEFT JOIN emails starter
                ON starter.message_id = t.root_message_id
               AND starter.mailing_list_id = t.mailing_list_id
            LEFT JOIN LATERAL (
                SELECT string_agg(COALESCE(e.search_body, ''), ' ' ORDER BY e.date) AS tail_text
                FROM thread_memberships tm
                JOIN emails e ON e.id = tm.email_id
                WHERE tm.thread_id = t.id
                  AND tm.mailing_list_id = t.mailing_list_id
                  AND (starter.id IS NULL OR e.id <> starter.id)
            ) rest ON TRUE
        ),
        ranked AS (
            SELECT
                td.id,
                td.mailing_list_id,
                td.last_date,
                ts_rank_cd(td.search_vector, query.tsq) AS text_score,
                exp(-GREATEST(0, EXTRACT(EPOCH FROM ((NOW() AT TIME ZONE 'utc') - td.last_date))) /
                    query.half_life_seconds) AS recency_factor
            FROM thread_docs td
            CROSS JOIN query
            WHERE td.search_vector @@ query.tsq
        )
        SELECT
            t.id,
            t.mailing_list_id,
            t.root_message_id,
            t.subject,
            t.start_date,
            t.last_date,
            CAST(t.message_count AS INTEGER) AS message_count,
            starter.author_id AS starter_id,
            a.canonical_name AS starter_name,
            a.email AS starter_email,
            (ranked.text_score * (1.0 + $7::double precision * ranked.recency_factor))::float4 AS blended_score
        FROM ranked
        JOIN threads t
          ON t.id = ranked.id
         AND t.mailing_list_id = ranked.mailing_list_id
        JOIN emails starter
          ON t.root_message_id = starter.message_id
         AND t.mailing_list_id = starter.mailing_list_id
        JOIN authors a ON starter.author_id = a.id
        ORDER BY blended_score DESC, t.last_date DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(mailing_list_id)
    .bind(query)
    .bind(limit)
    .bind(offset)
    .bind(start_date)
    .bind(end_date)
    .bind(THREAD_RECENCY_WEIGHT)
    .bind(THREAD_RECENCY_HALF_LIFE_SECONDS)
    .fetch_all(&mut *conn)
    .await?;

    let mut max_score = 0.0_f32;
    let mut raw_hits = Vec::with_capacity(rows.len());
    for row in rows {
        let thread = ThreadWithStarter {
            id: row.try_get("id")?,
            mailing_list_id: row.try_get("mailing_list_id")?,
            root_message_id: row.try_get("root_message_id")?,
            subject: row.try_get("subject")?,
            start_date: row.try_get("start_date")?,
            last_date: row.try_get("last_date")?,
            message_count: row.try_get("message_count")?,
            starter_id: row.try_get("starter_id")?,
            starter_name: row.try_get("starter_name")?,
            starter_email: row.try_get("starter_email")?,
        };
        let blended_score: f32 = row.try_get("blended_score")?;
        max_score = max_score.max(blended_score);
        raw_hits.push((thread, blended_score));
    }

    let mut hits = Vec::with_capacity(raw_hits.len());
    for (thread, score) in raw_hits {
        let mut hit = ThreadSearchHit::from_thread(thread);
        if max_score > 0.0 {
            hit.lexical_score = Some((score / max_score).clamp(0.0, 1.0));
        } else {
            hit.lexical_score = Some(0.0);
        }
        hits.push(hit);
    }

    let total_row = sqlx::query(
        r#"
        WITH query AS (
            SELECT websearch_to_tsquery('english', $2) AS tsq
        ),
        filtered_threads AS (
            SELECT t.*
            FROM threads t
            WHERE t.mailing_list_id = $1
              AND ($3::timestamptz IS NULL OR t.last_date >= $3::timestamptz)
              AND ($4::timestamptz IS NULL OR t.start_date <= $4::timestamptz)
        ),
        thread_docs AS (
            SELECT
                t.id,
                setweight(to_tsvector('english', COALESCE(t.subject, '')), 'A') ||
                setweight(to_tsvector('english', COALESCE(starter.search_body, '')), 'B') ||
                setweight(to_tsvector('english', COALESCE(rest.tail_text, '')), 'D') AS search_vector
            FROM filtered_threads t
            LEFT JOIN emails starter
                ON starter.message_id = t.root_message_id
               AND starter.mailing_list_id = t.mailing_list_id
            LEFT JOIN LATERAL (
                SELECT string_agg(COALESCE(e.search_body, ''), ' ' ORDER BY e.date) AS tail_text
                FROM thread_memberships tm
                JOIN emails e ON e.id = tm.email_id
                WHERE tm.thread_id = t.id
                  AND tm.mailing_list_id = t.mailing_list_id
                  AND (starter.id IS NULL OR e.id <> starter.id)
            ) rest ON TRUE
        ),
        ranked AS (
            SELECT td.id
            FROM thread_docs td
            CROSS JOIN query
            WHERE td.search_vector @@ query.tsq
        )
        SELECT COUNT(*) FROM ranked
        "#,
    )
    .bind(mailing_list_id)
    .bind(query)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(&mut *conn)
    .await?;
    let total: i64 = total_row.try_get(0)?;

    Ok((hits, total))
}

/// Sort emails into a depth-first order for deterministic thread rendering.
fn sort_emails_by_thread_order(emails: Vec<EmailHierarchy>) -> Vec<EmailHierarchy> {
    let email_map: HashMap<String, &EmailHierarchy> =
        emails.iter().map(|e| (e.message_id.clone(), e)).collect();

    let mut children_map: HashMap<Option<String>, Vec<&EmailHierarchy>> = HashMap::new();
    for email in &emails {
        children_map
            .entry(email.in_reply_to.clone())
            .or_insert_with(Vec::new)
            .push(email);
    }

    for children in children_map.values_mut() {
        children.sort_by(|a, b| a.date.cmp(&b.date));
    }

    let mut result = Vec::new();

    fn add_email_and_children(
        email: &EmailHierarchy,
        children_map: &HashMap<Option<String>, Vec<&EmailHierarchy>>,
        result: &mut Vec<EmailHierarchy>,
    ) {
        result.push(email.clone());

        if let Some(children) = children_map.get(&Some(email.message_id.clone())) {
            for child in children {
                add_email_and_children(child, children_map, result);
            }
        }
    }

    if let Some(roots) = children_map.get(&None) {
        for root in roots {
            add_email_and_children(root, &children_map, &mut result);
        }
    }

    for email in &emails {
        if let Some(ref parent_msg_id) = email.in_reply_to {
            if !email_map.contains_key(parent_msg_id) && !result.iter().any(|e| e.id == email.id) {
                add_email_and_children(email, &children_map, &mut result);
            }
        }
    }

    result
}
