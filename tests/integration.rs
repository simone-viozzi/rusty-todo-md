use git2::{Repository, Signature};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

use rusty_todo_md::git_utils::get_staged_files;

#[test]
fn test_get_staged_files() {
    // Create a temporary directory for the Git repository
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    // Initialize a new Git repository
    let repo = Repository::init(repo_path).expect("Failed to initialize Git repo");

    // Create an initial commit to set up HEAD
    let mut index = repo.index().expect("Failed to get Git index");
    let oid = index.write_tree().expect("Failed to write tree");
    let signature =
        Signature::now("Test User", "test@example.com").expect("Failed to create signature");
    let tree = repo.find_tree(oid).expect("Failed to find tree");
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .expect("Failed to create initial commit");

    // Create and stage test files
    let file1_path = repo_path.join("file1.txt");
    let mut file1 = File::create(&file1_path).expect("Failed to create file1.txt");
    writeln!(file1, "TODO: Implement feature A").expect("Failed to write to file1.txt");

    let file2_path = repo_path.join("file2.txt");
    let mut file2 = File::create(&file2_path).expect("Failed to create file2.txt");
    writeln!(file2, "This is a test file").expect("Failed to write to file2.txt");

    // Stage the files
    index
        .add_path(Path::new("file1.txt"))
        .expect("Failed to stage file1.txt");
    index
        .add_path(Path::new("file2.txt"))
        .expect("Failed to stage file2.txt");
    index.write().expect("Failed to write index");

    // Call `get_staged_files` to retrieve staged files
    let staged_files = get_staged_files(&repo).expect("Failed to retrieve staged files");

    // Verify that the staged files match the expected files
    let expected_files: Vec<_> = vec![file1_path, file2_path]
        .into_iter()
        .map(|p| p.strip_prefix(repo_path).unwrap().to_path_buf())
        .collect();

    assert_eq!(staged_files, expected_files);
}
