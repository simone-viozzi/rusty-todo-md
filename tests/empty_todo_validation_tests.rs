use rusty_todo_md::cli::{find_empty_todos, validate_no_empty_todos};
use rusty_todo_md::todo_extractor::MarkerConfig;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_empty_todo_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with empty TODOs
    let test_file = temp_path.join("test.rs");
    let mut file = fs::File::create(&test_file).unwrap();
    writeln!(file, "// TODO: This is a valid TODO").unwrap();
    writeln!(file, "fn main() {{").unwrap();
    writeln!(file, "    // TODO:").unwrap();
    writeln!(file, "    // FIXME:").unwrap();
    writeln!(file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(file, "}}").unwrap();
    drop(file);

    let marker_config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
    let files = vec![test_file.clone()];

    // Test that validation fails for empty TODOs
    let result = validate_no_empty_todos(&files, &marker_config);
    assert!(
        result.is_err(),
        "Expected validation to fail for empty TODOs"
    );

    let error_message = result.unwrap_err();
    assert!(error_message.contains("empty TODO comment found"));
    assert!(error_message.contains("empty FIXME comment found"));
    assert!(error_message.contains("test.rs:3"));
    assert!(error_message.contains("test.rs:4"));
}

#[test]
fn test_valid_todo_detection() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with only valid TODOs
    let test_file = temp_path.join("test.rs");
    let mut file = fs::File::create(&test_file).unwrap();
    writeln!(file, "// TODO: This is a valid TODO").unwrap();
    writeln!(file, "fn main() {{").unwrap();
    writeln!(file, "    // FIXME: This needs to be fixed").unwrap();
    writeln!(file, "    println!(\"Hello, world!\");").unwrap();
    writeln!(file, "}}").unwrap();
    drop(file);

    let marker_config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
    let files = vec![test_file.clone()];

    // Test that validation succeeds for valid TODOs
    let result = validate_no_empty_todos(&files, &marker_config);
    assert!(
        result.is_ok(),
        "Expected validation to succeed for valid TODOs"
    );
}

#[test]
fn test_find_empty_todos_function() {
    let content = r#"// TODO: This is a valid TODO
fn main() {
    // TODO:
    // FIXME:
    /* TODO:
       another empty one */
    println!("Hello, world!");
}"#;

    let marker_config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
    let empty_todos = find_empty_todos(&PathBuf::from("test.rs"), content, &marker_config);

    // Should find 3 empty TODOs
    assert_eq!(empty_todos.len(), 3);

    // Check line numbers
    assert_eq!(empty_todos[0].line_number, 3);
    assert_eq!(empty_todos[0].marker, "TODO");

    assert_eq!(empty_todos[1].line_number, 4);
    assert_eq!(empty_todos[1].marker, "FIXME");

    assert_eq!(empty_todos[2].line_number, 5);
    assert_eq!(empty_todos[2].marker, "TODO");
}

#[test]
fn test_python_empty_todos() {
    let content = r#"# TODO: Valid TODO
def main():
    # TODO:
    # FIXME:
    pass"#;

    let marker_config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);
    let empty_todos = find_empty_todos(&PathBuf::from("test.py"), content, &marker_config);

    // Should find 2 empty TODOs
    assert_eq!(empty_todos.len(), 2);
    assert_eq!(empty_todos[0].line_number, 3);
    assert_eq!(empty_todos[1].line_number, 4);
}
