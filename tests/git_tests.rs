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
    // Verify nested directory file is tracked with correct path (no double slashes)
    assert!(
        tracked.contains(&PathBuf::from("app/src/nested.txt")),
        "Expected 'app/src/nested.txt' in tracked files, got: {:?}",
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
    info!("Completed test_get_tracked_files");
}

/// During an unresolved merge, the index holds the same path at multiple
/// stages (1 = ancestor, 2 = ours, 3 = theirs). We must return each path
/// exactly once — otherwise the merge driver re-scans the same source file
/// up to three times and emits duplicate warnings.
#[test]
fn test_get_tracked_files_deduplicates_conflict_stages() {
    init_logger();
    let (_temp_dir, repo) = init_repo().unwrap();

    // Forge an index with three conflict-stage entries for one path. This
    // mirrors the state git leaves the index in mid-merge.
    let mut index = repo.index().unwrap();
    let oid = repo.blob(b"placeholder").unwrap();
    for stage in 1..=3u16 {
        let entry = git2::IndexEntry {
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: 11,
            id: oid,
            // bits 12-13 of flags encode the stage.
            flags: stage << 12,
            flags_extended: 0,
            path: b"conflicted.txt".to_vec(),
        };
        index.add(&entry).unwrap();
    }

    let tracked = GitOps.get_tracked_files(&repo).unwrap();
    let occurrences = tracked
        .iter()
        .filter(|p| p == &&PathBuf::from("conflicted.txt"))
        .count();
    assert_eq!(
        occurrences, 1,
        "expected conflicted.txt once, got {occurrences} (tracked: {tracked:?})"
    );
}

/// `get_tracked_files` must reflect the current index, not the HEAD tree:
/// during a rebase/merge-driver invocation files added in the replayed commit
/// are in the index but not yet in HEAD.
#[test]
fn test_get_tracked_files_includes_staged_but_uncommitted() {
    init_logger();
    info!("Starting test_get_tracked_files_includes_staged_but_uncommitted");
    let (temp_dir, repo) = init_repo().unwrap();

    // Add a brand-new file and stage it, but do NOT commit.
    let new_file = temp_dir.path().join("freshly_staged.rs");
    std::fs::write(&new_file, "// TODO: new\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("freshly_staged.rs")).unwrap();
    index.write().unwrap();

    let tracked = GitOps.get_tracked_files(&repo).unwrap();
    assert!(
        tracked.contains(&PathBuf::from("freshly_staged.rs")),
        "expected staged-but-uncommitted file in tracked list, got: {tracked:?}"
    );
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
