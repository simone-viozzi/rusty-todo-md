//! Extracts TODO/FIXME/HACK comments from source code files.

use std::path::Path;

// Re-export the new optimized functions
pub use crate::todo_extractor_internal::aggregator::{
    extract_marked_items_from_file, CommentLine, MarkedItem, MarkerConfig,
};

/// Extracts marked items from a file.
///
/// Automatically determines the appropriate parser based on file extension,
/// reads the file content, and extracts marked items.
///
/// Returns an empty vector for unsupported file types.
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
