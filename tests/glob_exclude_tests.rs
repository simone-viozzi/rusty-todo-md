mod utils;

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

    #[test]
    fn test_glob_exclude_wildcard_extension() {
        init_logger();
        log::info!("Starting test_glob_exclude_wildcard_extension");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create test files with different extensions
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: Rust file");
        let file2 = create_test_file(repo_path, "file2.log", "// TODO: Log file");
        let file3 = create_test_file(repo_path, "file3.rs", "// TODO: Another rust file");
        log::debug!("Created test files: {:?}, {:?}, {:?}", file1, file2, file3);

        // Build CLI arguments excluding all .log files
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "*.log".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone(), file3.clone()];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(content.contains("file1.rs"), "file1.rs should be included");
        assert!(content.contains("file3.rs"), "file3.rs should be included");
        assert!(
            !content.contains("file2.log"),
            "file2.log should be excluded"
        );
    }

    #[test]
    fn test_glob_exclude_directory_with_slash() {
        init_logger();
        log::info!("Starting test_glob_exclude_directory_with_slash");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create files in different directories
        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main file");
        let file2 = create_test_file(repo_path, "src/lib.rs", "// TODO: Library file");
        let file3 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test file");

        // Exclude src/ directory (directory-only pattern)
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src/".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone(), file3.clone()];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            !content.contains("src/main.rs"),
            "src/main.rs should be excluded"
        );
        assert!(
            !content.contains("src/lib.rs"),
            "src/lib.rs should be excluded"
        );
        assert!(
            content.contains("tests/test.rs"),
            "tests/test.rs should be included"
        );
    }

    #[test]
    fn test_glob_exclude_dir_flag() {
        init_logger();
        log::info!("Starting test_glob_exclude_dir_flag");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create files
        let file1 = create_test_file(repo_path, "build/output.rs", "// TODO: Build output");
        let file2 = create_test_file(repo_path, "src/main.rs", "// TODO: Main file");

        // Use --exclude-dir flag
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude-dir".to_string(),
            "build".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone()];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            !content.contains("build/output.rs"),
            "build/ should be excluded"
        );
        assert!(
            content.contains("src/main.rs"),
            "src/main.rs should be included"
        );
    }

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

    #[test]
    fn test_glob_no_exclude_processes_all() {
        init_logger();
        log::info!("Starting test_glob_no_exclude_processes_all");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: File 1");
        let file2 = create_test_file(repo_path, "file2.rs", "// TODO: File 2");

        // No exclusion
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];

        let (temp_dir_git, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1, file2];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir_git, staged_files, vec![]);

        run_cli_with_args(args, &fake_git_ops);

        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(content.contains("file1.rs"), "file1.rs should be included");
        assert!(content.contains("file2.rs"), "file2.rs should be included");
    }
}
