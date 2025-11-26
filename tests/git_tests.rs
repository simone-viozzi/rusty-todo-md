use log::info;
use log::LevelFilter;
use rusty_todo_md::git_utils::{GitOps, GitOpsTrait};
use rusty_todo_md::logger;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Once;
mod utils;
use utils::init_repo;

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

/// Tests that files in nested directories don't have double slashes in their paths.
/// This is a regression test for the issue where libgit2's tree walk callback
/// provides directory paths ending with '/', which when concatenated with '/'
/// would result in paths like "app//file.py" instead of "app/file.py".
#[test]
fn test_get_tracked_files_nested_directories() {
    init_logger();
    info!("Starting test_get_tracked_files_nested_directories");
    let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
    let repo = git2::Repository::init(temp_dir.path()).expect("failed to init repo");

    // Create HEAD file
    let head_path = temp_dir.path().join(".git").join("HEAD");
    std::fs::write(&head_path, "ref: refs/heads/master\n").expect("failed to write HEAD file");

    // Create nested directory structure
    let nested_dir = temp_dir.path().join("app").join("src").join("components");
    std::fs::create_dir_all(&nested_dir).expect("failed to create nested directories");

    // Create files in nested directories
    let file1 = temp_dir.path().join("app").join("file.py");
    File::create(&file1)
        .unwrap()
        .write_all(b"# test file")
        .unwrap();

    let file2 = nested_dir.join("Button.tsx");
    File::create(&file2)
        .unwrap()
        .write_all(b"// test file")
        .unwrap();

    // Stage and commit the files
    let mut index = repo.index().unwrap();
    index
        .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .unwrap();

    // Get tracked files and verify no double slashes
    let tracked = GitOps.get_tracked_files(&repo).unwrap();

    // Verify the files are tracked with correct paths (no double slashes)
    assert!(
        tracked.contains(&PathBuf::from("app/file.py")),
        "Expected 'app/file.py' in tracked files, got: {:?}",
        tracked
    );
    assert!(
        tracked.contains(&PathBuf::from("app/src/components/Button.tsx")),
        "Expected 'app/src/components/Button.tsx' in tracked files, got: {:?}",
        tracked
    );

    // Verify no path contains double slashes
    for path in &tracked {
        let path_str = path.to_string_lossy();
        assert!(
            !path_str.contains("//"),
            "Path '{}' contains double slashes",
            path_str
        );
    }

    info!("Completed test_get_tracked_files_nested_directories");
}
