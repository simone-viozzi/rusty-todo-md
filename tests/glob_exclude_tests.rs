mod utils;

/// Integration tests for glob-based file exclusion.
///
/// Note: Most exclusion logic is tested via unit tests in src/exclusion.rs.
/// These integration tests verify end-to-end CLI behavior with the exclusion flags.
mod glob_exclude_tests {
    use crate::utils::{init_repo, FakeGitOps};
    use log::LevelFilter;
    use rusty_todo_md::cli::run_cli_with_args;
    use rusty_todo_md::logger;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Once;
    use tempfile::tempdir;

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

    /// Helper to create a file in the provided directory.
    fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Integration test verifying recursive wildcard exclusion patterns work end-to-end
    #[test]
    fn test_glob_exclude_recursive_wildcard() {
        init_logger();
        log::info!("Starting test_glob_exclude_recursive_wildcard");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create nested structure
        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main");
        let file2 = create_test_file(repo_path, "src/utils/helper.rs", "// TODO: Helper");
        let file3 = create_test_file(repo_path, "src/deep/nested/file.rs", "// TODO: Nested");
        let file4 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test");

        // Exclude everything under src/ with **
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src/**".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
            file4.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1, file2, file3, file4.clone()];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            !content.contains("src/main.rs"),
            "src/main.rs should be excluded"
        );
        assert!(
            !content.contains("src/utils/helper.rs"),
            "src/utils/helper.rs should be excluded"
        );
        assert!(
            !content.contains("src/deep/nested/file.rs"),
            "nested file should be excluded"
        );
        assert!(
            content.contains("tests/test.rs"),
            "tests/test.rs should be included"
        );
    }

    /// Integration test verifying multiple exclusion patterns work correctly
    #[test]
    fn test_glob_multiple_exclude_patterns() {
        init_logger();
        log::info!("Starting test_glob_multiple_exclude_patterns");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main");
        let file2 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test");
        let file3 = create_test_file(repo_path, "docs/guide.rs", "// TODO: Doc");
        let file4 = create_test_file(repo_path, "lib.rs", "// TODO: Lib");

        // Multiple exclusions
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src/".to_string(),
            "--exclude".to_string(),
            "tests/".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
            file4.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1, file2, file3.clone(), file4.clone()];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(!content.contains("src/main.rs"), "src/ should be excluded");
        assert!(
            !content.contains("tests/test.rs"),
            "tests/ should be excluded"
        );
        assert!(
            content.contains("docs/guide.rs"),
            "docs/ should be included"
        );
        assert!(content.contains("lib.rs"), "lib.rs should be included");
    }
}
