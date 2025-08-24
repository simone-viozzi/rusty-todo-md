use git2::IndexAddOption;
use git2::{Error as GitError, Repository, Signature};
use log::debug;
use log::info;

use std::fs::File;

use std::io::Write;

use tempfile::TempDir;

/// Initializes a temporary repository exactly like a real repository would be:
/// - Calls `git init`
/// - Writes the HEAD file to point to "refs/heads/master"
/// - Creates an initial file, stages it, writes the tree, and commits
///
/// The result is a repository with HEAD as a symbolic ref to "refs/heads/master".
#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct FakeGitOps {
    pub _dummy_repo: Repository,
    pub temp_dir: tempfile::TempDir,
    pub staged_files: Vec<std::path::PathBuf>,
    pub tracked_files: Vec<std::path::PathBuf>,
}

#[allow(dead_code)]
impl FakeGitOps {
    pub fn new(
        _dummy_repo: Repository,
        temp_dir: tempfile::TempDir,
        staged_files: Vec<std::path::PathBuf>,
        tracked_files: Vec<std::path::PathBuf>,
    ) -> Self {
        FakeGitOps {
            _dummy_repo,
            temp_dir,
            staged_files,
            tracked_files,
        }
    }
}

impl rusty_todo_md::git_utils::GitOpsTrait for FakeGitOps {
    fn open_repository(&self, _repo_path: &std::path::Path) -> Result<Repository, GitError> {
        Repository::open(self.temp_dir.path())
    }
    fn get_staged_files(&self, _repo: &Repository) -> Result<Vec<std::path::PathBuf>, GitError> {
        Ok(self.staged_files.clone())
    }
    fn get_tracked_files(&self, _repo: &Repository) -> Result<Vec<std::path::PathBuf>, GitError> {
        Ok(self.tracked_files.clone())
    }
    fn add_file_to_index(
        &self,
        repo: &Repository,
        file_path: &std::path::Path,
    ) -> Result<(), GitError> {
        // For testing, actually add the file to the index like the real implementation
        let mut index = repo.index()?;
        index.add_path(file_path)?;
        index.write()?;
        Ok(())
    }
}
