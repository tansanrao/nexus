//! Helpers for managing search-related database state.
//!
//! Provides utilities to recompute derived search fields (FTS vectors) and to
//! refresh supporting indexes. These helpers are used by admin APIs to keep the
//! search schema in sync after bulk imports or manual maintenance.

use crate::models::PatchMetadata;
use crate::search::sanitize::strip_patch_payload;
use rocket_db_pools::sqlx::types::Json;
use rocket_db_pools::sqlx::{self, PgPool};

const BACKFILL_BATCH_SIZE: i64 = 1000;

/// Backfill lexical/body tsvector columns for emails.
///
/// Returns the number of rows updated. When `mailing_list_id` is provided the
/// refresh is limited to that list; otherwise all emails are refreshed.
pub async fn backfill_fts_columns(
    pool: &PgPool,
    mailing_list_id: Option<i32>,
) -> Result<u64, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct EmailRow {
        id: i32,
        body: Option<String>,
        patch_metadata: Option<Json<PatchMetadata>>,
        is_patch_only: bool,
    }

    let mut last_id: i32 = 0;
    let mut total_updated: u64 = 0;

    loop {
        let rows: Vec<EmailRow> = if let Some(list_id) = mailing_list_id {
            sqlx::query_as::<_, EmailRow>(
                r#"
                SELECT id, body, patch_metadata, is_patch_only
                FROM emails
                WHERE mailing_list_id = $1 AND id > $2
                ORDER BY id ASC
                LIMIT $3
                "#,
            )
            .bind(list_id)
            .bind(last_id)
            .bind(BACKFILL_BATCH_SIZE)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, EmailRow>(
                r#"
                SELECT id, body, patch_metadata, is_patch_only
                FROM emails
                WHERE id > $1
                ORDER BY id ASC
                LIMIT $2
                "#,
            )
            .bind(last_id)
            .bind(BACKFILL_BATCH_SIZE)
            .fetch_all(pool)
            .await?
        };

        if rows.is_empty() {
            break;
        }

        let mut ids: Vec<i32> = Vec::with_capacity(rows.len());
        let mut sanitized_bodies: Vec<String> = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            let sanitized = if let Some(body) = row.body.as_ref() {
                strip_patch_payload(
                    body,
                    row.patch_metadata.as_ref().map(|meta| &meta.0),
                    row.is_patch_only,
                )
                .into_owned()
            } else {
                String::new()
            };

            ids.push(row.id);
            sanitized_bodies.push(sanitized);
        }

        let update_result = sqlx::query(
            r#"
            UPDATE emails AS e
            SET
                search_body = data.search_body,
                lex_ts = to_tsvector('english', COALESCE(e.subject, '') || ' ' || COALESCE(data.search_body, '')),
                body_ts = to_tsvector('english', COALESCE(data.search_body, ''))
            FROM (
                SELECT UNNEST($1::int[]) AS id,
                       UNNEST($2::text[]) AS search_body
            ) AS data
            WHERE e.id = data.id
            "#,
        )
        .bind(&ids)
        .bind(&sanitized_bodies)
        .execute(pool)
        .await?;

        total_updated += update_result.rows_affected() as u64;
        last_id = rows.last().map(|row| row.id).unwrap_or(last_id);
    }

    Ok(total_updated)
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
        "REINDEX INDEX idx_threads_search_vector",
        "REINDEX INDEX idx_emails_embedding_hnsw",
        "REINDEX INDEX idx_thread_embeddings_hnsw",
    ];

    for statement in index_statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}

/// Drop and recreate the primary search indexes.
///
/// Useful when indexes become corrupted or when changing operator classes. This
/// operation acquires locks equivalent to `DROP INDEX` and `CREATE INDEX`, so it
/// should be executed during a maintenance window.
pub async fn rebuild_search_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    let statements = [
        "DROP INDEX IF EXISTS idx_emails_lex_ts",
        "DROP INDEX IF EXISTS idx_emails_body_ts",
        "DROP INDEX IF EXISTS idx_emails_subject_trgm",
        "DROP INDEX IF EXISTS idx_threads_search_vector",
        "DROP INDEX IF EXISTS idx_emails_embedding_hnsw",
        "DROP INDEX IF EXISTS idx_thread_embeddings_hnsw",
        "CREATE INDEX idx_emails_lex_ts ON emails USING GIN (lex_ts)",
        "CREATE INDEX idx_emails_body_ts ON emails USING GIN (body_ts)",
        "CREATE INDEX idx_emails_subject_trgm ON emails USING GIN (subject gin_trgm_ops)",
        "CREATE INDEX idx_threads_search_vector ON threads USING GIN (search_vector)",
        "CREATE INDEX idx_emails_embedding_hnsw ON emails USING vchordrq (embedding vector_cosine_ops)",
        "CREATE INDEX idx_thread_embeddings_hnsw ON thread_embeddings USING vchordrq (embedding vector_cosine_ops)",
    ];

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}
