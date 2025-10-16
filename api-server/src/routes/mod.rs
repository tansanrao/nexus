//! HTTP route handlers grouped by resource domain.
//!
//! Each submodule corresponds to a logical area of the API
//! (authors, threads, mailing lists, etc.) and exposes typed Rocket
//! handlers annotated with `#[openapi]` so `rocket_okapi` can derive
//! an OpenAPI document automatically.

pub mod admin;
pub mod authors;
pub mod emails;
pub(crate) mod helpers;
pub mod mailing_lists;
pub mod params;
pub mod stats;
pub mod threads;
