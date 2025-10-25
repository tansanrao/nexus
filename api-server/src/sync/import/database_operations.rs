//! Bulk database insert operations.
//!
//! Provides optimized batch insert operations using PostgreSQL's UNNEST
//! for efficient multi-row inserts.

use crate::sync::import::data_structures::{EmailsData, RecipientsData, ReferencesData};
use rocket_db_pools::sqlx::{Postgres, pool::PoolConnection};
use std::collections::HashMap;

/// Insert a batch of authors into the database.
///
/// Uses UNNEST for efficient bulk insertion. Handles conflicts by updating
/// the last_seen timestamp and canonical_name if not already set.
///
/// # Arguments
/// * `conn` - Database connection
/// * `authors` - Map of email addresses to display names
///
/// # Returns
/// Number of author records processed (inserted or updated)
pub async fn insert_authors_batch(
    conn: &mut PoolConnection<Postgres>,
    authors: HashMap<String, String>,
) -> Result<usize, sqlx::Error> {
    if authors.is_empty() {
        return Ok(0);
    }

    let mut emails = Vec::new();
    let mut names = Vec::new();

    for (email, name) in authors {
        emails.push(email);
        names.push(if name.is_empty() { None } else { Some(name) });
    }

    let count = emails.len();

    sqlx::query(
        r#"INSERT INTO authors (email, canonical_name, first_seen, last_seen)
           SELECT email, name, NOW(), NOW()
           FROM UNNEST($1::text[], $2::text[]) AS t(email, name)
           ON CONFLICT (email) DO UPDATE
           SET last_seen = NOW(),
               canonical_name = COALESCE(EXCLUDED.canonical_name, authors.canonical_name)"#,
    )
    .bind(&emails)
    .bind(&names)
    .execute(&mut **conn)
    .await?;

    log::trace!("bulk inserted {} authors", count);
    Ok(count)
}

/// Insert a batch of emails into the database.
///
/// Uses UNNEST for efficient bulk insertion. Skips emails that already exist
/// (based on mailing_list_id + message_id unique constraint).
///
/// # Arguments
/// * `conn` - Database connection
/// * `mailing_list_id` - Mailing list ID
/// * `data` - Prepared email data in columnar format
///
/// # Returns
/// Number of email records actually inserted (conflicts are skipped)
pub async fn insert_emails_batch(
    conn: &mut PoolConnection<Postgres>,
    mailing_list_id: i32,
    data: &EmailsData,
) -> Result<usize, sqlx::Error> {
    if data.message_ids.is_empty() {
        return Ok(0);
    }

    let count = data.message_ids.len();
    let list_ids = vec![mailing_list_id; count];

    let result = sqlx::query(
        r#"INSERT INTO emails (
            mailing_list_id, message_id, git_commit_hash, author_id,
            subject, normalized_subject, date, in_reply_to, body, search_body,
            series_id, series_number, series_total, epoch,
            patch_type, is_patch_only, patch_metadata, lex_ts, body_ts
           )
           SELECT
               list_id,
               message_id,
               commit_hash,
               author_id,
               subject,
               normalized_subject,
               mail_date,
               in_reply_to,
               body,
                search_body,
                series_id,
                series_number,
                series_total,
                epoch,
                patch_type,
                is_patch_only,
                patch_metadata,
                to_tsvector('english',
                   COALESCE(subject, '') || ' ' || COALESCE(search_body, '')
                ),
               to_tsvector('english', COALESCE(search_body, ''))
           FROM UNNEST(
               $1::int[],
               $2::text[],
               $3::text[],
               $4::int[],
               $5::text[],
               $6::text[],
               $7::timestamptz[],
               $8::text[],
               $9::text[],
               $10::text[],
               $11::text[],
               $12::int[],
               $13::int[],
               $14::int[],
               $15::patch_type[],
               $16::bool[],
               $17::jsonb[]
           ) AS t (
               list_id,
               message_id,
               commit_hash,
               author_id,
               subject,
               normalized_subject,
               mail_date,
               in_reply_to,
               body,
               search_body,
               series_id,
               series_number,
               series_total,
               epoch,
               patch_type,
               is_patch_only,
               patch_metadata
           )
           ON CONFLICT (mailing_list_id, message_id) DO NOTHING"#,
    )
    .bind(&list_ids)
    .bind(&data.message_ids)
    .bind(&data.commit_hashes)
    .bind(&data.author_ids)
    .bind(&data.subjects)
    .bind(&data.normalized_subjects)
    .bind(&data.dates)
    .bind(&data.in_reply_tos)
    .bind(&data.bodies)
    .bind(&data.search_bodies)
    .bind(&data.series_ids)
    .bind(&data.series_numbers)
    .bind(&data.series_totals)
    .bind(&data.epochs)
    .bind(&data.patch_types)
    .bind(&data.is_patch_only)
    .bind(&data.patch_metadata)
    .execute(&mut **conn)
    .await?;

    let rows_affected = result.rows_affected() as usize;
    if rows_affected < count {
        log::debug!(
            "insert_emails_batch: tried to insert {} emails, but only {} were inserted ({} skipped due to conflicts)",
            count,
            rows_affected,
            count - rows_affected
        );
    }

    log::trace!("bulk inserted {} emails", rows_affected);
    Ok(rows_affected)
}

/// Insert a batch of email recipients into the database.
///
/// Uses UNNEST for efficient bulk insertion. Does not handle conflicts
/// (assumes unique combinations of email_id + author_id + recipient_type).
///
/// # Arguments
/// * `conn` - Database connection
/// * `data` - Prepared recipient data in columnar format
///
/// # Returns
/// Number of recipient records inserted
pub async fn insert_recipients_batch(
    conn: &mut PoolConnection<Postgres>,
    data: RecipientsData,
) -> Result<usize, sqlx::Error> {
    if data.email_ids.is_empty() {
        return Ok(0);
    }

    let count = data.email_ids.len();

    sqlx::query(
        r#"INSERT INTO email_recipients (mailing_list_id, email_id, author_id, recipient_type)
           SELECT * FROM UNNEST($1::int[], $2::int[], $3::int[], $4::text[])"#,
    )
    .bind(&data.list_ids)
    .bind(&data.email_ids)
    .bind(&data.author_ids)
    .bind(&data.recipient_types)
    .execute(&mut **conn)
    .await?;

    log::trace!("bulk inserted {} recipients", count);
    Ok(count)
}

/// Insert a batch of email references into the database.
///
/// Uses UNNEST for efficient bulk insertion. Skips references that already exist
/// (based on unique constraint on mailing_list_id + email_id + referenced_message_id).
///
/// # Arguments
/// * `conn` - Database connection
/// * `data` - Prepared reference data in columnar format
///
/// # Returns
/// Number of reference records inserted
pub async fn insert_references_batch(
    conn: &mut PoolConnection<Postgres>,
    data: ReferencesData,
) -> Result<usize, sqlx::Error> {
    if data.email_ids.is_empty() {
        return Ok(0);
    }

    let count = data.email_ids.len();

    sqlx::query(
        r#"INSERT INTO email_references (mailing_list_id, email_id, referenced_message_id, position)
           SELECT * FROM UNNEST($1::int[], $2::int[], $3::text[], $4::int[])
           ON CONFLICT (mailing_list_id, email_id, referenced_message_id) DO NOTHING"#,
    )
    .bind(&data.list_ids)
    .bind(&data.email_ids)
    .bind(&data.referenced_message_ids)
    .bind(&data.positions)
    .execute(&mut **conn)
    .await?;

    log::trace!("bulk inserted {} references", count);
    Ok(count)
}
