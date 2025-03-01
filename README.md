# **TODO Extractor** 🚀  

A **multi-language TODO comment extractor** for source code files. This tool parses **Python, Rust, JavaScript, TypeScript, and Go** files to extract TODO comments, including **single-line and properly formatted multi-line TODOs**.  

---

## ✨ **Features**
- 📌 **Supports multiple programming languages**: Python, Rust, JavaScript, TypeScript, and Go.
- 📝 **Extracts TODO comments** from both single-line (`// TODO:`) and block (`/* TODO: */`) comments.
- 🔄 **Handles multi-line TODOs** by **merging only indented or structured comment blocks**.
- 🚀 **Fast and efficient** using the [Pest](https://pest.rs/) parser.
- 📍 **Provides accurate line numbers** for each extracted TODO.

---

## 📦 **Installation**
Clone the repository and build the project using **Cargo**:

```sh
# Clone the repository
git clone https://github.com/your-repo/todo-extractor.git
cd todo-extractor

# Build the project
cargo build --release

# Run the executable
./target/release/todo-extractor path/to/your/file.rs
```

---

## 🚀 **Usage**
### **Command Line**
Run the extractor by providing a **path** to a source file:

```sh
./todo-extractor path/to/source_file.rs
```

### **Example Output**
```sh
Found 2 TODOs:
3 - Refactor this function
10 - Handle edge cases properly
```

### **Example: Python File (`test.py`)**
```python
# TODO: Implement feature X
def function():
    """
    TODO: Improve performance
        by reducing loops
    """
    pass
```

#### **Extracted TODOs:**
```sh
Found 2 TODOs:
1 - Implement feature X
3 - Improve performance by reducing loops
```

---

## 🔍 **How It Works**
### **1. Detects File Type**
The parser checks the file extension (`.py`, `.rs`, `.js`, `.ts`, `.go`) and selects the appropriate parser.

### **2. Extracts Comment Lines**
Using **Pest grammars**, the extractor retrieves **only** comment lines, while ignoring **code and string literals**.

### **3. Identifies TODO Markers**
The extractor scans comment lines for the `TODO:` marker and **captures the message**.

### **4. Handles Multi-line TODOs Properly**
If a TODO is followed by **indented lines**, they are **merged** into a single entry.  
However, **unrelated lines are not merged**.

---

## 🛠️ **Supported Languages**
| Language   | Single-line Syntax  | Block Syntax       |
|------------|--------------------|--------------------|
| **Python** | `# TODO: ...`       | `""" TODO: ... """` |
| **Rust**   | `// TODO: ...`      | `/* TODO: ... */`  |
| **JavaScript** | `// TODO: ...`  | `/* TODO: ... */`  |
| **TypeScript** | `// TODO: ...`  | `/* TODO: ... */`  |
| **Go**     | `// TODO: ...`      | `/* TODO: ... */`  |

---

## 📝 **Handling Multi-line TODOs Correctly**
A `TODO:` comment may **span multiple lines**, but **only if**:
1. The **next line is indented** (with spaces or tabs).
2. The next line is **part of the same comment block**.

### ✅ **Example: Correct Multi-line TODO**
The second line **is indented**, so it is **merged** into the TODO.
```rust
/// TODO: Fix the parser
///     The tokenizer needs improvement
fn foo() {}
```
✔️ **Extracted TODO:**
```
Found 1 TODO:
1 - Fix the parser The tokenizer needs improvement
```

For block comments:
```rust
/*
   TODO: Refactor this function
       Ensure performance is optimized
*/
```
✔️ **Extracted TODO:**
```
Found 1 TODO:
2 - Refactor this function Ensure performance is optimized
```

---

### ❌ **Example: Incorrect Multi-line TODO**
The second line is **not indented**, so it **should NOT be merged**.
```rust
/// TODO: Fix the parser
/// The tokenizer needs improvement
fn foo() {}
```
❌ **Extracted TODOs:**
```
Found 1 TODO:
1 - Fix the parser
```

For block comments:
```rust
/*
   TODO: Refactor this function
   This line should NOT be merged
*/
```
❌ **Extracted TODOs:**
```
Found 1 TODO:
2 - Refactor this function
```

---

## 🧪 **Running Tests**
Run tests to validate TODO extraction:
```sh
cargo test
```

### **Example Test Case for Rust Files**
```rust
#[test]
fn test_rust_single_line() {
    let src = "// TODO: Refactor this code";
    let todos = extract_todos(Path::new("example.rs"), src);
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].message, "Refactor this code");
}
```

### **Example Test Case for Multi-line TODO (Indented)**
```rust
#[test]
fn test_rust_multiline_todo() {
    let src = r#"
/// TODO: Improve logging
///     Add detailed debug info
fn log() {}"#;

    let todos = extract_todos(Path::new("example.rs"), src);
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].message, "Improve logging Add detailed debug info");
}
```

---

## 🤝 Contributing
Want to add support for another language? Contributions are welcome! Feel free to open an issue or submit a pull request.
