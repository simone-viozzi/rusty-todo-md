// Allow deprecated functions for backward compatibility
#![allow(deprecated)]

//! # TODO Extractor API
//!
//! This module provides functions for extracting TODO/FIXME/HACK comments from source code files.
//!
//! ## Recommended API (v0.1.8+)
//! - [`extract_todos_with_parser`] - Optimized function that requires pre-validation of file extensions
//! - [`get_effective_extension`] - Get the effective file extension for parsing
//! - [`get_parser_for_extension`] - Get the appropriate parser for a file extension
//! - [`is_file_supported`] - Check if a file extension is supported
//!
//! ## Legacy API (deprecated)
//! - [`extract_todos_with_config`] - Deprecated: reads file content before checking if extension is supported
//! - [`extract_marked_items`] - Deprecated: internal function exposed for backward compatibility
//!
//! The new API allows you to check file extensions before reading file content, significantly
//! improving performance when processing many files with potentially unsupported extensions.

use std::path::Path;

// Re-export the new optimized functions
pub use crate::todo_extractor_internal::aggregator::{
    extract_marked_items_with_parser, get_effective_extension, get_parser_for_extension,
    CommentLine, MarkedItem, MarkerConfig,
};

// Re-export deprecated function for backward compatibility
#[deprecated(
    since = "0.1.8",
    note = "Use `extract_marked_items_with_parser` combined with `get_parser_for_extension` for better performance. This function checks file extension after reading content."
)]
pub use crate::todo_extractor_internal::aggregator::extract_marked_items;

#[deprecated(
    since = "0.1.8",
    note = "Use `extract_todos_with_parser` or `extract_marked_items_with_parser` for better performance. This function reads file content before checking if the extension is supported."
)]
pub fn extract_todos_with_config(
    file_path: &Path,
    content: &str,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items(file_path, content, config)
}

/// Extracts TODO comments from source code using a provided parser function.
///
/// **RECOMMENDED**: This is the preferred function for extracting TODO comments as it allows
/// you to validate file extensions before reading file content, improving performance.
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use rusty_todo_md::todo_extractor::*;
///
/// let path = Path::new("example.rs");
/// let effective_ext = get_effective_extension(path);
///
/// if let Some(parser_fn) = get_parser_for_extension(&effective_ext) {
///     let content = std::fs::read_to_string(path).unwrap();
///     let config = MarkerConfig::default();
///     let items = extract_todos_with_parser(path, &content, parser_fn, &config);
/// }
/// ```
pub fn extract_todos_with_parser(
    file_path: &Path,
    content: &str,
    parser_fn: fn(&str) -> Vec<CommentLine>,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items_with_parser(file_path, content, parser_fn, config)
}
