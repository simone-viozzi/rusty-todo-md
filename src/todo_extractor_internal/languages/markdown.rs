// src/languages/markdown.rs

use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/markdown.pest"]
pub struct MarkdownParser;

impl CommentParser for MarkdownParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::markdown_file, file_content)
    }
}

#[cfg(test)]
mod markdown_tests {
    use crate::logger;
    use crate::todo_extractor_internal::aggregator::{extract_marked_items, MarkerConfig};
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .format(logger::format_logger)
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });
    }

    #[test]
    fn test_markdown_html_comment() {
        init_logger();
        let src = "<!-- TODO: document -->\ntext";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("README.md"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "document");
    }
}
