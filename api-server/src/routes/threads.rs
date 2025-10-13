use rocket::serde::json::Json;
use rocket::get;
use rocket_db_pools::{sqlx, Connection};
use std::collections::HashMap;

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{EmailHierarchy, Thread, ThreadDetail, SearchType};

#[get("/<slug>/threads?<page>&<limit>&<sort_by>&<order>")]
pub async fn list_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    page: Option<i64>,
    limit: Option<i64>,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<Json<Vec<Thread>>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50).min(100); // Max 100 items per page
    let offset = (page - 1) * limit;

    // Parse sort parameters with defaults
    let sort_field = match sort_by.as_deref() {
        Some("start_date") => "start_date",
        Some("last_date") => "last_date",
        Some("message_count") => "message_count",
        _ => "last_date", // default
    };

    let sort_order = match order.as_deref() {
        Some("asc") => "ASC",
        Some("desc") => "DESC",
        _ => "DESC", // default
    };

    // Build query with dynamic ORDER BY and mailing_list_id filter
    let query = format!(
        r#"
        SELECT id, mailing_list_id, root_message_id, subject, start_date, last_date,
               CAST(message_count AS INTEGER) as message_count
        FROM threads
        WHERE mailing_list_id = $1
        ORDER BY {} {}
        LIMIT $2 OFFSET $3
        "#,
        sort_field, sort_order
    );

    let threads = sqlx::query_as::<_, Thread>(&query)
        .bind(mailing_list_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    Ok(Json(threads))
}

#[get("/<slug>/threads/<thread_id>")]
pub async fn get_thread(
    slug: String,
    mut db: Connection<NexusDb>,
    thread_id: i32,
) -> Result<Json<ThreadDetail>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    // Get thread info
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

    // Get all emails in thread with author info and depth
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

    // Sort emails in thread order (depth-first traversal)
    // This ensures each email appears immediately after its parent
    emails = sort_emails_by_thread_order(emails);

    Ok(Json(ThreadDetail { thread, emails }))
}

/// Sort emails in depth-first, pre-order traversal
///
/// This ensures that:
/// 1. Each email appears immediately after its parent
/// 2. Siblings are ordered by date
/// 3. The tree structure is preserved for correct UI rendering
fn sort_emails_by_thread_order(emails: Vec<EmailHierarchy>) -> Vec<EmailHierarchy> {
    // Build a map of message_id -> email for quick lookup
    let email_map: HashMap<String, &EmailHierarchy> = emails
        .iter()
        .map(|e| (e.message_id.clone(), e))
        .collect();

    // Build a map of parent message_id -> children emails
    let mut children_map: HashMap<Option<String>, Vec<&EmailHierarchy>> = HashMap::new();
    for email in &emails {
        children_map
            .entry(email.in_reply_to.clone())
            .or_insert_with(Vec::new)
            .push(email);
    }

    // Sort children by date within each parent
    for children in children_map.values_mut() {
        children.sort_by(|a, b| a.date.cmp(&b.date));
    }

    // Perform depth-first traversal starting from root (emails with no parent)
    let mut result = Vec::new();

    // Helper function to recursively add email and its children
    fn add_email_and_children(
        email: &EmailHierarchy,
        children_map: &HashMap<Option<String>, Vec<&EmailHierarchy>>,
        result: &mut Vec<EmailHierarchy>,
    ) {
        result.push(email.clone());

        // Add children in date order
        if let Some(children) = children_map.get(&Some(email.message_id.clone())) {
            for child in children {
                add_email_and_children(child, children_map, result);
            }
        }
    }

    // Start with emails that have no parent (roots)
    if let Some(roots) = children_map.get(&None) {
        for root in roots {
            add_email_and_children(root, &children_map, &mut result);
        }
    }

    // Handle orphans (emails whose parent is not in this thread - phantoms)
    // These should be processed as sub-trees
    for email in &emails {
        if let Some(ref parent_msg_id) = email.in_reply_to {
            if !email_map.contains_key(parent_msg_id) && !result.iter().any(|e| e.id == email.id) {
                // This is an orphan - its parent (phantom) is not in this thread
                add_email_and_children(email, &children_map, &mut result);
            }
        }
    }

    result
}

#[get("/<slug>/threads/search?<search>&<search_type>&<page>&<limit>&<sort_by>&<order>")]
pub async fn search_threads(
    slug: String,
    mut db: Connection<NexusDb>,
    search: Option<String>,
    search_type: Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
    sort_by: Option<String>,
    order: Option<String>,
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

    // Parse sort parameters with defaults
    let sort_field = match sort_by.as_deref() {
        Some("start_date") => "start_date",
        Some("last_date") => "last_date",
        Some("message_count") => "message_count",
        _ => "last_date", // default
    };

    let sort_order = match order.as_deref() {
        Some("asc") => "ASC",
        Some("desc") => "DESC",
        _ => "DESC", // default
    };

    // If no search term provided, return empty results
    let Some(search_term) = search else {
        return Ok(Json(vec![]));
    };

    // Parse search type, default to subject
    let search_mode = match search_type.as_deref() {
        Some("full_text") => SearchType::FullText,
        _ => SearchType::Subject, // default
    };

    // Search in thread subjects and email bodies
    let search_pattern = format!("%{}%", search_term.to_lowercase());

    // Build query based on search type
    let query = match search_mode {
        SearchType::Subject => {
            // Search only in subject lines (thread and email subjects)
            format!(
                r#"
                SELECT DISTINCT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
                       CAST(t.message_count AS INTEGER) as message_count
                FROM threads t
                LEFT JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
                LEFT JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
                WHERE t.mailing_list_id = $1
                  AND (LOWER(t.subject) LIKE $2
                   OR LOWER(e.subject) LIKE $2)
                ORDER BY {} {}
                LIMIT $3 OFFSET $4
                "#,
                sort_field, sort_order
            )
        }
        SearchType::FullText => {
            // Search in subject lines AND email bodies
            format!(
                r#"
                SELECT DISTINCT t.id, t.mailing_list_id, t.root_message_id, t.subject, t.start_date, t.last_date,
                       CAST(t.message_count AS INTEGER) as message_count
                FROM threads t
                LEFT JOIN thread_memberships tm ON t.id = tm.thread_id AND tm.mailing_list_id = $1
                LEFT JOIN emails e ON tm.email_id = e.id AND e.mailing_list_id = $1
                WHERE t.mailing_list_id = $1
                  AND (LOWER(t.subject) LIKE $2
                   OR LOWER(e.subject) LIKE $2
                   OR LOWER(e.body) LIKE $2)
                ORDER BY {} {}
                LIMIT $3 OFFSET $4
                "#,
                sort_field, sort_order
            )
        }
    };

    let threads = sqlx::query_as::<_, Thread>(&query)
        .bind(mailing_list_id)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&mut **db)
        .await?;

    Ok(Json(threads))
}
