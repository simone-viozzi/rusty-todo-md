use crate::MarkedItem;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// The single H1 at the top of a non-empty TODO.md.
const TITLE: &str = "# TODOs";

/// The source files an existing TODO.md has entries for.
///
/// TODO.md is only a hint about *which files to rescan*; its entries are
/// never reused. Parsing is lenient on purpose: it reads the path out of
/// every entry line and ignores everything else, so conflict markers,
/// `merge=union` leftovers (old and new line numbers side by side),
/// duplicates, hand edits and older layouts all still yield the right
/// files.
pub fn listed_files(content: &str) -> BTreeSet<PathBuf> {
    let entry_re = Regex::new(r"^\* \[(.+):\d+\]\(").unwrap();
    content
        .lines()
        .filter_map(|line| entry_re.captures(line))
        .map(|caps| PathBuf::from(&caps[1]))
        .collect()
}

/// Writes the given list of `TodoItem`s to the TODO.md file in markdown format.
///
/// The output has a single `# TODOs` title, then is grouped by marker (e.g., TODO,
/// FIXME) as H2 headers and by file as H3 headers, with each entry as a bullet:
///
/// # TODOs
///
/// ## FIXME
/// ### src/file2.rs
/// * [src/file2.rs:120](src/file2.rs#L120): Correct boundary condition
///
/// ## TODO
/// ### src/file1.rs
/// * [src/file1.rs:35](src/file1.rs#L35): Implement feature X
///
/// An empty item list produces an empty file.
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
    if !marker_map.is_empty() {
        content.push_str(&format!("{TITLE}\n\n"));
    }
    // Write each marker section, separated by a blank line
    for (m, (marker, files)) in marker_map.into_iter().enumerate() {
        if m > 0 {
            content.push('\n');
        }
        content.push_str(&format!("## {marker}\n"));
        // Write each file section under the marker
        let file_entries: Vec<_> = files.into_iter().collect();
        for (i, (file, items)) in file_entries.iter().enumerate() {
            content.push_str(&format!("### {file}\n", file = file.display()));
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

    fn listed(content: &str) -> Vec<String> {
        listed_files(content)
            .into_iter()
            .map(|p| p.display().to_string())
            .collect()
    }

    #[test]
    fn test_listed_files_reads_paths_from_every_entry_line() {
        init_logger();
        let current = "# TODOs\n\n## FIXME\n### src/b.rs\n* [src/b.rs:2](src/b.rs#L2): x\n\n## TODO\n### src/a.rs\n* [src/a.rs:1](src/a.rs#L1): y\n";
        assert_eq!(listed(current), ["src/a.rs", "src/b.rs"]);

        // Pre-#194 layout: marker as H1, file as H2.
        let old = "# TODO\n## src/a.rs\n* [src/a.rs:1](src/a.rs#L1): y\n";
        assert_eq!(listed(old), ["src/a.rs"]);
    }

    #[test]
    fn test_listed_files_tolerates_merge_leftovers() {
        // Both sides of a conflict, or of a `merge=union` result, name the
        // files that need rescanning; everything else is ignored.
        init_logger();
        let conflicted = "# TODOs\n\n## TODO\n### a.rs\n<<<<<<< HEAD\n* [a.rs:3](a.rs#L3): alpha\n=======\n* [b.rs:1](b.rs#L1): beta\n>>>>>>> feat\n";
        assert_eq!(listed(conflicted), ["a.rs", "b.rs"]);

        let union = "# TODOs\n\n## TODO\n### a.rs\n* [a.rs:10](a.rs#L10): alpha\n* [a.rs:11](a.rs#L11): alpha\nremember to ask Bob\n";
        assert_eq!(listed(union), ["a.rs"]);
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

        // One H1 title, markers as H2 in lexicographical order, files as H3.
        let expected = "\
# TODOs

## Add
### src/foo.rs
* [src/foo.rs:30](src/foo.rs#L30): Add tests for foo

## Fix
### src/foo.rs
* [src/foo.rs:20](src/foo.rs#L20): Fix bug in foo

## Refactor
### src/bar.rs
* [src/bar.rs:10](src/bar.rs#L10): Refactor bar
";
        assert_eq!(content, expected);
        assert_eq!(listed(&content), ["src/bar.rs", "src/foo.rs"]);
    }

    #[test]
    fn test_write_todo_file_empty_writes_empty_file() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let todo_path = temp_dir.path().join("TODO.md");

        write_todo_file(&todo_path, vec![]).unwrap();

        assert_eq!(fs::read_to_string(&todo_path).unwrap(), "");
    }
}
