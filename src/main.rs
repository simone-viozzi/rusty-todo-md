use rusty_todo_md::todo_md::read_todo_file;

fn main() {
    // Simulate the failing test content
    let test_todo_md = r#"
* [src/main.rs:12](src/main.rs#L12): Refactor this function
* [src/lib.rs:5](src/lib.rs#L5): Add error handling
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let todo_path = temp_dir.path().join("TODO.md");

    // Write the simulated content to a file
    std::fs::write(&todo_path, test_todo_md).unwrap();

    println!("Testing read_todo_file with simulated TODO.md...");
    let todos = read_todo_file(&todo_path);

    println!("Parsed TODO Items:");
    for todo in &todos {
        println!("{:?}", todo);
    }

    println!("Done testing!");
}
