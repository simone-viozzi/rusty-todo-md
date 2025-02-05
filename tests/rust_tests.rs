#[cfg(test)]
mod rust_tests {
    use todo_extractor::aggregator::extract_todos;
    use std::path::Path;

    #[test]
    fn test_rust_single_line() {
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
        let src = r#"
/// TODO: fix this doc
/// second line
fn foo() {}

/*
   stuff
   TODO: block
   more lines
*/
"#;
        let todos = extract_todos(Path::new("lib.rs"), src);
        assert_eq!(todos.len(), 2);

        // doc comment
        assert_eq!(todos[0].line_number, 2);
        assert!(todos[0].message.contains("fix this doc"));
        assert!(todos[0].message.contains("second line"));

        // block
        assert!(todos[1].message.contains("block"));
        assert!(todos[1].message.contains("more lines"));
    }
}
