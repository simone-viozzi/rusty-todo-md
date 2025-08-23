// src/languages/go.rs

use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/go.pest"]
pub struct GoParser;

impl CommentParser for GoParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::go_file, file_content)
    }
}

#[cfg(test)]
mod go_tests {
    use super::*;
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_go_single_line_comment() {
        init_logger();
        let src = r#"
// TODO: Fix this function
func main() {
    fmt.Println("Hello, World!")
}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("main.go"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "Fix this function");
    }

    #[test]
    fn test_go_block_comment() {
        init_logger();
        let src = r#"
/* TODO: Refactor this module
   Add better error handling */
func process() error {
    return nil
}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("process.go"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(
            todos[0].message,
            "Refactor this module Add better error handling"
        );
    }

    #[test]
    fn test_go_mixed_comments() {
        init_logger();
        let src = r#"
// TODO: Implement feature A
func foo() {
    /* FIXME: Handle edge cases
       such as nil pointers */
    return
}
// TODO: Add documentation
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("example.go"), src, &config);
        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].message, "Implement feature A");
        assert_eq!(todos[1].message, "Handle edge cases such as nil pointers");
        assert_eq!(todos[2].message, "Add documentation");
    }

    #[test]
    fn test_go_ignore_string_literals() {
        init_logger();
        let src = r#"
const message = "TODO: This should not be detected"
const single = 'F'  // Single char literal
const raw = `TODO: Raw string should be ignored`
// TODO: But this should be detected
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("strings.go"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "But this should be detected");
    }

    #[test]
    fn test_go_package_comments() {
        init_logger();
        let src = r#"
// Package main provides the application entry point
// TODO: Add package documentation
package main

/* FIXME: Implement proper error handling
   across the entire package */
import "fmt"
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string(), "FIXME:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("main.go"), src, &config);
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].message, "Add package documentation");
        assert_eq!(
            todos[1].message,
            "Implement proper error handling across the entire package"
        );
    }

    #[test]
    fn test_extract_go_comments() {
        let src = r#"
// This is a normal comment
// TODO: Implement feature Y
"#;
        let comments = GoParser::parse_comments(src);
        assert_eq!(comments.len(), 2); // Should extract both lines
    }

    #[test]
    fn test_ignore_non_comment_go() {
        let src = r#"
x := 10 // TODO: This is a comment
"#;
        let comments = GoParser::parse_comments(src);
        assert_eq!(comments.len(), 1); // Only extracts the inline comment
    }

    #[test]
    fn test_go_multiline_todo() {
        init_logger();
        let src = r#"
// TODO: Implement authentication
//       Add JWT token validation
//       Handle token expiration
func authenticate() error { return nil }
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("auth.go"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "Implement authentication Add JWT token validation Handle token expiration"
        );
    }

    #[test]
    fn test_go_nested_block_comments() {
        init_logger();
        let src = r#"
/*
    TODO: This is a complex task
    /* Note: Go doesn't actually support nested block comments,
       but our parser should handle this gracefully */
*/
func main() {}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("nested.go"), src, &config);
        // The parser should find at least one TODO
        assert!(!todos.is_empty());
        assert!(todos[0].message.contains("This is a complex task"));
    }
}
