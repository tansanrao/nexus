//! Checkpoint management for tracking sync progress.
//!
//! Checkpoints record which commits have been processed for each repository (epoch)
//! and when threading was last completed. This enables incremental synchronization
//! by allowing the sync system to resume from the last processed commit.

use rocket_db_pools::sqlx::PgPool;
use std::collections::HashMap;

/// Load the last indexed commit for each repository (epoch) of a mailing list.
///
/// This is used to determine where to resume synchronization. If no checkpoints
/// exist, a full sync will be performed.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `list_id` - Mailing list ID
///
/// # Returns
/// HashMap mapping epoch number (repo_order) to the last indexed commit hash.
/// Empty map indicates no previous sync has completed.
pub async fn load_last_indexed_commits(
    pool: &PgPool,
    list_id: i32,
) -> Result<HashMap<i32, String>, String> {
    let rows: Vec<(i32, Option<String>)> = sqlx::query_as(
        r#"SELECT repo_order, last_indexed_commit
           FROM mailing_list_repositories
           WHERE mailing_list_id = $1"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to load last indexed commits: {}", e))?;

    let mut map = HashMap::new();
    for (repo_order, last_commit) in rows {
        if let Some(commit_hash) = last_commit {
            map.insert(repo_order, commit_hash);
        }
    }

    Ok(map)
}

/// Save the last indexed commit for each processed epoch.
///
/// This checkpoint allows future syncs to resume from this point rather than
/// reprocessing all commits. Called after successful completion of a sync job.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `list_id` - Mailing list ID
/// * `commits` - HashMap mapping epoch number to the last processed commit hash
///
/// # Returns
/// `Ok(())` if checkpoints are saved successfully, error otherwise
pub async fn save_last_indexed_commits(
    pool: &PgPool,
    list_id: i32,
    commits: &HashMap<i32, String>,
) -> Result<(), String> {
    for (repo_order, commit_hash) in commits {
        sqlx::query(
            r#"UPDATE mailing_list_repositories
               SET last_indexed_commit = $1
               WHERE mailing_list_id = $2 AND repo_order = $3"#,
        )
        .bind(commit_hash)
        .bind(list_id)
        .bind(repo_order)
        .execute(pool)
        .await
        .map_err(|e| {
            format!(
                "Failed to save last indexed commit for repo {}: {}",
                repo_order, e
            )
        })?;
    }

    Ok(())
}

/// Save the timestamp when threading was last completed for a mailing list.
///
/// This is used for monitoring and debugging purposes to track when the
/// mailing list's thread structure was last updated.
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `list_id` - Mailing list ID
///
/// # Returns
/// `Ok(())` if timestamp is saved successfully, error otherwise
pub async fn save_last_threaded_at(pool: &PgPool, list_id: i32) -> Result<(), String> {
    let timestamp = chrono::Utc::now();
    sqlx::query(
        r#"UPDATE mailing_lists
           SET last_threaded_at = $1
           WHERE id = $2"#,
    )
    .bind(timestamp)
    .bind(list_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to save last threaded timestamp: {}", e))?;

    Ok(())
}
