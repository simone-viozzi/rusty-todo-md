// src/languages/sql.rs

use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/sql.pest"]
pub struct SqlParser;

impl CommentParser for SqlParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::sql_file, file_content)
    }
}

#[cfg(test)]
mod sql_tests {
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_sql_line_comment() {
        init_logger();
        let src = "-- TODO: optimize\nSELECT 1;";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("query.sql"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "optimize");
    }
}
