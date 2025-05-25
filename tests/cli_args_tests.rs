mod utils;

mod cli_args_tests {
    use crate::utils::FakeGitOps;
    use rusty_todo_md::cli::run_cli_with_args;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_markers_arg_parsing() {
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
            vec![],
        );

        run_cli_with_args(args, &fake_git_ops);
        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();
        assert!(content.contains("file1.rs"));
        assert!(content.contains("test marker"));
    }
}
