use rusty_todo_md::cli::validate_no_empty_todos;
use rusty_todo_md::{extract_marked_items_from_file, MarkerConfig};
use std::fs;
use std::io::Write;

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

    // Extract marked items first
    let todos = extract_marked_items_from_file(&test_file, &marker_config).unwrap();

    // Test that validation fails for empty TODOs
    let result = validate_no_empty_todos(&todos);
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

    // Extract marked items first
    let todos = extract_marked_items_from_file(&test_file, &marker_config).unwrap();

    // Test that validation succeeds for valid TODOs
    let result = validate_no_empty_todos(&todos);
    assert!(
        result.is_ok(),
        "Expected validation to succeed for valid TODOs"
    );
}

#[test]
fn test_extract_empty_todos_directly() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with mixed valid and empty TODOs
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

    // Extract all marked items
    let todos = extract_marked_items_from_file(&test_file, &marker_config).unwrap();

    // Should find 3 total items (1 valid, 2 empty)
    assert_eq!(todos.len(), 3);

    // Filter for empty ones
    let empty_todos: Vec<_> = todos
        .iter()
        .filter(|item| item.message.trim().is_empty())
        .collect();
    assert_eq!(empty_todos.len(), 2);

    // Check line numbers and markers
    assert_eq!(empty_todos[0].line_number, 3);
    assert_eq!(empty_todos[0].marker, "TODO");

    assert_eq!(empty_todos[1].line_number, 4);
    assert_eq!(empty_todos[1].marker, "FIXME");
}

#[test]
fn test_python_empty_todos() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a test file with empty TODOs in Python
    let test_file = temp_path.join("test.py");
    let mut file = fs::File::create(&test_file).unwrap();
    writeln!(file, "# TODO: Valid TODO").unwrap();
    writeln!(file, "def main():").unwrap();
    writeln!(file, "    # TODO:").unwrap();
    writeln!(file, "    # FIXME:").unwrap();
    writeln!(file, "    pass").unwrap();
    drop(file);

    let marker_config = MarkerConfig::normalized(vec!["TODO".to_string(), "FIXME".to_string()]);

    // Extract marked items first
    let todos = extract_marked_items_from_file(&test_file, &marker_config).unwrap();

    let result = validate_no_empty_todos(&todos);
    assert!(result.is_err());

    let error_message = result.unwrap_err();
    assert!(error_message.contains("empty TODO comment found"));
    assert!(error_message.contains("empty FIXME comment found"));
}
