use crate::exclusion::{build_exclusion_matcher, filter_excluded_files, ExclusionRule};
use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::merge_driver;
use crate::todo_md;
use crate::{extract_marked_items_from_file, MarkedItem, MarkerConfig};
use clap::{Arg, ArgAction, Command};
use git2::Repository;
use log::{error, info};
use std::path::{Path, PathBuf};

pub fn run_cli_with_args<I, T>(args: I, git_ops: &dyn GitOpsTrait)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = build_cli().get_matches_from(args);

    let todo_path_str = matches
        .get_one::<String>("todo_path")
        .expect("TODO.md path should have a default value")
        .clone();
    let todo_path = PathBuf::from(&todo_path_str);

    let repo = match git_ops.open_repository(Path::new(".")) {
        Ok(r) => r,
        Err(e) => {
            error!("Error opening repository: {e}");
            std::process::exit(1);
        }
    };

    let marker_config = {
        let markers: Vec<String> = matches
            .get_many::<String>("markers")
            .map(|vals| vals.map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["TODO".to_string()]);
        MarkerConfig::normalized(markers)
    };

    let exclude_patterns: Vec<String> = matches
        .get_many::<String>("exclude")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let exclude_dir_patterns: Vec<String> = matches
        .get_many::<String>("exclude_dir")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let exclusion_rules =
        match build_exclusion_matcher(exclude_patterns.clone(), exclude_dir_patterns.clone()) {
            Ok(rules) => rules,
            Err(e) => {
                error!("Error building exclusion patterns: {}", e);
                std::process::exit(1);
            }
        };

    let merge_driver_args: Option<Vec<String>> = matches
        .get_many::<String>("merge_driver")
        .map(|vals| vals.cloned().collect());

    if let Some(args) = merge_driver_args {
        // git passes %O %A %B as positional values: BASE, OURS, THEIRS.
        let ours_path = PathBuf::from(&args[1]);
        if let Err(e) = cmd_merge(&ours_path, git_ops, &repo, &marker_config, &exclusion_rules) {
            error!("Error: {e}");
            std::process::exit(1);
        }
    } else if matches.get_flag("regenerate") {
        if !todo_path.exists() {
            if let Err(e) = std::fs::write(&todo_path, "") {
                error!("Error creating TODO.md: {e}");
                std::process::exit(1);
            }
        }
        if let Err(e) = cmd_regenerate(&todo_path, git_ops, &repo, &marker_config, &exclusion_rules)
        {
            error!("Error: {e}");
            std::process::exit(1);
        }
    } else if matches.get_flag("install_merge_driver") {
        match merge_driver::install_driver(
            &repo,
            &marker_config,
            &exclude_patterns,
            &exclude_dir_patterns,
            &todo_path,
        ) {
            Ok(summary) => {
                print!("{}", merge_driver::format_install_summary(&summary));
            }
            Err(e) => {
                error!("Error installing merge driver: {e}");
                std::process::exit(1);
            }
        }
    } else {
        if !todo_path.exists() {
            if let Err(e) = std::fs::write(&todo_path, "") {
                error!("Error creating TODO.md: {e}");
                std::process::exit(1);
            }
        }

        if matches.get_flag("auto_install_merge_driver")
            && !merge_driver::is_driver_registered(&repo)
        {
            match merge_driver::install_driver(
                &repo,
                &marker_config,
                &exclude_patterns,
                &exclude_dir_patterns,
                &todo_path,
            ) {
                Ok(summary) => {
                    println!(
                        "rusty-todo-md: --auto-install-merge-driver enabled and driver not yet registered."
                    );
                    print!("{}", merge_driver::format_install_summary(&summary));
                }
                Err(e) => {
                    eprintln!(
                        "rusty-todo-md: --auto-install-merge-driver: failed to install driver: {e}"
                    );
                    // Non-fatal: continue with normal processing.
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string(&todo_path) {
            if content.lines().any(|l| l.starts_with("<<<<<<<")) {
                eprintln!("rusty-todo-md: detected conflict markers in TODO.md.");
                eprintln!("  - To rebuild TODO.md from source now: rusty-todo-md --regenerate");
                eprintln!("  - To eliminate this conflict shape in future rebases: rusty-todo-md --install-merge-driver");
            }
        }

        let files: Vec<PathBuf> = matches
            .get_many::<String>("files")
            .unwrap_or_default()
            .map(PathBuf::from)
            .collect();

        let auto_add = matches.get_flag("auto_add");

        if let Err(e) = process_files_from_list(
            &todo_path,
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
}

fn build_cli() -> Command {
    Command::new("rusty-todo-md")
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
                .global(true)
                .default_value("TODO.md"),
        )
        .arg(
            Arg::new("markers")
                .short('m')
                .long("markers")
                .value_name("KEYWORDS")
                .help("Specifies one or more marker keywords to search for (e.g., TODO FIXME HACK). Usage: --markers TODO FIXME HACK [-- file1.rs file2.rs]")
                .num_args(1..)
                .global(true),
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
                .action(ArgAction::Append)
                .global(true),
        )
        .arg(
            Arg::new("exclude_dir")
                .long("exclude-dir")
                .value_name("GLOB")
                .help("Exclude directories matching glob pattern (directory-only). Can be specified multiple times.")
                .action(ArgAction::Append)
                .global(true),
        )
        .arg(
            Arg::new("auto_install_merge_driver")
                .long("auto-install-merge-driver")
                .help("Opt-in: on first run per clone, register the TODO.md merge driver in .git/config and append a line to .gitattributes. Prints a loud summary of what changed. Intended for repo maintainers to put in pre-commit args.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("regenerate")
                .long("regenerate")
                .help("Re-scan all tracked files and rewrite TODO.md from scratch. Wipes any existing content (including conflict markers).")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["install_merge_driver", "merge_driver"]),
        )
        .arg(
            Arg::new("install_merge_driver")
                .long("install-merge-driver")
                .help("Register the TODO.md merge driver in .git/config and append a line to .gitattributes.")
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["regenerate", "merge_driver"]),
        )
        .arg(
            Arg::new("merge_driver")
                .long("merge-driver")
                .value_names(["BASE", "OURS", "THEIRS"])
                .num_args(3)
                .help("Git merge-driver entry point. Invoked by git as `--merge-driver %O %A %B`; regenerates TODO.md from working-tree source and writes it to OURS.")
                .conflicts_with_all(["regenerate", "install_merge_driver"]),
        )
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

/// Re-scan all tracked source files and rewrite TODO.md from scratch.
///
/// Used by both the user-facing `regenerate` subcommand and the `merge` merge
/// driver. Bypasses `sync_todo_file`'s read-then-merge step on purpose: a
/// fresh write is what wipes any conflict markers or stale entries.
fn regenerate_todo_md(
    todo_output_path: &Path,
    git_ops: &dyn GitOpsTrait,
    repo: &Repository,
    marker_config: &MarkerConfig,
    exclusion_rules: &[ExclusionRule],
    validate_empty: bool,
) -> Result<(), String> {
    let all_files = git_ops
        .get_tracked_files(repo)
        .map_err(|e| format!("failed to enumerate tracked files: {e}"))?;
    let filtered = filter_excluded_files(all_files, exclusion_rules);
    let todos = extract_todos_from_files(&filtered, marker_config);
    if validate_empty {
        validate_no_empty_todos(&todos)?;
    }
    todo_md::write_todo_file(todo_output_path, todos)
        .map_err(|e| format!("failed to write {}: {e}", todo_output_path.display()))?;
    Ok(())
}

fn cmd_regenerate(
    todo_path: &Path,
    git_ops: &dyn GitOpsTrait,
    repo: &Repository,
    marker_config: &MarkerConfig,
    exclusion_rules: &[ExclusionRule],
) -> Result<(), String> {
    regenerate_todo_md(
        todo_path,
        git_ops,
        repo,
        marker_config,
        exclusion_rules,
        true,
    )?;
    info!("TODO.md successfully regenerated.");
    Ok(())
}

/// Git merge driver. Ignores BASE/THEIRS — at merge-driver invocation time the
/// working tree already reflects the cumulative state of all replayed commits
/// for non-conflicting source files, so a fresh scan produces the canonical
/// TODO.md by construction. Skips empty-TODO validation: a half-merged source
/// file (with conflict markers) is already skipped at the extractor level, and
/// failing the merge here would surface the conflict to the user instead of
/// resolving it.
fn cmd_merge(
    ours_path: &Path,
    git_ops: &dyn GitOpsTrait,
    repo: &Repository,
    marker_config: &MarkerConfig,
    exclusion_rules: &[ExclusionRule],
) -> Result<(), String> {
    regenerate_todo_md(
        ours_path,
        git_ops,
        repo,
        marker_config,
        exclusion_rules,
        false,
    )?;
    info!("TODO.md merge driver wrote canonical output to {ours_path:?}.");
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
