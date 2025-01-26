use crate::todo_extractor::TodoItem;
use comrak::{nodes::AstNode, parse_document, Arena, ComrakOptions};
use std::fs;
use std::path::{Path, PathBuf};

/// Reads the existing TODO.md file and returns a vector of `TodoItem`s.
/// If the file does not exist, returns an empty vector.
///
/// # Arguments
/// - `todo_path`: The path to the TODO.md file.
///
/// # Returns
/// A vector of `TodoItem`s parsed from the existing TODO.md file.
pub fn read_todo_file(todo_path: &Path) -> Vec<TodoItem> {
    let mut todos = Vec::new();
    let arena = Arena::new();
    let options = ComrakOptions::default();

    if let Ok(content) = fs::read_to_string(todo_path) {
        let root = parse_document(&arena, &content, &options);

        // Traverse the AST to extract TODO items
        extract_todos_from_ast(root, &mut todos);
    }

    todos
}

/// Recursively processes the AST nodes to extract TODO items.
fn extract_todos_from_ast<'a>(node: &'a AstNode<'a>, todos: &mut Vec<TodoItem>) {
    let mut current_path: Option<String> = None;
    let mut current_line: Option<usize> = None;

    for child in node.children() {
        let data = &child.data.borrow();

        match data.value {
            comrak::nodes::NodeValue::Link(ref link) => {
                // Convert Vec<u8> to String for parsing
                if let Ok(link_str) = String::from_utf8(link.url.clone().into()) {
                    if let Some((path, line)) = parse_link(&link_str) {
                        current_path = Some(path.to_string());
                        current_line = Some(line);
                    }
                }
            }
            comrak::nodes::NodeValue::Text(ref comment) => {
                if let (Some(path), Some(line)) = (&current_path, current_line) {
                    // Trim leading ": " or whitespace
                    let cleaned_comment = comment.trim().trim_start_matches(':').trim();

                    todos.push(TodoItem {
                        file_path: PathBuf::from(path),
                        line_number: line,
                        comment: cleaned_comment.to_string(),
                    });
                }
                current_path = None;
                current_line = None;
            }
            _ => {}
        }

        // Recursively process child nodes
        extract_todos_from_ast(child, todos);
    }
}

/// Parses a link from TODO.md in the format `src/main.rs#L12`
/// and extracts the file path and line number.
///
/// # Arguments
/// - `link`: The Markdown link.
///
/// # Returns
/// An optional tuple containing the file path and line number.
fn parse_link(link: &str) -> Option<(&str, usize)> {
    if let Some((path, line)) = link.split_once("#L") {
        if let Ok(line_number) = line.parse::<usize>() {
            return Some((path, line_number));
        }
    }
    None
}

pub fn sync_todo_file(todo_path: &Path, new_todos: Vec<TodoItem>) -> Result<(), std::io::Error> {
    let mut existing_todos = read_todo_file(todo_path);
    existing_todos.retain(|existing| new_todos.contains(existing));

    for new_todo in new_todos {
        if !existing_todos.contains(&new_todo) {
            existing_todos.push(new_todo);
        }
    }

    existing_todos.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.line_number.cmp(&b.line_number))
    });

    write_todo_file(todo_path, &existing_todos)
}

/// Writes the given list of `TodoItem`s to the TODO.md file in markdown format.
///
/// # Arguments
/// - `todo_path`: The path to the TODO.md file.
/// - `todos`: A list of `TodoItem`s to write.
pub fn write_todo_file(todo_path: &Path, todos: &[TodoItem]) -> std::io::Result<()> {
    let mut content = String::new();

    for todo in todos {
        content.push_str(&format!(
            "* [{}:{}]({}#L{}): {}\n",
            todo.file_path.display(),
            todo.line_number,
            todo.file_path.display(),
            todo.line_number,
            todo.comment
        ));
    }

    fs::write(todo_path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sync_todo_file() {
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        let new_todos = vec![
            TodoItem {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 10,
                comment: "Refactor this function".to_string(),
            },
            TodoItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                comment: "Add error handling".to_string(),
            },
        ];

        let _ = sync_todo_file(&todo_path, new_todos.clone());

        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("src/main.rs:10"));
        assert!(content.contains("Refactor this function"));
        assert!(content.contains("src/lib.rs:5"));
        assert!(content.contains("Add error handling"));
    }

    #[test]
    fn test_read_todo_file_with_markdown_parser() {
        let content = r#"
* [src/main.rs:12](src/main.rs#L12): Refactor this function
* [src/lib.rs:5](src/lib.rs#L5): Add error handling
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // Write the test content to TODO.md
        std::fs::write(&todo_path, content).unwrap();

        // Read and parse the TODO.md file
        let todos = read_todo_file(&todo_path);

        assert_eq!(todos.len(), 2);
        assert_eq!(
            todos[0],
            TodoItem {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 12,
                comment: "Refactor this function".to_string(),
            }
        );
        assert_eq!(
            todos[1],
            TodoItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                comment: "Add error handling".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_link() {
        let link = "src/main.rs#L12";
        let parsed = parse_link(link).unwrap();
        assert_eq!(parsed.0, "src/main.rs");
        assert_eq!(parsed.1, 12);
    }
}
