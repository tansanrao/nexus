//! Shared helper functions for Rocket route handlers.

use crate::db::NexusDb;
use crate::error::ApiError;
use rocket_db_pools::{Connection, sqlx};

/// Resolve a mailing list slug to its numeric database identifier.
///
/// Returns [`ApiError::NotFound`] when the slug does not exist.
pub async fn resolve_mailing_list_id(
    slug: &str,
    db: &mut Connection<NexusDb>,
) -> Result<i32, ApiError> {
    let record: (i32,) = sqlx::query_as("SELECT id FROM mailing_lists WHERE slug = $1")
        .bind(slug)
        .fetch_one(db.as_mut())
        .await
        .map_err(|_| ApiError::NotFound(format!("Mailing list '{slug}' not found")))?;

    Ok(record.0)
}
