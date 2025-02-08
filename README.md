# TODO Extractor

A **multi-language TODO comment extractor** for source code files. This tool parses Python, Rust, JavaScript, TypeScript, and Go files to extract TODO comments, including single-line and multi-line TODOs.

## Features
- 📌 **Supports multiple programming languages**: Python, Rust, JavaScript, TypeScript, and Go.
- 📝 **Extracts TODO comments** from both single-line (`// TODO:`) and block (`/* TODO: */`) comments.
- 🔄 **Handles multi-line TODOs** by merging indented or docstring-style comments.
- 🚀 **Fast and efficient** using the [Pest](https://pest.rs/) parser.
- 📍 **Provides line numbers** for each TODO found.

---

## 📦 Installation

Clone the repository and build the project using Cargo:

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

## 🚀 Usage

### Command Line
Run the extractor by providing a path to a source file:

```sh
./todo-extractor path/to/source_file.rs
```

### Example Output
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

## 🔍 How It Works
### **1. Detects File Type**
The parser checks the file extension (`.py`, `.rs`, `.js`, `.ts`, `.go`) and selects the appropriate parser.

### **2. Extracts Comment Lines**
Using **Pest grammars**, the extractor retrieves comment lines while ignoring code and string literals.

### **3. Identifies TODO Markers**
The extractor searches for `TODO:` inside comment lines and captures the message.

### **4. Handles Multi-line TODOs**
If a TODO is followed by indented lines, they are merged into a single entry.

---

## 🛠️ Supported Languages
| Language | Single-line Syntax | Block Syntax |
|----------|-------------------|--------------|
| Python   | `# TODO: ...` | `""" TODO: ... """` |
| Rust     | `// TODO: ...` | `/* TODO: ... */` |
| JavaScript | `// TODO: ...` | `/* TODO: ... */` |
| TypeScript | `// TODO: ...` | `/* TODO: ... */` |
| Go       | `// TODO: ...` | `/* TODO: ... */` |

---

## 🧪 Running Tests
Run tests to validate the TODO extraction:
```sh
cargo test
```

Example test case for Rust files:
```rust
#[test]
fn test_rust_single_line() {
    let src = "// TODO: Refactor this code";
    let todos = extract_todos(Path::new("example.rs"), src);
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].message, "Refactor this code");
}
```

---

## 🤝 Contributing
Want to add support for another language? Contributions are welcome! Feel free to open an issue or submit a pull request.
