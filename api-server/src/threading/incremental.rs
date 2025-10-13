//! Incremental threading support
//!
//! This module provides functionality to determine which emails need to be re-threaded
//! when new emails arrive, enabling efficient incremental updates instead of full rebuilds.

use rocket_db_pools::sqlx::{Postgres, Transaction};
use std::collections::HashSet;

/// Find the complete set of email IDs that need to be re-threaded
///
/// Given a set of new email IDs, this function finds all emails that could be in the same
/// threads by following reference chains in both directions:
/// - Emails that reference these emails (via in_reply_to or references header)
/// - Emails that are referenced by these emails
/// - Recursively expand to find entire thread boundaries
///
/// ## Algorithm (Optimized with Recursive CTE)
///
/// Uses a single PostgreSQL recursive CTE to discover the complete affected set in one query.
/// This replaces the previous iterative approach that required multiple round-trips.
///
/// The recursive CTE:
/// 1. Base case: Start with new email IDs and their message_ids
/// 2. Recursive case: Find emails that:
///    - Reply to affected emails (in_reply_to)
///    - Reference affected emails (email_references table)
///    - Are referenced by affected emails (backward references)
/// 3. PostgreSQL handles the recursion efficiently in a single execution
///
/// ## Performance
///
/// Expected 5-10x speedup compared to iterative approach due to:
/// - Single database round-trip instead of multiple iterations
/// - PostgreSQL query planner optimizations for recursive CTEs
/// - Elimination of HashSet operations in application code
/// - Better use of indexes
///
/// ## Returns
///
/// A HashSet of all email IDs that need to be included in threading, including the original new emails.
pub async fn find_affected_email_set(
    tx: &mut Transaction<'_, Postgres>,
    mailing_list_id: i32,
    new_email_ids: &[i32],
) -> Result<HashSet<i32>, sqlx::Error> {
    if new_email_ids.is_empty() {
        return Ok(HashSet::new());
    }

    log::debug!(
        "finding affected emails: list {} with {} new emails",
        mailing_list_id,
        new_email_ids.len()
    );

    let start_time = std::time::Instant::now();

    // Single-pass recursive CTE for affected email discovery
    // This replaces the iterative approach with a single optimized query
    let rows: Vec<(i32,)> = sqlx::query_as(
        r#"WITH RECURSIVE affected_emails AS (
            -- Base case: new emails and their message_ids
            SELECT DISTINCT id, message_id
            FROM emails
            WHERE mailing_list_id = $1 AND id = ANY($2)

            UNION

            -- Recursive case: emails connected to affected emails
            SELECT DISTINCT e.id, e.message_id
            FROM emails e
            WHERE e.mailing_list_id = $1
            AND (
                -- Emails that reply to affected emails
                e.in_reply_to IN (SELECT message_id FROM affected_emails)
                OR
                -- Emails that reference affected emails in their References header
                EXISTS (
                    SELECT 1 FROM email_references er
                    JOIN affected_emails ae ON ae.message_id = er.referenced_message_id
                    WHERE er.mailing_list_id = $1
                    AND er.email_id = e.id
                )
                OR
                -- Emails referenced by affected emails (backward references)
                EXISTS (
                    SELECT 1 FROM email_references er
                    JOIN affected_emails ae ON ae.id = er.email_id
                    WHERE er.mailing_list_id = $1
                    AND e.message_id = er.referenced_message_id
                )
            )
        )
        SELECT id FROM affected_emails"#
    )
    .bind(mailing_list_id)
    .bind(new_email_ids)
    .fetch_all(&mut **tx)
    .await?;

    let affected_ids: HashSet<i32> = rows.into_iter().map(|(id,)| id).collect();
    let elapsed = start_time.elapsed();

    log::debug!(
        "affected set complete: {} emails discovered in {:.2}ms (recursive CTE)",
        affected_ids.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    Ok(affected_ids)
}

/// Find thread IDs that contain any of the given email IDs
///
/// This is used to identify which existing threads need to be deleted before rebuilding.
pub async fn find_affected_thread_ids(
    tx: &mut Transaction<'_, Postgres>,
    mailing_list_id: i32,
    email_ids: &[i32],
) -> Result<Vec<i32>, sqlx::Error> {
    if email_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<(i32,)> = sqlx::query_as(
        r#"SELECT DISTINCT thread_id
           FROM thread_memberships
           WHERE mailing_list_id = $1
           AND email_id = ANY($2)"#
    )
    .bind(mailing_list_id)
    .bind(email_ids)
    .fetch_all(&mut **tx)
    .await?;

    let thread_ids: Vec<i32> = rows.into_iter().map(|(id,)| id).collect();

    log::debug!(
        "found {} affected threads for {} emails",
        thread_ids.len(),
        email_ids.len()
    );

    Ok(thread_ids)
}

/// Delete threads and their memberships by thread IDs
///
/// Used to remove affected threads before rebuilding them incrementally.
pub async fn delete_threads_by_ids(
    tx: &mut Transaction<'_, Postgres>,
    mailing_list_id: i32,
    thread_ids: &[i32],
) -> Result<(), sqlx::Error> {
    if thread_ids.is_empty() {
        return Ok(());
    }

    log::debug!("deleting {} threads", thread_ids.len());

    // Delete thread memberships first (foreign key constraint)
    sqlx::query(
        r#"DELETE FROM thread_memberships
           WHERE mailing_list_id = $1 AND thread_id = ANY($2)"#
    )
    .bind(mailing_list_id)
    .bind(thread_ids)
    .execute(&mut **tx)
    .await?;

    // Delete threads
    sqlx::query(
        r#"DELETE FROM threads
           WHERE mailing_list_id = $1 AND id = ANY($2)"#
    )
    .bind(mailing_list_id)
    .bind(thread_ids)
    .execute(&mut **tx)
    .await?;

    log::debug!("deleted {} threads", thread_ids.len());

    Ok(())
}
