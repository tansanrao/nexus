//! Bulk email import system.
//!
//! This module provides a complete pipeline for importing parsed emails into
//! the database with optimal performance:
//!
//! 1. **Data Preparation** (`data_builder`) - Transforms parsed emails into columnar format
//! 2. **Database Operations** (`database_operations`) - Bulk inserts using PostgreSQL UNNEST
//! 3. **Coordination** (`coordinator`) - Orchestrates the entire import process
//! 4. **Statistics** (`stats`) - Tracks import metrics
//!
//! # Architecture
//!
//! The import system uses parallel database connections (up to 6 concurrent)
//! and PostgreSQL's UNNEST for efficient bulk operations. Data is processed
//! in chunks to avoid memory issues and connection timeouts.
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use crate::sync::import::{BulkImporter, ImportStats};
//! use crate::threading::MailingListCache;
//!
//! let importer = BulkImporter::new(pool, mailing_list_id);
//! let cache = MailingListCache::new(mailing_list_id);
//!
//! let stats = importer
//!     .import_chunk_with_epoch_cache(&parsed_emails, &cache)
//!     .await?;
//!
//! println!("Imported {} emails", stats.emails);
//! ```

pub mod coordinator;
pub mod data_builder;
pub mod data_structures;
pub mod database_operations;
pub mod stats;

// Re-export main types
pub use coordinator::BulkImporter;
pub use stats::ImportStats;
