//! Search utilities: embedding clients, configuration, and shared types.

pub mod client;
pub mod config;
pub mod text;
pub mod types;

pub use client::{EmbeddingClient, EmbeddingError};
pub use config::{EmbeddingConfig, SearchConfig};
pub use text::{build_email_embedding_text, build_embedding_text_from_parts};
pub use types::SearchMode;
