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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo_extractor::MarkedItem;
    use std::path::PathBuf;

    #[test]
    fn test_add_item() {
        let mut collection = TodoCollection::new();
        let item = MarkedItem {
            file_path: PathBuf::from("src/test.rs"),
            line_number: 42,
            message: "Test TODO".to_string(),
        };
        collection.add_item(item.clone());
        assert!(collection.todos.contains_key(&PathBuf::from("src/test.rs")));
        let items = collection.todos.get(&PathBuf::from("src/test.rs")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], item);
    }

    // TODO add more tests for the merge of the collections
    //     - test that items are added if they are not already present
    //     - test that items are not duplicated
    //     - test that items are not removed
    //     - test that items are removed if they are not present in the new collection

    #[test]
    fn test_merge_collections() {
        let mut col1 = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 10,
            message: "Fix bug".to_string(),
        };
        col1.add_item(item1.clone());

        let mut col2 = TodoCollection::new();
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/bar.rs"),
            line_number: 20,
            message: "Implement feature".to_string(),
        };
        let item3 = MarkedItem {
            file_path: PathBuf::from("src/foo.rs"),
            line_number: 30,
            message: "Add tests".to_string(),
        };
        col2.add_item(item2.clone());
        col2.add_item(item3.clone());

        // Merge col2 into col1
        col1.merge(col2);

        // Expect col1 to contain both items for src/foo.rs and one for src/bar.rs.
        assert!(col1.todos.contains_key(&PathBuf::from("src/foo.rs")));
        assert!(col1.todos.contains_key(&PathBuf::from("src/bar.rs")));
        let foo_items = col1.todos.get(&PathBuf::from("src/foo.rs")).unwrap();
        assert_eq!(foo_items.len(), 2);
        let bar_items = col1.todos.get(&PathBuf::from("src/bar.rs")).unwrap();
        assert_eq!(bar_items.len(), 1);
    }

    #[test]
    fn test_to_sorted_vec() {
        let mut collection = TodoCollection::new();
        let item1 = MarkedItem {
            file_path: PathBuf::from("src/z.rs"),
            line_number: 50,
            message: "Last item".to_string(),
        };
        let item2 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 10,
            message: "First item".to_string(),
        };
        let item3 = MarkedItem {
            file_path: PathBuf::from("src/a.rs"),
            line_number: 20,
            message: "Second item".to_string(),
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
}
