//! Core JWZ (Jamie Zawinski) threading algorithm implementation
//!
//! This module implements the main threading algorithm as described at:
//! https://www.jwz.org/doc/threading.html
//!
//! ## Algorithm Overview
//!
//! 1. **Build Container Tree**: Create containers for all messages and their references (PARALLEL)
//! 2. **Link References**: Build parent-child relationships from References header (PARALLEL)
//! 3. **Apply In-Reply-To**: Fallback for messages without References
//! 4. **Find Roots**: Identify messages with no parent (thread roots)
//! 5. **Collect Threads**: Gather all complete threads, preserving phantom structure (PARALLEL)

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dashmap::DashMap;
use rayon::prelude::*;

use super::container::{Container, EmailData, ThreadInfo};

/// Build email threads using the JWZ algorithm (PARALLELIZED)
///
/// This is the main entry point for the threading algorithm. It takes email data
/// and reference information from the database and returns a list of threads.
///
/// ## Arguments
///
/// - `email_data`: Map of email_id to EmailData for all emails
/// - `email_references`: Map of email_id to Vec<referenced_message_id>
///
/// ## Returns
///
/// A vector of ThreadInfo structures, each representing one complete thread.
///
/// ## Algorithm Steps (PARALLELIZED with Rayon + DashMap)
///
/// 1. Create all containers (real and phantom) - PARALLEL with DashMap
/// 2. Build parent-child relationships from References header - PARALLEL with DashMap
/// 3. Apply In-Reply-To fallback for messages without References
/// 4. Find root set (messages with no parent)
/// 5. Collect threads, preserving phantom structure - PARALLEL with Rayon
pub fn build_threads(
    email_data: HashMap<i32, EmailData>,
    email_references: HashMap<i32, Vec<String>>,
) -> Vec<ThreadInfo> {
    // Step 1: Build the container tree (PARALLELIZED)
    let id_table: Arc<DashMap<String, Container>> = Arc::new(DashMap::new());

    // Create containers for all real messages (PARALLEL)
    email_data.par_iter().for_each(|(email_id, data)| {
        let msg_id = data.message_id.clone();
        id_table
            .entry(msg_id.clone())
            .or_insert_with(|| Container::new_with_email(msg_id, *email_id));
    });

    // Create phantom containers for all referenced messages (PARALLEL)
    email_references.par_iter().for_each(|(_, refs)| {
        for ref_msg_id in refs {
            id_table
                .entry(ref_msg_id.clone())
                .or_insert_with(|| Container::new_phantom(ref_msg_id.clone()));
        }
    });

    // Step 2: Build parent-child relationships from References header (PARALLELIZED)
    link_references(&id_table, &email_data, &email_references);

    // Step 3: Apply In-Reply-To fallback
    link_in_reply_to(&id_table, &email_data);

    // Step 4: Find root set (messages with no parent)
    let root_set: Vec<String> = id_table
        .iter()
        .filter(|entry| entry.value().parent.is_none())
        .map(|entry| entry.key().clone())
        .collect();

    // Step 5: Collect threads (handle phantoms correctly) - PARALLELIZED
    collect_threads(root_set, &id_table, &email_data)
}

/// Link messages based on References header (PARALLELIZED)
///
/// The References header contains a space-separated list of message IDs
/// representing the full conversation history. We build the chain by linking
/// each reference to the next one, and finally linking the message to the
/// last reference.
///
/// ## Example
///
/// If a message has References: `<msg1> <msg2> <msg3>`, we create:
/// - msg1 (parent) → msg2 (child)
/// - msg2 (parent) → msg3 (child)
/// - msg3 (parent) → this_message (child)
///
/// This builds the complete conversation chain.
///
/// This function is now parallelized using Rayon to process multiple emails concurrently.
/// DashMap's concurrent access ensures thread-safe operations.
fn link_references(
    id_table: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
    email_references: &HashMap<i32, Vec<String>>,
) {
    email_data.par_iter().for_each(|(email_id, data)| {
        let msg_id = data.message_id.clone();

        if let Some(refs) = email_references.get(email_id) {
            // Link the references chain: refs[0] -> refs[1] -> refs[2] -> ... -> this_message
            let mut prev_ref: Option<String> = None;

            for ref_msg_id in refs {
                // Link previous reference to this one
                if let Some(prev) = &prev_ref {
                    link_if_no_parent(id_table, ref_msg_id, prev);
                }

                prev_ref = Some(ref_msg_id.clone());
            }

            // Link last reference to this message
            if let Some(last_ref) = prev_ref {
                link_if_no_parent(id_table, &msg_id, &last_ref);
            }
        }
    });
}

/// Link messages based on In-Reply-To header
///
/// For messages without References but with In-Reply-To, we can still
/// establish a parent-child relationship. This is a simpler form of threading
/// that only looks at the immediate parent.
fn link_in_reply_to(
    id_table: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
) {
    for (_email_id, data) in email_data {
        let msg_id = &data.message_id;

        // Skip if already has a parent from References
        if id_table
            .get(msg_id)
            .map(|c| c.parent.is_some())
            .unwrap_or(false)
        {
            continue;
        }

        if let Some(in_reply_to) = &data.in_reply_to {
            if id_table.contains_key(in_reply_to) {
                link_if_no_parent(id_table, msg_id, in_reply_to);
            }
        }
    }
}

/// Link a child to a parent if the child doesn't already have a parent
///
/// This helper function safely links two containers, checking for:
/// - Child doesn't already have a parent
/// - Avoids creating cycles
/// - Avoids duplicate children
///
/// Now uses DashMap for thread-safe concurrent access.
fn link_if_no_parent(
    id_table: &DashMap<String, Container>,
    child_msg_id: &str,
    parent_msg_id: &str,
) {
    // Don't link a message to itself
    if child_msg_id == parent_msg_id {
        return;
    }

    // Check if child already has a parent
    if id_table
        .get(child_msg_id)
        .map(|c| c.parent.is_some())
        .unwrap_or(false)
    {
        return;
    }

    // Check for potential cycle (parent is already a descendant of child)
    if would_create_cycle(id_table, child_msg_id, parent_msg_id) {
        return;
    }

    // Set parent on child
    if let Some(mut child) = id_table.get_mut(child_msg_id) {
        child.parent = Some(parent_msg_id.to_string());
    }

    // Add child to parent's children list
    if let Some(mut parent) = id_table.get_mut(parent_msg_id) {
        parent.add_child(child_msg_id.to_string());
    }
}

/// Check if linking child to parent would create a cycle
///
/// We need to ensure that parent is not already a descendant of child,
/// which would create a cycle in the tree.
///
/// Now uses DashMap for thread-safe access.
fn would_create_cycle(
    id_table: &DashMap<String, Container>,
    child_msg_id: &str,
    parent_msg_id: &str,
) -> bool {
    let mut visited = HashSet::new();
    let mut current = Some(parent_msg_id.to_string());

    while let Some(msg_id) = current {
        // If we've seen this before, we have a cycle
        if !visited.insert(msg_id.clone()) {
            return true;
        }

        // If parent would become descendant of child, that's a cycle
        if msg_id == child_msg_id {
            return true;
        }

        // Move up the tree
        current = id_table.get(&msg_id).and_then(|c| c.parent.clone());
    }

    false
}

/// Collect all threads from the root set (PARALLELIZED)
///
/// This function processes the root set to create ThreadInfo structures.
/// It handles both real roots (messages with email data) and phantom roots
/// (referenced but missing messages).
///
/// ## Phantom Root Handling
///
/// Unlike the old implementation, we DO NOT promote phantom children to roots.
/// Instead, we traverse into phantom containers to find the first real message
/// in the tree and use that as the thread root, preserving the phantom structure
/// in the depth calculations. This matches public-inbox behavior.
///
/// This function is now parallelized using Rayon - each root thread is processed
/// independently in parallel, which provides significant speedup for large datasets.
fn collect_threads(
    root_set: Vec<String>,
    id_table: &Arc<DashMap<String, Container>>,
    email_data: &HashMap<i32, EmailData>,
) -> Vec<ThreadInfo> {
    // Process each root in parallel
    root_set
        .par_iter()
        .filter_map(|root_msg_id| {
            if let Some(root_container) = id_table.get(root_msg_id) {
                // Case 1: Real root (has email data)
                if let Some(root_email_id) = root_container.email_id {
                    if let Some(root_data) = email_data.get(&root_email_id) {
                        let mut thread_info = ThreadInfo::new(
                            root_data.message_id.clone(),
                            root_data.subject.clone(),
                            root_data.date,
                        );

                        // Collect all emails in this thread
                        collect_thread_emails_dashmap(
                            root_msg_id,
                            id_table,
                            email_data,
                            0,
                            &mut thread_info.emails,
                        );

                        return Some(thread_info);
                    }
                }
                // Case 2: Phantom root - find first real message in subtree
                else {
                    // Find the first real (non-phantom) message in this tree
                    if let Some((_first_real_msg_id, first_real_data)) =
                        find_first_real_message_dashmap(root_msg_id, id_table, email_data)
                    {
                        let mut thread_info = ThreadInfo::new(
                            first_real_data.message_id.clone(),
                            first_real_data.subject.clone(),
                            first_real_data.date,
                        );

                        // Collect all real emails in this thread starting from the phantom root
                        // Since the phantom itself isn't added, start at depth -1 so that
                        // the phantom's direct children (first real messages) get depth 0
                        collect_thread_emails_dashmap(
                            root_msg_id,
                            id_table,
                            email_data,
                            -1,
                            &mut thread_info.emails,
                        );

                        return Some(thread_info);
                    }
                }
            }
            None
        })
        .collect()
}

/// Iteratively find the first real (non-phantom) message in a subtree (DashMap version)
///
/// This is used when the root of a thread is a phantom - we need to find
/// a real message to use for the thread metadata (subject, date).
///
/// Uses an iterative depth-first search to avoid stack overflow on deep threads.
fn find_first_real_message_dashmap<'a>(
    msg_id: &str,
    id_table: &DashMap<String, Container>,
    email_data: &'a HashMap<i32, EmailData>,
) -> Option<(String, &'a EmailData)> {
    // Use a stack for iterative DFS
    let mut stack = vec![msg_id.to_string()];

    while let Some(current_msg_id) = stack.pop() {
        if let Some(container) = id_table.get(&current_msg_id) {
            // If this container has real email data, return it
            if let Some(email_id) = container.email_id {
                if let Some(data) = email_data.get(&email_id) {
                    return Some((current_msg_id.clone(), data));
                }
            }

            // Clone children to avoid holding the DashMap lock
            // Add children to stack in reverse order for correct DFS order
            let children = container.children.clone();
            drop(container);

            for child_msg_id in children.iter().rev() {
                stack.push(child_msg_id.clone());
            }
        }
    }

    None
}

/// Iteratively collect all emails in a thread with their depths (DashMap version)
///
/// This function performs a depth-first traversal of the thread tree,
/// collecting all real emails (non-phantoms) and their depth in the tree.
///
/// Uses an iterative approach with an explicit stack to avoid stack overflow on deep threads.
fn collect_thread_emails_dashmap(
    msg_id: &str,
    id_table: &DashMap<String, Container>,
    _email_data: &HashMap<i32, EmailData>,
    depth: i32,
    result: &mut Vec<(i32, i32)>,
) {
    // Use a stack for iterative DFS: (message_id, depth)
    let mut stack = vec![(msg_id.to_string(), depth)];

    while let Some((current_msg_id, current_depth)) = stack.pop() {
        if let Some(container) = id_table.get(&current_msg_id) {
            // Add this email if it exists (not a phantom)
            if let Some(email_id) = container.email_id {
                result.push((email_id, current_depth));
            }

            // Clone children to avoid holding the DashMap lock
            // Add children to stack in reverse order for correct DFS order
            let children = container.children.clone();
            drop(container);

            for child_msg_id in children.iter().rev() {
                stack.push((child_msg_id.clone(), current_depth + 1));
            }
        }
    }
}
