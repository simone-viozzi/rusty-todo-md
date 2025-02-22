#[cfg(test)]
mod aggregator_tests {
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::extract_todos;
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
    fn test_valid_rust_extension() {
        init_logger();
        let src = "// TODO: Implement feature X";
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 1);
    }

    #[test]
    fn test_invalid_extension() {
        init_logger();
        let src = "// TODO: This should not be processed";
        let todos = extract_todos(Path::new("file.unknown"), src);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_merge_multiline_todo() {
        init_logger();
        let src = r#"
// TODO: Fix bug
//     Improve error handling
//     Add logging
"#;
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "Fix bug Improve error handling Add logging"
        );
    }

    #[test]
    fn test_stop_merge_on_unindented_line() {
        init_logger();
        let src = r#"
// TODO: Improve API
// Refactor later
"#;
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "Improve API"); // Does not merge second line
    }

    #[test]
    fn test_todo_with_line_number() {
        init_logger();
        let src = r#"
// Some comment
// TODO: Implement caching
"#;
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 3);
        assert_eq!(todos[0].message, "Implement caching");
    }

    #[test]
    fn test_empty_input_no_todos() {
        init_logger();
        let src = "";
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_display_todo_output() {
        init_logger();
        let src = "// TODO: Improve logging";
        let todos = extract_todos(Path::new("file.rs"), src);

        let output = format!("{} - {}", todos[0].line_number, todos[0].message);
        assert_eq!(output, "1 - Improve logging");
    }

    #[test]
    fn test_display_no_todos() {
        init_logger();
        let src = "fn main() {}";
        let todos = extract_todos(Path::new("file.rs"), src);
        assert!(todos.is_empty());
    }

    #[test]
    fn test_basic_framework() {
        init_logger();
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_false_positive_detection() {
        init_logger();
        let src = r#"
let message = "TODO: This should not be detected";
"#;
        let todos = extract_todos(Path::new("file.rs"), src);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_multiple_consecutive_todos() {
        init_logger();
        let src = r#"
// TODO: todo1
// TODO: todo2
"#;

        // We simulate a .rs file so it picks the Rust parser:
        let todos = extract_todos(Path::new("file.rs"), src);

        // We expect 2 separate TODOs because there's no indentation to merge them
        assert_eq!(todos.len(), 2);

        // Check their line numbers and messages
        // The first TODO should be on line 2, the second on line 3 (1-based from Pest)
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "todo1");

        assert_eq!(todos[1].line_number, 3);
        assert_eq!(todos[1].message, "todo2");
    }
}

// TODO: add tests where the TODO is not at the beginning of the line, 
//      this should not be detected