// src/languages/python.rs

use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser; // Import the trait
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/python.pest"]
pub struct PythonParser;

impl CommentParser for PythonParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::python_file, file_content)
    }
}

#[cfg(test)]
mod python_tests {
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_python_single_line() {
        init_logger();

        let src = r#"
# TODO: do something
x = "TODO: not a comment"
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.py"), src, &config);
        println!("{todos:?}");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2); // line is 1-based
        assert_eq!(todos[0].message, "do something");
    }

    #[test]
    fn test_python_docstring() {
        init_logger();
        let src = r#"
def f():
    """
    normal doc line
    TODO: fix f
      some more text
    """
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("test.py"), src, &config);
        assert_eq!(todos.len(), 1);
        let item = &todos[0];

        println!("{item:?}");

        // The TODO appears on line 5, not 4.
        assert_eq!(item.line_number, 5);
        // The text merges "fix f" + "some more text" due to aggregator's logic
        assert!(item.message.contains("fix f"));
        assert!(item.message.contains("some more text"));
    }

    #[test]
    fn test_extract_python_todo() {
        init_logger();
        let src = r#"
# TODO: Fix performance issues
# Regular comment
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.py"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "Fix performance issues");
    }

    #[test]
    fn test_ignore_non_todo_python() {
        init_logger();
        let src = r#"
# This is just a comment
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.py"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_python_docstring_multiple_todos() {
        init_logger();
        let src = r#"
def big_function():
    """
    some text
    TODO: first
        more text in the todo
    TODO: second
    some unrelated text
    """
    x = 42
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("multi_todos.py"), src, &config);

        // Print to see the aggregator's actual behavior
        println!("Todos = {todos:?}");

        assert_eq!(todos.len(), 2, "");

        let item = &todos[0];
        assert!(
            item.message.contains("first"),
            "Should contain the first TODO message"
        );
        assert!(
            item.message.contains("more text in the todo"),
            "Should contain the indented line after 'TODO: first'"
        );

        // Check line number of the first "TODO:" line
        assert_eq!(item.line_number, 5, "Docstring TODO line is probably 5");
    }
}
