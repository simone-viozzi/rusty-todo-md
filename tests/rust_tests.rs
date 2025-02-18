#[cfg(test)]
mod rust_tests {
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_todos;
    use todo_extractor::languages::common::CommentParser;
    use todo_extractor::languages::rust::RustParser;
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
    fn test_rust_single_line() {
        init_logger();
        let src = r#"
// normal comment
// TODO: single line
fn main() {
   let s = "TODO: not a comment in string";
}
"#;
        let todos = extract_todos(Path::new("example.rs"), src);
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
        let todos = extract_todos(Path::new("lib.rs"), src);

        // Now we should expect only the correctly merged lines
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
// 1: This file is simulating ~50 lines of code
// 2: Some normal comment
// 3:
fn example() {   // 4
    // 5: Another normal comment
    // 6: TODO: first_todo
    let x = 10;  // 7
    println!("hello"); // 8
    // 9:
    /*
     10: Multi-line block
       TODO: second_todo
          still part of second_todo
     */
    let y = 20; // 15
    // 16:
    // 17: TODO: third_todo
    if x + y > 20 {
        // 20: no todo
        println!("sum > 20");
    }
    // 23: normal
    // 24: We can check line numbers carefully
}

// 28: Another function
fn foo() { // 29
    // 30: Another random comment
    // 31: TODO: fourth_todo
    /* 32: Some block comment with no TODO inside */
    let z = "string that says TODO: but inside quotes, so aggregator ignores it";
    println!("{}", z); // 34
}

// 36: The end is near
// 37: Just some padding
// 38: 
// 39: 
// 40:
"#;

        let todos = extract_todos(Path::new("large_file.rs"), src);

        // Let's see the aggregator's results:
        println!("Found {} TODOs: {:#?}", todos.len(), todos);

        // We *expect* 4 TODOs from lines:
        //   - line 6 => "first_todo"
        //   - line ~10 => "second_todo" (plus the indented line "still part of second_todo")
        //   - line 17 => "third_todo"
        //   - line 31 => "fourth_todo"
        // Adjust if your real snippet changes line spacing.

        assert_eq!(
            todos.len(),
            4,
            "Should find exactly four TODOs in this snippet"
        );

        // Check line numbers:
        assert_eq!(todos[0].line_number, 6);
        assert_eq!(todos[0].message, "first_todo");

        // second_todo likely merges the line "still part of second_todo"
        // The aggregator merges indented lines. So the final message might be:
        //   "second_todo still part of second_todo"
        assert!(
            todos[1].message.contains("second_todo"),
            "Should contain second_todo text"
        );
        assert!(
            todos[1].message.contains("still part of second_todo"),
            "Should also include the indented line"
        );

        // third_todo on line 17
        assert_eq!(todos[2].line_number, 17);
        assert_eq!(todos[2].message, "third_todo");

        // fourth_todo on line 31
        assert_eq!(todos[3].line_number, 31);
        assert_eq!(todos[3].message, "fourth_todo");
    }
}
