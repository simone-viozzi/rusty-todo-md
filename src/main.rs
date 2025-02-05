use std::env;
use std::path::Path;
use log::LevelFilter;

fn main() {
    // Initialize the logger based on RUST_LOG or default to Debug.
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Debug)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <path/to/file>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let content = std::fs::read_to_string(path).expect("cannot read file");

    // Use the library's extractor
    let todos = todo_extractor::extract_todos(path, &content);

    if todos.is_empty() {
        println!("No TODOs found.");
    } else {
        println!("Found {} TODOs:", todos.len());
        for todo in todos {
            println!("{} - {}", todo.line_number, todo.message);
        }
    }
}
