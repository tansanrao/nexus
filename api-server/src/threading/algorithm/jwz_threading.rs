//! Core JWZ (Jamie Zawinski) threading algorithm implementation
//!
//! Implements the standard email threading algorithm as described at:
//! https://www.jwz.org/doc/threading.html
//!
//! This implementation is parallelized using Rayon and DashMap for performance
//! on large email datasets (Linux kernel mailing lists can have millions of emails).
//!
//! ## Algorithm Overview
//!
//! 1. **Create Containers**: Build container objects for all messages and references
//! 2. **Link References**: Build parent-child relationships from References header
//! 3. **Apply In-Reply-To**: Fallback linking for messages without References
//! 4. **Find Roots**: Identify messages with no parent (thread roots)
//! 5. **Assemble Threads**: Collect complete threads, handling phantom roots correctly

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use rayon::prelude::*;

use super::super::container::{Container, EmailData, ThreadInfo};
use super::cycle_detection::detect_cycle_in_ancestry;
use super::tree_traversal::{collect_thread_members, find_first_real_message};

/// Build email threads using the JWZ algorithm
///
/// This is the main entry point for threading. Takes email data and reference
/// information from the database and returns organized threads.
///
/// ## Arguments
///
/// * `email_data` - Map of email_id → EmailData for all emails to thread
/// * `email_references` - Map of email_id → Vec<referenced_message_ids> in order
///
/// ## Returns
///
/// Vector of ThreadInfo structures, each representing one complete conversation thread
///
/// ## Performance
///
/// Parallelized using Rayon for container creation and DashMap for thread-safe
/// concurrent access. Can process millions of emails efficiently.
pub fn build_email_threads(
    email_data: HashMap<i32, EmailData>,
    email_references: HashMap<i32, Vec<String>>,
) -> Vec<ThreadInfo> {
    // Step 1: Create all message containers (real and phantom)
    let message_containers = create_message_containers(&email_data, &email_references);

    // Step 2: Build parent-child relationships from References header
    build_reference_links(&message_containers, &email_data, &email_references);

    // Step 3: Apply In-Reply-To fallback for messages without References
    apply_in_reply_to_fallbacks(&message_containers, &email_data);

    // Step 4: Find root set (messages with no parent)
    let root_message_ids = identify_thread_roots(&message_containers);

    // Step 5: Assemble complete threads
    assemble_threads(root_message_ids, &message_containers, &email_data)
}

/// Create containers for all messages (real and phantom)
///
/// Creates Container objects for:
/// - Real messages: messages we have in the database
/// - Phantom messages: messages that are referenced but not in our database
///
/// Uses parallel processing for performance on large datasets.
///
/// ## Why Phantoms?
///
/// Email threads often reference messages we don't have (e.g., private replies,
/// messages from before we started archiving). We create phantom containers
/// to maintain the complete thread structure.
fn create_message_containers(
    email_data: &HashMap<i32, EmailData>,
    email_references: &HashMap<i32, Vec<String>>,
) -> Arc<DashMap<String, Container>> {
    // DashMap allows concurrent inserts from multiple threads
    let message_containers: Arc<DashMap<String, Container>> = Arc::new(DashMap::new());

    // Create containers for all real messages (parallel)
    email_data.par_iter().for_each(|(email_id, data)| {
        let msg_id = data.message_id.clone();
        message_containers
            .entry(msg_id.clone())
            .or_insert_with(|| Container::new_with_email(msg_id, *email_id));
    });

    // Create phantom containers for all referenced messages (parallel)
    email_references.par_iter().for_each(|(_, refs)| {
        for referenced_message_id in refs {
            message_containers
                .entry(referenced_message_id.clone())
                .or_insert_with(|| Container::new_phantom(referenced_message_id.clone()));
        }
    });

    message_containers
}

/// Build parent-child relationships from References headers
///
/// The References header contains the full conversation history as a chain
/// of message IDs. We build links between each adjacent pair, creating the
/// complete thread structure.
///
/// ## Example
///
/// ```text
/// Email has References: <msg1> <msg2> <msg3>
///
/// Creates links:
///   msg1 (parent) → msg2 (child)
///   msg2 (parent) → msg3 (child)
///   msg3 (parent) → this_email (child)
/// ```
///
/// Parallelized across emails using Rayon.
fn build_reference_links(
    message_containers: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
    email_references: &HashMap<i32, Vec<String>>,
) {
    // Process emails in parallel - DashMap handles concurrent access safely
    email_data.par_iter().for_each(|(email_id, data)| {
        let msg_id = data.message_id.clone();

        if let Some(refs) = email_references.get(email_id) {
            // Build the reference chain: refs[0] → refs[1] → ... → this_message
            let mut previous_reference: Option<String> = None;

            for referenced_message_id in refs {
                // Link previous reference to this one
                if let Some(prev) = &previous_reference {
                    link_child_to_parent(message_containers, referenced_message_id, prev);
                }

                previous_reference = Some(referenced_message_id.clone());
            }

            // Link the last reference to this message
            if let Some(last_ref) = previous_reference {
                link_child_to_parent(message_containers, &msg_id, &last_ref);
            }
        }
    });
}

/// Apply In-Reply-To header as fallback
///
/// For messages without References but with In-Reply-To, we can still
/// establish a parent-child relationship. This is simpler than References
/// as it only links to the immediate parent.
///
/// Sequential processing is fine here as it's a fast operation.
fn apply_in_reply_to_fallbacks(
    message_containers: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
) {
    for (_email_id, data) in email_data {
        let msg_id = &data.message_id;

        // Skip if already has a parent from References
        if message_containers
            .get(msg_id)
            .map(|c| c.parent.is_some())
            .unwrap_or(false)
        {
            continue;
        }

        // Link to In-Reply-To if available
        if let Some(in_reply_to) = &data.in_reply_to {
            if message_containers.contains_key(in_reply_to) {
                link_child_to_parent(message_containers, msg_id, in_reply_to);
            }
        }
    }
}

/// Link a child to a parent if safe to do so
///
/// Safely establishes a parent-child relationship with multiple safety checks:
/// - Child doesn't already have a parent
/// - Not linking a message to itself
/// - Doesn't create a cycle in the tree
/// - Avoids duplicate children
///
/// Thread-safe via DashMap's concurrent access.
fn link_child_to_parent(
    message_containers: &DashMap<String, Container>,
    child_message_id: &str,
    parent_message_id: &str,
) {
    // Don't link a message to itself (self-loop check)
    if child_message_id == parent_message_id {
        return;
    }

    // Check if child already has a parent
    if message_containers
        .get(child_message_id)
        .map(|c| c.parent.is_some())
        .unwrap_or(false)
    {
        return;
    }

    // Check for potential cycle (parent is already a descendant of child)
    if detect_cycle_in_ancestry(message_containers, child_message_id, parent_message_id) {
        return;
    }

    // Safe to link - set parent on child
    if let Some(mut child_container) = message_containers.get_mut(child_message_id) {
        child_container.parent = Some(parent_message_id.to_string());
    }

    // Add child to parent's children list
    if let Some(mut parent_container) = message_containers.get_mut(parent_message_id) {
        parent_container.add_child(child_message_id.to_string());
    }
}

/// Find all thread roots (messages with no parent)
///
/// Root messages are the starting points of conversation threads.
/// They have no parent in the tree structure.
fn identify_thread_roots(message_containers: &Arc<DashMap<String, Container>>) -> Vec<String> {
    message_containers
        .iter()
        .filter(|entry| entry.value().parent.is_none())
        .map(|entry| entry.key().clone())
        .collect()
}

/// Assemble complete threads from root set
///
/// Processes each root to create ThreadInfo structures. Handles both:
/// - Real roots: messages with actual email data
/// - Phantom roots: referenced messages we don't have (finds first real child)
///
/// Parallelized using Rayon for performance.
fn assemble_threads(
    root_message_ids: Vec<String>,
    message_containers: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
) -> Vec<ThreadInfo> {
    root_message_ids
        .par_iter()
        .filter_map(|root_msg_id| {
            assemble_single_thread(root_msg_id, message_containers, email_data)
        })
        .collect()
}

/// Assemble a single thread from its root
///
/// Handles both real and phantom roots appropriately:
/// - Real root: use its data for thread metadata
/// - Phantom root: find first real message in subtree for metadata
fn assemble_single_thread(
    root_message_id: &str,
    message_containers: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
) -> Option<ThreadInfo> {
    let root_container = message_containers.get(root_message_id)?;

    // Case 1: Real root (has email data)
    if let Some(root_email_id) = root_container.email_id {
        if let Some(root_data) = email_data.get(&root_email_id) {
            // Collect all members in this thread and get date range
            let mut emails = Vec::new();
            let dates = collect_thread_members(
                root_message_id,
                message_containers,
                email_data,
                0, // Real root starts at depth 0
                &mut emails,
            );

            // Use dates from collection, or fall back to root date if no dates found
            let (start_date, last_date) = dates.unwrap_or((root_data.date, root_data.date));

            // Create thread with this email as the root
            let mut thread_info = ThreadInfo::new(
                root_data.message_id.clone(),
                root_data.subject.clone(),
                start_date,
                last_date,
            );
            thread_info.emails = emails;

            return Some(thread_info);
        }
    }

    // Case 2: Phantom root - find first real message in subtree
    let (_first_real_msg_id, first_real_data) =
        find_first_real_message(root_message_id, message_containers, email_data)?;

    // Collect all members starting from phantom root and get date range
    // Start at depth -1 so phantom's direct children (first real messages) get depth 0
    let mut emails = Vec::new();
    let dates = collect_thread_members(
        root_message_id,
        message_containers,
        email_data,
        -1, // Phantom root depth offset
        &mut emails,
    );

    // Use dates from collection, or fall back to first real message date if no dates found
    let (start_date, last_date) = dates.unwrap_or((first_real_data.date, first_real_data.date));

    // Create thread using the first real message for metadata
    let mut thread_info = ThreadInfo::new(
        first_real_data.message_id.clone(),
        first_real_data.subject.clone(),
        start_date,
        last_date,
    );
    thread_info.emails = emails;

    Some(thread_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_email(id: i32, message_id: &str, in_reply_to: Option<String>) -> EmailData {
        EmailData {
            id,
            message_id: message_id.to_string(),
            subject: format!("Test {}", id),
            in_reply_to,
            date: Utc::now(),
            series_id: None,
            series_number: None,
            series_total: None,
        }
    }

    #[test]
    fn test_simple_thread() {
        let mut email_data = HashMap::new();
        let mut email_references = HashMap::new();

        // Email 1: root
        email_data.insert(1, create_test_email(1, "msg1", None));
        email_references.insert(1, vec![]);

        // Email 2: reply to email 1
        email_data.insert(2, create_test_email(2, "msg2", Some("msg1".to_string())));
        email_references.insert(2, vec!["msg1".to_string()]);

        let threads = build_email_threads(email_data, email_references);

        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].emails.len(), 2);
    }

    #[test]
    fn test_phantom_root() {
        let mut email_data = HashMap::new();
        let mut email_references = HashMap::new();

        // Email 2: references phantom message
        email_data.insert(2, create_test_email(2, "msg2", Some("msg1".to_string())));
        email_references.insert(2, vec!["msg1".to_string()]);

        let threads = build_email_threads(email_data, email_references);

        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].emails.len(), 1);
        // Email 2 should be at depth 0 (phantom parent not counted)
        assert_eq!(threads[0].emails[0].1, 0);
    }
}
