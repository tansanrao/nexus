//! Caching system for email threading operations
//!
//! This module provides caching infrastructure to avoid repeated database queries
//! during threading operations. The cache stores email metadata and reference
//! information needed by the JWZ threading algorithm.
//!
//! ## Architecture
//!
//! Uses a unified cache approach where all emails for a mailing list are cached
//! together, providing a complete view for threading operations.

mod mailing_list_cache;
mod types;

// Re-export public types
pub use mailing_list_cache::MailingListCache;
pub use types::{CacheError, EmailThreadingInfo, UnifiedCacheStats};
