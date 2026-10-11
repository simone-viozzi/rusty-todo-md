// Allow deprecated functions for backward compatibility in public API

pub mod cli;
pub mod exclusion;
pub mod git_utils;
pub mod logger;
pub mod todo_md;
pub mod todo_md_internal;

// Private implementation modules
mod todo_extractor_internal;

// Re-export the public API directly at the crate root
pub use todo_extractor_internal::aggregator::{
    extract_marked_items_from_file, CommentLine, MarkedItem, MarkerConfig,
};

#[cfg(test)]
pub mod test_utils;
