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
        // The docstring starts presumably on line 3 or 4
        assert_eq!(item.line_number, 4);
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
}
