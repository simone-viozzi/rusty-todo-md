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
    extract_marked_items_with_parser, get_effective_extension,
    get_parser_for_extension, CommentLine, MarkedItem, MarkerConfig, extract_marked_items_from_file
};

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
#[deprecated(
    since = "0.1.8",
    note = "Use pub_extract_marked_items_from_file for optimized extraction. This function is kept for backward compatibility."
)]
pub fn extract_todos_with_parser(
    file_path: &Path,
    content: &str,
    parser_fn: fn(&str) -> Vec<CommentLine>,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items_with_parser(file_path, content, parser_fn, config)
}

/// Extracts TODO comments from a file by automatically determining the appropriate parser.
///
/// This is a convenient high-level function that handles the complete workflow:
/// 1. Determines the effective file extension
/// 2. Gets the appropriate parser for that extension
/// 3. Reads the file content
/// 4. Extracts marked items using the parser
///
/// **Note**: This function is optimized to check file extension support before reading
/// file content, so it efficiently skips unsupported file types without I/O overhead.
/// For cases where you already have the file content in memory, you can use
/// [`extract_todos_with_parser`] directly.
///
/// # Arguments
/// - `file`: Path to the source code file to analyze
/// - `marker_config`: Configuration specifying which markers to look for (e.g., TODO, FIXME, HACK)
///
/// # Returns
/// - `Ok(Vec<MarkedItem>)`: Vector of found marked items, empty if file is unsupported
/// - `Err(String)`: Error message if the file cannot be read
///
/// # Example
/// ```rust,no_run
/// use std::path::Path;
/// use rusty_todo_md::todo_extractor::*;
///
/// let path = Path::new("src/main.rs");
/// let config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
///
/// match pub_extract_marked_items_from_file(path, &config) {
///     Ok(items) => {
///         for item in items {
///             println!("{}:{} [{}] {}",
///                 item.file_path.display(),
///                 item.line_number,
///                 item.marker,
///                 item.message
///             );
///         }
///     }
///     Err(e) => eprintln!("Error processing file: {}", e),
/// }
/// ```
pub fn pub_extract_marked_items_from_file(
    file: &Path,
    marker_config: &MarkerConfig,
) -> Result<Vec<MarkedItem>, String> {
    extract_marked_items_from_file(file, marker_config)
}
