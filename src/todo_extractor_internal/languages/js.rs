// src/languages/js.rs

use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/js.pest"]
pub struct JsParser;

impl CommentParser for JsParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::js_file, file_content)
    }
}

#[cfg(test)]
mod js_tests {
    use super::*;
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_js_single_line_comment() {
        init_logger();
        let src = r#"
// TODO: Fix this function
function init() {
    console.log("Hello");
}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.js"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "Fix this function");
    }

    #[test]
    fn test_js_block_comment() {
        init_logger();
        let src = r#"
/* TODO: Refactor this module
   Add better error handling */
function init() {
    console.log("Hello");
}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.js"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(
            todos[0].message,
            "Refactor this module Add better error handling"
        );
    }

    #[test]
    fn test_js_mixed_comments() {
        init_logger();
        let src = r#"
// TODO: Implement feature A
function foo() {
    /* FIXME: Handle edge cases
       such as null responses */
    return null;
}
// TODO: Add documentation
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.js"), src, &config);
        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].message, "Implement feature A");
        assert_eq!(todos[1].message, "Handle edge cases such as null responses");
        assert_eq!(todos[2].message, "Add documentation");
    }

    #[test]
    fn test_js_ignore_string_literals() {
        init_logger();
        let src = r#"
const message = "TODO: This should not be detected";
const single = 'FIXME: Neither should this';
const template = `TODO: Or this ${variable}`;
// TODO: But this should be detected
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.js"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "But this should be detected");
    }

    #[test]
    fn test_js_jsx_syntax() {
        init_logger();
        let src = r#"
// TODO: Add prop validation
const Component = () => {
    /* FIXME: Handle loading state */
    return <div>Hello World</div>;
};
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("component.jsx"), src, &config);
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].message, "Add prop validation");
        assert_eq!(todos[1].message, "Handle loading state");
    }

    #[test]
    fn test_extract_js_comments() {
        let src = r#"
// This is a normal comment
// TODO: Implement feature Y
"#;
        let comments = JsParser::parse_comments(src);
        assert_eq!(comments.len(), 2); // Should extract both lines
    }

    #[test]
    fn test_ignore_non_comment_js() {
        let src = r#"
const x = 10; // TODO: This is a comment
"#;
        let comments = JsParser::parse_comments(src);
        assert_eq!(comments.len(), 1); // Only extracts the inline comment
    }

    #[test]
    fn test_js_multiline_todo() {
        init_logger();
        let src = r#"
// TODO: Implement authentication
//       Add JWT token validation
//       Handle token expiration
function authenticate() {}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("auth.js"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "Implement authentication Add JWT token validation Handle token expiration"
        );
    }
}
