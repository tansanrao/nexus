//! Backward compatibility re-exports for bulk import functionality.
//!
//! This module provides re-exports from the refactored `import` module
//! to maintain backward compatibility with existing code.
//!
//! **Deprecated**: New code should use `crate::sync::import` directly.

pub use crate::sync::import::{BulkImporter, ImportStats};
