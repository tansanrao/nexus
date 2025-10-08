//! Container data structures for the JWZ threading algorithm
//!
//! The JWZ algorithm uses a "container" abstraction to represent both real messages
//! and "phantom" messages (messages referenced but not present in our dataset).

use std::collections::HashMap;

/// A container represents a node in the thread tree.
///
/// Each container has:
/// - A message_id (unique identifier)
/// - An optional email_id (None for phantom containers)
/// - An optional parent reference
/// - A list of children
///
/// ## Phantom Containers
///
/// Phantom containers are created for messages that are referenced (e.g., in the
/// References header) but not present in our database. This allows us to maintain
/// the complete thread structure even when we're missing some messages.
#[derive(Debug, Clone)]
pub struct Container {
    /// The Message-ID of this container
    pub message_id: String,

    /// Database email ID if this message exists in our dataset (None for phantoms)
    pub email_id: Option<i32>,

    /// Message-ID of the parent container (None for root messages)
    pub parent: Option<String>,

    /// List of child message IDs
    pub children: Vec<String>,
}

impl Container {
    /// Create a new container for a real email
    pub fn new_with_email(message_id: String, email_id: i32) -> Self {
        Container {
            message_id,
            email_id: Some(email_id),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Create a new phantom container (for referenced but missing messages)
    pub fn new_phantom(message_id: String) -> Self {
        Container {
            message_id,
            email_id: None,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Check if this container is a phantom (no associated email)
    pub fn is_phantom(&self) -> bool {
        self.email_id.is_none()
    }

    /// Add a child to this container (avoiding duplicates)
    pub fn add_child(&mut self, child_msg_id: String) {
        if !self.children.contains(&child_msg_id) {
            self.children.push(child_msg_id);
        }
    }
}

/// Email data loaded from the database
///
/// This structure contains all the information we need about an email
/// to perform threading operations using the JWZ algorithm.
#[derive(Debug, Clone)]
pub struct EmailData {
    /// Database ID
    pub id: i32,

    /// Message-ID from email header
    pub message_id: String,

    /// Original subject line
    pub subject: String,

    /// In-Reply-To header value
    pub in_reply_to: Option<String>,

    /// Date the email was sent
    pub date: chrono::DateTime<chrono::Utc>,

    /// Patch series information (metadata only, not used for threading)
    pub series_id: Option<String>,
    pub series_number: Option<i32>,
    pub series_total: Option<i32>,
}

impl EmailData {
    /// Check if this email is part of a patch series
    pub fn is_patch_series(&self) -> bool {
        self.series_number.is_some()
    }

    /// Check if this email is a cover letter (patch 0/N)
    pub fn is_cover_letter(&self) -> bool {
        self.series_number == Some(0)
    }
}

/// Information about a complete thread after building
///
/// This is the final output of the threading algorithm, containing
/// all the information needed to insert the thread into the database.
#[derive(Debug)]
pub struct ThreadInfo {
    /// Message-ID of the root message
    pub root_message_id: String,

    /// Subject of the thread (from root message)
    pub subject: String,

    /// Earliest date in the thread
    pub start_date: chrono::DateTime<chrono::Utc>,

    /// List of (email_id, depth) pairs for all messages in the thread
    pub emails: Vec<(i32, i32)>,
}

impl ThreadInfo {
    /// Create a new ThreadInfo
    pub fn new(
        root_message_id: String,
        subject: String,
        start_date: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        ThreadInfo {
            root_message_id,
            subject,
            start_date,
            emails: Vec::new(),
        }
    }

    /// Add an email to this thread with its depth
    pub fn add_email(&mut self, email_id: i32, depth: i32) {
        self.emails.push((email_id, depth));
    }
}

/// Recursively collect all emails in a thread with their depths
///
/// This function performs a depth-first traversal of the thread tree,
/// collecting all real emails (non-phantoms) and their depth in the tree.
///
/// ## Arguments
///
/// - `msg_id`: Message-ID of the current node to process
/// - `id_table`: The complete container table
/// - `email_data`: Map of email_id to EmailData
/// - `depth`: Current depth in the tree (0 for root)
/// - `result`: Output vector to accumulate (email_id, depth) pairs
pub fn collect_thread_emails(
    msg_id: &str,
    id_table: &HashMap<String, Container>,
    email_data: &HashMap<i32, EmailData>,
    depth: i32,
    result: &mut Vec<(i32, i32)>,
) {
    if let Some(container) = id_table.get(msg_id) {
        // Add this email if it exists (not a phantom)
        if let Some(email_id) = container.email_id {
            result.push((email_id, depth));
        }

        // Recursively process all children
        for child_msg_id in &container.children {
            collect_thread_emails(child_msg_id, id_table, email_data, depth + 1, result);
        }
    }
}
