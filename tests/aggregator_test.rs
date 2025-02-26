#[cfg(test)]
mod aggregator_tests {
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::extract_marked_items;
    use todo_extractor::logger;
    use todo_extractor::MarkerConfig;

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
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
    }

    #[test]
    fn test_invalid_extension() {
        init_logger();
        let src = "// TODO: This should not be processed";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.unknown"), src, &config);
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
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
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
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
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
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 3);
        assert_eq!(todos[0].message, "Implement caching");
    }

    #[test]
    fn test_empty_input_no_todos() {
        init_logger();
        let src = "";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_display_todo_output() {
        init_logger();
        let src = "// TODO: Improve logging";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);

        let output = format!("{} - {}", todos[0].line_number, todos[0].message);
        assert_eq!(output, "1 - Improve logging");
    }

    #[test]
    fn test_display_no_todos() {
        init_logger();
        let src = "fn main() {}";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
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
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_multiple_consecutive_todos() {
        init_logger();
        let src = r#"
// TODO: todo1
// TODO: todo2
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);

        assert_eq!(todos.len(), 2);

        // Check their line numbers and messages
        // The first TODO should be on line 2, the second on line 3 (1-based from Pest)
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "todo1");
        assert_eq!(todos[1].line_number, 3);
        assert_eq!(todos[1].message, "todo2");
    }

    #[test]
    fn test_ignore_todo_not_at_beginning() {
        let src = r#"
// This is a comment with a TODO: not at the beginning
fn main() {}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(
            todos.len(),
            0,
            "A TODO not at the beginning should not be detected"
        );
    }

    #[test]
    fn test_fixme_with_colon() {
        // Test a comment that uses FIXME with a colon.
        let src = r#"
    // FIXME: Correct the error handling
    "#;
        let config = MarkerConfig {
            markers: vec!["FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].message, "Correct the error handling");
    }

    #[test]
    fn test_fixme_without_colon() {
        // Test a comment that uses FIXME without a colon.
        let src = r#"
    // FIXME Correct the error handling
    "#;
        let config = MarkerConfig {
            markers: vec!["FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].message, "Correct the error handling");
    }

    #[test]
    fn test_mixed_markers() {
        // Test a file that mixes both TODO and FIXME comments,
        // with and without the colon.
        let src = r#"
    // TODO: Implement feature A
    // FIXME: Fix bug in module
    // Some regular comment
    // TODO Implement feature B
    // FIXME Fix another bug
    "#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);

        // We expect four items in order.
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].message, "Implement feature A");
        assert_eq!(items[1].message, "Fix bug in module");
        assert_eq!(items[2].message, "Implement feature B");
        assert_eq!(items[3].message, "Fix another bug");
    }

    #[test]
    fn test_mixed_markers_complex() {
        // This test mixes both TODO and FIXME comments (with and without a colon),
        // includes multiline comment blocks, and interleaves non-comment code.
        let src = r#"
// TODO: Implement feature A

fn some_function() {
    // This is a normal comment
    // FIXME: Fix bug in module
    println!("Hello, world!");
}

/*
   TODO: Implement feature C
       with additional multiline details
*/

/// FIXME Fix critical bug
///   that occurs on edge cases

// TODO Implement feature B

// FIXME Fix another bug
"#;

        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);

        // We expect six separate marked items:
        // 1. "Implement feature A"
        // 2. "Fix bug in module"
        // 3. "Implement feature C with additional multiline details"
        // 4. "Fix critical bug that occurs on edge cases"
        // 5. "Implement feature B"
        // 6. "Fix another bug"
        assert_eq!(items.len(), 6);

        assert_eq!(items[0].message, "Implement feature A");
        assert_eq!(items[1].message, "Fix bug in module");
        assert_eq!(
            items[2].message,
            "Implement feature C with additional multiline details"
        );
        assert_eq!(
            items[3].message,
            "Fix critical bug that occurs on edge cases"
        );
        assert_eq!(items[4].message, "Implement feature B");
        assert_eq!(items[5].message, "Fix another bug");
    }
}
