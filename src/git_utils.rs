use git2::{DiffOptions, Error as GitError, ObjectType, Repository, TreeWalkMode, TreeWalkResult};
use log::{debug, info};
use std::path::{Path, PathBuf};

/// Trait that abstracts the Git operations.
pub trait GitOpsTrait {
    fn open_repository(&self, repo_path: &Path) -> Result<Repository, GitError>;
    fn get_staged_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError>;
    fn get_tracked_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError>;
    fn add_file_to_index(&self, repo: &Repository, file_path: &Path) -> Result<(), GitError>;
}

/// Real implementation that uses git2 directly.
pub struct GitOps;

impl GitOpsTrait for GitOps {
    /// Opens the Git repository at the specified path.
    /// Returns an error if the specified path is not a Git repository.
    fn open_repository(&self, repo_path: &Path) -> Result<Repository, GitError> {
        debug!("Opening repository at path: {repo_path:?}",);
        let repo = Repository::open(repo_path)?;
        info!("Successfully opened repository at path: {repo_path:?}",);
        Ok(repo)
    }

    /// Retrieves the list of staged files that contain meaningful content changes.
    /// Uses DiffOptions to optimize for the intended use case, ignoring irrelevant changes.
    fn get_staged_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
        debug!("Retrieving staged files with meaningful content changes");
        let mut diff_opts = DiffOptions::new();
        diff_opts
            .ignore_whitespace(true)
            .ignore_whitespace_change(true)
            .ignore_whitespace_eol(true)
            .include_untracked(false)
            .force_text(true)
            .skip_binary_check(true);

        let head_tree = repo.head()?.peel_to_tree()?;
        let diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

        let mut staged_files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    debug!("Staged file added/modified: {path:?}",);
                    staged_files.push(path.to_path_buf());
                }
                true
            },
            None,
            None,
            None,
        )?;
        info!(
            "Found {staged_files_len} staged files",
            staged_files_len = staged_files.len()
        );
        Ok(staged_files)
    }

    /// Retrieves all files that are currently tracked by Git by walking the HEAD tree.
    /// This function ignores directories (like the .git folder) and returns file paths relative to the repo root.
    fn get_tracked_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
        debug!("Retrieving all tracked files");
        let head_tree = repo.head()?.peel_to_tree()?;
        let mut tracked_files = Vec::new();

        head_tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() == Some(ObjectType::Blob) {
                let path = if root.is_empty() {
                    entry.name().unwrap_or("").into()
                } else {
                    format!("{}/{}", root, entry.name().unwrap_or(""))
                };
                debug!("Tracked file: {path:?}",);
                tracked_files.push(PathBuf::from(path));
            }
            TreeWalkResult::Ok
        })?;
        info!(
            "Found {tracked_files_len} tracked files",
            tracked_files_len = tracked_files.len()
        );
        Ok(tracked_files)
    }

    /// Adds a file to the Git index (stages it for commit).
    /// This is equivalent to running `git add <file_path>`.
    fn add_file_to_index(&self, repo: &Repository, file_path: &Path) -> Result<(), GitError> {
        debug!("Adding file to index: {file_path:?}");
        let mut index = repo.index()?;
        index.add_path(file_path)?;
        index.write()?;
        info!("Successfully added file to index: {file_path:?}");
        Ok(())
    }
}
