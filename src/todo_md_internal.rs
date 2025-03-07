use crate::todo_extractor::MarkedItem;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoCollection {
    // Maps a file path to a list of TODO items found in that file.
    pub todos: HashMap<PathBuf, Vec<MarkedItem>>,
}

impl TodoCollection {
    /// Creates a new, empty TodoCollection.
    /// New Function.
    pub fn new() -> Self {
        TodoCollection {
            todos: HashMap::new(),
        }
    }

    /// Adds a `MarkedItem` to the collection.
    /// If the file already has TODO items, the new item is appended.
    /// New Function.
    pub fn add_item(&mut self, item: MarkedItem) {
        self.todos
            .entry(item.file_path.clone())
            .or_default()
            .push(item);
    }

    /// Merges another `TodoCollection` into this one.
    /// For each file, new items are added if they are not already present.
    /// New Function.
    pub fn merge(&mut self, other: TodoCollection) {
        for (file, mut items) in other.todos {
            let entry = self.todos.entry(file).or_default();
            // TODO this logic seams to be wrong, why only add new items if they are not already present?
            //     what about removing items that are not present in the new collection?
            //     given that this is divide by file we can just replace the entry for a file with the new one
            for new_item in items.drain(..) {
                if !entry.contains(&new_item) {
                    entry.push(new_item);
                }
            }
        }
    }

    /// Returns a sorted vector of all `MarkedItem` entries.
    /// The sorting is done first by the file path, then by the line number.
    /// New Function.
    pub fn to_sorted_vec(&self) -> Vec<MarkedItem> {
        let mut all_items: Vec<_> = self.todos.values().flat_map(|v| v.clone()).collect();
        all_items.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.line_number.cmp(&b.line_number))
        });
        all_items
    }
}

impl Default for TodoCollection {
    fn default() -> Self {
        Self::new()
    }
}
