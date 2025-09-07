use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/yaml.pest"]
pub struct YamlParser;

impl CommentParser for YamlParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::yaml_file, file_content)
    }
}

#[cfg(test)]
mod yaml_tests {
    use super::*;
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_yaml_single_comment() {
        init_logger();
        let src = r#"# TODO: configure
key: value"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("config.yaml"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "configure");
    }

    #[test]
    fn test_yaml_ignore_triple_quoted_strings() {
        init_logger();
        let src = r#"services:
  test-service:
    description: """
    This is a string, not a comment
    TODO: This should NOT be detected
    """
    environment:
      # TODO: This SHOULD be detected
      - KEY=value"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("config.yaml"), src, &config);

        // Should only find the one in the actual comment, not in the string
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "This SHOULD be detected");
        assert_eq!(todos[0].line_number, 8);
    }

    #[test]
    fn test_yaml_multiple_comments() {
        init_logger();
        let src = r#"# TODO: Header comment
services:
  service1:
    # FIXME: Service-level comment
    image: nginx
  service2:
    # TODO: Another comment
    image: apache"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("docker-compose.yaml"), src, &config);

        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].message, "Header comment");
        assert_eq!(todos[1].message, "Service-level comment");
        assert_eq!(todos[2].message, "Another comment");
    }

    #[test]
    fn test_yaml_quoted_strings() {
        init_logger();
        let src = r#"config:
  message1: "This contains TODO: but should be ignored"
  message2: 'Another TODO: that should be ignored'
  # TODO: This is a real comment
  message3: "Normal value""#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("config.yaml"), src, &config);

        // Should only find the real comment, not the TODOs in strings
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "This is a real comment");
        assert_eq!(todos[0].line_number, 4);
    }

    #[test]
    fn test_yaml_direct_parser() {
        init_logger();
        let src = r#"# First comment
key: value
# Second comment with TODO: test message
another: "string with TODO: ignored""#;

        let comments = YamlParser::parse_comments(src);

        // Should extract 2 comment lines, not the string content
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].line_number, 1);
        assert_eq!(comments[0].text, "# First comment");
        assert_eq!(comments[1].line_number, 3);
        assert_eq!(comments[1].text, "# Second comment with TODO: test message");
    }
}
