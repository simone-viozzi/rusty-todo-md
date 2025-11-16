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
        // Create parent directories if needed for nested files
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
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
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

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
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

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
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

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
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

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
        let fake_git_ops = FakeGitOps::new(repo, temp_dir, staged_files, tracked_files);

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

    /// Test that the --auto-add flag automatically stages TODO.md when it gets modified.
    /// This test mimics a real-world pre-commit scenario.
    ///
    /// Manual testing steps (if needed):
    /// 1. Initialize a git repo: `git init`
    /// 2. Create files with TODOs: `echo "// TODO: test" > sample.rs`
    /// 3. Add and commit files: `git add sample.rs && git commit -m "initial"`
    /// 4. Run without auto-add: `rusty-todo-md sample.rs`
    /// 5. Check status: `git status` (TODO.md should be untracked)
    /// 6. Add another TODO file: `echo "# TODO: another" > sample.py`
    /// 7. Run with auto-add: `rusty-todo-md --auto-add sample.rs sample.py`
    /// 8. Check status: `git status` (TODO.md should be staged)
    #[test]
    fn test_auto_add_functionality() {
        init_logger();
        log::info!("Starting test_auto_add_functionality");

        // Save the current working directory
        let original_cwd = std::env::current_dir().expect("Failed to get current dir");

        // Create a real git repository using the test utility
        let (temp_dir, repo) = init_repo().expect("Failed to init repo");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Change to the git repository directory for the test
        std::env::set_current_dir(repo_path).expect("Failed to change directory");

        // Create initial test file with TODO comments in the git repo directory
        let _file1 = create_test_file(
            repo_path,
            "sample.rs",
            "// TODO: Implement user authentication\nfn main() {}",
        );

        // Commit the test file to git
        let mut index = repo.index().expect("Failed to get index");
        index
            .add_path(std::path::Path::new("sample.rs"))
            .expect("Failed to add sample.rs");
        index.write().expect("Failed to write index");

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let sig = git2::Signature::now("Test User", "test@example.com")
            .expect("Failed to create signature");
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Add test files",
            &tree,
            &[&repo.head().unwrap().peel_to_commit().unwrap()],
        )
        .expect("Failed to commit");

        // Test 1: Run without --auto-add flag (TODO.md should NOT be staged)
        let args_no_auto = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            "TODO.md".to_string(), // Use relative path since we're in the repo dir
            "sample.rs".to_string(), // Use relative path
        ];

        let git_ops = rusty_todo_md::git_utils::GitOps;
        run_cli_with_args(args_no_auto, &git_ops);

        // Verify TODO.md was created
        assert!(todo_path.exists(), "TODO.md should be created");
        let todo_content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        assert!(
            todo_content.contains("Implement user authentication"),
            "Should contain TODO from sample.rs"
        );

        // Check git status - TODO.md should NOT be staged
        let status = repo.statuses(None).expect("Failed to get git status");
        let todo_md_status = status.iter().find(|s| s.path() == Some("TODO.md"));
        assert!(
            todo_md_status.is_some(),
            "TODO.md should appear in git status"
        );
        if let Some(status_entry) = todo_md_status {
            assert!(
                status_entry.status().is_wt_new() || status_entry.status().is_wt_modified(),
                "TODO.md should be untracked/modified, not staged"
            );
        }

        // Test 2: Add another file and run with --auto-add flag (TODO.md should be staged)
        let _file2 = create_test_file(
            repo_path,
            "sample.py",
            "# TODO: Add error handling\ndef main():\n    pass",
        );

        let args_with_auto = vec![
            "rusty-todo-md".to_string(),
            "--auto-add".to_string(),
            "--todo-path".to_string(),
            "TODO.md".to_string(),   // Use relative path
            "sample.rs".to_string(), // Use relative path
            "sample.py".to_string(), // Use relative path
        ];

        run_cli_with_args(args_with_auto, &git_ops);

        // Verify TODO.md was updated with both files
        let updated_content =
            fs::read_to_string(&todo_path).expect("Failed to read updated TODO.md");
        assert!(
            updated_content.contains("Implement user authentication"),
            "Should still contain TODO from sample.rs"
        );
        assert!(
            updated_content.contains("Add error handling"),
            "Should now contain TODO from sample.py"
        );

        // Check git status - TODO.md should NOW be staged
        let status_after = repo
            .statuses(None)
            .expect("Failed to get git status after auto-add");
        let todo_md_status_after = status_after.iter().find(|s| s.path() == Some("TODO.md"));
        assert!(
            todo_md_status_after.is_some(),
            "TODO.md should appear in git status"
        );
        if let Some(status_entry) = todo_md_status_after {
            assert!(
                status_entry.status().is_index_new() || status_entry.status().is_index_modified(),
                "TODO.md should be staged after --auto-add"
            );
        }

        // Restore the original working directory
        std::env::set_current_dir(original_cwd).expect("Failed to restore original directory");

        log::info!("test_auto_add_functionality completed successfully");
    }

    #[test]
    fn test_markers_arg_parsing() {
        init_logger();
        log::info!("Starting test_markers_arg_parsing");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");
        let file1 = repo_path.join("file1.rs");
        fs::write(&file1, "// TODO: test marker").unwrap();

        let args = vec![
            "rusty-todo-md",
            "--markers",
            "TODO",
            "FIXME",
            "HACK",
            "--todo-path",
            todo_path.to_str().unwrap(),
            file1.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![file1.clone()],
            vec![file1.clone()],
        );

        run_cli_with_args(args, &fake_git_ops);
        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("file1.rs"));
        assert!(content.contains("test marker"));

        log::info!("test_markers_arg_parsing completed successfully");
    }

    #[test]
    fn test_markers_with_separator() {
        init_logger();
        log::info!("Starting test_markers_with_separator");

        // This test verifies that using -- separator works correctly
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");
        let file1 = repo_path.join("file1.rs");
        let file2 = repo_path.join("file2.rs");

        fs::write(&file1, "// TODO: test marker in file1").unwrap();
        fs::write(&file2, "// FIXME: test marker in file2").unwrap();

        // Using -- to separate markers from files
        let args = vec![
            "rusty-todo-md",
            "--auto-add",
            "--todo-path",
            todo_path.to_str().unwrap(),
            "--markers",
            "TODO",
            "FIXME",
            "HACK",
            "--",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![file1.clone(), file2.clone()],
            vec![file1.clone(), file2.clone()],
        );

        run_cli_with_args(args, &fake_git_ops);

        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();

        // Both files should be processed correctly
        assert!(content.contains("file1.rs"));
        assert!(content.contains("file2.rs"));
        assert!(content.contains("test marker in file1"));
        assert!(content.contains("test marker in file2"));

        log::info!("test_markers_with_separator completed successfully");
    }

    /// Integration test for file exclusion with glob patterns
    #[test]
    fn test_exclude_files_with_glob_patterns() {
        init_logger();
        log::info!("Starting test_exclude_files_with_glob_patterns");

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create multiple files with TODO comments
        let file1 = create_test_file(repo_path, "src/main.rs", "// TODO: Main file");
        let file2 = create_test_file(repo_path, "src/lib.rs", "// TODO: Library file");
        let file3 = create_test_file(repo_path, "build/output.rs", "// TODO: Build output");
        let file4 = create_test_file(repo_path, "tests/test.rs", "// TODO: Test file");

        // Exclude src/ directory and all build output
        let args = vec![
            "rusty-todo-md".to_string(),
            "--todo-path".to_string(),
            todo_path.to_str().unwrap().to_string(),
            "--exclude".to_string(),
            "src/".to_string(),
            "--exclude-dir".to_string(),
            "build".to_string(),
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

        // src/ should be excluded
        assert!(!content.contains("src/main.rs"), "src/ should be excluded");
        assert!(!content.contains("src/lib.rs"), "src/ should be excluded");

        // build should be excluded
        assert!(
            !content.contains("build/output.rs"),
            "build/ should be excluded"
        );

        // tests should be included
        assert!(
            content.contains("tests/test.rs"),
            "tests/ should be included"
        );

        log::info!("test_exclude_files_with_glob_patterns completed successfully");
    }
}
