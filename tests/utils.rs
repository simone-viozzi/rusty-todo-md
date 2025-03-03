use git2::{Repository, Signature};
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

impl TempGitRepo {
    /// Creates a new temporary Git repository.
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        let repo = Repository::init(&repo_path).expect("Failed to initialize Git repository");

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
    }

    /// Stages a file for commit.
    pub fn stage_file(&self, file_name: &str) {
        let mut index = self.repo.index().unwrap();
        println!("Staging file: {}", file_name);
        index.add_path(Path::new(file_name)).unwrap();
        index.write().unwrap();
    }

    /// Commits staged files with a message.
    pub fn commit(&self, message: &str) {
        let mut index = self.repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test User", "test@example.com").unwrap();

        // Check if there's a HEAD commit. If yes, use it as the parent.
        let commit_result = match self.repo.head() {
            Ok(head) => {
                if let Ok(parent_commit) = head.peel_to_commit() {
                    self.repo
                        .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit])
                } else {
                    self.repo
                        .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                }
            }
            Err(_) => self
                .repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[]),
        };

        commit_result.unwrap();
    }
}
