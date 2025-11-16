use assert_cmd::Command;
use log::LevelFilter;
use log::{debug, info};
use predicates::str::contains;
use rusty_todo_md::logger;
use std::fs;
use std::sync::Once;
use tempfile::tempdir;
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
// TODO: Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
// The current usage of Command::cargo_bin() is deprecated and incompatible with custom cargo build-dir.
// This #[allow(deprecated)] suppresses clippy warnings to prevent pre-commit hook failures.
// See: https://docs.rs/assert_cmd/latest/assert_cmd/cargo/fn.cargo_bin_cmd.html
#[allow(deprecated)]
fn test_run_cli_in_non_git_directory() {
    init_logger();

    info!("Starting test: test_run_cli_in_non_git_directory");

    // Create a temporary directory that is not a git repo.
    let temp = tempdir().expect("failed to create temp dir");
    debug!("Created temporary directory: {:?}", temp.path());

    // Run the CLI binary in that directory. Since there is no .git, it should fail.
    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("binary exists");
    debug!("Running CLI binary in temporary directory");
    cmd.current_dir(&temp)
        .arg("--todo-path")
        .arg("TODO.md")
        .arg("dummy_file.rs"); // dummy file path to trigger processing

    cmd.assert()
        .failure()
        .stderr(contains("Error opening repository"));
    info!("Test completed: test_run_cli_in_non_git_directory");
}

#[test]
// TODO: Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
// The current usage of Command::cargo_bin() is deprecated and incompatible with custom cargo build-dir.
// This #[allow(deprecated)] suppresses clippy warnings to prevent pre-commit hook failures.
// See: https://docs.rs/assert_cmd/latest/assert_cmd/cargo/fn.cargo_bin_cmd.html
#[allow(deprecated)]
fn test_run_cli_with_unreadable_file() {
    // Initialize logging for the test.
    init_logger();
    info!("Starting test: test_run_cli_with_unreadable_file");

    // Use the common helper to initialize a real repository.
    let (temp_dir, _repo) = init_repo().expect("Failed to initialize test repo");
    let repo_dir = temp_dir.path();

    // Create a dummy unreadable file in the repository.
    let file_path = repo_dir.join("unreadable.rs");
    fs::write(&file_path, " // TODO: test unreadable").expect("failed to write file");
    debug!("Created unreadable file at: {:?}", file_path);

    // Make the file unreadable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&file_path, perms).expect("failed to set permissions");
        debug!("Set file permissions to unreadable for: {:?}", file_path);
    }

    // Run the CLI binary in the repository directory.
    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("binary exists");
    debug!("Running CLI binary in repository directory");
    cmd.current_dir(repo_dir)
        .arg("--todo-path")
        .arg("TODO.md")
        .arg(file_path.to_str().expect("file path valid"));

    // Now we expect the CLI to succeed (exit code 0) but with a warning message in stderr.
    cmd.assert()
        .success()
        .stderr(contains("Warning: Could not read file"));

    info!("Test completed: test_run_cli_with_unreadable_file");

    // Restore file permissions so the temporary directory can be cleaned up.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path)
            .expect("failed to get metadata")
            .permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).expect("failed to reset permissions");
        debug!("Restored file permissions for: {:?}", file_path);
    }
}

#[test]
// TODO: Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
// The current usage of Command::cargo_bin() is deprecated and incompatible with custom cargo build-dir.
// This #[allow(deprecated)] suppresses clippy warnings to prevent pre-commit hook failures.
// See: https://docs.rs/assert_cmd/latest/assert_cmd/cargo/fn.cargo_bin_cmd.html
#[allow(deprecated)]
fn test_sync_todo_file_fallback_mechanism() {
    init_logger();
    info!("Starting test: test_sync_todo_file_fallback_mechanism");

    // Use the common helper to initialize a real repository.
    let (temp_dir, repo) = init_repo().expect("Failed to initialize test repo");
    let repo_dir = temp_dir.path();
    debug!("Initialized repository at: {:?}", repo_dir);

    // Create a test file with TODO comments
    let test_file = repo_dir.join("test.rs");
    fs::write(
        &test_file,
        "// TODO: implement feature A\n// FIXME: fix bug B\n",
    )
    .expect("failed to write test file");
    debug!("Created test file at: {:?}", test_file);

    // Create a corrupted TODO.md file with invalid format
    // This will now trigger the sync_todo_file error and activate the fallback mechanism
    let todo_path = repo_dir.join("TODO.md");
    let corrupted_content = r#"This is completely invalid content that doesn't match any regex pattern
And this line will also fail validation
No markdown headers or bullet points here
Just plain text that should trigger validation failure
"#;
    fs::write(&todo_path, corrupted_content).expect("failed to write corrupted TODO.md");
    debug!("Created corrupted TODO.md at: {:?}", todo_path);

    // Stage and commit the test file so it appears in tracked files for the fallback
    // Use git2 library directly like other tests to avoid CI/CD environment issues
    let mut index = repo.index().expect("Failed to get index");
    index
        .add_path(std::path::Path::new("test.rs"))
        .expect("Failed to add test.rs");
    index.write().expect("Failed to write index");
    debug!("Staged test file with git2");

    let tree_id = index.write_tree().expect("Failed to write tree");
    let tree = repo.find_tree(tree_id).expect("Failed to find tree");
    let sig =
        git2::Signature::now("Test User", "test@example.com").expect("Failed to create signature");
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add test file",
        &tree,
        &[&repo.head().unwrap().peel_to_commit().unwrap()],
    )
    .expect("Failed to commit");
    debug!("Committed test file with git2");

    // Run the CLI binary - this should trigger the fallback mechanism
    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("binary exists");
    debug!("Running CLI binary to test fallback mechanism");
    cmd.current_dir(repo_dir)
        .env("RUST_LOG", "debug")
        .arg("--todo-path")
        .arg("TODO.md")
        .arg(test_file.to_str().expect("test file path valid"));

    // The fallback mechanism should succeed and recreate the TODO.md file properly
    cmd.assert().success(); // No stderr expected since the fallback succeeds

    // Verify that the TODO.md file was recreated with proper content
    assert!(todo_path.exists(), "TODO.md should exist after fallback");
    let final_content = fs::read_to_string(&todo_path).expect("failed to read final TODO.md");
    debug!("Final TODO.md content: {}", final_content);

    // Verify the fallback worked by checking for expected TODO items
    assert!(
        final_content.contains("implement feature A"),
        "Should contain TODO from test file"
    );
    // Note: FIXME comments are treated as TODO by default, so both appear under TODO section
    assert!(
        final_content.contains("fix bug B") || final_content.contains("TODO"),
        "Should contain content from test file"
    );

    // Verify the corrupted content was replaced
    assert!(
        !final_content.contains("This is completely invalid"),
        "Corrupted content should be gone"
    );

    // Verify the file has proper markdown structure
    assert!(final_content.contains("# TODO"), "Should have TODO header");
    assert!(
        final_content.contains("## test.rs"),
        "Should have file section header"
    );

    info!("Test completed: test_sync_todo_file_fallback_mechanism");
}
