mod embeddings;
mod error;
mod indexer;
mod models;
pub mod sanitize;
mod service;

pub use embeddings::EmbeddingsClient;
pub use error::SearchError;
pub use indexer::{reindex_authors, reindex_threads};
pub use models::{AuthorDocument, AuthorMailingListStats, ThreadDocument};
pub use service::{
    AuthorHit, AuthorSearchPayload, AuthorSearchResults, SearchService, ThreadHit,
    ThreadMailingListFilter, ThreadSearchPayload, ThreadSearchResults,
};
