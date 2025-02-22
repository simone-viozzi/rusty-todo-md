#[cfg(test)]
mod python_tests {
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_todos;
    use todo_extractor::logger;

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
    fn test_python_single_line() {
        init_logger();

        let src = r#"
# TODO: do something
x = "TODO: not a comment"
"#;
        // We'll pass an artificial .py path
        let todos = extract_todos(Path::new("test.py"), src);
        println!("{:?}", todos);
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
        let todos = extract_todos(Path::new("test.py"), src);
        assert_eq!(todos.len(), 1);
        let item = &todos[0];

        println!("{:?}", item);

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
        let todos = extract_todos(Path::new("file.py"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "Fix performance issues");
    }

    #[test]
    fn test_ignore_non_todo_python() {
        init_logger();
        let src = r#"
# This is just a comment
"#;
        let todos = extract_todos(Path::new("file.py"), src);
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

        // TODO: review the logic in this test        

        let todos = extract_todos(Path::new("multi_todos.py"), src);

        // Print to see the aggregator's actual behavior
        println!("Todos = {:#?}", todos);

    
        assert_eq!(todos.len(), 2, "");

        // If you want it to detect both TODOs separately, you'd confirm how you intend to handle multiple “TODO:” lines in the same docstring block.
        // For now, we test the existing behavior (which likely sees 1).
        let item = &todos[0];
        assert!(
            item.message.contains("first"),
            "Should contain the first TODO message"
        );
        assert!(
            item.message.contains("more text in the todo"),
            "Should contain the indented line after 'TODO: first'"
        );
        // The aggregator usually won't parse the second 'TODO:' line in the same block unless you modify the code to do so.

        // Check line number of the first "TODO:" line
        // Likely line_number = 5 or 6 (depending on how the docstring is parsed).
        // Adjust the assertion after you see the actual aggregator result in printout:
        assert_eq!(item.line_number, 5, "Docstring TODO line is probably 5");
    }
}
