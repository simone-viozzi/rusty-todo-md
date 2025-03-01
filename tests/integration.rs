mod utils;

mod integration_tests {
    use super::*;
    use log::LevelFilter;
    use rusty_todo_md::cli::run_workflow;
    use rusty_todo_md::git_utils;
    use rusty_todo_md::git_utils::get_staged_files;
    use rusty_todo_md::logger;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Once;
    use utils::TempGitRepo;

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
    fn test_get_staged_files() {
        init_logger();
        // Set up a temporary Git repository
        let temp_repo = TempGitRepo::new();

        // Create and stage test files
        temp_repo.create_file("file1.txt", "TODO: Implement feature A");
        temp_repo.stage_file("file1.txt");

        temp_repo.create_file("file2.txt", "This is a test file");
        temp_repo.stage_file("file2.txt");

        // Call `get_staged_files` to retrieve staged files
        let staged_files =
            get_staged_files(&temp_repo.repo).expect("Failed to retrieve staged files");

        // Verify that the staged files match the expected files
        let expected_files: Vec<_> = vec!["file1.txt", "file2.txt"]
            .into_iter()
            .map(PathBuf::from)
            .collect();

        assert_eq!(staged_files, expected_files);
    }

    #[test]
    fn test_run_workflow() {
        init_logger();
        // Set up a temporary Git repository
        let temp_repo = TempGitRepo::new();
        let todo_path = temp_repo.repo_path.join("TODO.md");

        // Create and stage a file with a TODO comment
        temp_repo.create_file("file1.rs", "// TODO: Refactor this function");
        temp_repo.stage_file("file1.rs");

        // Debug: Verify the index state
        let staged_files =
            git_utils::get_staged_files(&temp_repo.repo).expect("Failed to retrieve staged files");
        log::debug!("Staged files from index: {:?}", staged_files);

        // Run the workflow
        let result = run_workflow(&todo_path, &temp_repo.repo_path, false);

        assert!(result.is_ok(), "Workflow failed with error: {:?}", result);

        // Verify TODO.md exists
        assert!(todo_path.exists(), "TODO.md should have been created");

        // Verify TODO.md was updated
        let content = fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content:\n{}", content);
        assert!(content.contains("file1.rs"), "Expected file1.rs in TODO.md");
        assert!(
            content.contains("Refactor this function"),
            "Expected TODO comment in TODO.md"
        );
    }

    #[test]
    fn test_run_workflow_all_files() {
        init_logger();
        // Set up a temporary Git repository using our test helper.
        let temp_repo = TempGitRepo::new();
        let todo_path = temp_repo.repo_path.join("TODO.md");

        // Create a new file with a TODO comment and commit it so that it is tracked.
        temp_repo.create_file("file_all.rs", "// TODO: Implement all features");
        temp_repo.stage_file("file_all.rs");
        temp_repo.commit("Add file_all.rs");

        // Run the workflow with the --all-files option enabled (i.e. all_files flag set to true).
        let result = run_workflow(&todo_path, &temp_repo.repo_path, true);
        assert!(result.is_ok(), "Workflow failed with error: {:?}", result);

        // Verify that TODO.md has been created.
        assert!(todo_path.exists(), "TODO.md should have been created");

        // Read and verify that the content of TODO.md includes our file and TODO comment.
        let content = std::fs::read_to_string(&todo_path).expect("Failed to read TODO.md");
        log::debug!("TODO.md content:\n{}", content);
        assert!(
            content.contains("file_all.rs"),
            "Expected file_all.rs in TODO.md"
        );
        assert!(
            content.contains("Implement all features"),
            "Expected TODO comment in TODO.md"
        );
    }
}
