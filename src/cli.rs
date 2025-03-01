use crate::git_utils;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use log::{error, info};
use std::path::Path;

pub fn run_cli() {
    // Define CLI arguments, including the new --all-files flag.
    let matches = Command::new("rusty-todo-md")
        .version("0.1.5")
        .author("Simone Viozzi <you@example.com>")
        .about("Automatically scans files for TODO comments and updates TODO.md.")
        .arg(
            Arg::new("todo_path")
                .short('p')
                .long("todo-path")
                .value_name("FILE")
                .help("Specifies the path to the TODO.md file")
                .action(ArgAction::Set)
                .default_value("TODO.md"),
        )
        .arg(
            Arg::new("all_files")
                .long("all-files")
                .help("Ignore staged files logic and scan all tracked files in the repository")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // Get the path to TODO.md and the flag value.
    let todo_path = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value");
    let all_files = *matches.get_one::<bool>("all_files").unwrap_or(&false);

    info!("Updating TODO file at: {}", todo_path);

    if let Err(e) = run_workflow(Path::new(todo_path), Path::new("."), all_files) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Main workflow for scanning files and updating TODO.md.
/// 
/// When `all_files` is true, it retrieves all tracked files from Git;
/// otherwise, it gets only the staged files.
pub fn run_workflow(todo_path: &Path, repo_path: &Path, all_files: bool) -> Result<(), String> {
    // Open the Git repository.
    let repo = git_utils::open_repository(repo_path)
        .map_err(|e| format!("Failed to open Git repository: {}", e))?;

    // Choose the appropriate file list based on the flag.
    let file_paths = if all_files {
        info!("Scanning all tracked files in repository: {:?}", repo_path);
        git_utils::get_tracked_files(&repo)
            .map_err(|e| format!("Failed to retrieve tracked files: {}", e))?
    } else {
        info!("Scanning staged files in repository: {:?}", repo_path);
        git_utils::get_staged_files(&repo)
            .map_err(|e| format!("Failed to retrieve staged files: {}", e))?
    };

    if file_paths.is_empty() {
        info!("No files found.");
        return Ok(());
    }

    let mut new_todos = Vec::new();
    // Process each file in the list.
    for file in file_paths {
        let absolute_path = repo_path.join(&file);
        if let Ok(content) = std::fs::read_to_string(&absolute_path) {
            // Extract TODO comments using the relative file path.
            let todos = todo_extractor::extract_todos(&file, &content);
            new_todos.extend(todos);
        } else {
            error!("Warning: Could not read file {:?}, skipping.", file);
        }
    }

    if new_todos.is_empty() {
        info!("No TODO comments found.");
        return Ok(());
    }

    // Update the TODO.md file.
    todo_md::sync_todo_file(todo_path, new_todos)
        .map_err(|e| format!("Failed to update TODO.md: {}", e))?;
    info!("TODO.md successfully updated.");
    Ok(())
}