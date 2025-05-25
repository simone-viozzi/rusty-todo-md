mod utils;

mod integration_tests {
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
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Test that running the CLI with a single file containing a TODO comment creates a proper TODO.md section.
    #[test]
    fn test_process_files_list_single_run() {
        init_logger();
        log::info!("Starting test_process_files_list_single_run");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create a test file with a TODO comment.
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: Implement feature X");
        log::debug!("Created test file: {:?}", file1);

        // Build CLI arguments (only a list of files is supported now).
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        // FakeGitOps setup inlined
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone()];
        let tracked_files = vec![];
        let deleted_files = vec![];
        let fake_git_ops =
            FakeGitOps::new(repo, temp_dir, staged_files, tracked_files, deleted_files);

        // Run the CLI.
        run_cli_with_args(args, &fake_git_ops);

        // Verify that TODO.md has been created and contains the expected section and message.
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content: {}", content);
        assert!(
            content.contains("file1.rs"),
            "Expected TODO entry for file1.rs"
        );
        assert!(
            content.contains("Implement feature X"),
            "Expected TODO message in file1.rs"
        );
    }

    /// Test that if a file is updated (its TODO message changes), the TODO.md file is updated accordingly.
    #[test]
    fn test_update_todo_md_on_file_change() {
        init_logger();
        log::info!("Starting test_update_todo_md_on_file_change");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create a file with an initial TODO comment.
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: Initial implementation");
        log::debug!("Created test file: {:?}", file1);

        // Build arguments.
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        // FakeGitOps setup inlined
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone()];
        let tracked_files = vec![];
        let deleted_files = vec![];
        let fake_git_ops =
            FakeGitOps::new(repo, temp_dir, staged_files, tracked_files, deleted_files);

        // Run the CLI.
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content_initial = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("Initial TODO.md content: {}", content_initial);
        assert!(
            content_initial.contains("Initial implementation"),
            "Expected initial TODO message"
        );

        // Update the file with a new TODO comment.
        fs::write(&file1, "// TODO: Updated implementation").expect("Failed to update file");
        log::debug!("Updated test file: {:?}", file1);

        // Second run.
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content_updated =
            fs::read_to_string(&todo_path).expect("Failed to read TODO.md after update");
        log::debug!("Updated TODO.md content: {}", content_updated);
        assert!(
            content_updated.contains("Updated implementation"),
            "Expected updated TODO message"
        );
        assert!(
            !content_updated.contains("Initial implementation"),
            "Old TODO message should be removed"
        );
    }

    /// Test that if a file no longer contains a TODO comment, its section is removed from TODO.md.
    #[test]
    fn test_update_todo_md_on_file_removal() {
        init_logger();
        log::info!("Starting test_update_todo_md_on_file_removal");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create a file with a TODO comment.
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: Remove this code");
        log::debug!("Created test file: {:?}", file1);

        // Build arguments.
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        // FakeGitOps setup inlined
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone()];
        let tracked_files = vec![];
        let deleted_files = vec![];
        let fake_git_ops =
            FakeGitOps::new(repo, temp_dir, staged_files, tracked_files, deleted_files);

        // First run: file has a TODO.
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content_initial = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("Initial TODO.md content: {}", content_initial);
        assert!(
            content_initial.contains("Remove this code"),
            "Expected TODO message initially"
        );

        // Update the file to remove the TODO comment.
        fs::write(&file1, "// No TODO here anymore").expect("Failed to update file to remove TODO");
        log::debug!("Updated test file: {:?}", file1);

        // Second run.
        run_cli_with_args(args, &fake_git_ops);
        let content_updated =
            fs::read_to_string(&todo_path).expect("Failed to read updated TODO.md");
        log::debug!("Updated TODO.md content: {}", content_updated);
        // The section for file1 should now be removed.
        assert!(
            !content_updated.contains("file1.rs"),
            "Section for file1.rs should be removed when no TODO is present"
        );
        assert!(
            !content_updated.contains("Remove this code"),
            "Old TODO message should be removed"
        );
    }

    /// Test running the CLI multiple times on the same file, simulating real-world updates.
    #[test]
    fn test_multiple_runs_update() {
        init_logger();
        log::info!("Starting test_multiple_runs_update");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create a file with an initial TODO comment.
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: First version");
        log::debug!("Created test file: {:?}", file1);

        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        // FakeGitOps setup inlined
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone()];
        let tracked_files = vec![];
        let deleted_files = vec![];
        let fake_git_ops =
            FakeGitOps::new(repo, temp_dir, staged_files, tracked_files, deleted_files);

        // Run 1: initial TODO.
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content1 = fs::read_to_string(&todo_path).expect("Failed to read TODO.md after run 1");
        log::debug!("TODO.md content after run 1: {}", content1);
        assert!(
            content1.contains("First version"),
            "Expected first version of TODO"
        );

        // Run 2: update the TODO message.
        fs::write(&file1, "// TODO: Second version")
            .expect("Failed to update file with second version");
        log::debug!("Updated test file: {:?}", file1);
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content2 = fs::read_to_string(&todo_path).expect("Failed to read TODO.md after run 2");
        log::debug!("TODO.md content after run 2: {}", content2);
        assert!(
            content2.contains("Second version"),
            "Expected second version of TODO"
        );
        assert!(
            !content2.contains("First version"),
            "Old TODO should be removed"
        );

        // Run 3: remove the TODO comment altogether.
        fs::write(&file1, "// No TODO now").expect("Failed to update file to remove TODO");
        log::debug!("Updated test file: {:?}", file1);
        run_cli_with_args(args, &fake_git_ops);
        let content3 = fs::read_to_string(&todo_path).expect("Failed to read TODO.md after run 3");
        log::debug!("TODO.md content after run 3: {}", content3);
        assert!(
            !content3.contains("file1.rs"),
            "Section for file1.rs should be removed when no TODO exists"
        );
        assert!(
            !content3.contains("Second version"),
            "Previous TODO message should be removed"
        );
    }

    /// Test handling multiple files simultaneously:
    /// one file gets updated and the other removed.
    #[test]
    fn test_multiple_files_update() {
        init_logger();
        log::info!("Starting test_multiple_files_update");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create two test files with TODO comments.
        let file1 = create_test_file(repo_path, "file1.rs", "// TODO: Feature A");
        let file2 = create_test_file(repo_path, "file2.rs", "// TODO: Feature B");
        log::debug!("Created test files: {:?}, {:?}", file1, file2);

        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            file1.to_str().unwrap().to_string(),
            file2.to_str().unwrap().to_string(),
        ];
        log::debug!("CLI arguments: {:?}", args);

        // FakeGitOps setup inlined
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let staged_files = vec![file1.clone(), file2.clone()];
        let tracked_files = vec![];
        let deleted_files = vec![];
        let fake_git_ops =
            FakeGitOps::new(repo, temp_dir, staged_files, tracked_files, deleted_files);

        // Run 1: both files processed.
        run_cli_with_args(args.clone(), &fake_git_ops);
        let content_initial =
            fs::read_to_string(&todo_path).expect("Failed to read initial TODO.md");
        log::debug!("Initial TODO.md content:\n{}", content_initial);
        assert!(
            content_initial.contains("Feature A"),
            "Expected Feature A in TODO.md"
        );
        assert!(
            content_initial.contains("Feature B"),
            "Expected Feature B in TODO.md"
        );

        // Update: change file1's TODO and remove file2's TODO.
        fs::write(&file1, "// TODO: Updated Feature A").expect("Failed to update file1");
        fs::write(&file2, "// No TODO in file2").expect("Failed to update file2");
        log::debug!(
            "Updated test files:\nfile1\n{:?}\nfile2\n{:?}",
            file1,
            file2
        );

        // Run 2: process updates.
        run_cli_with_args(args, &fake_git_ops);
        let content_updated =
            fs::read_to_string(&todo_path).expect("Failed to read updated TODO.md");
        log::debug!("Updated TODO.md content: {}", content_updated);

        // Extract the section for file1
        let file1_section = content_updated
            .split("##")
            .find(|section| section.contains("file1.rs"))
            .unwrap_or("");

        assert!(
            file1_section.contains("Updated Feature A"),
            "Expected updated Feature A"
        );
        // Check that the old line is not present (using a more precise pattern)
        assert!(
            !file1_section.contains("): Feature A"),
            "Old Feature A should be removed"
        );

        // File2 should be removed because it no longer contains a TODO.
        assert!(
            !content_updated.contains("file2.rs"),
            "Section for file2.rs should be removed"
        );
        assert!(
            !content_updated.contains("Feature B"),
            "Feature B should be removed"
        );
    }
}
