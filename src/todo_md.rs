use crate::todo_md_internal::TodoCollection;
use crate::MarkedItem;
use log::{debug, info, warn};
use regex::Regex;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub enum TodoError {
    Io(io::Error),
    Parse(String),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::Io(e) => write!(f, "I/O error: {e}"),
            TodoError::Parse(msg) => write!(f, "Parse error: {msg}"),
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
            // Expected patterns for a marker header, section header, and a TODO item line.
            let marker_re = Regex::new(r"^#\s+\w+").unwrap();
            let section_re = Regex::new(r"^##\s+(.*)$").unwrap();
            let todo_re = Regex::new(r"^\*\s+\[(.+):(\d+)\]\(.+#L\d+\):\s*(.+)$").unwrap();
            // Check each non‑empty line for a valid pattern.
            for (i, line) in content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if !(marker_re.is_match(line)
                    || section_re.is_match(line)
                    || todo_re.is_match(line))
                {
                    warn!(
                        "Invalid format on line {line_num}: {line}",
                        line_num = i + 1,
                        line = line
                    );
                    return false;
                }
            }
            true
        }
        Err(e) => {
            warn!(
                "Failed to read {path}: {e}",
                path = todo_path.display(),
                e = e
            );
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
    let marker_re = Regex::new(r"^#\s+(\w+)").unwrap();
    let section_re = Regex::new(r"^##\s+(.*)$").unwrap();
    let todo_re = Regex::new(r"^\*\s+\[(.+):(\d+)\]\(.+#L\d+\):\s*(.+)$").unwrap();
    let mut current_file: Option<String> = None;
    let mut current_marker: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // If the line is a marker header, update the current marker
        if let Some(caps) = marker_re.captures(line) {
            current_marker = Some(caps[1].to_string());
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
            let marker = current_marker.clone().unwrap_or_else(|| "TODO".to_string());
            todos.push(MarkedItem {
                file_path,
                line_number,
                message,
                marker,
            });
        }
    }
    Ok(todos)
}

pub fn sync_todo_file(
    todo_path: &Path,
    new_todos: Vec<MarkedItem>,
    scanned_files: Vec<PathBuf>,
) -> Result<(), TodoError> {
    // TODO maybe simplify the logic of this function

    let mut existing_collection = TodoCollection::new();

    match read_todo_file(todo_path) {
        Ok(existing_todos) => {
            let filtered_todos: Vec<MarkedItem> = existing_todos
                .into_iter()
                .filter(|item| item.file_path.exists())
                .collect();

            debug!("Filtered out TODOs for non-existent files");

            // Create a TodoCollection from the filtered existing TODO items.
            for item in filtered_todos {
                existing_collection.add_item(item);
            }
        }

        Err(e) => {
            // Propagate the error to trigger fallback mechanism in CLI
            return Err(e);
        }
    }

    // Create a TodoCollection from the new TODO items.
    let mut new_collection = TodoCollection::new();
    for item in new_todos {
        new_collection.add_item(item);
    }

    // Merge new TODO items into the existing collection, updating only scanned files.
    existing_collection.merge(new_collection, scanned_files);

    // Convert the merged collection back into a sorted vector of MarkedItems.
    let merged_todos = existing_collection.to_sorted_vec();

    // Write the merged and sorted TODO items back to the TODO.md file in the new sectioned format.
    write_todo_file(todo_path, merged_todos)?;
    Ok(())
}

/// Writes the given list of `TodoItem`s to the TODO.md file in markdown format.
///
/// The output format is grouped by marker (e.g., TODO, FIXME) as top-level headers,
/// then by file as secondary headers, with each entry as a bullet:
///
/// # TODO
/// ## src/file1.rs
/// - [src/file1.rs:35](src/file1.rs#L35): Implement feature X
///
/// # FIXME
/// ## src/file2.rs
/// - [src/file2.rs:120](src/file2.rs#L120): Correct boundary condition
///
pub fn write_todo_file(todo_path: &Path, todos: Vec<MarkedItem>) -> std::io::Result<()> {
    // Group by marker, then by file using BTreeMap for sorted output
    let mut marker_map: BTreeMap<String, BTreeMap<PathBuf, Vec<MarkedItem>>> = BTreeMap::new();
    for item in todos {
        marker_map
            .entry(item.marker.clone())
            .or_default()
            .entry(item.file_path.clone())
            .or_default()
            .push(item);
    }

    let mut content = String::new();
    // Write each marker section
    for (marker, files) in marker_map {
        content.push_str(&format!("# {marker}\n"));
        // Write each file section under the marker
        let file_entries: Vec<_> = files.into_iter().collect();
        for (i, (file, items)) in file_entries.iter().enumerate() {
            content.push_str(&format!("## {file}\n", file = file.display()));
            // Sort items by line number for consistency
            let mut sorted_items = items.clone();
            sorted_items.sort_by_key(|item| item.line_number);
            for item in sorted_items.iter() {
                content.push_str(&format!(
                    "* [{file}:{line}]({file}#L{line}): {message}\n",
                    file = item.file_path.display(),
                    line = item.line_number,
                    message = item.message
                ));
            }
            // Add an extra newline between file sections (but not after the last one)
            if i < file_entries.len() - 1 {
                content.push('\n');
            }
        }
    }
    // Write the final content to the TODO.md file
    fs::write(todo_path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_logger;
    use crate::MarkedItem;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_sync_todo_file() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // create the empty TODO.md file
        fs::write(&todo_path, "").unwrap();

        let new_todos = vec![
            MarkedItem {
                file_path: PathBuf::from("src/main.rs"),
                line_number: 10,
                message: "Refactor this function".to_string(),
                marker: "TODO".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                message: "Add error handling".to_string(),
                marker: "TODO".to_string(),
            },
        ];

        let res = sync_todo_file(&todo_path, new_todos.clone(), vec![]);

        assert!(res.is_ok());

        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("src/main.rs:10"));
        assert!(content.contains("Refactor this function"));
        assert!(content.contains("src/lib.rs:5"));
        assert!(content.contains("Add error handling"));

        // Check that the file ends with exactly one newline (not two)
        assert!(content.ends_with('\n'), "File should end with a newline");
        assert!(
            !content.ends_with("\n\n"),
            "File should not end with double newlines"
        );
    }

    #[test]
    fn test_sync_todo_file_filters_nonexistent_files() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // Create an existing TODO.md with entries for both existing and non-existent files
        let existing_content = r#"# TODO
## src/existing.rs
* [src/existing.rs:10](src/existing.rs#L10): This file exists

## src/deleted.rs
* [src/deleted.rs:5](src/deleted.rs#L5): This file does not exist
"#;
        fs::write(&todo_path, existing_content).unwrap();

        // Create only one of the files to simulate that the other was deleted
        // We need to change to the temp directory so relative paths work correctly
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let existing_file = PathBuf::from("src").join("existing.rs");
        fs::create_dir_all(existing_file.parent().unwrap()).unwrap();
        fs::write(&existing_file, "// TODO: This file exists\nfn main() {}").unwrap();
        // Note: We don't create src/deleted.rs to simulate it being deleted

        // Run sync_todo_file with no new todos, which should filter out the non-existent file
        let new_todos = vec![];
        let res = sync_todo_file(&todo_path, new_todos, vec![]);
        assert!(res.is_ok());

        // Read the updated TODO.md content
        let content = fs::read_to_string(&todo_path).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // The content should only contain the entry for the existing file
        assert!(
            content.contains("src/existing.rs"),
            "Should contain existing file"
        );
        assert!(
            !content.contains("src/deleted.rs"),
            "Should not contain deleted file"
        );
        assert!(
            !content.contains("This file does not exist"),
            "Should not contain deleted file's TODO"
        );
        assert!(
            content.contains("This file exists"),
            "Should still contain existing file's TODO"
        );
    }

    #[test]
    fn test_read_todo_file_with_markdown_parser() {
        init_logger();
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
                marker: "TODO".to_string(),
            }
        );
        assert_eq!(
            todos[1],
            MarkedItem {
                file_path: PathBuf::from("src/lib.rs"),
                line_number: 5,
                message: "Add error handling".to_string(),
                marker: "TODO".to_string(),
            }
        );
    }

    #[test]
    fn test_write_todo_file_sectioned() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        // Create a list of TODO items from two different files.
        let items = vec![
            MarkedItem {
                file_path: PathBuf::from("src/foo.rs"),
                line_number: 20,
                message: "Fix bug in foo".to_string(),
                marker: "Fix".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/bar.rs"),
                line_number: 10,
                message: "Refactor bar".to_string(),
                marker: "Refactor".to_string(),
            },
            MarkedItem {
                file_path: PathBuf::from("src/foo.rs"),
                line_number: 30,
                message: "Add tests for foo".to_string(),
                marker: "Add".to_string(),
            },
        ];

        // Write the TODO items using the new sectioned format.
        let result = write_todo_file(&todo_path, items);
        assert!(result.is_ok());

        let content = fs::read_to_string(&todo_path).unwrap();

        // Check for marker header
        assert!(content.contains("# Fix"), "Missing marker section header");
        assert!(
            content.contains("# Refactor"),
            "Missing marker section header"
        );
        assert!(content.contains("# Add"), "Missing marker section header");
        // Check for file headers
        assert!(
            content.contains("## src/bar.rs"),
            "Missing section for src/bar.rs"
        );
        assert!(
            content.contains("## src/foo.rs"),
            "Missing section for src/foo.rs"
        );

        // Check for correct TODO item formatting
        let expected_bar = "* [src/bar.rs:10](src/bar.rs#L10): Refactor bar";
        let expected_foo_20 = "* [src/foo.rs:20](src/foo.rs#L20): Fix bug in foo";
        let expected_foo_30 = "* [src/foo.rs:30](src/foo.rs#L30): Add tests for foo";
        assert!(content.contains(expected_bar));
        assert!(content.contains(expected_foo_20));
        assert!(content.contains(expected_foo_30));

        // Check that marker sections appear in lexicographical order
        let marker_fix_index = content.find("# Fix").unwrap_or(usize::MAX);
        let marker_refactor_index = content.find("# Refactor").unwrap_or(usize::MAX);
        let marker_add_index = content.find("# Add").unwrap_or(usize::MAX);
        assert!(
            marker_add_index < marker_fix_index && marker_fix_index < marker_refactor_index,
            "Marker section ordering is incorrect"
        );
    }
}
