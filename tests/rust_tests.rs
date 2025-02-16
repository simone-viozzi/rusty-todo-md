#[cfg(test)]
mod rust_tests {
    use log::LevelFilter;
    use todo_extractor::languages::common::CommentParser;
    use std::path::Path;
    use std::sync::Once;
    use todo_extractor::aggregator::extract_todos;
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
}
