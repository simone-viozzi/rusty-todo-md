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
        .about("Automatically scans files for TODO comments and updates TODO.md. Use '--' to separate markers from files when markers is the last option.")
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
                .help("Specifies one or more marker keywords to search for (e.g., TODO FIXME HACK). Usage: --markers TODO FIXME HACK [-- file1.rs file2.rs]")
                .num_args(1..)
        )
        .arg(
            Arg::new("files")
                .value_name("FILE")
                .help("Optional list of files to process (passed by pre-commit)")
                .num_args(0..)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("auto_add")
                .long("auto-add")
                .help("Automatically add TODO.md file to git staging if it was modified")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .value_name("PATH")
                .help("Exclude specific files or directories from processing. Can be specified multiple times.")
                .action(ArgAction::Append),
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

    // Parse markers from CLI args (if any)
    let markers: Vec<String> = matches
        .get_many::<String>("markers")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| vec!["TODO".to_string()]);
    let marker_config = MarkerConfig::normalized(markers);

    let auto_add = matches.get_flag("auto_add");

    // Parse exclude paths from CLI args (if any)
    let exclude_paths: Vec<PathBuf> = matches
        .get_many::<String>("exclude")
        .map(|vals| vals.map(PathBuf::from).collect())
        .unwrap_or_default();

    // Process files (empty vec if no files provided) to ensure cleanup happens
    if let Err(e) = process_files_from_list(
        Path::new(todo_path),
        files,
        git_ops,
        repo,
        &marker_config,
        auto_add,
        &exclude_paths,
    ) {
        error!("Error: {e}");
        std::process::exit(1);
    }
}

pub fn run_cli() {
    run_cli_with_args(std::env::args(), &GitOps);
}

/// Check if a file path should be excluded based on the exclude patterns.
/// Returns true if the file should be excluded.
fn should_exclude(file_path: &Path, exclude_paths: &[PathBuf]) -> bool {
    for exclude_path in exclude_paths {
        // Get the file name for simple file name matching
        if let Some(file_name) = file_path.file_name() {
            if let Some(exclude_name) = exclude_path.file_name() {
                // Check if it's a simple file name match
                if exclude_path.components().count() == 1 && file_name == exclude_name {
                    return true;
                }
            }
        }

        // Normalize both paths
        let file_normalized = file_path.components().collect::<PathBuf>();
        let exclude_normalized = exclude_path.components().collect::<PathBuf>();

        // Check if file path ends with the exclude pattern (handles relative paths)
        if file_normalized.ends_with(&exclude_normalized) {
            return true;
        }

        // Check if any component sequence matches
        let file_components: Vec<_> = file_normalized.components().collect();
        let exclude_components: Vec<_> = exclude_normalized.components().collect();

        // Check if exclude pattern matches from any position in the file path
        for i in 0..=file_components
            .len()
            .saturating_sub(exclude_components.len())
        {
            let slice: PathBuf = file_components[i..i + exclude_components.len()]
                .iter()
                .collect();
            if slice == exclude_normalized {
                return true;
            }
        }
    }
    false
}

fn extract_todos_from_files(
    files: &Vec<PathBuf>,
    marker_config: &MarkerConfig,
    exclude_paths: &[PathBuf],
) -> Vec<MarkedItem> {
    let mut new_todos = Vec::new();
    for file in files {
        // Skip excluded files
        if should_exclude(file, exclude_paths) {
            info!("Excluding file: {:?}", file);
            continue;
        }

        match extract_marked_items_from_file(file, marker_config) {
            Ok(mut todos) => new_todos.append(&mut todos),
            Err(e) => error!("Error processing file {:?}: {}", file, e),
        }
    }
    new_todos
}

pub fn validate_no_empty_todos(new_todos: &[MarkedItem]) -> Result<(), String> {
    let empty_todos: Vec<&MarkedItem> = new_todos
        .iter()
        .filter(|item| item.message.trim().is_empty())
        .collect();

    if !empty_todos.is_empty() {
        let errors: Vec<String> = empty_todos
            .iter()
            .map(|item| {
                format!(
                    "error: empty {} comment found\n  --> {}:{}",
                    item.marker,
                    item.file_path.display(),
                    item.line_number
                )
            })
            .collect();

        return Err(format!(
            "{}\n\nPlease add descriptions to the empty TODO comments above.",
            errors.join("\n\n")
        ));
    }

    Ok(())
}

pub fn process_files_from_list(
    todo_path: &Path,
    scanned_files: Vec<PathBuf>,
    git_ops: &dyn GitOpsTrait,
    repo: Repository,
    marker_config: &MarkerConfig,
    auto_add: bool,
    exclude_paths: &[PathBuf],
) -> Result<(), String> {
    let new_todos = extract_todos_from_files(&scanned_files, marker_config, exclude_paths);

    // Capture the TODO file content before modification (if it exists)
    let todo_content_before = std::fs::read_to_string(todo_path).ok();

    // Validate that there are no empty TODO comments
    validate_no_empty_todos(&new_todos)?;

    // Pass the list of scanned files to sync_todo_file.
    if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, scanned_files) {
        info!("There was an error updating TODO.md: {err}");

        // This branch is tested by test_sync_todo_file_fallback_mechanism.
        // It does not show in code coverage because it is an integration test
        // that calls the binary, not a unit test that calls this function directly.

        let all_files = match git_ops.get_tracked_files(&repo) {
            Ok(files) => files,
            Err(e) => {
                error!("Error retrieving tracked files: {e}");
                std::process::exit(1);
            }
        };

        let new_todos = extract_todos_from_files(&all_files, marker_config, exclude_paths);

        if let Err(err) = todo_md::write_todo_file(todo_path, new_todos) {
            error!("Error updating TODO.md: {err}");
            std::process::exit(1);
        }
    }
    info!("TODO.md successfully updated.");

    // If auto_add is enabled, check if the TODO file was modified and stage it
    // TODO simplify this, maybe move to git_utils and maybe do not check if content changed
    //      but just try to add it and ignore errors in case it was not modified
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
