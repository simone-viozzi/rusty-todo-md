//! Extracts TODO/FIXME/HACK comments from source code files.
//!
//! This module provides the main API for extracting marked comments from source files.
//! It automatically determines the appropriate parser based on file extension and
//! supports multiple programming languages.

// Private implementation modules
mod todo_extractor_internal;

// Re-export the public API
pub use todo_extractor_internal::aggregator::{
    extract_marked_items_from_file, CommentLine, MarkedItem, MarkerConfig,
};
