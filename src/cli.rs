use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use log::{error, info};
use std::path::Path;
use std::path::PathBuf;

pub fn run_cli_with_args<I, T>(args: I, git_ops: &dyn GitOpsTrait)
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
            Arg::new("files")
                .value_name("FILE")
                .help("Optional list of files to process (passed by pre-commit)")
                .num_args(0..),
        )
        // TODO add a flag to enable debug logging
        // TODO add configuration to specify the Markers to search for
        .get_matches_from(args);

    let todo_path = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value");

    if !Path::new(todo_path).exists() {
        if let Err(e) = std::fs::write(todo_path, "") {
            error!("Error creating TODO.md: {}", e);
            std::process::exit(1);
        }
    }

    let files: Vec<PathBuf> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .map(PathBuf::from)
        .collect();

    let repo = match git_ops.open_repository(Path::new(".")) {
        Ok(r) => r,
        Err(e) => {
            error!("Error opening repository: {}", e);
            std::process::exit(1);
        }
    };

    // Retrieve the list of deleted files from the repository.
    let deleted_files = match git_ops.get_deleted_files(&repo) {
        Ok(list) => list,
        Err(e) => {
            error!("Error retrieving deleted files: {}", e);
            std::process::exit(1);
        }
    };

    if !files.is_empty() || !deleted_files.is_empty() {
        if let Err(e) =
            crate::cli::process_files_from_list(Path::new(todo_path), files, deleted_files)
        {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        info!("No files provided, nothing to do.");
        std::process::exit(0);
    }
}

pub fn run_cli() {
    run_cli_with_args(std::env::args(), &GitOps);
}

pub fn process_files_from_list(
    todo_path: &Path,
    scanned_files: Vec<PathBuf>,
    deleted_files: Vec<PathBuf>,
) -> Result<(), String> {
    let mut new_todos = Vec::new();
    // Each file provided in the CLI is scanned.
    for file in &scanned_files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let todos = todo_extractor::extract_todos(file, &content);
            new_todos.extend(todos);
        } else {
            error!("Warning: Could not read file {:?}, skipping.", file);
        }
    }

    // Pass the list of scanned files to sync_todo_file.
    if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, scanned_files, deleted_files) {
        info!("Error: {:?}", err);
    }
    info!("TODO.md successfully updated.");
    Ok(())
}
