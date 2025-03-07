# **Rusty TODO MD** — A Pre-Commit Hook for Managing TODOs

Rusty TODO MD is a **pre-commit hook** designed to help you keep track of all your code's **TODO** comments by automatically extracting them and syncing them to a central **`TODO.md`** file. It uses custom parsers to accurately find and handle multi-line TODOs in supported languages, ensuring that TODO markers stay organized and visible.

---

## ✨ **Key Features**
1. **Automatic TODO Collection**  
   By default, Rusty TODO MD scans only your **staged** files for `TODO` (or other markers like `FIXME`), then updates `TODO.md` with any new entries.
   
2. **Selective or Full-Project Scan**  
   - **Selective Scan** (default): Only fetches changes from **staged** files.
   - **Full-Project Scan**: Use the `--all-files` flag to scan the entire repository at once.

3. **Multi-line TODO Support**  
   - Properly merges **indented** lines under a single TODO block (e.g., continuing on new lines).

4. **Sync Mechanism**  
   - Entries in `TODO.md` reflect your code’s `TODO` lines (with file path and line number).  
   - If a TODO is removed from the code, it will be removed from `TODO.md` in subsequent runs (assuming it's no longer present in newly staged or scanned files).

5. **Language-Aware Parsing**  
   Currently supports **Python** and **Rust**. JavaScript, TypeScript, and Go are partially supported or planned.

6. **Easy Integration with Pre-Commit**  
   Just add a snippet to your `.pre-commit-config.yaml` to start tracking your TODOs effortlessly.

---

## 🚀 **Objective**

**Why use Rusty TODO MD?**  
Keeping a scattered collection of `TODO` comments in code can quickly become unmanageable. Rusty TODO MD streamlines the process by **centralizing** all TODO notes into a single `TODO.md` file. This makes it easy for teams to monitor outstanding tasks, track them through commits, and keep documentation in sync with the actual codebase.

---

## ⚙️ **Installation & Setup**

### 1. **Install Pre-Commit**
If you don’t already have [pre-commit](https://pre-commit.com/) installed:
```sh
pip install pre-commit
```
*(Or use your preferred package manager.)*

### 2. **Add Rusty TODO MD to `.pre-commit-config.yaml`**
In your repository, create or update a `.pre-commit-config.yaml` file at the root:
```yaml
repos:
  - repo: https://github.com/your-username/rusty-todo-md
    rev: v0.1.8-alpha.11
    hooks:
      - id: rusty-todo-md
```
Replace `rev` with the actual tag or commit hash you’d like to use.

### 3. **Install the Pre-Commit Hook**
From your repository root:
```sh
pre-commit install
```
Now, whenever you `git commit`, Rusty TODO MD will run on staged files.

---

## 🧩 **Usage as a Pre-Commit Hook**

1. **Stage Your Changes**  
   ```sh
   git add <files...>
   ```
2. **Commit**  
   ```sh
   git commit -m "feat: add new feature"
   ```
3. **Observe TODO.md**  
   - Any `TODO` or `FIXME` lines (in code comments) that were part of your **staged** changes will be extracted.
   - Rusty TODO MD updates (or creates) the `TODO.md` file with the new entries.

> **Tip**: If `TODO.md` changes were made during the pre-commit hook, you’ll need to re-stage `TODO.md` (if the hook is configured to fail on changes). This ensures your commit accurately captures the updated `TODO.md`.

---

## 🖥️ **CLI Usage (Optional)**

You can also run `rusty-todo-md` manually from the command line:

```bash
rusty-todo-md --all-files
```
- **`--all-files`**: Scans **all tracked files** in your repo instead of just staged ones, updating `TODO.md` accordingly.

Or to specify a custom `TODO.md` location:
```bash
rusty-todo-md --todo-path docs/TODOS.md
```

---

## 📝 **How It Works**

1. **File Discovery**  
   - **Default**: Rusty TODO MD looks at the **staged files** in your Git index.  
   - **`--all-files`**: Rusty TODO MD collects **all tracked files** from your repo.

2. **Comment Extraction**  
   - It parses each file’s **comments** (not code/string literals) and looks for **markers** (e.g., `TODO`, `FIXME`).
   - Handles **multi-line** and **indented** comment structures, merging them into one block.

3. **Write/Sync `TODO.md`**  
   - Each TODO is recorded in the format:  
     ```
     * [path/to/file:line_number](path/to/file#Lline_number): TODO message
     ```
   - If you remove a TODO from the source code, that entry disappears from `TODO.md` after the next run (provided you are scanning or staging those file changes).

---

## 🔧 **Configuration**

- **Markers**: By default, only `TODO` is searched. You can customize markers (like `FIXME`, `HACK`, etc.) by modifying the source or through future CLI options (work in progress).  
- **File Extensions**: Rusty TODO MD currently includes specialized grammars for **Python** and **Rust**, with partial or upcoming support for JS, TS, and Go. Feel free to open a PR to add more languages.

---

## 📚 **Example**

### **Python**

```python
# TODO: Implement data validation
def process_data(data):
    """
    FIXME: Optimize this logic
        Possibly reduce nested loops
    """
    pass
```

**In `TODO.md`**:
```
* [path/to/your_file.py:2](path/to/your_file.py#L2): Implement data validation
* [path/to/your_file.py:4](path/to/your_file.py#L4): Optimize this logic Possibly reduce nested loops
```

### **Rust**

```rust
// TODO: Refactor main function
fn main() {
    /*
       FIXME: Add error handling
           Possibly a custom result type
    */
}
```

**In `TODO.md`**:
```
* [src/main.rs:2](src/main.rs#L2): Refactor main function
* [src/main.rs:5](src/main.rs#L5): Add error handling Possibly a custom result type
```

---

## 🤝 **Contributing**

1. **Open an Issue** if you find a bug or want a new feature.
2. **Submit a Pull Request** to add support for a new language, marker, or enhancement.

We welcome all contributions, from docs to code changes!

---

## ⚖️ **License**

This project is licensed under the [MIT License](LICENSE). You’re free to use, modify, and distribute it, as long as the original license is included.

---

## ❤️ **Support**

If you find Rusty TODO MD helpful, a ⭐️ on the repository is always appreciated! It helps others discover the project and shows your support.

---

Happy coding, and let Rusty TODO MD handle your TODOs so you can focus on building amazing features!