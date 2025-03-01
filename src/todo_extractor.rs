use std::path::{Path, PathBuf};

pub use crate::todo_extractor_internal::aggregator::{
    extract_marked_items, MarkedItem, MarkerConfig,
};

pub fn extract_todos(file_path: &Path, content: &str) -> Vec<MarkedItem> {
    let todos = extract_marked_items(file_path, content, &MarkerConfig::default());

    return todos;
}
