use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::todo_extractor;
use crate::todo_md;
use clap::{Arg, ArgAction, Command};
use git2::Repository;
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
            Arg::new("markers")
                .short('m')
                .long("markers")
                .value_name("KEYWORDS")
                .help("Specifies one or more marker keywords to search for (e.g., TODO FIXME HACK). Usage: --markers TODO FIXME HACK")
                .num_args(1..)
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Optional list of files to process (passed by pre-commit)")
                .num_args(0..),
        )
        // TODO add a flag to enable debug logging
        .get_matches_from(args);

    let todo_path = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value");

    if !Path::new(todo_path).exists() {
        if let Err(e) = std::fs::write(todo_path, "") {
            error!("Error creating TODO.md: {e}");
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
            error!("Error opening repository: {e}");
            std::process::exit(1);
        }
    };

    // Retrieve the list of deleted files from the repository.
    let deleted_files = match git_ops.get_deleted_files(&repo) {
        Ok(list) => list,
        Err(e) => {
            error!("Error retrieving deleted files: {e}");
            std::process::exit(1);
        }
    };

    // Parse markers from CLI args (if any)
    let markers: Vec<String> = matches
        .get_many::<String>("markers")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| vec!["TODO".to_string()]);
    let marker_config = todo_extractor::MarkerConfig::normalized(markers);

    if !files.is_empty() || !deleted_files.is_empty() {
        if let Err(e) = process_files_from_list(
            Path::new(todo_path),
            files,
            deleted_files,
            git_ops,
            repo,
            &marker_config,
        ) {
            error!("Error: {e}");
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

fn extract_todos_from_files(
    files: &Vec<PathBuf>,
    marker_config: &todo_extractor::MarkerConfig,
) -> Vec<todo_extractor::MarkedItem> {
    let mut new_todos = Vec::new();
    for file in files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let todos = todo_extractor::extract_todos_with_config(file, &content, marker_config);
            new_todos.extend(todos);
        } else {
            error!("Warning: Could not read file {file:?}, skipping.");
        }
    }
    new_todos
}

pub fn process_files_from_list(
    todo_path: &Path,
    scanned_files: Vec<PathBuf>,
    deleted_files: Vec<PathBuf>,
    git_ops: &dyn GitOpsTrait,
    repo: Repository,
    marker_config: &todo_extractor::MarkerConfig,
) -> Result<(), String> {
    let new_todos = extract_todos_from_files(&scanned_files, marker_config);

    // Pass the list of scanned files to sync_todo_file.
    if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, scanned_files, deleted_files) {
        info!("There was an error updating TODO.md: {err}");

        // TODO add tests for this branch

        let all_files = match git_ops.get_tracked_files(&repo) {
            Ok(files) => files,
            Err(e) => {
                error!("Error retrieving tracked files: {e}");
                std::process::exit(1);
            }
        };

        let new_todos = extract_todos_from_files(&all_files, marker_config);

        if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, all_files, vec![]) {
            error!("Error updating TODO.md: {err}");
            std::process::exit(1);
        }
    }
    info!("TODO.md successfully updated.");
    Ok(())
}
