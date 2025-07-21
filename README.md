# **Rusty TODO MD** — A Pre-Commit Hook for Managing TODOs

Rusty TODO MD is a **pre-commit hook** designed to help you manage and centralize all your code's **TODO** comments by automatically extracting them and synchronizing them into a single **`TODO.md`** file. With support for multiple languages—currently **Python**, **Rust**, **JavaScript**, and **Go**—Rusty TODO MD now organizes TODOs using a new sectioned format for enhanced readability and easier maintenance.

---

## ✨ Key Features

1. **Automatic TODO Collection**  
   By default, Rusty TODO MD scans your **staged files** (or all tracked files using the `--all-files` flag) for markers like `TODO` and `FIXME`, and updates your `TODO.md` with any new entries.

2. **Sectioned TODO.md Format**  
   The `TODO.md` file is now organized into sections, grouped first by marker (e.g., `# TODO`, `# FIXME`), then by file. Each marker section begins with a header (`# <MARKER>`), each file with a sub-header (`## <file-path>`), followed by a list of extracted TODO items.

3. **Multi-line TODO Support**  
   Handles multi-line and indented TODO comments, merging them into a single entry.

4. **Sync Mechanism**  
   - Automatically merges new TODO entries with existing ones, using an internal representation.
   - Removes entries when their corresponding TODOs are no longer present in the source code.

5. **Language-Aware Parsing**  
   Supports precise parsing for **Python**, **Rust**, **JavaScript**, and **Go** out-of-the-box, with plans for additional languages such as TypeScript, PHP, and Java.

6. **Seamless Pre-Commit Integration**  
   Easily integrate Rusty TODO MD into your workflow by adding it to your `.pre-commit-config.yaml`.

---

## 🚀 Objective

Scattered TODO comments can be hard to track and maintain. Rusty TODO MD centralizes these comments in a structured, sectioned `TODO.md` file—making it simpler to review outstanding tasks and keep your documentation in sync with your codebase.

---

## ⚙️ Installation & Setup

### Option 1: PyPI Installation (Recommended)

Install directly from PyPI using pip:
```sh
pip install rusty_todo_md
```

Then you can use it directly:
```sh
rusty-todo-md --help
```

### Option 2: Pre-Commit Hook Integration

#### With PyPI (Recommended - No Rust toolchain required)
Add the following to your `.pre-commit-config.yaml`:
```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md
    rev: v1.1.0  # Use the latest version
    hooks:
      - id: rusty-todo-md
        language: python
        additional_dependencies: ["rusty_todo_md==1.1.0"]
```

#### With Git Repository
Add the following snippet to your `.pre-commit-config.yaml` file at the root of your repository:
```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md
    rev: v1.1.0  # Use the latest version
    hooks:
      - id: rusty-todo-md
```

### 1. Install Pre-Commit
If you haven't already installed [pre-commit](https://pre-commit.com/):
```sh
pip install pre-commit
```
*(Alternatively, use your preferred package manager.)*

### 2. Configure Pre-Commit
Add the following snippet to your `.pre-commit-config.yaml` file at the root of your repository:
```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md
    rev: v0.1.8-alpha.11
    hooks:
      - id: rusty-todo-md
```
Replace `rev` with the desired tag or commit hash.

### 3. Install the Pre-Commit Hook
Run the following command from your repository root:
```sh
pre-commit install
```
Rusty TODO MD will now run on your staged files each time you commit.

---

## 🧩 CLI Usage (Optional)

You can also run Rusty TODO MD manually:
```sh
rusty-todo-md --all-files
```
- **`--all-files`**: Scans all tracked files instead of just the staged ones.
- **`--marker`/`-m`**: Specify one or more keywords to search for (e.g., TODO, FIXME, HACK). Can be used multiple times.

Example:
```sh
rusty-todo-md --markers TODO FIXME HACK
```

Or specify a custom location for your TODO file:
```sh
rusty-todo-md --todo-path docs/TODOS.md
```

---

## 📝 How It Works

1. **File Discovery**  
   - By default, scans the staged files in your Git index.
   - With the `--all-files` flag, scans all tracked files in the repository.

2. **Comment Extraction**  
   - Parses file comments (ignoring code or string literals) to extract markers like `TODO` and `FIXME`.
   - Supports multi-line and indented comment structures.

3. **Sectioned TODO.md Format**  
   - Organizes extracted TODOs into sections grouped by marker and file.
   - Each marker section starts with a header (`# <MARKER>`), each file with a sub-header (`## <file-path>`), followed by formatted entries:
     ```
     * [<file-path>:<line_number>](<file-path>#L<line_number>): TODO message
     ```

4. **Sync Mechanism**  
   - Reads the existing `TODO.md` using the new parser.
   - Merges new TODO entries with the existing ones using an internal representation.
   - Writes the updated, sectioned list back to `TODO.md`.

---

## 🔧 Configuration

- **Markers**: The tool searches for `TODO` by default. You can customize markers (e.g., `FIXME`, `HACK`) using the `--marker`/`-m` CLI argument. Multiple markers are supported and can be specified multiple times.
- **Language Support**: Rusty TODO MD provides built-in parsing for **Python** (`.py`), **Rust** (`.rs`), **JavaScript** (`.js`, `.jsx`), and **Go** (`.go`), with planned support for additional languages.

---

## 📊 Test Coverage

Rusty TODO MD maintains comprehensive test coverage to ensure reliability and code quality.

### CI/CD Coverage

- **Automated Coverage**: Every pull request automatically generates coverage reports
- **Coverage Reports**: Available as downloadable artifacts from GitHub Actions
- **Format**: Reports are generated in LCOV format for broad tool compatibility

### VS Code Integration

For the best local development experience:

1. Install the [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) extension
2. Run `cargo tarpaulin --out Lcov --output-dir ./coverage`
3. Open your project in VS Code to see coverage highlights directly in your source files

---

## 📚 Example

### Python Example
```python
# TODO: Implement data validation
def process_data(data):
    """
    FIXME: Optimize this logic
        Possibly reduce nested loops
    """
    pass
```
This produces a section in `TODO.md` like:
```
# TODO
## path/to/your_file.py
* [path/to/your_file.py:2](path/to/your_file.py#L2): Implement data validation

# FIXME
## path/to/your_file.py
* [path/to/your_file.py:4](path/to/your_file.py#L4): Optimize this logic Possibly reduce nested loops
```

### Rust Example
```rust
// TODO: Refactor main function
fn main() {
    /*
       FIXME: Add error handling
           Possibly a custom result type
    */
}
```
This produces a section in `TODO.md` like:
```
# TODO
## src/main.rs
* [src/main.rs:2](src/main.rs#L2): Refactor main function

# FIXME
## src/main.rs
* [src/main.rs:5](src/main.rs#L5): Add error handling Possibly a custom result type
```

### JavaScript Example
```javascript
// TODO: Refactor this into separate modules
function init() {
  /* FIXME: Handle edge cases 
     such as null responses */
  fetchData();
}
```
This produces a section in `TODO.md` like:
```
# TODO
## src/app.js
* [src/app.js:1](src/app.js#L1): Refactor this into separate modules

# FIXME
## src/app.js
* [src/app.js:4](src/app.js#L4): Handle edge cases such as null responses
```

### Go Example
```go
// TODO: Add proper logging
func main() {
    /* FIXME: Implement proper error handling
       across the entire application */
    fmt.Println("Hello, World!")
}
```
This produces a section in `TODO.md` like:
```
# TODO
## main.go
* [main.go:1](main.go#L1): Add proper logging

# FIXME
## main.go
* [main.go:3](main.go#L3): Implement proper error handling across the entire application
```

---

## 🤝 Contributing

Contributions are welcome!  
- **Open an Issue** for bug reports or feature requests.
- **Submit a Pull Request** with enhancements, additional language support, or configuration options.

---

## ⚖️ License

This project is licensed under the [MIT License](LICENSE). You are free to use, modify, and distribute the software as long as the original license is included.

---

## ❤️ Support

If you find Rusty TODO MD helpful, please consider giving it a ⭐ on GitHub to help others discover the project.

---

Happy coding, and let Rusty TODO MD handle your TODOs so you can focus on building amazing features!
