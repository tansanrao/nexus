use rocket::serde::json::Json;
use rocket::get;
use rocket_db_pools::{sqlx, Connection};

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::EmailWithAuthor;

#[get("/<slug>/emails/<email_id>")]
pub async fn get_email(
    slug: String,
    mut db: Connection<NexusDb>,
    email_id: i32,
) -> Result<Json<EmailWithAuthor>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let email = sqlx::query_as::<_, EmailWithAuthor>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name as author_name, a.email as author_email
        FROM emails e
        JOIN authors a ON e.author_id = a.id
        WHERE e.mailing_list_id = $1 AND e.id = $2
        "#,
    )
    .bind(mailing_list_id)
    .bind(email_id)
    .fetch_one(&mut **db)
    .await?;

    Ok(Json(email))
}
