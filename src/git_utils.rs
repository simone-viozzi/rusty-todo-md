use git2::{DiffOptions, Error as GitError, Repository};
use log::info;
use std::path::{Path, PathBuf};

/// Opens the Git repository at the specified path.
/// Returns an error if the specified path is not a Git repository.
pub fn open_repository(repo_path: &Path) -> Result<Repository, GitError> {
    Repository::open(repo_path)
}

/// Retrieves the list of staged files that contain meaningful content changes.
/// Uses DiffOptions to optimize for the intended use case, ignoring irrelevant files and changes.
pub fn get_staged_files(repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
    let mut diff_opts = DiffOptions::new();

    diff_opts
        .ignore_whitespace(true) // Ignore all whitespace differences
        .ignore_whitespace_change(true) // Ignore changes in the amount of whitespace
        .ignore_whitespace_eol(true) // Ignore trailing whitespace changes
        .include_untracked(false) // Focus only on staged changes
        .force_text(true) // Treat all files as text
        .skip_binary_check(true); // Skip binary file checks for efficiency

    // Create the diff between the HEAD tree and the index
    let diff = repo.diff_tree_to_index(
        Some(&repo.head()?.peel_to_tree()?),
        None,
        Some(&mut diff_opts),
    )?;

    // Collect staged files with meaningful changes
    let mut staged_files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            // Only include files that have been added or modified (no deletions or renames)
            if let Some(path) = delta.new_file().path() {
                staged_files.push(path.to_path_buf());
            }
            true
        },
        None,
        None,
        None,
    )?;

    info!("found {} staged files", staged_files.len());

    Ok(staged_files)
}
