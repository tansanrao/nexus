//! Tree traversal utilities for email threading
//!
//! Functions for traversing the thread tree to collect emails and find
//! real messages in phantom-rooted trees. All functions use iterative
//! approaches to avoid stack overflow on deeply nested threads.

use dashmap::DashMap;
use std::collections::HashMap;

use super::super::container::{Container, EmailData};

/// Find the first real (non-phantom) message in a subtree
///
/// When a thread root is a phantom (referenced but missing message), we need
/// to find a real message to use for the thread metadata (subject, date).
/// This performs a depth-first search to find the first real message.
///
/// ## Algorithm
///
/// Uses iterative depth-first search with an explicit stack to avoid
/// stack overflow on deeply nested threads.
///
/// ## Arguments
///
/// * `root_message_id` - Message ID to start searching from
/// * `message_containers` - Map of message_id → Container
/// * `email_data` - Map of email_id → EmailData for real emails
///
/// ## Returns
///
/// `Some((message_id, EmailData))` if a real message is found,
/// `None` if the entire subtree contains only phantoms
pub fn find_first_real_message<'a>(
    root_message_id: &str,
    message_containers: &DashMap<String, Container>,
    email_data: &'a HashMap<i32, EmailData>,
) -> Option<(String, &'a EmailData)> {
    // Stack for iterative depth-first search
    // Avoids recursion to prevent stack overflow on deep threads
    let mut search_stack = vec![root_message_id.to_string()];

    while let Some(current_message_id) = search_stack.pop() {
        if let Some(container) = message_containers.get(&current_message_id) {
            // If this container has real email data, we found it!
            if let Some(email_id) = container.email_id {
                if let Some(data) = email_data.get(&email_id) {
                    return Some((current_message_id.clone(), data));
                }
            }

            // No real message here - add children to search stack
            // Clone children to release the DashMap lock
            let children = container.children.clone();
            drop(container);

            // Add children in reverse order to maintain DFS left-to-right order
            for child_message_id in children.iter().rev() {
                search_stack.push(child_message_id.clone());
            }
        }
    }

    // No real messages found in entire subtree
    None
}

/// Collect all emails in a thread with their depth values
///
/// Performs a depth-first traversal of the thread tree starting from the
/// given message, collecting all real emails (non-phantoms) along with
/// their depth in the tree. Phantoms are skipped but their depth is
/// counted to preserve tree structure.
///
/// ## Algorithm
///
/// Uses iterative depth-first search with an explicit stack. Each stack
/// entry contains (message_id, depth_in_tree).
///
/// ## Arguments
///
/// * `root_message_id` - Message ID to start traversal from
/// * `message_containers` - Map of message_id → Container
/// * `email_data` - Map of email_id → EmailData (not directly used but kept for API consistency)
/// * `starting_depth` - Initial depth value (usually 0, or -1 for phantom roots)
/// * `collected_members` - Output vector to accumulate (email_id, depth) pairs
///
/// ## Depth Handling
///
/// - Real messages: depth is current depth value
/// - Phantom messages: not added to results, but depth is incremented for their children
/// - For phantom roots: start at depth -1 so first real children get depth 0
pub fn collect_thread_members(
    root_message_id: &str,
    message_containers: &DashMap<String, Container>,
    _email_data: &HashMap<i32, EmailData>,
    starting_depth: i32,
    collected_members: &mut Vec<(i32, i32)>,
) {
    // Stack for iterative DFS: (message_id, depth_in_tree)
    let mut traversal_stack = vec![(root_message_id.to_string(), starting_depth)];

    while let Some((current_message_id, current_depth)) = traversal_stack.pop() {
        if let Some(container) = message_containers.get(&current_message_id) {
            // Add this email if it's real (not a phantom)
            if let Some(email_id) = container.email_id {
                collected_members.push((email_id, current_depth));
            }

            // Add children to stack for processing
            // Clone children to release DashMap lock quickly
            let children = container.children.clone();
            drop(container);

            // Add children in reverse order to maintain DFS left-to-right order
            // Each child's depth is one more than current depth
            for child_message_id in children.iter().rev() {
                traversal_stack.push((child_message_id.clone(), current_depth + 1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_container(
        message_id: String,
        email_id: Option<i32>,
        children: Vec<String>,
    ) -> Container {
        Container {
            message_id: message_id.clone(),
            email_id,
            parent: None,
            children,
        }
    }

    fn create_test_email_data(email_id: i32, message_id: String) -> EmailData {
        EmailData {
            id: email_id,
            message_id,
            subject: format!("Test {}", email_id),
            in_reply_to: None,
            date: Utc::now(),
            series_id: None,
            series_number: None,
            series_total: None,
        }
    }

    #[test]
    fn test_find_real_message_immediate() {
        let containers = DashMap::new();
        let mut email_data = HashMap::new();

        // Root is real
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), Some(1), vec![]),
        );
        email_data.insert(1, create_test_email_data(1, "A".to_string()));

        let result = find_first_real_message("A", &containers, &email_data);
        assert!(result.is_some());
        let (msg_id, _) = result.unwrap();
        assert_eq!(msg_id, "A");
    }

    #[test]
    fn test_find_real_message_in_child() {
        let containers = DashMap::new();
        let mut email_data = HashMap::new();

        // Root is phantom, child is real
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), None, vec!["B".to_string()]),
        );
        containers.insert(
            "B".to_string(),
            create_test_container("B".to_string(), Some(2), vec![]),
        );
        email_data.insert(2, create_test_email_data(2, "B".to_string()));

        let result = find_first_real_message("A", &containers, &email_data);
        assert!(result.is_some());
        let (msg_id, _) = result.unwrap();
        assert_eq!(msg_id, "B");
    }

    #[test]
    fn test_collect_simple_thread() {
        let containers = DashMap::new();
        let email_data = HashMap::new();

        // A → B → C (all real)
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), Some(1), vec!["B".to_string()]),
        );
        containers.insert(
            "B".to_string(),
            create_test_container("B".to_string(), Some(2), vec!["C".to_string()]),
        );
        containers.insert(
            "C".to_string(),
            create_test_container("C".to_string(), Some(3), vec![]),
        );

        let mut members = Vec::new();
        collect_thread_members("A", &containers, &email_data, 0, &mut members);

        assert_eq!(members.len(), 3);
        assert_eq!(members[0], (1, 0)); // A at depth 0
        assert_eq!(members[1], (2, 1)); // B at depth 1
        assert_eq!(members[2], (3, 2)); // C at depth 2
    }

    #[test]
    fn test_collect_with_phantom() {
        let containers = DashMap::new();
        let email_data = HashMap::new();

        // A (phantom) → B (real) → C (real)
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), None, vec!["B".to_string()]),
        );
        containers.insert(
            "B".to_string(),
            create_test_container("B".to_string(), Some(2), vec!["C".to_string()]),
        );
        containers.insert(
            "C".to_string(),
            create_test_container("C".to_string(), Some(3), vec![]),
        );

        let mut members = Vec::new();
        // Start at -1 so phantom's children get depth 0
        collect_thread_members("A", &containers, &email_data, -1, &mut members);

        assert_eq!(members.len(), 2);
        assert_eq!(members[0], (2, 0)); // B at depth 0 (phantom parent not counted)
        assert_eq!(members[1], (3, 1)); // C at depth 1
    }
}
