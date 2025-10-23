//! Search utilities: embedding clients, configuration, and shared types.

pub mod client;
pub mod config;
pub mod types;

pub use client::{EmbeddingClient, EmbeddingError};
pub use config::{EmbeddingConfig, SearchConfig};
pub use types::SearchMode;
