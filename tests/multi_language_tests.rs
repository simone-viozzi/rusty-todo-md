mod utils;

mod multi_language_tests {
    use crate::utils::FakeGitOps;
    use rusty_todo_md::cli::run_cli_with_args;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mixed_language_todo_extraction() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");

        // Create files with different languages
        let rust_file = repo_path.join("main.rs");
        fs::write(&rust_file, "// TODO: Implement Rust feature\nfn main() {}").unwrap();

        let js_file = repo_path.join("app.js");
        fs::write(
            &js_file,
            "// TODO: Implement JS feature\nconsole.log('hello');",
        )
        .unwrap();

        let jsx_file = repo_path.join("component.jsx");
        fs::write(
            &jsx_file,
            "// TODO: Add prop validation\nconst App = () => <div>Hello</div>;",
        )
        .unwrap();

        let go_file = repo_path.join("main.go");
        fs::write(&go_file, "// TODO: Implement Go feature\npackage main").unwrap();

        let python_file = repo_path.join("script.py");
        fs::write(
            &python_file,
            "# TODO: Implement Python feature\nprint('hello')",
        )
        .unwrap();

        let args = vec![
            "rusty-todo-md",
            "--todo-path",
            todo_path.to_str().unwrap(),
            rust_file.to_str().unwrap(),
            js_file.to_str().unwrap(),
            jsx_file.to_str().unwrap(),
            go_file.to_str().unwrap(),
            python_file.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![
                rust_file.clone(),
                js_file.clone(),
                jsx_file.clone(),
                go_file.clone(),
                python_file.clone(),
            ],
            vec![
                rust_file.clone(),
                js_file.clone(),
                jsx_file.clone(),
                go_file.clone(),
                python_file.clone(),
            ],
        );

        run_cli_with_args(args, &fake_git_ops);

        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();

        // Verify all language files are included
        assert!(content.contains("main.rs"));
        assert!(content.contains("app.js"));
        assert!(content.contains("component.jsx"));
        assert!(content.contains("main.go"));
        assert!(content.contains("script.py"));

        // Verify TODO messages are extracted
        assert!(content.contains("Implement Rust feature"));
        assert!(content.contains("Implement JS feature"));
        assert!(content.contains("Add prop validation"));
        assert!(content.contains("Implement Go feature"));
        assert!(content.contains("Implement Python feature"));
    }

    #[test]
    fn test_js_with_fixme_markers() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");
        let js_file = repo_path.join("complex.js");

        let js_content = r#"
// TODO: Refactor this function
function init() {
    /* FIXME: Handle edge cases
       such as null responses */
    fetchData();
}

// Regular comment without marker
const x = 10;

/* TODO: Add error handling */
"#;
        fs::write(&js_file, js_content).unwrap();

        let args = vec![
            "rusty-todo-md",
            "--markers",
            "TODO",
            "FIXME",
            "--todo-path",
            todo_path.to_str().unwrap(),
            js_file.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![js_file.clone()],
            vec![js_file.clone()],
        );

        run_cli_with_args(args, &fake_git_ops);
        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();

        // Check for TODO section
        assert!(content.contains("# TODO"));
        assert!(content.contains("Refactor this function"));
        assert!(content.contains("Add error handling"));

        // Check for FIXME section
        assert!(content.contains("# FIXME"));
        assert!(content.contains("Handle edge cases such as null responses"));
    }

    #[test]
    fn test_go_with_mixed_comments() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");
        let go_file = repo_path.join("server.go");

        let go_content = r#"
package main

// TODO: Add proper logging
import "fmt"

/* FIXME: Implement proper error handling
   across the entire package */

func main() {
    // Regular comment
    fmt.Println("Hello, World!")

    /* TODO: Add configuration support */
}
"#;
        fs::write(&go_file, go_content).unwrap();

        let args = vec![
            "rusty-todo-md",
            "--markers",
            "TODO",
            "FIXME",
            "--todo-path",
            todo_path.to_str().unwrap(),
            go_file.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![go_file.clone()],
            vec![go_file.clone()],
        );

        run_cli_with_args(args, &fake_git_ops);
        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();

        // Check for TODO section
        assert!(content.contains("# TODO"));
        assert!(content.contains("Add proper logging"));
        assert!(content.contains("Add configuration support"));

        // Check for FIXME section
        assert!(content.contains("# FIXME"));
        assert!(content.contains("Implement proper error handling across the entire package"));
    }

    #[test]
    fn test_dockerfile_with_multiple_markers() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();
        let todo_path = repo_path.join("TODO.md");
        let dockerfile = repo_path.join("Dockerfile");

        let dockerfile_content = r#"FROM node:18-alpine

# TODO: Optimize base image size
RUN apk add --no-cache \
    python3 \
    make \
    g++ \
    # FIXME: Pin package versions for reproducibility
    && npm install -g pnpm

WORKDIR /app

# HACK: Temporary workaround for build issues
COPY package*.json ./

# TODO: Add health check endpoint
RUN npm install

COPY . .
EXPOSE 3000
CMD ["npm", "start"]"#;
        fs::write(&dockerfile, dockerfile_content).unwrap();

        let args = vec![
            "rusty-todo-md",
            "--markers",
            "TODO",
            "FIXME",
            "HACK",
            "--todo-path",
            todo_path.to_str().unwrap(),
            dockerfile.to_str().unwrap(),
        ];

        let fake_git_ops = FakeGitOps::new(
            git2::Repository::init(repo_path).unwrap(),
            temp_dir,
            vec![dockerfile.clone()],
            vec![dockerfile.clone()],
        );

        run_cli_with_args(args, &fake_git_ops);
        assert!(todo_path.exists());
        let content = fs::read_to_string(&todo_path).unwrap();

        // Check for TODO section
        assert!(content.contains("# TODO"));
        assert!(content.contains("Optimize base image size"));
        assert!(content.contains("Add health check endpoint"));

        // Check for FIXME section
        assert!(content.contains("# FIXME"));
        assert!(content.contains("Pin package versions for reproducibility"));

        // Check for HACK section
        assert!(content.contains("# HACK"));
        assert!(content.contains("Temporary workaround for build issues"));
    }
}
