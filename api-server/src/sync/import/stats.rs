//! Import statistics tracking.
//!
//! Tracks the number of records inserted during email import operations.

/// Statistics for a single import operation.
///
/// Tracks the number of records inserted across all related tables.
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    /// Number of author records inserted (may include updates)
    pub authors: usize,
    /// Number of email records inserted
    pub emails: usize,
    /// Number of recipient records inserted
    pub recipients: usize,
    /// Number of reference records inserted
    pub references: usize,
    /// Number of thread records inserted
    pub threads: usize,
    /// Number of thread membership records inserted
    pub thread_memberships: usize,
}

impl ImportStats {
    /// Merge another ImportStats into this one by summing all counts.
    ///
    /// Used to combine statistics from multiple import chunks.
    pub fn merge(&mut self, other: ImportStats) {
        self.authors += other.authors;
        self.emails += other.emails;
        self.recipients += other.recipients;
        self.references += other.references;
        self.threads += other.threads;
        self.thread_memberships += other.thread_memberships;
    }
}
