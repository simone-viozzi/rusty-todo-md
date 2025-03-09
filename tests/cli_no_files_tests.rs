use assert_cmd::Command;
use log::LevelFilter;
mod utils;
use utils::init_repo;

use rusty_todo_md::logger;

use std::sync::Once;

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
fn test_run_cli_no_files() {
    init_logger();

    let (temp_dir, _repo) = init_repo().expect("Failed to initialize test repo");
    let repo_dir = temp_dir.path();

    // Run the CLI binary from the temporary directory with only the TODO file specified.
    // Since no files are provided, it should log "No files provided, nothing to do." and exit with code 0.
    let mut cmd = Command::cargo_bin("rusty-todo-md").expect("binary exists");
    cmd.current_dir(repo_dir).arg("--todo-path").arg("TODO.md"); // no file arguments

    // Because the CLI calls std::process::exit(0) in this branch,
    // assert_cmd will capture the exit code and output.
    cmd.assert().code(0);
}
