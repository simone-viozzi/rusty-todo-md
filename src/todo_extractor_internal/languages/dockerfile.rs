use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/dockerfile.pest"]
pub struct DockerfileParser;

impl CommentParser for DockerfileParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::dockerfile_file, file_content)
    }
}

#[cfg(test)]
mod dockerfile_tests {
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_dockerfile_single_comment() {
        init_logger();
        let src = r#"# TODO: install packages
FROM alpine"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };

        // TODO now in the tests i need to actually create the file instead of passing a fake path and a content
        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);

        // TODO and also need to check errors
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "install packages");
    }
}
