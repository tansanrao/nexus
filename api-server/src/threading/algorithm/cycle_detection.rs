//! Cycle detection for email threading
//!
//! Ensures that parent-child relationships don't create cycles in the thread tree.
//! A cycle would occur if we tried to make a parent a descendant of its own child.

use dashmap::DashMap;
use std::collections::HashSet;

use super::super::container::Container;

/// Check if linking a child to a parent would create a cycle
///
/// Traverses up the ancestry chain from the proposed parent to ensure
/// it's not already a descendant of the proposed child. This prevents
/// circular relationships in the thread tree.
///
/// ## Algorithm
///
/// Starting from the parent, walk up the tree following parent links.
/// If we encounter the child anywhere in this chain, linking would
/// create a cycle.
///
/// ## Arguments
///
/// * `message_containers` - The complete container map (message_id → Container)
/// * `child_message_id` - Message ID of the proposed child
/// * `parent_message_id` - Message ID of the proposed parent
///
/// ## Returns
///
/// `true` if linking would create a cycle, `false` if safe to link
///
/// ## Example
///
/// ```text
/// Current tree:  A → B → C
///
/// Trying to link: C → A (would create cycle A → B → C → A)
/// Result: true (cycle detected)
///
/// Trying to link: C → D (D is not in chain)
/// Result: false (safe to link)
/// ```
pub fn detect_cycle_in_ancestry(
    message_containers: &DashMap<String, Container>,
    child_message_id: &str,
    parent_message_id: &str,
) -> bool {
    // Track visited nodes to detect cycles in the parent chain itself
    let mut visited_message_ids = HashSet::new();
    let mut current_message_id = Some(parent_message_id.to_string());

    // Walk up the ancestry chain from parent
    while let Some(msg_id) = current_message_id {
        // If we've seen this node before, there's a cycle in the parent chain
        if !visited_message_ids.insert(msg_id.clone()) {
            return true;
        }

        // If we find the child in the parent's ancestry, linking would create a cycle
        if msg_id == child_message_id {
            return true;
        }

        // Move up to the next parent in the chain
        current_message_id = message_containers
            .get(&msg_id)
            .and_then(|container| container.parent.clone());
    }

    // No cycle detected - safe to link
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_container(message_id: String, parent: Option<String>) -> Container {
        Container {
            message_id: message_id.clone(),
            email_id: None,
            parent,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_no_cycle_simple_chain() {
        // Setup: A → B → C
        let containers = DashMap::new();
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), None),
        );
        containers.insert(
            "B".to_string(),
            create_test_container("B".to_string(), Some("A".to_string())),
        );
        containers.insert(
            "C".to_string(),
            create_test_container("C".to_string(), Some("B".to_string())),
        );

        // Trying to link D → C is safe
        assert!(!detect_cycle_in_ancestry(&containers, "D", "C"));
    }

    #[test]
    fn test_cycle_detected() {
        // Setup: A → B → C
        let containers = DashMap::new();
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), None),
        );
        containers.insert(
            "B".to_string(),
            create_test_container("B".to_string(), Some("A".to_string())),
        );
        containers.insert(
            "C".to_string(),
            create_test_container("C".to_string(), Some("B".to_string())),
        );

        // Trying to link A → C would create a cycle
        assert!(detect_cycle_in_ancestry(&containers, "A", "C"));
    }

    #[test]
    fn test_self_loop() {
        // Setup: A (no parent)
        let containers = DashMap::new();
        containers.insert(
            "A".to_string(),
            create_test_container("A".to_string(), None),
        );

        // Trying to link A → A (self-loop) should be detected
        // Note: This is handled by the caller, but cycle detection catches it too
        assert!(detect_cycle_in_ancestry(&containers, "A", "A"));
    }
}
