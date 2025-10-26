//! Database management for the sync system.
//!
//! This module provides database operations including:
//! - Schema migrations
//! - Partition management per mailing list
//! - Checkpoint tracking for incremental sync

pub mod checkpoint;
pub mod migration;
pub mod partition;

// Re-export commonly used functions
pub use checkpoint::{load_last_indexed_commits, save_last_indexed_commits, save_last_threaded_at};
pub use migration::{reset_database, run_migrations};
pub use partition::{create_mailing_list_partitions, drop_mailing_list_partitions};
