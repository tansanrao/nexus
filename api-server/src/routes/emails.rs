//! Email-centric endpoints.

use rocket::{get, serde::json::Json};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::EmailWithAuthor;
use crate::routes::helpers::resolve_mailing_list_id;

/// Retrieve a single email in the context of a mailing list.
#[openapi(tag = "Emails")]
#[get("/<slug>/emails/<email_id>")]
pub async fn get_email(
    slug: String,
    mut db: Connection<NexusDb>,
    email_id: i32,
) -> Result<Json<EmailWithAuthor>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let email = sqlx::query_as::<_, EmailWithAuthor>(
        r#"
        SELECT
            e.id, e.mailing_list_id, e.message_id, e.git_commit_hash, e.author_id,
            e.subject, e.date, e.in_reply_to, e.body, e.created_at,
            a.canonical_name as author_name, a.email as author_email,
            e.patch_type, e.is_patch_only, e.patch_metadata
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
