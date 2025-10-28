//! Aggregate statistics endpoints for mailing lists.

use crate::db::NexusDb;
use crate::error::ApiError;
use crate::models::{ApiResponse, ListAggregateStats, MailingListStats, ResponseMeta};
use crate::routes::helpers::resolve_mailing_list_id;
use rocket::get;
use rocket::serde::json::Json;
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;

#[openapi(tag = "Lists")]
#[get("/lists/stats")]
pub async fn aggregate_stats(
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<ListAggregateStats>>, ApiError> {
    let row: (i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            (SELECT COUNT(*) FROM mailing_lists) AS total_lists,
            (SELECT COUNT(*) FROM emails) AS total_emails,
            (SELECT COUNT(*) FROM threads) AS total_threads,
            (SELECT COUNT(DISTINCT author_id) FROM emails) AS total_authors
        "#,
    )
    .fetch_one(&mut **db)
    .await?;

    let stats = ListAggregateStats {
        total_lists: row.0,
        total_emails: row.1,
        total_threads: row.2,
        total_authors: row.3,
    };

    Ok(Json(ApiResponse::new(stats)))
}

#[openapi(tag = "Lists")]
#[get("/lists/<slug>/stats")]
pub async fn list_stats(
    slug: String,
    mut db: Connection<NexusDb>,
) -> Result<Json<ApiResponse<MailingListStats>>, ApiError> {
    let mailing_list_id = resolve_mailing_list_id(&slug, &mut db).await?;

    let stats = sqlx::query_as::<_, MailingListStats>(
        r#"
        SELECT
            CAST((SELECT COUNT(*) FROM emails WHERE mailing_list_id = $1) AS BIGINT) AS total_emails,
            CAST((SELECT COUNT(*) FROM threads WHERE mailing_list_id = $1) AS BIGINT) AS total_threads,
            CAST((SELECT COUNT(*) FROM author_mailing_list_activity WHERE mailing_list_id = $1) AS BIGINT) AS total_authors,
            (SELECT MIN(date) FROM emails WHERE mailing_list_id = $1) AS date_range_start,
            (SELECT MAX(date) FROM emails WHERE mailing_list_id = $1) AS date_range_end
        "#,
    )
    .bind(mailing_list_id)
    .fetch_one(&mut **db)
    .await?;

    let meta = ResponseMeta::default().with_list_id(slug);
    Ok(Json(ApiResponse::with_meta(stats, meta)))
}
