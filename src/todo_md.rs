use crate::todo_extractor::MarkedItem;
use crate::todo_md_internal::TodoCollection;
use log::info;
use regex::Regex;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use log::warn;

#[derive(Debug)]
pub enum TodoError {
    Io(io::Error),
    Parse(String),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::Io(e) => write!(f, "I/O error: {}", e),
            TodoError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for TodoError {}

impl From<io::Error> for TodoError {
    fn from(e: io::Error) -> Self {
        TodoError::Io(e)
    }
}

pub fn validate_todo_file(todo_path: &std::path::Path) -> bool {
    // TODO: add tests for this function
    match fs::read_to_string(todo_path) {
        Ok(content) => {
            if content.is_empty() {
                info!("Empty TODO.md file");
                return true;
            }
            // Expected patterns for a section header and a TODO item line.
            let section_re = Regex::new(r"^##\s+(.*)$").unwrap();
            let todo_re = Regex::new(r"^\*\s+\[(.+):(\d+)\]\(.+#L\d+\):\s*(.+)$").unwrap();
            // Check each non‑empty line for a valid pattern.
            for (i, line) in content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if !(section_re.is_match(line) || todo_re.is_match(line)) {
                    warn!("Invalid format on line {}: {}", i + 1, line);
                    return false;
                }
            }
            true
        }
        Err(e) => {
            warn!("Failed to read {}: {}", todo_path.display(), e);
            false
        }
    }
}

/// Reads the existing TODO.md file (in the new sectioned format) and returns a vector of `MarkedItem`s.
///
/// The new format groups TODO items under section headers of the form:
///
/// ```markdown
/// ## <file-path>
/// * [<file-path>:<line_number>](<file-path>#L<line_number>): <message>
/// ```
///
/// This function uses regex to detect section headers to set the current file context, and then
/// parses subsequent todo item lines accordingly.
pub fn read_todo_file(todo_path: &Path) -> Result<Vec<MarkedItem>, TodoError> {
    if !validate_todo_file(todo_path) {
        return Err(TodoError::Parse("TODO.md validation failed".to_string()));
    }

    let content = fs::read_to_string(todo_path)?;

    let mut todos = Vec::new();

    // TODO what happen here if the file is malformed?
    //     is the file is malformed, we should raise an error
    //     and rerun with --all-files to regenerate the file from scratch

    // TODO this will need to be a 3 way scan
    //     1. scan for the markers (# TODO, # FIXME, etc)
    //     2. scan for the file path (## src/main.rs)
    //     3. scan for the marker line (* [src/main.rs:12](src/main.rs#L12): Refactor this function)

    // Regex for matching a section header, e.g., "## src/cli.rs"
    let section_re = Regex::new(r"^##\s+(.*)$").unwrap();
    // Regex for matching a TODO item line, e.g.,
    // "* [src/cli.rs:10](src/cli.rs#L10): add a new argument to specify what markers to look for"
    let todo_re = Regex::new(r"^\*\s+\[(.+):(\d+)\]\(.+#L\d+\):\s*(.+)$").unwrap();

    let mut current_file: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // If the line is a section header, update the current file context.
        if let Some(caps) = section_re.captures(line) {
            current_file = Some(caps[1].trim().to_string());
            continue;
        }
        // If the line matches a TODO item, parse it.
        if let Some(caps) = todo_re.captures(line) {
            let file_path_str = current_file.clone().unwrap_or_else(|| caps[1].to_string());
            let file_path = PathBuf::from(file_path_str);
            let line_number = caps[2].parse::<usize>().unwrap_or(0);
            let message = caps[3].to_string();
            todos.push(MarkedItem {
                file_path,
                line_number,
                message,
            });
        }
    }
    Ok(todos)
}

pub fn sync_todo_file(
    todo_path: &Path,
    new_todos: Vec<MarkedItem>,
    scanned_files: Vec<PathBuf>,
    deleted_files: Vec<PathBuf>,
) -> Result<(), TodoError> {
    // Read existing TODO items from the file using the new parser.
    let existing_todos = read_todo_file(todo_path)?;

    // Create a TodoCollection from the existing TODO items.
    let mut existing_collection = TodoCollection::new();
    for item in existing_todos {
        existing_collection.add_item(item);
    }

    // Create a TodoCollection from the new TODO items.
    let mut new_collection = TodoCollection::new();
    for item in new_todos {
        new_collection.add_item(item);
    }

    // Merge new TODO items into the existing collection, updating only scanned files.
    existing_collection.merge(new_collection, scanned_files, deleted_files);

    // Convert the merged collection back into a sorted vector of MarkedItems.
    let merged_todos = existing_collection.to_sorted_vec();

    // Write the merged and sorted TODO items back to the TODO.md file in the new sectioned format.
    write_todo_file(todo_path, &merged_todos)?;
    Ok(())
}

/// Writes the given list of `TodoItem`s to the TODO.md file in markdown format.
///
/// # Arguments
/// - `todo_path`: The path to the TODO.md file.
/// - `todos`: A list of `TodoItem`s to write.
pub fn write_todo_file(todo_path: &Path, todos: &[MarkedItem]) -> std::io::Result<()> {
    // Create a TodoCollection from the provided TODO items.
    let mut collection = TodoCollection::new();
    for todo in todos {
        collection.add_item(todo.clone());
    }

    // Sort the file paths (keys) to ensure a consistent order.
    let mut files: Vec<_> = collection.todos.keys().collect();
    files.sort_by_key(|a| a.display().to_string());

    let mut content = String::new();
    // For each file, write a markdown section header and its TODO items.
    for file in files {
        // Write a section header for the file.
        content.push_str(&format!("## {}\n", file.display()));

        // Retrieve and sort TODO items for the current file by line number.
        let mut items = collection.todos.get(file).unwrap().clone();
        items.sort_by_key(|item| item.line_number);
        for item in items {
            content.push_str(&format!(
                "* [{}:{}]({}#L{}): {}\n",
                item.file_path.display(),
                item.line_number,
                item.file_path.display(),
                item.line_number,
                item.message
            ));
        }
        // Add an extra newline between sections.
        content.push('\n');
    }

    fs::write(todo_path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo_extractor::MarkedItem;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_sync_todo_file() {
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // create the empty TODO.md file
        fs::write(&todo_path, "").unwrap();

        let new_todos = vec![
            MarkedItem {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 10,
                message: "Refactor this function".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                message: "Add error handling".to_string(),
            },
        ];

        let res = sync_todo_file(&todo_path, new_todos.clone(), vec![], vec![]);

        assert!(res.is_ok());

        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("src/main.rs:10"));
        assert!(content.contains("Refactor this function"));
        assert!(content.contains("src/lib.rs:5"));
        assert!(content.contains("Add error handling"));
    }

    #[test]
    fn test_read_todo_file_with_markdown_parser() {
        let content = r#"
## src/main.rs
* [src/main.rs:12](src/main.rs#L12): Refactor this function

## src/lib.rs
* [src/lib.rs:5](src/lib.rs#L5): Add error handling
"#;

        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // Write the test content to TODO.md
        fs::write(&todo_path, content).unwrap();

        // Read and parse the TODO.md file
        let todos = read_todo_file(&todo_path);

        assert!(todos.is_ok());
        let todos = todos.unwrap();

        assert_eq!(todos.len(), 2);
        assert_eq!(
            todos[0],
            MarkedItem {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 12,
                message: "Refactor this function".to_string(),
            }
        );
        assert_eq!(
            todos[1],
            MarkedItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                message: "Add error handling".to_string(),
            }
        );
    }

    #[test]
    fn test_write_todo_file_sectioned() {
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // Create a list of TODO items from two different files.
        let items = vec![
            MarkedItem {
                file_path: PathBuf::from("src/foo.rs"),
                line_number: 20,
                message: "Fix bug in foo".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/bar.rs"),
                line_number: 10,
                message: "Refactor bar".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/foo.rs"),
                line_number: 30,
                message: "Add tests for foo".to_string(),
            },
        ];

        // Write the TODO items using the new sectioned format.
        let result = write_todo_file(&todo_path, &items);
        assert!(result.is_ok());

        let content = fs::read_to_string(&todo_path).unwrap();

        // Verify that each file has its own section header.
        assert!(
            content.contains("## src/bar.rs"),
            "Missing section for src/bar.rs"
        );
        assert!(
            content.contains("## src/foo.rs"),
            "Missing section for src/foo.rs"
        );

        // Verify that the TODO items are correctly formatted.
        let expected_bar = "* [src/bar.rs:10](src/bar.rs#L10): Refactor bar";
        let expected_foo_20 = "* [src/foo.rs:20](src/foo.rs#L20): Fix bug in foo";
        let expected_foo_30 = "* [src/foo.rs:30](src/foo.rs#L30): Add tests for foo";
        assert!(content.contains(expected_bar));
        assert!(content.contains(expected_foo_20));
        assert!(content.contains(expected_foo_30));

        // Ensure that the sections appear in lexicographical order.
        let bar_index = content.find("## src/bar.rs").unwrap();
        let foo_index = content.find("## src/foo.rs").unwrap();
        assert!(bar_index < foo_index, "Section ordering is incorrect");
    }
}
