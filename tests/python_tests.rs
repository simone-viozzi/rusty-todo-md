#[cfg(test)]
mod python_tests {
    use todo_extractor::{extract_todos, aggregator::TodoItem};
    use std::path::Path;

    #[test]
    fn test_python_single_line() {
        let src = r#"
# TODO: do something
x = "TODO: not a comment"
"#;
        // We'll pass an artificial .py path
        let todos = extract_todos(Path::new("test.py"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 2); // line is 1-based
        assert_eq!(todos[0].message, "do something");
    }

    #[test]
    fn test_python_docstring() {
        let src = r#"
def f():
    """
    normal doc line
    TODO: fix f
      some more text
    """
"#;
        let todos = extract_todos(Path::new("test.py"), src);
        assert_eq!(todos.len(), 1);
        let item = &todos[0];
        // The docstring starts presumably on line 3 or 4
        assert_eq!(item.line_number, 4); 
        // The text merges "fix f" + "some more text" due to aggregator's logic
        assert!(item.message.contains("fix f"));
        assert!(item.message.contains("some more text"));
    }
}
