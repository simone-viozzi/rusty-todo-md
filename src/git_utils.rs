use git2::{
    Delta, DiffOptions, Error as GitError, ObjectType, Repository, TreeWalkMode, TreeWalkResult,
};
use log::info;
use std::path::{Path, PathBuf};

/// Opens the Git repository at the specified path.
/// Returns an error if the specified path is not a Git repository.
pub fn open_repository(repo_path: &Path) -> Result<Repository, GitError> {
    Repository::open(repo_path)
}

/// Retrieves the list of staged files that contain meaningful content changes.
/// Uses DiffOptions to optimize for the intended use case, ignoring irrelevant changes.
pub fn get_staged_files(repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
    let mut diff_opts = DiffOptions::new();
    diff_opts
        .ignore_whitespace(true)
        .ignore_whitespace_change(true)
        .ignore_whitespace_eol(true)
        .include_untracked(false)
        .force_text(true)
        .skip_binary_check(true);

    // Create the diff between the HEAD tree and the index
    let head_tree = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

    let mut staged_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            // Only include files that have been added or modified.
            if let Some(path) = delta.new_file().path() {
                staged_files.push(path.to_path_buf());
            }
            true
        },
        None,
        None,
        None,
    )?;

    info!("Found {} staged files", staged_files.len());
    Ok(staged_files)
}

/// Retrieves all files that are currently tracked by Git by walking the HEAD tree.
/// This function ignores directories (like the .git folder) and returns file paths relative to the repo root.
pub fn get_tracked_files(repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
    let head_tree = repo.head()?.peel_to_tree()?;
    let mut tracked_files = Vec::new();

    head_tree.walk(TreeWalkMode::PreOrder, |root, entry| {
        if entry.kind() == Some(ObjectType::Blob) {
            let path = if root.is_empty() {
                entry.name().unwrap_or("").into()
            } else {
                format!("{}/{}", root, entry.name().unwrap_or(""))
            };
            tracked_files.push(PathBuf::from(path));
        }
        TreeWalkResult::Ok
    })?;

    info!("Found {} tracked files", tracked_files.len());
    Ok(tracked_files)
}

/// Retrieves the list of staged files that have been deleted.
/// It creates a diff between the HEAD tree and the index, then collects files where the diff
/// status indicates deletion.
pub fn get_deleted_files(repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
    let mut diff_opts = DiffOptions::new();
    diff_opts.include_untracked(false).skip_binary_check(true);

    let head_tree = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

    let mut deleted_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if delta.status() == Delta::Deleted {
                if let Some(path) = delta.old_file().path() {
                    deleted_files.push(path.to_path_buf());
                }
            }
            true
        },
        None,
        None,
        None,
    )?;

    info!("Found {} deleted files", deleted_files.len());
    Ok(deleted_files)
}
