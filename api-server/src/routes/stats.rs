use rocket::serde::json::Json;
use rocket::get;
use rocket_db_pools::{sqlx, Connection};

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::Stats;

#[get("/<slug>/stats")]
pub async fn get_stats(
    slug: String,
    mut db: Connection<NexusDb>,
) -> Result<Json<Stats>, ApiError> {
    // Get mailing list ID from slug
    let list: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(&slug)
        .fetch_one(&mut **db)
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{}' not found", slug)))?;
    let mailing_list_id = list.0;

    let stats = sqlx::query_as::<_, Stats>(
        r#"
        SELECT
            CAST((SELECT COUNT(*) FROM emails WHERE mailing_list_id = $1) AS BIGINT) as total_emails,
            CAST((SELECT COUNT(*) FROM threads WHERE mailing_list_id = $1) AS BIGINT) as total_threads,
            CAST((SELECT COUNT(*) FROM authors WHERE mailing_list_id = $1) AS BIGINT) as total_authors,
            (SELECT MIN(date) FROM emails WHERE mailing_list_id = $1) as date_range_start,
            (SELECT MAX(date) FROM emails WHERE mailing_list_id = $1) as date_range_end
        "#,
    )
    .bind(mailing_list_id)
    .fetch_one(&mut **db)
    .await?;

    Ok(Json(stats))
}
