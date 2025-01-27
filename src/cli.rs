use crate::git_utils;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use std::path::Path;

pub fn run_cli() {
    // Define CLI arguments using clap
    let matches = Command::new("rusty-todo-md")
        .version("0.1.5")
        .author("Simone Viozzi <you@example.com>")
        .about("Automatically scans staged files for TODO comments and updates TODO.md.")
        .arg(
            Arg::new("todo_path")
                .short('p')
                .long("todo-path")
                .value_name("FILE")
                .help("Specifies the path to the TODO.md file")
                .action(ArgAction::Set)
                .default_value("TODO.md"),
        )
        .get_matches();

    // Get the path to TODO.md from the arguments
    let todo_path = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value");

    println!("Updating TODO file at: {}", todo_path);

    // Run the workflow
    if let Err(e) = run_workflow(Path::new(todo_path)) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main workflow for scanning staged files and updating TODO.md.
fn run_workflow(todo_path: &Path) -> Result<(), String> {
    // Open the Git repository
    let repo = git_utils::open_repository()
        .map_err(|e| format!("Failed to open Git repository: {}", e))?;

    // Get staged files
    let staged_files = git_utils::get_staged_files(&repo)
        .map_err(|e| format!("Failed to retrieve staged files: {}", e))?;

    if staged_files.is_empty() {
        println!("No staged files found.");
        return Ok(());
    }

    println!("Staged files:");
    for file in &staged_files {
        println!("- {:?}", file);
    }

    // Extract TODO comments from staged files
    let mut new_todos = Vec::new();
    for file in staged_files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            let todos = todo_extractor::extract_todos(&file, &content);
            new_todos.extend(todos);
        } else {
            eprintln!("Warning: Could not read file {:?}, skipping.", file);
        }
    }

    if new_todos.is_empty() {
        println!("No TODO comments found in staged files.");
        return Ok(());
    }

    // Update the TODO.md file
    todo_md::sync_todo_file(todo_path, new_todos)
        .map_err(|e| format!("Failed to update TODO.md: {}", e))?;

    println!("TODO.md successfully updated.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_run_workflow() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Set up a fake Git repository
        let repo = git_utils::open_repository().expect("Failed to initialize repo");
        let file_path = repo_path.join("file1.rs");
        fs::write(&file_path, "// TODO: Refactor this function").unwrap();

        let mut index = repo.index().unwrap();
        index
            .add_path(file_path.strip_prefix(repo_path).unwrap())
            .unwrap();
        index.write().unwrap();

        // Run the workflow
        let result = run_workflow(&todo_path);

        assert!(result.is_ok());

        // Verify TODO.md was updated
        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("file1.rs"));
        assert!(content.contains("Refactor this function"));
    }
}
