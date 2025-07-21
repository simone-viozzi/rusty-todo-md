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
    fn test_sql_line_comment() {
        init_logger();
        let src = "-- TODO: optimize\nSELECT 1;";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("query.sql"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "optimize");
    }
}
