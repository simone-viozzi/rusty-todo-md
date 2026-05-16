use crate::exclusion::{build_exclusion_matcher, filter_excluded_files, ExclusionRule};
use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::merge_driver;
use crate::todo_md;
use crate::{extract_marked_items_from_file, MarkedItem, MarkerConfig};
use clap::{Arg, ArgAction, ArgMatches, Command};
use git2::Repository;
use log::{error, info};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn run_cli() {
    run_cli_with_args(std::env::args(), &GitOps);
}

pub fn run_cli_with_args<I, T>(args: I, git_ops: &dyn GitOpsTrait)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let parsed = match ParsedArgs::from_clap_matches(build_cli().get_matches_from(args)) {
        Ok(p) => p,
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = dispatch(&parsed, git_ops) {
        error!("Error: {e}");
        std::process::exit(1);
    }
}

// Re-exported because integration tests in `tests/` use it directly.
pub fn validate_no_empty_todos(new_todos: &[MarkedItem]) -> Result<(), String> {
    let empty_todos: Vec<&MarkedItem> = new_todos
        .iter()
        .filter(|item| item.message.trim().is_empty())
        .collect();
    if empty_todos.is_empty() {
        return Ok(());
    }
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
    Err(format!(
        "{}\n\nPlease add descriptions to the empty TODO comments above.",
        errors.join("\n\n")
    ))
}

// ---------------------------------------------------------------------------
// Parsed args + mode dispatch
// ---------------------------------------------------------------------------

/// What the four mutually-exclusive operating modes do.
///
/// Each top-level invocation lands in exactly one variant; `Scan` is the
/// default when no mode-selecting flag is present and is the only mode that
/// honors `auto_add` / `auto_install_merge_driver`.
enum Mode {
    Scan,
    Regenerate,
    Install,
    MergeDriver { ours: PathBuf },
}

/// Everything the CLI needs after parsing. Kept as a flat struct (rather
/// than one-per-mode) because most fields are mode-agnostic (markers,
/// exclusions, todo-path) and the cost of a few unused fields per mode is
/// smaller than the cost of variant duplication.
struct ParsedArgs {
    mode: Mode,
    todo_path: PathBuf,
    marker_config: MarkerConfig,
    exclude_patterns: Vec<String>,
    exclude_dir_patterns: Vec<String>,
    exclusion_rules: Vec<ExclusionRule>,
    files: Vec<PathBuf>,
    auto_add: bool,
    auto_install_merge_driver: bool,
}

impl ParsedArgs {
    fn from_clap_matches(matches: ArgMatches) -> Result<Self, String> {
        let todo_path = PathBuf::from(
            matches
                .get_one::<String>("todo_path")
                .expect("--todo-path has a default value"),
        );

        let markers: Vec<String> = matches
            .get_many::<String>("markers")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_else(|| vec!["TODO".to_string()]);
        let marker_config = MarkerConfig::normalized(markers);

        let exclude_patterns: Vec<String> = matches
            .get_many::<String>("exclude")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default();
        let exclude_dir_patterns: Vec<String> = matches
            .get_many::<String>("exclude_dir")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default();
        let exclusion_rules =
            build_exclusion_matcher(exclude_patterns.clone(), exclude_dir_patterns.clone())
                .map_err(|e| format!("Error building exclusion patterns: {e}"))?;

        let files: Vec<PathBuf> = matches
            .get_many::<String>("files")
            .map(|vals| vals.map(PathBuf::from).collect())
            .unwrap_or_default();

        let mode = if let Some(vals) = matches.get_many::<String>("merge_driver") {
            // git passes %O %A %B; OURS is the second value and the only one
            // the driver writes to.
            let triple: Vec<&String> = vals.collect();
            let ours = PathBuf::from(triple[1]);
            Mode::MergeDriver { ours }
        } else if matches.get_flag("regenerate") {
            Mode::Regenerate
        } else if matches.get_flag("install_merge_driver") {
            Mode::Install
        } else {
            Mode::Scan
        };

        Ok(ParsedArgs {
            mode,
            todo_path,
            marker_config,
            exclude_patterns,
            exclude_dir_patterns,
            exclusion_rules,
            files,
            auto_add: matches.get_flag("auto_add"),
            auto_install_merge_driver: matches.get_flag("auto_install_merge_driver"),
        })
    }
}

fn dispatch(args: &ParsedArgs, git_ops: &dyn GitOpsTrait) -> Result<(), String> {
    let repo = git_ops
        .open_repository(Path::new("."))
        .map_err(|e| format!("Error opening repository: {e}"))?;
    match &args.mode {
        Mode::MergeDriver { ours } => mode::merge_driver(args, &repo, git_ops, ours),
        Mode::Regenerate => mode::regenerate(args, &repo, git_ops),
        Mode::Install => mode::install(args, &repo),
        Mode::Scan => mode::scan(args, repo, git_ops),
    }
}

// ---------------------------------------------------------------------------
// Modes
// ---------------------------------------------------------------------------

mod mode {
    use super::*;

    /// Default mode: process the files pre-commit passed us, merge into the
    /// existing TODO.md, optionally auto-add it back to the index, and
    /// optionally self-install the merge driver.
    pub(super) fn scan(
        args: &ParsedArgs,
        repo: Repository,
        git_ops: &dyn GitOpsTrait,
    ) -> Result<(), String> {
        ensure_todo_path_exists(&args.todo_path)?;
        if args.auto_install_merge_driver {
            maybe_auto_install(args, &repo);
        }
        warn_if_todo_md_has_conflict_markers(&args.todo_path);
        process_files(args, repo, git_ops)
    }

    /// `--regenerate`: rebuild TODO.md from scratch (current index ⇒ TODO.md).
    pub(super) fn regenerate(
        args: &ParsedArgs,
        repo: &Repository,
        git_ops: &dyn GitOpsTrait,
    ) -> Result<(), String> {
        ensure_todo_path_exists(&args.todo_path)?;
        regenerate_todo_md(args, repo, git_ops, &args.todo_path, true)?;
        info!("TODO.md successfully regenerated.");
        Ok(())
    }

    /// `--install-merge-driver`: register the driver in `.git/config` and
    /// `.gitattributes`. Convergent — running it twice with the same args is
    /// a no-op on disk.
    pub(super) fn install(args: &ParsedArgs, repo: &Repository) -> Result<(), String> {
        let summary = merge_driver::install_driver(
            repo,
            &args.marker_config,
            &args.exclude_patterns,
            &args.exclude_dir_patterns,
            &args.todo_path,
        )
        .map_err(|e| format!("Error installing merge driver: {e}"))?;
        print!("{}", merge_driver::format_install_summary(&summary));
        Ok(())
    }

    /// Git merge-driver entry point. Ignores BASE/THEIRS — at invocation
    /// time the working tree's source files already reflect the cumulative
    /// state of all replayed commits (for files that didn't themselves
    /// conflict), so a fresh scan produces canonical TODO.md by
    /// construction. Skips empty-TODO validation: a half-merged source file
    /// (with conflict markers) is already skipped at the extractor level,
    /// and failing the merge here would just surface the conflict back to
    /// the user instead of resolving it.
    pub(super) fn merge_driver(
        args: &ParsedArgs,
        repo: &Repository,
        git_ops: &dyn GitOpsTrait,
        ours: &Path,
    ) -> Result<(), String> {
        regenerate_todo_md(args, repo, git_ops, ours, false)?;
        info!("TODO.md merge driver wrote canonical output to {ours:?}.");
        Ok(())
    }

    /// Auto-install side-effect. Only called from scan mode when
    /// `--auto-install-merge-driver` is set. Non-fatal on failure: a flaky
    /// install must never block the actual pre-commit work.
    fn maybe_auto_install(args: &ParsedArgs, repo: &Repository) {
        if merge_driver::is_driver_registered(repo) {
            return;
        }
        match merge_driver::install_driver(
            repo,
            &args.marker_config,
            &args.exclude_patterns,
            &args.exclude_dir_patterns,
            &args.todo_path,
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
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (used by multiple modes)
// ---------------------------------------------------------------------------

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

fn ensure_todo_path_exists(todo_path: &Path) -> Result<(), String> {
    if todo_path.exists() {
        return Ok(());
    }
    std::fs::write(todo_path, "").map_err(|e| format!("Error creating TODO.md: {e}"))
}

fn warn_if_todo_md_has_conflict_markers(todo_path: &Path) {
    if let Ok(content) = std::fs::read_to_string(todo_path) {
        if content.lines().any(|l| l.starts_with("<<<<<<<")) {
            eprintln!("rusty-todo-md: detected conflict markers in TODO.md.");
            eprintln!("  - To rebuild TODO.md from source now: rusty-todo-md --regenerate");
            eprintln!("  - To eliminate this conflict shape in future rebases: rusty-todo-md --install-merge-driver");
        }
    }
}

/// Re-scan the current index and rewrite TODO.md from scratch.
///
/// Shared by the `--regenerate` user command and the `--merge-driver` git
/// entry point. Bypasses `sync_todo_file`'s read-merge-write step on
/// purpose: writing from scratch is what wipes prior conflict markers.
fn regenerate_todo_md(
    args: &ParsedArgs,
    repo: &Repository,
    git_ops: &dyn GitOpsTrait,
    output_path: &Path,
    validate_empty: bool,
) -> Result<(), String> {
    let all_files = git_ops
        .get_tracked_files(repo)
        .map_err(|e| format!("failed to enumerate tracked files: {e}"))?;
    let filtered = filter_excluded_files(all_files, &args.exclusion_rules);
    let todos = extract_todos_from_files(&filtered, &args.marker_config);
    if validate_empty {
        validate_no_empty_todos(&todos)?;
    }
    todo_md::write_todo_file(output_path, todos)
        .map_err(|e| format!("failed to write {}: {e}", output_path.display()))?;
    Ok(())
}

fn process_files(
    args: &ParsedArgs,
    repo: Repository,
    git_ops: &dyn GitOpsTrait,
) -> Result<(), String> {
    let filtered_files = filter_excluded_files(args.files.clone(), &args.exclusion_rules);
    let new_todos = extract_todos_from_files(&filtered_files, &args.marker_config);
    let todo_content_before = std::fs::read_to_string(&args.todo_path).ok();

    validate_no_empty_todos(&new_todos)?;

    if let Err(err) = todo_md::sync_todo_file(&args.todo_path, new_todos, filtered_files) {
        info!("There was an error updating TODO.md: {err}");
        sync_fallback_full_rescan(args, &repo, git_ops);
    }
    info!("TODO.md successfully updated.");

    if args.auto_add {
        maybe_stage_todo_file(&args.todo_path, &repo, git_ops, &todo_content_before)?;
    }
    Ok(())
}

/// Last-resort recovery when `sync_todo_file` can't parse the existing
/// TODO.md: rescan everything tracked and overwrite from scratch. Exit
/// (rather than return Err) because at this point the TODO.md is already
/// broken and propagating the error would leave the user with two failures
/// to read.
fn sync_fallback_full_rescan(args: &ParsedArgs, repo: &Repository, git_ops: &dyn GitOpsTrait) {
    let all_files = match git_ops.get_tracked_files(repo) {
        Ok(files) => files,
        Err(e) => {
            error!("Error retrieving tracked files: {e}");
            std::process::exit(1);
        }
    };
    let filtered = filter_excluded_files(all_files, &args.exclusion_rules);
    let todos = extract_todos_from_files(&filtered, &args.marker_config);
    if let Err(err) = todo_md::write_todo_file(&args.todo_path, todos) {
        error!("Error updating TODO.md: {err}");
        std::process::exit(1);
    }
}

fn maybe_stage_todo_file(
    todo_path: &Path,
    repo: &Repository,
    git_ops: &dyn GitOpsTrait,
    todo_content_before: &Option<String>,
) -> Result<(), String> {
    let todo_content_after = std::fs::read_to_string(todo_path).ok();
    if todo_content_before == &todo_content_after {
        info!("TODO file was not modified, skipping auto-add");
        return Ok(());
    }
    info!("TODO file was modified, staging it for commit");

    let repo_workdir = repo
        .workdir()
        .ok_or("Repository has no working directory")?;
    let absolute = if todo_path.is_absolute() {
        todo_path.to_path_buf()
    } else {
        repo_workdir.join(todo_path)
    };
    let relative = absolute
        .strip_prefix(repo_workdir)
        .map_err(|_| "TODO path is not within repository")?;

    if let Err(e) = git_ops.add_file_to_index(repo, relative) {
        // Warn but don't fail: staging failure shouldn't kill the commit.
        error!("Warning: Failed to add TODO file to git index: {e}");
    } else {
        info!("Successfully staged TODO file: {relative:?}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Clap configuration
// ---------------------------------------------------------------------------

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
