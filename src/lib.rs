// Allow deprecated functions for backward compatibility in public API

pub mod cli;
pub mod git_utils;
pub mod logger;
pub mod todo_extractor;
pub mod todo_md;
pub mod todo_md_internal;

mod todo_extractor_internal;

// Re-export the public API from the todo_extractor module at the crate root.
// Note: extract_marked_items is deprecated in favor of the new optimized API
pub use todo_extractor::{extract_marked_items_from_file, MarkerConfig};

#[cfg(test)]
pub mod test_utils;
