use crate::MarkedItem;
use log::{debug, info};
use std::collections::HashMap;
use std::path::PathBuf;

// TODO: generalize in maker collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoCollection {
    // Maps a file path to a list of TODO items found in that file.
    pub todos: HashMap<PathBuf, Vec<MarkedItem>>,
}

impl TodoCollection {
    /// Creates and returns a new, empty TodoCollection instance.
    pub fn new() -> Self {
        info!("Creating a new TodoCollection");
        TodoCollection {
            todos: HashMap::new(),
        }
    }

    /// Adds a MarkedItem to the collection. If the file already has associated TODO items,
    /// the new item is appended to the existing list.
    pub fn add_item(&mut self, item: MarkedItem) {
        info!("Adding item to collection: {item:?}");
        self.todos
            .entry(item.file_path.clone())
            .or_default()
            .push(item);
    }

    /// Merges a new TodoCollection (representing the latest scan results) into the
    /// existing collection, updating only those files that were scanned.
    ///
    /// Merge Logic:
    ///     For each file in the provided scanned_files, remove any existing TODO items.
    ///     For each file in the new collection, insert the new TODO items (which replaces any previous
    ///         entries for that file).
    ///     Files not included in scanned_files remain unchanged.
    pub fn merge(&mut self, new: TodoCollection, scanned_files: Vec<PathBuf>) {
        info!("Merging new TodoCollection into existing one");

        // For each file that was scanned, remove its previous entries.
        for file in scanned_files {
            self.todos.remove(&file);
        }

        // Insert new todos for files that were scanned.
        for (file, new_items) in new.todos {
            debug!("Updating todos for file: {file:?}");
            self.todos.insert(file, new_items);
        }
    }

    /// Returns a vector containing all MarkedItem entries sorted first lexicographically by
    /// file path and then in ascending order by line number.
    pub fn to_sorted_vec(&self) -> Vec<MarkedItem> {
        info!("Converting TodoCollection to a sorted vector");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_logger;
    use crate::MarkedItem;
    use std::path::PathBuf;

    #[test]
    fn test_add_item() {
        init_logger();
        let mut collection = TodoCollection::new();
        let item = MarkedItem {
            file_path: PathBuf::from("src/test.rs"),
            line_number: 42,
            message: "Test TODO".to_string(),
            marker: "TODO".to_string(),
        };
        collection.add_item(item.clone());
        assert!(collection.todos.contains_key(&PathBuf::from("src/test.rs")));
        let items = collection.todos.get(&PathBuf::from("src/test.rs")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], item);
    }

    // Test that missing items from the new collection are added to the existing collection.
    #[test]
    fn test_merge_adds_missing_items() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 10,
            message: "Fix bug".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item1.clone());

        let mut col2 = TodoCollection::new();
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 20,
            message: "Implement new feature".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(item1.clone());
        col2.add_item(item2.clone());

        // Updated merge call.
        col1.merge(col2, vec![]);

        let foo_items = col1.todos.get(&PathBuf::from("src/foo.rs")).unwrap();
        assert_eq!(foo_items.len(), 2, "Expected two items for src/foo.rs");
        assert!(foo_items.contains(&item1));
        assert!(foo_items.contains(&item2));
    }

    // Test that merging collections does not duplicate items when the same item exists.
    #[test]
    fn test_merge_no_duplicates() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item = MarkedItem {
            file_path: PathBuf::from("src/bar.rs"),
            line_number: 15,
            message: "Refactor code".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item.clone());

        let mut col2 = TodoCollection::new();
        // Add the same item in the second collection.
        col2.add_item(item.clone());

        col1.merge(col2, vec![]);

        let bar_items = col1.todos.get(&PathBuf::from("src/bar.rs")).unwrap();
        assert_eq!(bar_items.len(), 1, "Expected no duplicates for src/bar.rs");
        assert_eq!(bar_items[0], item);
    }

    // Test that merging an empty collection leaves the existing collection unchanged.
    #[test]
    fn test_merge_keeps_existing_items_when_new_empty() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item = MarkedItem {
            file_path: PathBuf::from("src/baz.rs"),
            line_number: 25,
            message: "Optimize performance".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item.clone());

        let col2 = TodoCollection::new(); // empty collection

        col1.merge(col2, vec![]);

        let baz_items = col1.todos.get(&PathBuf::from("src/baz.rs")).unwrap();
        assert_eq!(baz_items.len(), 1, "Existing item should not be removed");
        assert_eq!(baz_items[0], item);
    }

    // Test merging collections across different files.
    #[test]
    fn test_merge_multiple_files() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 5,
            message: "Improve variable naming".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item1.clone());

        let mut col2 = TodoCollection::new();
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/b.rs"),
            line_number: 10,
            message: "Add unit tests".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(item2.clone());

        col1.merge(col2, vec![]);

        // Both files should be present with their respective items.
        assert!(col1.todos.contains_key(&PathBuf::from("src/a.rs")));
        assert!(col1.todos.contains_key(&PathBuf::from("src/b.rs")));
        let a_items = col1.todos.get(&PathBuf::from("src/a.rs")).unwrap();
        let b_items = col1.todos.get(&PathBuf::from("src/b.rs")).unwrap();
        assert_eq!(a_items.len(), 1);
        assert_eq!(b_items.len(), 1);
        assert_eq!(a_items[0], item1);
        assert_eq!(b_items[0], item2);
    }

    // Test that the sorted vector output is in the expected order across multiple files.
    #[test]
    fn test_merge_sorting_order() {
        init_logger();
        let mut collection = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/z.rs"),
            line_number: 50,
            message: "Last item".to_string(),
            marker: "TODO".to_string(),
        };
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 10,
            message: "First item".to_string(),
            marker: "TODO".to_string(),
        };
        let item3 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 20,
            message: "Second item".to_string(),
            marker: "TODO".to_string(),
        };
        // Add items in non-sorted order.
        collection.add_item(item1.clone());
        collection.add_item(item3.clone());
        collection.add_item(item2.clone());

        let sorted = collection.to_sorted_vec();
        // Expected order: items from src/a.rs (line 10, then 20) followed by src/z.rs.
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], item2);
        assert_eq!(sorted[1], item3);
        assert_eq!(sorted[2], item1);
    }

    #[test]
    fn test_merge_collections() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 10,
            message: "Fix bug".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item1.clone());

        let mut col2 = TodoCollection::new();
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/bar.rs"),
            line_number: 20,
            message: "Implement feature".to_string(),
            marker: "TODO".to_string(),
        };
        let item3 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 30,
            message: "Add tests".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(item2.clone());
        col2.add_item(item3.clone());

        // Merge col2 into col1
        col1.merge(col2, vec![]);

        // Expect col1 to contain both items for src/foo.rs and one for src_bar.rs.
        assert!(col1.todos.contains_key(&PathBuf::from("src/foo.rs")));
        assert!(col1.todos.contains_key(&PathBuf::from("src/bar.rs")));
        let foo_items = col1.todos.get(&PathBuf::from("src/foo.rs")).unwrap();
        assert_eq!(foo_items.len(), 1);
        let bar_items = col1.todos.get(&PathBuf::from("src/bar.rs")).unwrap();
        assert_eq!(bar_items.len(), 1);
    }

    #[test]
    fn test_to_sorted_vec() {
        init_logger();
        let mut collection = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/z.rs"),
            line_number: 50,
            message: "Last item".to_string(),
            marker: "TODO".to_string(),
        };
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 10,
            message: "First item".to_string(),
            marker: "TODO".to_string(),
        };
        let item3 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 20,
            message: "Second item".to_string(),
            marker: "TODO".to_string(),
        };
        collection.add_item(item1.clone());
        collection.add_item(item2.clone());
        collection.add_item(item3.clone());

        let sorted = collection.to_sorted_vec();
        // Expected order: items from src/a.rs (line 10, then 20) followed by src/z.rs.
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], item2);
        assert_eq!(sorted[1], item3);
        assert_eq!(sorted[2], item1);
    }

    #[test]
    fn test_merge_replaces_existing_items() {
        init_logger();
        let mut col1 = TodoCollection::new();
        let item_old = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 10,
            message: "Fix bug".to_string(),
            marker: "TODO".to_string(),
        };
        let item_stale = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 15,
            message: "Old note".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(item_old);
        col1.add_item(item_stale);

        let mut col2 = TodoCollection::new();
        let item_new = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 20,
            message: "Implement feature".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(item_new.clone());

        // Updated merge call.
        col1.merge(col2, vec![]);

        let foo_items = col1.todos.get(&PathBuf::from("src/foo.rs")).unwrap();
        // We expect that the stale items have been replaced and only the new one remains.
        assert_eq!(
            foo_items.len(),
            1,
            "Expected old items to be replaced by the new list"
        );
        assert_eq!(foo_items[0], item_new);
    }

    #[test]
    fn test_merge_complex_replacement() {
        init_logger();
        let mut col1 = TodoCollection::new();
        // File A: initially two items.
        let a_item1 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 5,
            message: "A: initial task".to_string(),
            marker: "TODO".to_string(),
        };
        let a_item2 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 15,
            message: "A: old task".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(a_item1);
        col1.add_item(a_item2);

        // File B: initially one item.
        let b_item1 = MarkedItem {
            file_path: PathBuf::from("src/b.rs"),
            line_number: 10,
            message: "B: fix issue".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(b_item1.clone());

        // File C: exists only in col1.
        let c_item1 = MarkedItem {
            file_path: PathBuf::from("src/c.rs"),
            line_number: 20,
            message: "C: temporary note".to_string(),
            marker: "TODO".to_string(),
        };
        col1.add_item(c_item1);

        // Create col2 with updated items.
        let mut col2 = TodoCollection::new();
        // For File A, new list with one updated item.
        let a_item_new = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 7,
            message: "A: new task".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(a_item_new.clone());

        // For File B, new list with an additional item.
        let b_item2 = MarkedItem {
            file_path: PathBuf::from("src/b.rs"),
            line_number: 12,
            message: "B: additional improvement".to_string(),
            marker: "TODO".to_string(),
        };
        // Note: Even though b_item1 is already in col1, intended behavior is to replace the list.
        col2.add_item(b_item1.clone());
        col2.add_item(b_item2.clone());

        // For File D, a new file not in col1.
        let d_item1 = MarkedItem {
            file_path: PathBuf::from("src/d.rs"),
            line_number: 1,
            message: "D: start here".to_string(),
            marker: "TODO".to_string(),
        };
        col2.add_item(d_item1.clone());

        // No scanned_files provided, so File C should remain unchanged
        col1.merge(col2, vec![]);

        // File A should now have only the new item.
        let a_items = col1.todos.get(&PathBuf::from("src/a.rs")).unwrap();
        assert_eq!(a_items.len(), 1, "File A's items should have been replaced");
        assert_eq!(a_items[0], a_item_new);

        // File B should have exactly the two items from col2.
        let b_items = col1.todos.get(&PathBuf::from("src/b.rs")).unwrap();
        assert_eq!(
            b_items.len(),
            2,
            "File B should have been replaced with two items"
        );
        assert!(b_items.contains(&b_item1));
        assert!(b_items.contains(&b_item2));

        // File D should be newly added.
        let d_items = col1.todos.get(&PathBuf::from("src/d.rs")).unwrap();
        assert_eq!(d_items.len(), 1);
        assert_eq!(d_items[0], d_item1);

        // File C should still be present since it wasn't in scanned_files
        assert!(
            col1.todos.contains_key(&PathBuf::from("src/c.rs")),
            "File C should remain present since it wasn't scanned"
        );
    }

    #[test]
    fn test_merge_scanned_file_removal() {
        // Initialize a collection with a TODO for a file.
        let mut original = TodoCollection::new();
        let item = MarkedItem {
            file_path: PathBuf::from("src/old.rs"),
            line_number: 100,
            message: "Obsolete TODO".to_string(),
            marker: "TODO".to_string(),
        };
        original.add_item(item);

        // Create an empty new collection (simulating that no new TODO was found for that file).
        let new_collection = TodoCollection::new();

        // Call merge with scanned_files containing "src/old.rs".
        original.merge(new_collection, vec![PathBuf::from("src/old.rs")]);

        // Assert that "src/old.rs" has been removed from the collection.
        assert!(
            !original.todos.contains_key(&PathBuf::from("src/old.rs")),
            "Expected 'src/old.rs' to be removed when no new TODOs are provided."
        );
    }
}
