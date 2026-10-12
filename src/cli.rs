use crate::exclusion::{build_exclusion_matcher, filter_excluded_files, ExclusionRule};
use crate::git_utils::GitOps;
use crate::git_utils::GitOpsTrait;
use crate::todo_extractor_internal::aggregator::content_has_conflict_markers;
use crate::todo_md;
use crate::{extract_marked_items_from_file, MarkedItem, MarkerConfig};
use clap::{Arg, ArgAction, ArgMatches, Command};
use git2::Repository;
use log::{error, info};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

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

/// Everything the CLI needs after parsing. `regenerate` selects the
/// `--regenerate` mode; otherwise the default scan mode runs, which is the
/// only one that honors `auto_add`.
struct ParsedArgs {
    regenerate: bool,
    todo_path: PathBuf,
    marker_config: MarkerConfig,
    exclusion_rules: Vec<ExclusionRule>,
    files: Vec<PathBuf>,
    auto_add: bool,
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
        let exclusion_rules = build_exclusion_matcher(exclude_patterns, exclude_dir_patterns)
            .map_err(|e| format!("Error building exclusion patterns: {e}"))?;

        let files: Vec<PathBuf> = matches
            .get_many::<String>("files")
            .map(|vals| vals.map(PathBuf::from).collect())
            .unwrap_or_default();

        Ok(ParsedArgs {
            regenerate: matches.get_flag("regenerate"),
            todo_path,
            marker_config,
            exclusion_rules,
            files,
            auto_add: matches.get_flag("auto_add"),
        })
    }
}

fn dispatch(args: &ParsedArgs, git_ops: &dyn GitOpsTrait) -> Result<(), String> {
    let repo = git_ops
        .open_repository(Path::new("."))
        .map_err(|e| format!("Error opening repository: {e}"))?;
    if args.regenerate {
        mode::regenerate(args, &repo, git_ops)
    } else {
        mode::scan(args, repo, git_ops)
    }
}

// ---------------------------------------------------------------------------
// Modes
// ---------------------------------------------------------------------------

mod mode {
    use super::*;

    /// Default mode: rebuild TODO.md from the files pre-commit passed us
    /// plus the files it already lists, and optionally auto-add it back to
    /// the index.
    pub(super) fn scan(
        args: &ParsedArgs,
        repo: Repository,
        git_ops: &dyn GitOpsTrait,
    ) -> Result<(), String> {
        ensure_todo_path_exists(&args.todo_path)?;
        process_files(args, repo, git_ops)
    }

    /// `--regenerate`: rebuild TODO.md from scratch (current index ⇒ TODO.md).
    pub(super) fn regenerate(
        args: &ParsedArgs,
        repo: &Repository,
        git_ops: &dyn GitOpsTrait,
    ) -> Result<(), String> {
        ensure_todo_path_exists(&args.todo_path)?;
        regenerate_todo_md(args, repo, git_ops)?;
        info!("TODO.md successfully regenerated.");
        Ok(())
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

/// Re-scan the current index and rewrite TODO.md from scratch.
fn regenerate_todo_md(
    args: &ParsedArgs,
    repo: &Repository,
    git_ops: &dyn GitOpsTrait,
) -> Result<(), String> {
    let all_files = git_ops
        .get_tracked_files(repo)
        .map_err(|e| format!("failed to enumerate tracked files: {e}"))?;
    let filtered = filter_excluded_files(all_files, &args.exclusion_rules);
    let todos = extract_todos_from_files(&filtered, &args.marker_config);
    validate_no_empty_todos(&todos)?;
    todo_md::write_todo_file(&args.todo_path, todos)
        .map_err(|e| format!("failed to write {}: {e}", args.todo_path.display()))?;
    Ok(())
}

/// Rebuild TODO.md from the files pre-commit passed plus every file TODO.md
/// already lists. Nothing is merged from the old TODO.md, so whatever a
/// merge, rebase or hand edit left in it (conflict markers, `merge=union`
/// leftovers, stale line numbers) is replaced by a fresh scan of the files
/// it names. Files that were deleted or are now excluded drop out.
fn process_files(
    args: &ParsedArgs,
    repo: Repository,
    git_ops: &dyn GitOpsTrait,
) -> Result<(), String> {
    let todo_content_before = std::fs::read_to_string(&args.todo_path).ok();
    if todo_content_before
        .as_deref()
        .is_some_and(content_has_conflict_markers)
    {
        print_merge_union_hint(&args.todo_path, &repo);
    }

    // Compare and record every path in its repository-relative form: the
    // index lists paths that way, and absolute entries in TODO.md would
    // neither match it nor make working links.
    let normalize = |path: PathBuf| repo_relative(&path, &repo).unwrap_or(path);
    let passed: Vec<PathBuf> = args.files.iter().cloned().map(normalize).collect();
    let passed = filter_excluded_files(passed, &args.exclusion_rules);
    let passed_todos = extract_todos_from_files(&passed, &args.marker_config);
    // Only the files being committed must be free of empty TODOs; one that
    // slipped in elsewhere (e.g. via --no-verify) shouldn't block this commit.
    validate_no_empty_todos(&passed_todos)?;

    // TODO.md is committed input, so whoever wrote it chooses what we read.
    // Only rescan files git tracks: anything else (`../outside.rs`, an
    // untracked `.env`) could leak its TODO lines into TODO.md.
    let tracked: BTreeSet<PathBuf> = git_ops
        .get_tracked_files(&repo)
        .map_err(|e| format!("failed to enumerate tracked files: {e}"))?
        .into_iter()
        .collect();
    let passed: BTreeSet<PathBuf> = passed.into_iter().collect();
    let listed: Vec<PathBuf> = todo_md::listed_files(todo_content_before.as_deref().unwrap_or(""))
        .into_iter()
        .map(normalize)
        .filter(|file| tracked.contains(file) && file.exists() && !passed.contains(file))
        .collect();
    let mut todos = passed_todos;
    todos.extend(extract_todos_from_files(
        &filter_excluded_files(listed, &args.exclusion_rules),
        &args.marker_config,
    ));

    todo_md::write_todo_file(&args.todo_path, todos)
        .map_err(|e| format!("Error updating TODO.md: {e}"))?;
    info!("TODO.md successfully updated.");

    if args.auto_add {
        maybe_stage_todo_file(&args.todo_path, &repo, git_ops, &todo_content_before)?;
    }
    Ok(())
}

/// Suggest the `.gitattributes` rule that avoids TODO.md conflicts. The
/// pattern must be repository-relative; when the path would need
/// gitattributes quoting or escaping, point at the README instead of
/// printing a rule that wouldn't match.
fn print_merge_union_hint(todo_path: &Path, repo: &Repository) {
    let pattern = repo_relative(todo_path, repo).map(|p| p.display().to_string());
    match pattern {
        Some(p)
            if !p.starts_with('!')
                && !p
                    .chars()
                    .any(|c| c.is_whitespace() || matches!(c, '*' | '?' | '[' | '\\' | '"' | '#')) =>
        {
            // Leading `/` anchors the pattern; without it the rule would
            // also apply to every same-named file in any subdirectory.
            eprintln!("hint: add \"/{p} merge=union\" to .gitattributes to avoid conflicts in it");
        }
        _ => eprintln!(
            "hint: add a \"merge=union\" rule for {} to .gitattributes to avoid conflicts in it (see the README)",
            todo_path.display()
        ),
    }
}

/// `path` relative to the repository root, if it is inside it.
fn repo_relative(path: &Path, repo: &Repository) -> Option<PathBuf> {
    let workdir = repo.workdir()?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workdir.join(path)
    };
    let relative = absolute
        .strip_prefix(workdir)
        .ok()
        .map(Path::to_path_buf)
        .or_else(|| {
            // libgit2 resolves symlinks in the workdir, so a path typed
            // through a symlink only matches once it's resolved too.
            let resolved = absolute.canonicalize().ok()?;
            let workdir = workdir.canonicalize().ok()?;
            resolved.strip_prefix(workdir).ok().map(Path::to_path_buf)
        })?;
    // `workdir/../x` still starts with `workdir`; it isn't inside it.
    (!relative.components().any(|c| c == Component::ParentDir)).then_some(relative)
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

    let relative = repo_relative(todo_path, repo).ok_or("TODO path is not within repository")?;

    if let Err(e) = git_ops.add_file_to_index(repo, &relative) {
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
            Arg::new("regenerate")
                .long("regenerate")
                .help("Re-scan all tracked files and rewrite TODO.md from scratch. Wipes any existing content (including conflict markers).")
                .action(ArgAction::SetTrue),
        )
}
