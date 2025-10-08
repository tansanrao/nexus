//! Subject normalization and matching for email threading
//!
//! When email headers (In-Reply-To, References) are missing or incomplete,
//! we can use subject-based matching as a fallback to detect conversations.
//!
//! This module provides normalization to remove common prefixes and noise
//! so we can match emails that are part of the same conversation.

use std::collections::HashMap;

use super::container::{Container, EmailData};

/// Normalize an email subject for threading comparison
///
/// This function removes common prefixes and noise from email subjects to enable
/// matching of related emails. It repeatedly strips prefixes until no more can be removed.
///
/// ## Removed Patterns
///
/// - Reply prefixes: `Re:`, `Fwd:`, `Fw:`, `Aw:`
/// - Bracketed tags: `[PATCH]`, `[PATCH v2]`, `[RFC]`, `[PATCH 1/3]`, etc.
/// - Multiple spaces collapsed to single space
/// - Leading/trailing whitespace
///
/// ## Examples
///
/// ```rust
/// assert_eq!(
///     normalize_subject("Re: [PATCH] Fix memory leak"),
///     "fix memory leak"
/// );
/// assert_eq!(
///     normalize_subject("[PATCH v2 1/3] Add new feature"),
///     "add new feature"
/// );
/// assert_eq!(
///     normalize_subject("Re: Fwd: [RFC PATCH] Test"),
///     "test"
/// );
/// ```
pub fn normalize_subject(subject: &str) -> String {
    let mut normalized = subject.trim().to_lowercase();

    // Keep removing prefixes until none match
    loop {
        let before = normalized.clone();

        // Remove Re:, Fwd:, Fw:, Aw: prefixes (case insensitive, already lowercase)
        for prefix in &["re:", "fwd:", "fw:", "aw:"] {
            if normalized.starts_with(prefix) {
                normalized = normalized[prefix.len()..].trim_start().to_string();
            }
        }

        // Remove bracketed tags like [PATCH], [PATCH v2], [RFC], etc.
        if normalized.starts_with('[') {
            if let Some(end_bracket) = normalized.find(']') {
                normalized = normalized[end_bracket + 1..].trim_start().to_string();
            }
        }

        // If nothing changed, we're done
        if before == normalized {
            break;
        }
    }

    // Collapse multiple spaces into one
    let words: Vec<&str> = normalized.split_whitespace().collect();
    words.join(" ")
}

/// Link emails based on subject similarity
///
/// This function implements subject-based threading as a fallback when
/// email headers are missing or incomplete.
///
/// ## Algorithm
///
/// 1. Index all emails by normalized subject
/// 2. For each email without a parent:
///    - Find emails with the same normalized subject
///    - Link to the earliest email with that subject
///
/// This creates threads based on subject similarity, useful for emails
/// where the References/In-Reply-To headers are missing.
///
/// ## Arguments
///
/// - `id_table`: The container table to modify
/// - `email_data`: Map of email_id to EmailData
pub fn link_by_subject(
    id_table: &mut HashMap<String, Container>,
    email_data: &HashMap<i32, EmailData>,
) {
    // Build subject index: normalized_subject -> Vec<email_id>
    let mut subject_index: HashMap<String, Vec<i32>> = HashMap::new();

    for (email_id, data) in email_data {
        subject_index
            .entry(data.normalized_subject.clone())
            .or_insert_with(Vec::new)
            .push(*email_id);
    }

    // Link emails with matching subjects
    for (email_id, data) in email_data {
        let msg_id = &data.message_id;

        // Skip if already has a parent
        if id_table
            .get(msg_id)
            .map(|c| c.parent.is_some())
            .unwrap_or(false)
        {
            continue;
        }

        // Skip if normalized subject is empty
        if data.normalized_subject.is_empty() {
            continue;
        }

        // Find other emails with same normalized subject
        if let Some(similar_emails) = subject_index.get(&data.normalized_subject) {
            // Find the earliest email with this subject (likely the root)
            let potential_parent = similar_emails
                .iter()
                .filter(|&&other_id| other_id != *email_id)
                .filter_map(|other_id| email_data.get(other_id))
                .filter(|other_data| other_data.date < data.date)
                .min_by_key(|other_data| other_data.date);

            if let Some(parent_data) = potential_parent {
                let parent_msg_id = &parent_data.message_id;

                // Set parent
                if let Some(container) = id_table.get_mut(msg_id) {
                    container.parent = Some(parent_msg_id.clone());
                }

                // Add to parent's children
                if let Some(parent_container) = id_table.get_mut(parent_msg_id) {
                    parent_container.add_child(msg_id.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic_reply() {
        assert_eq!(
            normalize_subject("Re: [PATCH] Fix memory leak"),
            "fix memory leak"
        );
    }

    #[test]
    fn test_normalize_versioned_patch() {
        assert_eq!(
            normalize_subject("[PATCH v2 1/3] Add new feature"),
            "add new feature"
        );
    }

    #[test]
    fn test_normalize_multiple_prefixes() {
        assert_eq!(
            normalize_subject("Re: Fwd: [RFC PATCH] Test"),
            "test"
        );
    }

    #[test]
    fn test_normalize_nested_re() {
        assert_eq!(
            normalize_subject("Re: Re: [PATCH v3] Important fix"),
            "important fix"
        );
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            normalize_subject("  Re:   [PATCH]   Multiple    spaces  "),
            "multiple spaces"
        );
    }
}
