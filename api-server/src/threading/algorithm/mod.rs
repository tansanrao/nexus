//! Email threading algorithm implementation
//!
//! This module implements the JWZ (Jamie Zawinski) threading algorithm
//! for organizing email messages into conversation threads.
//!
//! ## Main Entry Point
//!
//! Use `build_email_threads()` to thread a collection of emails.

mod cycle_detection;
mod jwz_threading;
mod tree_traversal;

// Re-export the main threading function
pub use jwz_threading::build_email_threads;
