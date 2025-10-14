//! Email threading module
//!
//! This module implements the JWZ (Jamie Zawinski) threading algorithm for organizing
//! email messages into conversation threads. The implementation is based on the algorithm
//! from https://www.jwz.org/doc/threading.html with specific adaptations for the Linux
//! kernel mailing list structure.
//!
//! ## Threading Strategy
//!
//! The algorithm uses the standard JWZ threading algorithm, relying solely on email headers:
//!
//! 1. **References Header**: The primary method - uses the full chain of message IDs from
//!    the References header to build parent-child relationships
//! 2. **In-Reply-To Header**: Fallback for messages without References but with In-Reply-To
//!
//! This matches the exact behavior of public-inbox and lore.kernel.org.
//!
//! ## Module Structure
//!
//! - `container`: Data structures for the threading algorithm
//! - `jwz_algorithm`: Core JWZ threading implementation
//! - `patch_series`: Patch series detection (metadata extraction only)
//! - `epoch_cache`: Caching system for threading data

pub mod container;
pub mod epoch_cache;
pub mod jwz_algorithm;
pub mod patch_series;

// Re-export main types and functions
pub use container::EmailData;
pub use epoch_cache::{EmailThreadingInfo, MailingListCache};
pub use jwz_algorithm::build_threads;
pub use patch_series::extract_patch_series_info;
