use std::path::PathBuf;

use rusty_todo_md::cli::run_workflow;
use rusty_todo_md::git_utils::get_staged_files;
use std::fs;
mod utils;
use rusty_todo_md::git_utils;
use utils::TempGitRepo;

#[test]
fn test_get_staged_files() {
    // Set up a temporary Git repository
    let temp_repo = TempGitRepo::new();

    // Create and stage test files
    temp_repo.create_file("file1.txt", "TODO: Implement feature A");
    temp_repo.stage_file("file1.txt");

    temp_repo.create_file("file2.txt", "This is a test file");
    temp_repo.stage_file("file2.txt");

    // Call `get_staged_files` to retrieve staged files
    let staged_files = get_staged_files(&temp_repo.repo).expect("Failed to retrieve staged files");

    // Verify that the staged files match the expected files
    let expected_files: Vec<_> = vec!["file1.txt", "file2.txt"]
        .into_iter()
        .map(PathBuf::from)
        .collect();

    assert_eq!(staged_files, expected_files);
}


#[test]
fn test_run_workflow() {
    // Set up a temporary Git repository
    let temp_repo = TempGitRepo::new();
    let todo_path = temp_repo.repo_path.join("TODO.md");

    // Create and stage a file with a TODO comment
    temp_repo.create_file("file1.rs", "// TODO: Refactor this function");
    temp_repo.stage_file("file1.rs");

    // Debug: Verify the index state
    let staged_files =
        git_utils::get_staged_files(&temp_repo.repo).expect("Failed to retrieve staged files");
    println!("Debug: Staged files from index: {:?}", staged_files);

    // Run the workflow
    let result = run_workflow(&todo_path, &temp_repo.repo_path);

    assert!(result.is_ok(), "Workflow failed with error: {:?}", result);

    // Verify TODO.md exists
    assert!(todo_path.exists(), "TODO.md should have been created");

    // Verify TODO.md was updated
    let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
    println!("Debug: TODO.md content:\n{}", content);
    assert!(content.contains("file1.rs"), "Expected file1.rs in TODO.md");
    assert!(
        content.contains("Refactor this function"),
        "Expected TODO comment in TODO.md"
    );
}
