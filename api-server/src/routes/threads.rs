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
use rocket::{State, get, serde::json::Json};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;
use std::collections::HashMap;

use crate::search::{SearchService, ThreadDocument};

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
    search_service: &State<SearchService>,
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
    let semantic_ratio = params
        .semantic_ratio()
        .unwrap_or(search_service.default_semantic_ratio());

    let results = search_service
        .search_threads(
            &slug,
            mailing_list_id,
            &query,
            page,
            size,
            start_bound,
            end_bound,
            semantic_ratio,
        )
        .await?;

    let max_score = results
        .hits
        .iter()
        .filter_map(|hit| hit.ranking_score)
        .fold(0.0_f32, f32::max);

    let mut hits = Vec::with_capacity(results.hits.len());
    for hit in results.hits {
        let thread = thread_from_document(hit.document)?;
        let mut api_hit = ThreadSearchHit::from_thread(thread);
        if let Some(score) = hit.ranking_score {
            let normalized = if max_score > 0.0 {
                (score / max_score).clamp(0.0, 1.0)
            } else {
                0.0
            };
            api_hit.lexical_score = Some(normalized);
        }
        hits.push(api_hit);
    }

    Ok(Json(ThreadSearchResponse {
        query,
        page,
        size,
        total: results.total,
        results: hits,
    }))
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

fn thread_from_document(doc: ThreadDocument) -> Result<ThreadWithStarter, ApiError> {
    let start_date = doc.start_date().ok_or_else(|| {
        ApiError::InternalError(format!(
            "Thread {} missing start timestamp in search document",
            doc.thread_id
        ))
    })?;

    let last_date = doc.last_date().ok_or_else(|| {
        ApiError::InternalError(format!(
            "Thread {} missing last timestamp in search document",
            doc.thread_id
        ))
    })?;

    Ok(ThreadWithStarter {
        id: doc.thread_id,
        mailing_list_id: doc.mailing_list_id,
        root_message_id: doc.root_message_id,
        subject: doc.subject,
        start_date,
        last_date,
        message_count: Some(doc.message_count),
        starter_id: doc.starter_id,
        starter_name: doc.starter_name,
        starter_email: doc.starter_email,
    })
}
