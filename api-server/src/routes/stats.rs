//! Statistics endpoints for mailing lists.

use rocket::{get, serde::json::Json};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::Stats;
use crate::routes::helpers::resolve_mailing_list_id;

/// Retrieve aggregate statistics for a specific mailing list.
#[openapi(tag = "Stats")]
#[get("/<slug>/stats")]
pub async fn get_stats(slug: String, mut db: Connection<NexusDb>) -> Result<Json<Stats>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

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
