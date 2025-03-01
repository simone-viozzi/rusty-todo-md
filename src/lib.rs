// src/lib.rs
pub mod logger;
pub mod todo_extractor;

mod todo_extractor_internal;

// Re-export the public API from the todo_extractor module at the crate root.
pub use todo_extractor::{extract_marked_items, MarkerConfig};
