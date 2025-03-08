use git2::{Repository, Signature};
use log::{debug, error, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Struct to manage a temporary Git repository for testing.
pub struct TempGitRepo {
    /// Keep the temp directory alive, preventing it from being deleted until this struct is dropped.
    _temp_dir: TempDir, // renamed from temp_dir to _temp_dir
    pub repo: Repository,
    pub repo_path: PathBuf,
}

impl Default for TempGitRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl TempGitRepo {
    /// Creates a new temporary Git repository.
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        let repo = Repository::init(&repo_path).expect("Failed to initialize Git repository");

        info!("Initialized new Git repository at {:?}", repo_path);

        let temp_repo = TempGitRepo {
            _temp_dir: temp_dir,
            repo,
            repo_path,
        };

        temp_repo.commit("Initial commit");

        temp_repo
    }

    /// Creates and writes content to a file in the repo.
    pub fn create_file(&self, file_name: &str, content: &str) {
        let file_path = self.repo_path.join(file_name);
        fs::write(&file_path, content).expect("Failed to write file");
        debug!("Created file: {:?}", file_path);
    }

    /// Stages a file for commit.
    pub fn stage_file(&self, file_name: &str) {
        let mut index = self.repo.index().unwrap();
        info!("Staging file: {}", file_name);
        if let Err(e) = index.add_path(Path::new(file_name)) {
            error!("Failed to stage file: {}: {:?}", file_name, e);
        }
        if let Err(e) = index.write() {
            error!("Failed to write index: {:?}", e);
        }
    }

    /// Commits staged files with a message.
    /// Commits staged files with a message.
    pub fn commit(&self, message: &str) {
        let mut index = self.repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test User", "test@example.com").unwrap();

        // Use "refs/heads/master" as the update reference to ensure HEAD is updated correctly.
        let commit_result = match self.repo.head() {
            Ok(head) => {
                if let Ok(parent_commit) = head.peel_to_commit() {
                    info!("Committing with parent commit");
                    self.repo.commit(
                        Some("refs/heads/master"),
                        &sig,
                        &sig,
                        message,
                        &tree,
                        &[&parent_commit],
                    )
                } else {
                    warn!("HEAD exists but no parent commit found");
                    self.repo
                        .commit(Some("refs/heads/master"), &sig, &sig, message, &tree, &[])
                }
            }
            Err(_) => {
                warn!("No HEAD found, committing without parent commit");
                self.repo
                    .commit(Some("refs/heads/master"), &sig, &sig, message, &tree, &[])
            }
        };

        commit_result.unwrap();
        info!("Committed with message: {}", message);
    }
}
