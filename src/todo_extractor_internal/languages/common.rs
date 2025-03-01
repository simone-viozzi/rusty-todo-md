// src/languages/common.rs

use crate::todo_extractor_internal::aggregator::CommentLine;

/// A trait for parsing comments from source code.
pub trait CommentParser {
    /// Parses the provided file content and returns a vector of comment lines.
    fn parse_comments(file_content: &str) -> Vec<CommentLine>;
}
