use regex::Regex;
use std::path::{Path, PathBuf};

/// Represents a TODO comment extracted from a file.
#[derive(Debug, PartialEq, Eq)]
pub struct TodoItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub comment: String,
}

/// Extracts TODO comments from the given file content.
///
/// # Arguments
/// - `file_path`: The path to the file being scanned.
/// - `content`: The content of the file as a string.
///
/// # Returns
/// A vector of `TodoItem` structs containing the extracted TODO comments.
///
/// # Example
/// ```
/// use rusty_todo_md::todo_extractor::extract_todos;
/// use std::path::PathBuf;
///
/// let file_path = PathBuf::from("path/to/file");
/// let content = "// TODO: Refactor this function";
/// let todos = extract_todos(&file_path, content);
///
/// assert_eq!(todos.len(), 1);
/// assert_eq!(todos[0].comment, "Refactor this function");
/// ```
pub fn extract_todos(file_path: &Path, content: &str) -> Vec<TodoItem> {
    // Regex to match TODO comments (case-insensitive)
    let todo_regex = Regex::new(r"(?i)\bTODO\b[:\s]*(.*)").expect("Failed to compile regex");

    // Collect TODO items with line numbers and comments
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if let Some(captures) = todo_regex.captures(line) {
                let comment = captures
                    .get(1)
                    .map_or("", |m| m.as_str())
                    .trim()
                    .to_string();
                Some(TodoItem {
                    file_path: file_path.to_path_buf().clone(),
                    line_number: index,
                    comment,
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_todos() {
        let file_path = PathBuf::from("test_file.rs");
        let content = r#"
// This is a regular comment
// TODO: Refactor this function
fn example() {}

/*
 * TODO: Add more tests
 */
// todo lowercase should also work
"#;

        let todos = extract_todos(&file_path, content);

        assert_eq!(todos.len(), 3);
        assert_eq!(
            todos[0],
            TodoItem {
                file_path: file_path.clone(),
                line_number: 2, // Line 2 in the content
                comment: "Refactor this function".to_string(),
            }
        );
        assert_eq!(
            todos[1],
            TodoItem {
                file_path: file_path.clone(),
                line_number: 6, // Line 6 in the content
                comment: "Add more tests".to_string(),
            }
        );
        assert_eq!(
            todos[2],
            TodoItem {
                file_path: file_path.clone(),
                line_number: 8, // Line 8 in the content
                comment: "lowercase should also work".to_string(),
            }
        );
    }

    #[test]
    fn test_extract_todos_no_matches() {
        let file_path = PathBuf::from("empty.rs");
        let content = "This file has no TODOs.";
        let todos = extract_todos(&file_path, content);
        assert!(todos.is_empty());
    }
}
