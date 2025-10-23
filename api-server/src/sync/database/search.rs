//! Helpers for managing search-related database state.
//!
//! Provides utilities to recompute derived search fields (FTS vectors) and to
//! refresh supporting indexes. These helpers are used by admin APIs to keep the
//! search schema in sync after bulk imports or manual maintenance.

use rocket_db_pools::sqlx::{self, PgPool};

/// Backfill lexical/body tsvector columns for emails.
///
/// Returns the number of rows updated. When `mailing_list_id` is provided the
/// refresh is limited to that list; otherwise all emails are refreshed.
pub async fn backfill_fts_columns(
    pool: &PgPool,
    mailing_list_id: Option<i32>,
) -> Result<u64, sqlx::Error> {
    let query = if mailing_list_id.is_some() {
        r#"
        UPDATE emails
        SET
            lex_ts = to_tsvector('english', COALESCE(subject, '') || ' ' || COALESCE(body, '')),
            body_ts = to_tsvector('english', COALESCE(body, ''))
        WHERE mailing_list_id = $1
        "#
    } else {
        r#"
        UPDATE emails
        SET
            lex_ts = to_tsvector('english', COALESCE(subject, '') || ' ' || COALESCE(body, '')),
            body_ts = to_tsvector('english', COALESCE(body, ''))
        "#
    };

    // Embeddings intentionally stay NULL here; future inference jobs populate them.
    let result = if let Some(list_id) = mailing_list_id {
        sqlx::query(query).bind(list_id).execute(pool).await?
    } else {
        sqlx::query(query).execute(pool).await?
    };

    Ok(result.rows_affected())
}

/// Refresh search indexes used for lexical/vector queries.
///
/// Runs REINDEX on the main search-related indexes to ensure statistics are
/// up-to-date after large backfills. These commands run sequentially to avoid
/// excessive lock contention.
pub async fn refresh_search_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    let index_statements = [
        "REINDEX INDEX idx_emails_lex_ts",
        "REINDEX INDEX idx_emails_body_ts",
        "REINDEX INDEX idx_emails_subject_trgm",
        "REINDEX INDEX idx_emails_embedding_hnsw",
        "REINDEX INDEX idx_thread_embeddings_hnsw",
    ];

    for statement in index_statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}
