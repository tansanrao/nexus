//! Thread-focused REST endpoints.
//!
//! Provides listing, detail retrieval, and search capabilities for threads
//! within a specific mailing list. Query parameters are parsed via helpers in
//! [`crate::routes::params`] to keep OpenAPI metadata in sync with Rocket.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{EmailHierarchy, PaginatedResponse, Thread, ThreadDetail};
use crate::routes::{
    helpers::resolve_mailing_list_id,
    params::{ThreadListParams, ThreadSearchParams, ThreadSearchType},
};
use rocket::{get, serde::json::Json};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;
use std::collections::HashMap;

/// List threads in a mailing list with pagination and sorting.
#[openapi(tag = "Threads")]
#[get("/<slug>/threads?<params..>")]
pub async fn list_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadListParams>,
) -> Result<Json<PaginatedResponse<Thread>>, ApiError> {
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
        SELECT id, mailing_list_id, root_message_id, subject, start_date, last_date,
               CAST(message_count AS INTEGER) as message_count
        FROM threads
        WHERE mailing_list_id = $1
        ORDER BY {} {}
        LIMIT $2 OFFSET $3
        "#,
        sort_column, sort_order
    );

    let threads = sqlx::query_as::<_, Thread>(&query)
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
            CAST(COALESCE(tm.depth, 0) AS INTEGER) as depth
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

/// Search threads inside a mailing list by subject or full-text content.
#[openapi(tag = "Threads")]
#[get("/<slug>/threads/search?<params..>")]
pub async fn search_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    params: Option<ThreadSearchParams>,
) -> Result<Json<PaginatedResponse<Thread>>, ApiError> {
    let params = params.unwrap_or_default();
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let page = params.page();
    let size = params.size();
    let Some(search_term) = params.normalized_query() else {
        return Ok(Json(PaginatedResponse::new(Vec::new(), page, size, 0)));
    };
    let offset = (page - 1) * size;
    let sort_column = params.sort_column();
    let sort_order = params.sort_order();
    let search_mode = params.search_type();
    let search_pattern = format!("%{search_term}%");

    let base_count_query = r#"
        SELECT COUNT(DISTINCT t.id)
        FROM threads t
        LEFT JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
        LEFT JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
        WHERE t.mailing_list_id = $1
    "#;

    let count_query = if matches!(search_mode, ThreadSearchType::FullText) {
        format!(
            r#"
            {}
              AND (LOWER(t.subject) LIKE $2
               OR LOWER(e.subject) LIKE $2
               OR LOWER(e.body) LIKE $2)
            "#,
            base_count_query
        )
    } else {
        format!(
            r#"
            {}
              AND (LOWER(t.subject) LIKE $2
               OR LOWER(e.subject) LIKE $2)
            "#,
            base_count_query
        )
    };

    let total: (i64,) = sqlx::query_as(&count_query)
        .bind(mailing_list_id)
        .bind(&search_pattern)
        .fetch_one(&mut **db)
        .await?;
    let total_elements = total.0;

    let base_select = r#"
        SELECT DISTINCT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
               CAST(t.message_count AS INTEGER) as message_count
        FROM threads t
        LEFT JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
        LEFT JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
        WHERE t.mailing_list_id = $1
    "#;

    let query = if matches!(search_mode, ThreadSearchType::FullText) {
        format!(
            r#"
            {}
              AND (LOWER(t.subject) LIKE $2
               OR LOWER(e.subject) LIKE $2
               OR LOWER(e.body) LIKE $2)
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            base_select, sort_column, sort_order
        )
    } else {
        format!(
            r#"
            {}
              AND (LOWER(t.subject) LIKE $2
               OR LOWER(e.subject) LIKE $2)
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            base_select, sort_column, sort_order
        )
    };

    let threads = sqlx::query_as::<_, Thread>(&query)
        .bind(mailing_list_id)
        .bind(&search_pattern)
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
