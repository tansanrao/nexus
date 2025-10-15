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
//! - `algorithm`: Core JWZ threading implementation with cycle detection and tree traversal
//! - `cache`: Caching system for threading data to avoid repeated database queries
//! - `container`: Data structures used by the threading algorithm
//! - `patch_series`: Patch series detection (metadata extraction only)

mod algorithm;
mod cache;
pub mod container;
pub mod patch_series;

// Re-export main types and functions
pub use algorithm::build_email_threads;
pub use cache::{EmailThreadingInfo, MailingListCache};
pub use patch_series::extract_patch_series_info;
