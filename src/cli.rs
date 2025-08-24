use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::todo_md;
use crate::{extract_marked_items_from_file, MarkedItem, MarkerConfig};
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
        .author("Simone Viozzi simoneviozzi97@gmail.com")
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
        .arg(
            Arg::new("auto_add")
                .long("auto-add")
                .help("Automatically add TODO.md file to git staging if it was modified")
                .action(ArgAction::SetTrue),
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
    let marker_config = MarkerConfig::normalized(markers);

    let auto_add = matches.get_flag("auto_add");

    if !files.is_empty() || !deleted_files.is_empty() {
        if let Err(e) = process_files_from_list(
            Path::new(todo_path),
            files,
            deleted_files,
            git_ops,
            repo,
            &marker_config,
            auto_add,
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

fn extract_todos_from_files(files: &Vec<PathBuf>, marker_config: &MarkerConfig) -> Vec<MarkedItem> {
    let mut new_todos = Vec::new();
    for file in files {
        match extract_marked_items_from_file(file, marker_config) {
            Ok(mut todos) => new_todos.append(&mut todos),
            Err(e) => error!("Error processing file {:?}: {}", file, e),
        }
    }
    new_todos
}

pub fn validate_no_empty_todos(
    files: &Vec<PathBuf>,
    marker_config: &todo_extractor::MarkerConfig,
) -> Result<(), String> {
    let mut errors = Vec::new();

    for file in files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let empty_todos = find_empty_todos(file, &content, marker_config);
            for empty_todo in empty_todos {
                errors.push(format!(
                    "error: empty {} comment found\n  --> {}:{}\n   |\n{:3} | {}\n   | {}^ empty {} comment requires a message",
                    empty_todo.marker,
                    file.display(),
                    empty_todo.line_number,
                    empty_todo.line_number,
                    empty_todo.line_content.trim(),
                    " ".repeat(empty_todo.marker_position),
                    empty_todo.marker
                ));
            }
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "{}\n\nPlease add descriptions to the empty TODO comments above.",
            errors.join("\n\n")
        ));
    }

    Ok(())
}

#[derive(Debug)]
pub struct EmptyTodo {
    pub line_number: usize,
    pub line_content: String,
    pub marker: String,
    pub marker_position: usize,
}

pub fn find_empty_todos(
    _file: &Path,
    content: &str,
    marker_config: &todo_extractor::MarkerConfig,
) -> Vec<EmptyTodo> {
    let mut empty_todos = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_number = line_idx + 1;
        let trimmed_line = line.trim();

        for marker in &marker_config.markers {
            // Check for comment-style markers like "// TODO:" or "# TODO:"
            let patterns = vec![
                format!("// {}:", marker),
                format!("/* {}:", marker),
                format!("# {}:", marker),
                format!("-- {}:", marker),
            ];

            for pattern in patterns {
                if let Some(start_pos) = trimmed_line.find(&pattern) {
                    // Extract the part after the marker
                    let after_marker = &trimmed_line[start_pos + pattern.len()..];

                    // Check if there's only whitespace or closing comment after the marker
                    let content_after_marker = after_marker.trim_start();
                    let is_empty = content_after_marker.is_empty()
                        || content_after_marker == "*/"
                        || content_after_marker.starts_with("*/");

                    if is_empty {
                        // Find the position of the marker in the original line
                        let marker_pos_in_line = line.find(&pattern).unwrap_or(0);

                        empty_todos.push(EmptyTodo {
                            line_number,
                            line_content: line.to_string(),
                            marker: marker.clone(),
                            marker_position: marker_pos_in_line,
                        });
                        break; // Don't check other patterns for this line
                    }
                }
            }
        }
    }

    empty_todos
}

pub fn process_files_from_list(
    todo_path: &Path,
    scanned_files: Vec<PathBuf>,
    deleted_files: Vec<PathBuf>,
    git_ops: &dyn GitOpsTrait,
    repo: Repository,
    marker_config: &MarkerConfig,
    auto_add: bool,
) -> Result<(), String> {
    let new_todos = extract_todos_from_files(&scanned_files, marker_config);

    // Capture the TODO file content before modification (if it exists)
    let todo_content_before = std::fs::read_to_string(todo_path).ok();
    
    // Validate that there are no empty TODO comments
    if let Err(empty_todo_errors) = validate_no_empty_todos(&scanned_files, marker_config) {
        return Err(empty_todo_errors);
    }

    // Pass the list of scanned files to sync_todo_file.
    if let Err(err) =
        todo_md::sync_todo_file(todo_path, new_todos, scanned_files, deleted_files, false)
    {
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

        if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, all_files, vec![], true) {
            error!("Error updating TODO.md: {err}");
            std::process::exit(1);
        }
    }
    info!("TODO.md successfully updated.");

    // If auto_add is enabled, check if the TODO file was modified and stage it
    if auto_add {
        let todo_content_after = std::fs::read_to_string(todo_path).ok();
        if todo_content_before != todo_content_after {
            info!("TODO file was modified, staging it for commit");

            // Convert todo_path to absolute path, then to relative path from repo root
            let repo_workdir = repo
                .workdir()
                .ok_or("Repository has no working directory")?;
            let absolute_todo_path = if todo_path.is_absolute() {
                todo_path.to_path_buf()
            } else {
                repo_workdir.join(todo_path)
            };
            let relative_todo_path = absolute_todo_path
                .strip_prefix(repo_workdir)
                .map_err(|_| "TODO path is not within repository")?;

            if let Err(e) = git_ops.add_file_to_index(&repo, relative_todo_path) {
                error!("Warning: Failed to add TODO file to git index: {e}");
                // Don't fail the entire operation just because we couldn't stage the file
            } else {
                info!("Successfully staged TODO file: {relative_todo_path:?}");
            }
        } else {
            info!("TODO file was not modified, skipping auto-add");
        }
    }

    Ok(())
}
