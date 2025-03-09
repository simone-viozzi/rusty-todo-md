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
