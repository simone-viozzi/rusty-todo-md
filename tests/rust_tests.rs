#[cfg(test)]
mod rust_tests {
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_marked_items;
    use todo_extractor::languages::common::CommentParser;
    use todo_extractor::languages::rust::RustParser;
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
    fn test_rust_single_line() {
        init_logger();
        let src = r#"
// normal comment
// TODO: single line
fn main() {
   let s = "TODO: not a comment in string";
}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("example.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 3);
        assert_eq!(todos[0].message, "single line");
    }

    #[test]
    fn test_rust_block_doc() {
        init_logger();
        let src = r#"
/// TODO: fix this doc
///     second line
fn foo() {}

/*
    TODO: block
        more lines
*/
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("lib.rs"), src, &config);

        assert_eq!(todos.len(), 2);

        // Doc comment
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "fix this doc second line");

        // Block comment
        assert_eq!(todos[1].message, "block more lines");
    }

    #[test]
    fn test_extract_rust_comments() {
        let src = r#"
// This is a normal comment
// TODO: Implement feature Y
"#;
        let comments = RustParser::parse_comments(src);
        assert_eq!(comments.len(), 2); // Should extract both lines
    }

    #[test]
    fn test_ignore_non_comment_rust() {
        let src = r#"
let x = 10; // TODO: Not a comment
"#;
        let comments = RustParser::parse_comments(src);
        assert_eq!(comments.len(), 1); // Only extracts the inline comment
    }

    #[test]
    fn test_large_rust_file_scenario() {
        init_logger();
        let src = r#"
// This file is simulating ~50 lines of code
// Some normal comment
//
fn example() {   // 4
    // Another normal comment
    // TODO: first_todo
    let x = 10;  // 7
    println!("hello"); // 8
    //
    /*
        Multi-line block
        TODO: second_todo
            still part of second_todo
     */
    let y = 20; // 15
    // 
    // TODO: third_todo
    if x + y > 20 {
        // no todo
        println!("sum > 20");
    }
    // normal
    // We can check line numbers carefully
}

// Another function
fn foo() { 
    // Another random comment
    // TODO: fourth_todo
    /* Some block comment with no TODO inside */
    let z = "string that says TODO: but inside quotes, so aggregator ignores it";
    println!("{}", z); // 34
}

// The end is near
// Just some padding
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("large_file.rs"), src, &config);

        println!("Found {} TODOs: {:#?}", todos.len(), todos);

        assert_eq!(
            todos.len(),
            4,
            "Should find exactly four TODOs in this snippet"
        );

        // Check line numbers:
        assert_eq!(todos[0].line_number, 7);
        assert_eq!(todos[0].message, "first_todo");

        // second_todo likely merges the line "still part of second_todo"
        assert_eq!(todos[1].line_number, 13);
        assert!(todos[1].message.contains("second_todo"));
        assert!(todos[1].message.contains("still part of second_todo"));

        // third_todo on line 17
        assert_eq!(todos[2].line_number, 18);
        assert_eq!(todos[2].message, "third_todo");

        // fourth_todo on line 31
        assert_eq!(todos[3].line_number, 30);
        assert_eq!(todos[3].message, "fourth_todo");
    }
}
