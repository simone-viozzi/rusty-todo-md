use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::todo_md;
use crate::{extract_marked_items_from_file, MarkedItem, MarkerConfig};
use clap::{Arg, ArgAction, Command};
use git2::Repository;
use globset::Glob;
use log::{error, info};
use std::path::{Path, PathBuf};

/// Exclusion rule type
#[derive(Debug, Clone)]
enum ExclusionKind {
    /// Matches files and directories
    Exclude,
    /// Matches directories only
    ExcludeDir,
}

/// An exclusion rule with its pattern and kind
pub struct ExclusionRule {
    pattern: String,
    kind: ExclusionKind,
    glob: globset::GlobMatcher,
}

/// Build the exclusion matcher from CLI arguments
fn build_exclusion_matcher(
    exclude_patterns: Vec<String>,
    exclude_dir_patterns: Vec<String>,
) -> Result<Vec<ExclusionRule>, String> {
    let mut rules = Vec::new();

    // Add --exclude patterns
    for pattern in exclude_patterns {
        let normalized = normalize_pattern(&pattern);
        let glob = Glob::new(&normalized)
            .map_err(|e| format!("Invalid exclude pattern '{}': {}", pattern, e))?
            .compile_matcher();
        rules.push(ExclusionRule {
            pattern: pattern.clone(),
            kind: ExclusionKind::Exclude,
            glob,
        });
    }

    // Add --exclude-dir patterns (ensure they end with /)
    for pattern in exclude_dir_patterns {
        let pattern_with_slash = if pattern.ends_with('/') {
            pattern.clone()
        } else {
            format!("{}/", pattern)
        };
        let normalized = normalize_pattern(&pattern_with_slash);
        let glob = Glob::new(&normalized)
            .map_err(|e| format!("Invalid exclude-dir pattern '{}': {}", pattern, e))?
            .compile_matcher();
        rules.push(ExclusionRule {
            pattern: pattern_with_slash, // Store pattern with trailing slash
            kind: ExclusionKind::ExcludeDir,
            glob,
        });
    }

    Ok(rules)
}

/// Normalize a glob pattern to use forward slashes
fn normalize_pattern(pattern: &str) -> String {
    pattern.replace('\\', "/")
}

/// Check if a path should be excluded based on exclusion rules
/// Returns true if the path matches any exclusion rule (last match wins)
fn should_exclude(path: &Path, is_dir: bool, rules: &[ExclusionRule]) -> bool {
    // Try to match against both the full path and just the file/dir name components
    let path_str = path.to_str().unwrap_or("");
    let normalized_full_path = normalize_pattern(path_str);

    // Also get just the filename/dirname for simple pattern matching
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Get path components for relative path matching
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut excluded = false;

    for rule in rules {
        let mut matches = false;

        // Determine if this is a directory-only pattern
        let is_dir_pattern =
            rule.pattern.ends_with('/') || matches!(rule.kind, ExclusionKind::ExcludeDir);

        if is_dir_pattern {
            // This is a directory pattern - check if this is a dir OR if any parent is this dir
            if is_dir {
                // Check if the directory itself matches
                matches =
                    rule.glob.is_match(&normalized_full_path) || rule.glob.is_match(file_name);

                if !matches {
                    for i in 0..components.len() {
                        let partial_path = components[i..].join("/") + "/";
                        if rule.glob.is_match(&partial_path) {
                            matches = true;
                            break;
                        }
                    }
                }
            } else {
                // For files, check if any parent directory matches the pattern
                // e.g., if pattern is "src/" or "build" and file is "/path/build/output.rs", exclude it
                for i in 0..components.len() - 1 {
                    // -1 to exclude the filename itself
                    // Build the path up to (but not including) the filename
                    for j in (i + 1)..components.len() {
                        // Start from i+1 to get directory paths
                        let dir_path = components[i..j].join("/");
                        // Check if this directory path matches the pattern
                        if rule.glob.is_match(&dir_path)
                            || rule.glob.is_match(&(dir_path.clone() + "/"))
                        {
                            matches = true;
                            break;
                        }
                    }
                    if matches {
                        break;
                    }
                }
            }
        } else {
            // Regular file/dir pattern
            matches = rule.glob.is_match(&normalized_full_path) || rule.glob.is_match(file_name);

            if !matches {
                for i in 0..components.len() {
                    let partial_path = components[i..].join("/");
                    if rule.glob.is_match(&partial_path) {
                        matches = true;
                        break;
                    }
                }
            }
        }

        if matches {
            excluded = true; // Last match wins
        }
    }

    excluded
}

/// Filter files based on exclusion rules
fn filter_excluded_files(files: Vec<PathBuf>, rules: &[ExclusionRule]) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|file| {
            let is_dir = file.is_dir();
            let should_exclude_file = should_exclude(file, is_dir, rules);
            if should_exclude_file {
                info!("Excluding: {:?}", file);
            }
            !should_exclude_file
        })
        .collect()
}

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
                .value_name("GLOB")
                .help("Exclude files or directories matching glob pattern (relative to scan root). Can be specified multiple times. Use '/' suffix for directory-only patterns. Supports *, ?, and **.")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("exclude_dir")
                .long("exclude-dir")
                .value_name("GLOB")
                .help("Exclude directories matching glob pattern (directory-only). Can be specified multiple times.")
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

    // Parse exclude patterns from CLI args
    let exclude_patterns: Vec<String> = matches
        .get_many::<String>("exclude")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let exclude_dir_patterns: Vec<String> = matches
        .get_many::<String>("exclude_dir")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Build exclusion rules
    let exclusion_rules = match build_exclusion_matcher(exclude_patterns, exclude_dir_patterns) {
        Ok(rules) => rules,
        Err(e) => {
            error!("Error building exclusion patterns: {}", e);
            std::process::exit(1);
        }
    };

    // Process files (empty vec if no files provided) to ensure cleanup happens
    if let Err(e) = process_files_from_list(
        Path::new(todo_path),
        files,
        git_ops,
        repo,
        &marker_config,
        auto_add,
        &exclusion_rules,
    ) {
        error!("Error: {e}");
        std::process::exit(1);
    }
}

pub fn run_cli() {
    run_cli_with_args(std::env::args(), &GitOps);
}

fn extract_todos_from_files(files: &[PathBuf], marker_config: &MarkerConfig) -> Vec<MarkedItem> {
    let mut new_todos = Vec::new();
    for file in files {
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
    exclusion_rules: &[ExclusionRule],
) -> Result<(), String> {
    // Filter files based on exclusion rules before extraction
    let filtered_files = filter_excluded_files(scanned_files.clone(), exclusion_rules);

    let new_todos = extract_todos_from_files(&filtered_files, marker_config);

    // Capture the TODO file content before modification (if it exists)
    let todo_content_before = std::fs::read_to_string(todo_path).ok();

    // Validate that there are no empty TODO comments
    validate_no_empty_todos(&new_todos)?;

    // Pass the list of scanned files to sync_todo_file.
    if let Err(err) = todo_md::sync_todo_file(todo_path, new_todos, filtered_files.clone()) {
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

        // Filter all files with exclusion rules
        let filtered_all_files = filter_excluded_files(all_files, exclusion_rules);
        let new_todos = extract_todos_from_files(&filtered_all_files, marker_config);

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
