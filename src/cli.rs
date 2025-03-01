use crate::git_utils;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use std::path::Path;
use log::{debug, error, info};

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

    info!("Updating TODO file at: {}", todo_path);

    // Run the workflow
    if let Err(e) = run_workflow(Path::new(todo_path), Path::new(".")) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main workflow for scanning staged files and updating TODO.md.
pub fn run_workflow(todo_path: &Path, repo_path: &Path) -> Result<(), String> {
    // Open the Git repository
    let repo = git_utils::open_repository(repo_path)
        .map_err(|e| format!("Failed to open Git repository: {}", e))?;

    // Get staged files
    let staged_files = git_utils::get_staged_files(&repo)
        .map_err(|e| format!("Failed to retrieve staged files: {}", e))?;

    if staged_files.is_empty() {
        info!("No staged files found.");
        return Ok(());
    }

    info!("Staged files:");
    for file in &staged_files {
        info!("- {:?}", file);
    }

    // Extract TODO comments from staged files
    let mut new_todos = Vec::new();
    for file in staged_files {
        let absolute_path = repo_path.join(&file);
        if let Ok(content) = std::fs::read_to_string(&absolute_path) {
            let todos = todo_extractor::extract_todos(&file, &content);
            new_todos.extend(todos);
        } else {
            error!("Warning: Could not read file {:?}, skipping.", file);
        }
    }

    if new_todos.is_empty() {
        info!("No TODO comments found in staged files.");
        return Ok(());
    }

    // Update the TODO.md file
    todo_md::sync_todo_file(todo_path, new_todos)
        .map_err(|e| format!("Failed to update TODO.md: {}", e))?;

    info!("TODO.md successfully updated.");
    Ok(())
}
