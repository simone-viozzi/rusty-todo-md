use std::path::Path;

pub use crate::todo_extractor_internal::aggregator::{
    extract_marked_items, MarkedItem, MarkerConfig,
};

pub fn extract_todos_with_config(
    file_path: &Path,
    content: &str,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items(file_path, content, config)
}
