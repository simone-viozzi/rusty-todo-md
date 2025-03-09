use git2::{
    Delta, DiffOptions, Error as GitError, ObjectType, Repository, TreeWalkMode, TreeWalkResult,
};
use log::{debug, info};
use std::path::{Path, PathBuf};

/// Trait that abstracts the Git operations.
pub trait GitOpsTrait {
    fn open_repository(&self, repo_path: &Path) -> Result<Repository, GitError>;
    fn get_staged_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError>;
    fn get_tracked_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError>;
    fn get_deleted_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError>;
}

/// Real implementation that uses git2 directly.
pub struct GitOps;

impl GitOpsTrait for GitOps {
    /// Opens the Git repository at the specified path.
    /// Returns an error if the specified path is not a Git repository.
    fn open_repository(&self, repo_path: &Path) -> Result<Repository, GitError> {
        debug!("Opening repository at path: {:?}", repo_path);
        let repo = Repository::open(repo_path)?;
        info!("Successfully opened repository at path: {:?}", repo_path);
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
                    debug!("Staged file added/modified: {:?}", path);
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
                debug!("Tracked file: {:?}", path);
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
    fn get_deleted_files(&self, repo: &Repository) -> Result<Vec<PathBuf>, GitError> {
        debug!("Retrieving staged files that have been deleted");
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(false).skip_binary_check(true);

        let head_tree = repo.head()?.peel_to_tree()?;
        let diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

        let mut deleted_files = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if delta.status() == Delta::Deleted {
                    if let Some(path) = delta.old_file().path() {
                        debug!("Deleted file: {:?}", path);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger;
    use git2::{IndexAddOption, Repository, Signature};
    use log::LevelFilter;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Once;
    use tempfile::TempDir;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .format(logger::format_logger)
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });
    }

    /// Initializes a temporary repository exactly like a real repository would be:
    /// - Calls `git init`
    /// - Writes the HEAD file to point to "refs/heads/master"
    /// - Creates an initial file, stages it, writes the tree, and commits
    ///
    /// The result is a repository with HEAD as a symbolic ref to "refs/heads/master".
    pub fn init_repo() -> Result<(TempDir, Repository), GitError> {
        // 1. Create a temporary directory and initialize the repository.
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let repo = Repository::init(temp_dir.path())?;
        debug!("Initialized repository at {:?}", temp_dir.path());

        // 2. Mimic 'git init': create the HEAD file with the symbolic ref.
        let head_path = temp_dir.path().join(".git").join("HEAD");
        std::fs::write(&head_path, "ref: refs/heads/master\n").expect("failed to write HEAD file");
        debug!("Set HEAD to 'refs/heads/master'");

        // 3. Create an initial file.
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = File::create(&file_path).expect("failed to create initial file");
            writeln!(file, "initial content").expect("failed to write to initial file");
        }
        debug!("Created initial file: {:?}", file_path);

        // 4. Stage the file (simulate `git add .`).
        let mut index = repo.index()?;
        index.add_all(["."].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        debug!("Staged files via index.add_all");

        // 5. Write the index to a tree.
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        debug!("Wrote tree with id: {}", tree_id);

        // 6. Create the initial commit.
        // Passing Some("HEAD") tells git2 to update the ref that HEAD points to (i.e. "refs/heads/master")
        let sig = Signature::now("Test User", "test@example.com")?;
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])?;
        debug!("Created initial commit: {}", commit_oid);

        // 7. Verify HEAD: it should now be a symbolic ref to "refs/heads/master"
        let head_ref = repo.head()?;
        debug!("Final HEAD is: {:?}", head_ref.name().unwrap_or("unknown"));

        info!("Repository initialized with HEAD pointing to 'refs/heads/master'");
        drop(tree);
        drop(head_ref);
        Ok((temp_dir, repo))
    }

    #[test]
    fn test_get_tracked_files() {
        init_logger();
        info!("Starting test_get_tracked_files");
        let (_temp_dir, repo) = init_repo().unwrap();
        let tracked = GitOps.get_tracked_files(&repo).unwrap();
        assert!(tracked.contains(&PathBuf::from("test.txt")));
        info!("Completed test_get_tracked_files");
    }

    #[test]
    fn test_get_staged_files() {
        init_logger();
        info!("Starting test_get_staged_files");
        let (temp_dir, repo) = init_repo().unwrap();

        // Modify the file to simulate staged changes.
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "modified content").unwrap();
        }

        // Stage the modified file.
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let staged = GitOps.get_staged_files(&repo).unwrap();
        assert!(staged.contains(&PathBuf::from("test.txt")));
        info!("Completed test_get_staged_files");
    }

    #[test]
    fn test_get_deleted_files() {
        init_logger();
        info!("Starting test_get_deleted_files");
        let (temp_dir, repo) = init_repo().unwrap();

        // Delete the file and stage the deletion.
        let file_path = temp_dir.path().join("test.txt");
        fs::remove_file(&file_path).unwrap();

        // Update the index to reflect the deletion.
        let mut index = repo.index().unwrap();
        index.remove_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let deleted = GitOps.get_deleted_files(&repo).unwrap();
        assert!(deleted.contains(&PathBuf::from("test.txt")));
        info!("Completed test_get_deleted_files");
    }
}
