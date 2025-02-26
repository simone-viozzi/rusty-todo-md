use log::{info, warn};
use std::env;
use std::path::Path;
mod logger;
use todo_extractor::{extract_marked_items, MarkerConfig};

fn main() {
    // Initialize the logger
    env_logger::Builder::from_default_env()
        .format(logger::format_logger)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        warn!("Usage: {} <path/to/file>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let content = std::fs::read_to_string(path).expect("cannot read file");

    // TODO: make this configurable!
    //     also remove this main altogether
    let config = MarkerConfig {
        markers: vec!["TODO:".to_string()],
    };

    // Use the library's extractor
    let todos = extract_marked_items(path, &content, &config);

    if todos.is_empty() {
        info!("No TODOs found.");
    } else {
        info!("Found {} TODOs:", todos.len());
        for todo in todos {
            info!("{} - {}", todo.line_number, todo.message);
        }
    }
}
