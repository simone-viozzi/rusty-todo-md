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
