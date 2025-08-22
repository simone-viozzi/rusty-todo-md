use std::path::Path;

pub use crate::todo_extractor_internal::aggregator::{
    extract_marked_items, extract_marked_items_with_parser, get_effective_extension,
    get_parser_for_extension, CommentLine, MarkedItem, MarkerConfig,
};

// TODO: deprecated
pub fn extract_todos_with_config(
    file_path: &Path,
    content: &str,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items(file_path, content, config)
}

pub fn extract_todos_with_parser(
    file_path: &Path,
    content: &str,
    parser_fn: fn(&str) -> Vec<CommentLine>,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    extract_marked_items_with_parser(file_path, content, parser_fn, config)
}
