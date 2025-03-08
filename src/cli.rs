use crate::git_utils;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use log::{error, info};
use std::path::Path;

pub fn run_cli_with_args<I, T>(args: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
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
            // TODO remove this functionality, it's not needed, pre-commit will pass the list of files
            Arg::new("all_files")
                .long("all-files")
                .help("Ignore staged files logic and scan all tracked files in the repository")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Optional list of files to process (passed by pre-commit)")
                .num_args(0..),
        )
        .get_matches_from(args);

    let todo_path = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value");

    let all_files = *matches.get_one::<bool>("all_files").unwrap_or(&false);

    let files: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    if !files.is_empty() {
        if let Err(e) = crate::cli::process_files_from_list(Path::new(todo_path), files) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        // this funcionaity is not needed, pre-commit will pass the list of files
    } else if let Err(e) = crate::cli::run_workflow(Path::new(todo_path), Path::new("."), all_files)
    {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

pub fn run_cli() {
    run_cli_with_args(std::env::args());
}

/// Main workflow for scanning files and updating TODO.md.
///
/// When `all_files` is true, it retrieves all tracked files from Git;
/// otherwise, it gets only the staged files.
pub fn run_workflow(todo_path: &Path, repo_path: &Path, all_files: bool) -> Result<(), String> {
    // TODO delete this function
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
        // TODO remove this functionality
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

pub fn process_files_from_list(todo_path: &Path, files: Vec<String>) -> Result<(), String> {
    let mut new_todos = Vec::new();
    for file in files {
        let path = Path::new(&file);
        if let Ok(content) = std::fs::read_to_string(path) {
            let todos = todo_extractor::extract_todos(path, &content);
            new_todos.extend(todos);
        } else {
            error!("Warning: Could not read file {:?}, skipping.", path);
        }
    }

    if new_todos.is_empty() {
        info!("No TODO comments found in provided files.");
        return Ok(());
    }

    todo_md::sync_todo_file(todo_path, new_todos)
        .map_err(|e| format!("Failed to update TODO.md: {}", e))?;
    info!("TODO.md successfully updated.");
    Ok(())
}
