mod utils;

mod exclude_tests {
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
    fn test_exclude_single_file() {
        init_logger();
        log::info!("Starting test_exclude_single_file");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create two test files with TODO comments
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: This should be included");
        let file2 = create_test_file(repo_path, "file2.rs", "// TODO: This should be excluded");
        log::debug!("Created test files: {:?}, {:?}", file1, file2);

        // Build CLI arguments excluding file2.rs
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "file2.rs".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone()];
        let tracked_files = vec![];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md only contains file1, not file2
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            content.contains("file1.rs"),
            "Expected TODO entry for file1.rs"
        );
        assert!(
            content.contains("This should be included"),
            "Expected TODO message from file1.rs"
        );
        assert!(!content.contains("file2.rs"), "file2.rs should be excluded");
        assert!(
            !content.contains("This should be excluded"),
            "TODO from file2.rs should not appear"
        );
    }

    #[test]
    fn test_exclude_directory() {
        init_logger();
        log::info!("Starting test_exclude_directory");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create files in different directories
        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main file");
        let file2 = create_test_file(repo_path, "src/lib.rs", "// TODO: Library file");
        let file3 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test file");
        log::debug!("Created test files: {:?}, {:?}, {:?}", file1, file2, file3);

        // Build CLI arguments excluding the src directory
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone(), file3.clone()];
        let tracked_files = vec![];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md only contains the test file, not src files
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
        assert!(
            content.contains("Test file"),
            "TODO from test file should appear"
        );
    }

    #[test]
    fn test_exclude_multiple_patterns() {
        init_logger();
        log::info!("Starting test_exclude_multiple_patterns");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create multiple test files
        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main file");
        let file2 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test file");
        let file3 = create_test_file(repo_path, "docs/readme.md", "<!-- TODO: Documentation -->");
        let file4 = create_test_file(repo_path, "lib.rs", "// TODO: Root lib");
        log::debug!(
            "Created test files: {:?}, {:?}, {:?}, {:?}",
            file1,
            file2,
            file3,
            file4
        );

        // Build CLI arguments excluding both src and tests directories
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src".to_string(),
            "--exclude".to_string(),
            "tests".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
            file3.to_str().unwrap().to_string(),
            file4.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone(), file3.clone(), file4.clone()];
        let tracked_files = vec![];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md only contains docs and root files
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            !content.contains("src/main.rs"),
            "src/main.rs should be excluded"
        );
        assert!(
            !content.contains("tests/test.rs"),
            "tests/test.rs should be excluded"
        );
        assert!(
            content.contains("docs/readme.md"),
            "docs/readme.md should be included"
        );
        assert!(content.contains("lib.rs"), "lib.rs should be included");
    }

    #[test]
    fn test_exclude_with_relative_path() {
        init_logger();
        log::info!("Starting test_exclude_with_relative_path");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create nested directory structure
        let file1 = create_test_file(
            repo_path,
            "src/utils/helper.rs",
            "// TODO: Helper utilities",
        );
        let file2 = create_test_file(repo_path, "src/main.rs", "// TODO: Main application");
        log::debug!("Created test files: {:?}, {:?}", file1, file2);

        // Build CLI arguments excluding src/utils directory with relative path
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src/utils".to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone()];
        let tracked_files = vec![];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md only contains src/main.rs, not src/utils/helper.rs
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(
            !content.contains("src/utils/helper.rs"),
            "src/utils/helper.rs should be excluded"
        );
        assert!(
            content.contains("src/main.rs"),
            "src/main.rs should be included"
        );
    }

    #[test]
    fn test_no_exclude_processes_all_files() {
        init_logger();
        log::info!("Starting test_no_exclude_processes_all_files");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create test files
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: File 1");
        let file2 = create_test_file(repo_path, "file2.rs", "// TODO: File 2");
        log::debug!("Created test files: {:?}, {:?}", file1, file2);

        // Build CLI arguments WITHOUT exclude option
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone()];
        let tracked_files = vec![];
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md contains both files
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);

        assert!(content.contains("file1.rs"), "file1.rs should be included");
        assert!(content.contains("file2.rs"), "file2.rs should be included");
        assert!(
            content.contains("File 1"),
            "TODO from file1.rs should appear"
        );
        assert!(
            content.contains("File 2"),
            "TODO from file2.rs should appear"
        );
    }
}
