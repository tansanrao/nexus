//! Data preparation for bulk database operations.
//!
//! Transforms parsed emails into columnar data structures optimized for
//! PostgreSQL UNNEST bulk insert operations.

use crate::search::sanitize::strip_patch_payload;
use crate::sync::import::data_structures::{
    ChunkCacheData, EmailsData, RecipientsData, ReferencesData,
};
use crate::sync::parser::ParsedEmail;
use crate::threading::extract_patch_series_info;
use rocket_db_pools::sqlx::PgPool;
use serde_json;
use std::collections::{HashMap, HashSet};

/// Extract unique authors from a chunk of parsed emails.
///
/// Collects all unique author emails (senders, To recipients, Cc recipients)
/// and their names. If multiple names are seen for the same email, the first
/// one encountered is used.
///
/// # Arguments
/// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
///
/// # Returns
/// HashMap mapping email addresses to display names
pub fn extract_unique_authors_from_chunk(
    chunk: &[(String, ParsedEmail, i32)],
) -> HashMap<String, String> {
    let mut authors = HashMap::new();

    for (_, email, _) in chunk {
        // Add sender (pick first name if multiple are seen)
        authors
            .entry(email.author_email.clone())
            .or_insert_with(|| email.author_name.clone());

        // Add recipients
        for (name, addr) in &email.to_addrs {
            authors.entry(addr.clone()).or_insert_with(|| name.clone());
        }
        for (name, addr) in &email.cc_addrs {
            authors.entry(addr.clone()).or_insert_with(|| name.clone());
        }
    }

    authors
}

/// Build email batch data for database insertion.
///
/// Prepares email data in columnar format for bulk insert. Requires that
/// all referenced authors have already been inserted into the database.
///
/// # Arguments
/// * `pool` - Database connection pool (for loading author IDs)
/// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
///
/// # Returns
/// EmailsData structure with parallel vectors ready for UNNEST insertion
///
/// # Errors
/// Returns database errors if author ID loading fails
pub async fn build_email_batch_data(
    pool: &PgPool,
    chunk: &[(String, ParsedEmail, i32)],
) -> Result<EmailsData, sqlx::Error> {
    // Load author IDs first
    let unique_emails: Vec<String> = chunk
        .iter()
        .map(|(_, e, _)| e.author_email.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let author_rows: Vec<(String, i32)> =
        sqlx::query_as("SELECT email, id FROM authors WHERE email = ANY($1)")
            .bind(&unique_emails)
            .fetch_all(pool)
            .await?;

    let author_map: HashMap<String, i32> = author_rows.into_iter().collect();

    // Diagnostic: log if we have missing authors
    let unique_email_count = unique_emails.len();
    let found_author_count = author_map.len();
    if unique_email_count != found_author_count {
        log::warn!(
            "Author lookup mismatch: expected {} unique authors, found {} (missing: {})",
            unique_email_count,
            found_author_count,
            unique_email_count - found_author_count
        );
    }

    // Build parallel vectors for UNNEST
    let mut data = EmailsData::default();
    let mut skipped_count = 0;

    for (commit_hash, email, epoch) in chunk {
        if let Some(&author_id) = author_map.get(&email.author_email) {
            let series_info = extract_patch_series_info(&email.subject);

            data.message_ids.push(email.message_id.clone());
            data.commit_hashes.push(commit_hash.clone());
            data.author_ids.push(author_id);
            data.subjects.push(email.subject.clone());
            data.normalized_subjects
                .push(email.normalized_subject.clone());
            data.dates.push(email.date);
            data.in_reply_tos.push(email.in_reply_to.clone());
            data.bodies.push(email.body.clone());
            let sanitized = strip_patch_payload(
                &email.body,
                email.patch_metadata.as_ref(),
                email.is_patch_only,
            )
            .into_owned();
            data.search_bodies.push(sanitized);

            if let Some((series_id, series_num, series_total)) = series_info {
                data.series_ids.push(Some(series_id));
                data.series_numbers.push(Some(series_num));
                data.series_totals.push(Some(series_total));
            } else {
                data.series_ids.push(None);
                data.series_numbers.push(None);
                data.series_totals.push(None);
            }

            // Store epoch for this email
            data.epochs.push(*epoch);
            data.patch_types.push(email.patch_type);
            data.is_patch_only.push(email.is_patch_only);
            let metadata_value = email
                .patch_metadata
                .as_ref()
                .and_then(|meta| serde_json::to_value(meta).ok());
            data.patch_metadata.push(metadata_value);
        } else {
            // DIAGNOSTIC: Email skipped due to missing author
            skipped_count += 1;
            if skipped_count <= 5 {
                log::warn!(
                    "Skipping email {} (commit {}) - author '{}' not found in author_map",
                    &email.message_id,
                    &commit_hash[..8],
                    &email.author_email
                );
            }
        }
    }

    if skipped_count > 0 {
        log::warn!(
            "build_email_batch_data: skipped {} emails due to missing author IDs (chunk size: {})",
            skipped_count,
            chunk.len()
        );
    }

    Ok(data)
}

/// Build recipient batch data for database insertion.
///
/// Uses pre-loaded email and author ID maps to avoid redundant database queries.
/// Automatically deduplicates recipients (same email + author + type).
///
/// # Arguments
/// * `mailing_list_id` - Mailing list ID
/// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
/// * `email_id_map` - Map from message_id to email database ID
/// * `author_map` - Map from email address to author database ID
///
/// # Returns
/// RecipientsData structure with parallel vectors ready for UNNEST insertion
pub fn build_recipient_batch_data(
    mailing_list_id: i32,
    chunk: &[(String, ParsedEmail, i32)],
    email_id_map: &HashMap<String, i32>,
    author_map: &HashMap<String, i32>,
) -> RecipientsData {
    // Use a HashSet to deduplicate (email_id, author_id, recipient_type) tuples
    let mut recipient_set: HashSet<(i32, i32, String)> = HashSet::new();

    for (_, email, _) in chunk {
        if let Some(&email_id) = email_id_map.get(&email.message_id) {
            // Collect To recipients
            for (_, addr) in &email.to_addrs {
                if let Some(&author_id) = author_map.get(addr) {
                    recipient_set.insert((email_id, author_id, "to".to_string()));
                }
            }

            // Collect Cc recipients
            for (_, addr) in &email.cc_addrs {
                if let Some(&author_id) = author_map.get(addr) {
                    recipient_set.insert((email_id, author_id, "cc".to_string()));
                }
            }
        }
    }

    // Build parallel vectors
    let mut data = RecipientsData::default();
    for (email_id, author_id, recipient_type) in recipient_set {
        data.list_ids.push(mailing_list_id);
        data.email_ids.push(email_id);
        data.author_ids.push(author_id);
        data.recipient_types.push(recipient_type);
    }

    data
}

/// Build reference batch data for database insertion.
///
/// Preserves the order of references from the email headers using position numbers.
/// Automatically deduplicates references for the same email.
///
/// # Arguments
/// * `mailing_list_id` - Mailing list ID
/// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
/// * `email_id_map` - Map from message_id to email database ID
///
/// # Returns
/// ReferencesData structure with parallel vectors ready for UNNEST insertion
pub fn build_reference_batch_data(
    mailing_list_id: i32,
    chunk: &[(String, ParsedEmail, i32)],
    email_id_map: &HashMap<String, i32>,
) -> ReferencesData {
    // Use a HashMap to deduplicate (email_id, referenced_message_id) pairs
    // Keep the first occurrence's position
    let mut reference_map: HashMap<(i32, String), i32> = HashMap::new();

    for (_, email, _) in chunk {
        if let Some(&email_id) = email_id_map.get(&email.message_id) {
            // Collect references with their position to preserve order
            for (position, ref_msg_id) in email.references.iter().enumerate() {
                let key = (email_id, ref_msg_id.clone());
                reference_map.entry(key).or_insert(position as i32);
            }
        }
    }

    // Build parallel vectors
    let mut data = ReferencesData::default();
    for ((email_id, ref_msg_id), position) in reference_map {
        data.list_ids.push(mailing_list_id);
        data.email_ids.push(email_id);
        data.referenced_message_ids.push(ref_msg_id);
        data.positions.push(position);
    }

    data
}

/// Extract cache data from imported email chunk.
///
/// Builds the data structure needed to populate the threading cache after
/// successful database insertion. Includes email metadata and references.
///
/// # Arguments
/// * `chunk` - Slice of (commit_hash, parsed_email, epoch) tuples
/// * `email_id_map` - Map from message_id to email database ID
///
/// # Returns
/// ChunkCacheData containing email metadata and references for cache population
pub fn extract_cache_data_from_chunk(
    chunk: &[(String, ParsedEmail, i32)],
    email_id_map: &HashMap<String, i32>,
) -> ChunkCacheData {
    let mut cache_emails = Vec::new();
    let mut cache_refs_map: HashMap<i32, Vec<String>> = HashMap::new();
    let mut cache_miss_count = 0;

    // Build email data for cache
    for (_, email, _) in chunk {
        if let Some(&email_id) = email_id_map.get(&email.message_id) {
            let series_info = extract_patch_series_info(&email.subject);
            let (series_id, series_number, series_total) =
                if let Some((sid, snum, stot)) = series_info {
                    (Some(sid), Some(snum), Some(stot))
                } else {
                    (None, None, None)
                };

            cache_emails.push((
                email_id,
                email.message_id.clone(),
                email.subject.clone(),
                email.in_reply_to.clone(),
                email.date,
                series_id,
                series_number,
                series_total,
            ));

            // Build references map for this email
            let mut refs = Vec::new();
            for ref_msg_id in &email.references {
                refs.push(ref_msg_id.clone());
            }
            if !refs.is_empty() {
                cache_refs_map.insert(email_id, refs);
            }
        } else {
            // DIAGNOSTIC: Email's ID not found in email_id_map
            cache_miss_count += 1;
            if cache_miss_count <= 5 {
                log::warn!(
                    "Email {} not found in email_id_map (not inserted to DB?)",
                    &email.message_id
                );
            }
        }
    }

    if cache_miss_count > 0 {
        log::warn!(
            "extract_cache_data_from_chunk: {} emails not added to cache (not in email_id_map), chunk size: {}",
            cache_miss_count,
            chunk.len()
        );
    }

    ChunkCacheData {
        emails: cache_emails,
        references: cache_refs_map.into_iter().collect(),
    }
}
