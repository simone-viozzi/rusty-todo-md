use git2::{DiffOptions, Error as GitError, Repository};
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

    /// Retrieves all files git considers tracked **right now** — i.e. the
    /// current index. Equivalent to `git ls-files`.
    ///
    /// We deliberately read the index rather than walking the HEAD tree:
    /// during `git rebase` (and during a partial-commit pre-commit run) the
    /// index reflects the state being committed, while HEAD still points at
    /// the previous commit. The merge driver invoked mid-rebase needs the
    /// index view, otherwise files added by the replayed commit are missed
    /// and their TODOs silently disappear from TODO.md.
    ///
    /// During an unresolved merge, the index can hold multiple entries for
    /// the same path — one per conflict stage (1 = ancestor, 2 = ours,
    /// 3 = theirs). The working-tree file is the same on disk for all
    /// stages, so we deduplicate by path: the first entry we see per path
    /// wins (stage 0 if present, otherwise stage 1).
    fn get_tracked_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
        debug!("Retrieving all tracked files from index");
        let index = repo.index()?;
        let mut tracked_files = Vec::with_capacity(index.len());
        let mut seen = std::collections::HashSet::new();
        for entry in index.iter() {
            // index path bytes are git's internal encoding; on unix this is
            // raw filesystem bytes, on windows it's UTF-8.
            let path = match std::str::from_utf8(&entry.path) {
                Ok(s) => PathBuf::from(s),
                Err(_) => {
                    debug!("Skipping index entry with non-UTF-8 path");
                    continue;
                }
            };
            if !seen.insert(path.clone()) {
                continue;
            }
            debug!("Tracked file: {path:?}");
            tracked_files.push(path);
        }
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
