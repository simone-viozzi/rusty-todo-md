# **Rusty TODO.md** — A Pre-Commit Hook & CLI for Managing TODOs

[![PyPI - Version](https://img.shields.io/pypi/v/rusty-todo-md.svg)](https://pypi.org/project/rusty-todo-md/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/rusty-todo-md.svg)](https://pypi.org/project/rusty-todo-md/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Rusty TODO.md helps you **find, centralize, and maintain** all your `TODO` comments across your codebase.
It can run as a **[pre-commit](https://pre-commit.com/) hook** or from the **CLI**, automatically extracting TODO-style comments into a structured `TODO.md` file.

Supports a wide range of languages and file types, with **sectioned formatting**, **multi-line support**, and **smart sync**.

---

## 📌 Recommended usage: Pre-commit via shim repo

When `pre-commit` installs a hook from a Git repo, it runs `pip install .` from that repo — which would normally build Rusty TODO.md from source (requiring a Rust toolchain).

The **shim repository** ([`rusty-todo-md-pre-commit`](https://github.com/simone-viozzi/rusty-todo-md-pre-commit)) solves this by depending on the `rusty_todo_md` PyPI package, ensuring **prebuilt wheels** are used.

Add this to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md-pre-commit
    rev: v1.9.1  # Use the latest upstream tag (shim mirrors upstream)
    hooks:
      - id: rusty-todo-md
        args: ["--auto-add", "--markers", "TODO", "FIXME", "HACK", "--"]
        language_version: python3.11
```

- `args` customise the markers to scan for and enable `--auto-add` to stage `TODO.md` automatically.
- `language_version` forces the hook to run with a specific Python interpreter.

**Example with exclusions:**

```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md-pre-commit
    rev: v1.9.1
    hooks:
      - id: rusty-todo-md
        args:
          - "--auto-add"
          - "--markers"
          - "TODO"
          - "FIXME"
          - "HACK"
          - "--exclude-dir"
          - "node_modules"
          - "--exclude-dir"
          - "target"
          - "--exclude"
          - "**/*.test.js"
          - --
        language_version: python3.11
```

Then install the hook:

```sh
pre-commit install
```

> ✅ No Rust toolchain is required when using the shim and a supported platform.

---

## ⚙️ CLI installation

You can also install Rusty TODO.md directly for manual CLI use:

```sh
pip install rusty_todo_md
```

Then run:

```sh
rusty-todo-md --help
```

---

## ✨ Key Features

1. **Automatic TODO Collection**
   By default, Rusty TODO.md scans the files passed to it (typically staged files from pre-commit) for markers like `TODO` and `FIXME`, and updates your `TODO.md` with any new entries.

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
   Easily integrate Rusty TODO.md into your workflow by adding it to your `.pre-commit-config.yaml`.

7. **Auto-stage Updated TODO.md**
   With the `--auto-add` flag, the tool can automatically stage the `TODO.md` file after updates.

---

## 🧩 CLI usage

### Scan staged files
```sh
rusty-todo-md
```
### Use multiple markers
```sh
rusty-todo-md --markers TODO FIXME HACK
```

### Specify files to process with markers
When using `--markers` as the last option before specifying files, use `--` to separate markers from files:
```sh
rusty-todo-md --markers TODO FIXME HACK -- file1.rs file2.rs
```

Without the `--` separator, the files would be incorrectly treated as additional markers.

### Automatically stage TODO.md
```sh
rusty-todo-md --auto-add path/to/file.rs
```

### Custom TODO.md path
```sh
rusty-todo-md --todo-path docs/TODOS.md
```

### Exclude files and directories

Rusty TODO.md supports glob-based exclusion patterns to filter out files and directories from TODO extraction.

#### Exclude specific files or patterns
```sh
# Exclude all log files
rusty-todo-md --exclude "*.log"

# Exclude specific file
rusty-todo-md --exclude "config.rs"

# Exclude files in a specific directory
rusty-todo-md --exclude "src/generated/*"
```

#### Exclude directories
```sh
# Exclude a directory (and all files within it)
rusty-todo-md --exclude-dir "build"

# Or use --exclude with trailing slash
rusty-todo-md --exclude "build/"

# Exclude multiple directories
rusty-todo-md --exclude-dir "node_modules" --exclude-dir "target"
```

#### Recursive exclusion with wildcards
```sh
# Exclude all files under src/ recursively
rusty-todo-md --exclude "src/**"

# Exclude all .test.js files anywhere in the tree
rusty-todo-md --exclude "**/*.test.js"

# Exclude all 'vendor' directories at any depth
rusty-todo-md --exclude-dir "**/vendor"
```

#### Multiple exclusion patterns
```sh
# Combine multiple exclusions
rusty-todo-md \
  --exclude "*.log" \
  --exclude "*.tmp" \
  --exclude-dir "build" \
  --exclude-dir "dist" \
  --markers TODO FIXME
```

#### Glob pattern syntax
- `*` — matches any sequence of characters within a single path component
- `?` — matches any single character
- `**` — matches zero or more path components (recursive)
- `/` suffix — indicates directory-only matching (when used with `--exclude`)

> **Note:** Patterns are matched relative to the scan root. The `--exclude-dir` flag automatically ensures directory-only matching.

---

## 🔀 Rebase conflicts in TODO.md

Because each `TODO.md` bullet embeds the line number twice (`* [file:line](file#L<line>)`), branches that shift TODOs to different line numbers produce a Git conflict at every replayed commit during a rebase. The fix is a **custom git merge driver** that regenerates `TODO.md` from working-tree source instead of trying to text-merge the file.

### Default flow (no install)

Without the driver installed, `TODO.md` conflicts the way it always has. To rebuild it after a conflict:

```sh
rusty-todo-md --regenerate
```

This re-scans every tracked file and rewrites `TODO.md` from scratch, wiping any conflict markers. The tool also prints a one-line advisory whenever it detects `<<<<<<<` in `TODO.md`, pointing at `--regenerate` and `--install-merge-driver`.

### Install the merge driver (per-clone, once)

To eliminate this conflict shape entirely on a given clone:

```sh
rusty-todo-md --install-merge-driver
```

This writes two pieces of state, both local to the clone:

1. `merge.rusty-todo-md.name` and `merge.rusty-todo-md.driver` in `.git/config`.
2. `TODO.md merge=rusty-todo-md` appended to `.gitattributes` (committable).

The command prints exactly what it changed, plus the `git config --unset` lines to undo. Pass `--markers`, `--exclude`, `--exclude-dir`, or `--todo-path` to bake non-default settings into the driver command. After install, future rebases call `rusty-todo-md --merge-driver` for `TODO.md` and resolve cleanly without text-merging line numbers.

### Repo-maintainer opt-in (auto-install)

Maintainers who want every collaborator's clone to register the driver automatically can add the flag to pre-commit args:

```yaml
- id: rusty-todo-md
  args: ["--auto-add", "--auto-install-merge-driver", "--markers", "TODO", "FIXME", "--"]
```

On the first pre-commit run in each clone, the tool registers the driver and prints a loud summary to stdout describing exactly what was modified. On subsequent runs it's a no-op. The maintainer's PR adding the flag serves as team consent; the visible-on-first-run message ensures no collaborator is surprised.

### Failure mode

If a source file is in a conflicted state (contains `<<<<<<<`) when the merge driver runs, the extractor skips it with a stderr warning. `TODO.md` is then approximate until you resolve the source-file conflicts and re-run `rusty-todo-md --regenerate`.

---

## 📝 Supported languages & extensions

Rusty TODO.md detects comment syntax based on file extension:

| Language / Type    | Extensions                                       |
|--------------------|--------------------------------------------------|
| Python             | `py`                                             |
| Rust               | `rs`                                             |
| JavaScript / JSX   | `js`, `jsx`, `mjs`                                |
| TypeScript         | `ts`, `tsx`                                      |
| Java               | `java`                                           |
| C / C++ headers    | `cpp`, `hpp`, `cc`, `hh`                          |
| C#                 | `cs`                                             |
| Swift              | `swift`                                          |
| Kotlin             | `kt`, `kts`                                      |
| JSON               | `json`                                           |
| Go                 | `go`                                             |
| Shell              | `sh`                                             |
| YAML               | `yml`, `yaml`                                    |
| TOML               | `toml`                                           |
| Dockerfile         | `dockerfile`                                     |
| Markdown           | `md`                                             |

> Many extensions share the same parser (e.g., JS-style comment parsing for TS, Java, C-like languages).

---

## 🔍 Output format (stable)

Entries in `TODO.md` use this format:

```
* [path/to/file.ext:LINE](path/to/file.ext#L{LINE}): MESSAGE
```

This format is stable and designed for easy linking to code in hosted repos.

Example:

```
# TODO
## src/main.rs
* [src/main.rs:10](src/main.rs#L10): Refactor initialization logic
```

---

## 📦 Requirements & Supported Platforms

- **Python** ≥ 3.10
- **No Rust toolchain** needed if using the shim or PyPI wheels

Prebuilt wheels are published for:

| OS / libc         | Architectures                           |
|-------------------|------------------------------------------|
| **Linux (manylinux)** | `x86_64`, `x86`, `aarch64`, `armv7`, `ppc64le` |
| **Linux (musllinux)** | `x86_64`, `x86`, `aarch64`, `armv7`          |
| **Windows**           | `x64`, `x86`                               |
| **macOS**             | `x86_64` (macOS 15), `aarch64` (macOS 14)  |

---

## 🛠 Troubleshooting

- If no wheel is available for your platform, `pip` will try to build from source — which **requires a Rust toolchain**.
- If you encounter build errors, please:
  1. Check the [latest releases](https://github.com/simone-viozzi/rusty-todo-md/releases) to confirm wheel availability.
  2. Open an [issue](https://github.com/simone-viozzi/rusty-todo-md/issues) with your OS/arch details.

---

## 👩‍💻 Development

If you want to run Rusty TODO.md directly from the main repo via pre-commit (building from source):

```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md
    rev: v1.7.5
    hooks:
      - id: rusty-todo-md
```

> ⚠️ This will compile the Rust source and requires a working Rust toolchain.

---

## 🤝 Contributing

Contributions are welcome!
- Open an issue for bug reports or feature requests.
- Submit a pull request with improvements, new parsers, or fixes.

---

## 📚 Links

- **Shim repo** (recommended for pre-commit): [rusty-todo-md-pre-commit](https://github.com/simone-viozzi/rusty-todo-md-pre-commit)
- **PyPI** package: [rusty_todo_md](https://pypi.org/project/rusty-todo-md/)
- **Releases**: [GitHub Releases](https://github.com/simone-viozzi/rusty-todo-md/releases)

---

## ⚖️ License

Licensed under the [MIT License](LICENSE).

---

## ❤️ Support

If you find Rusty TODO.md helpful, please consider giving it a ⭐ on GitHub to help others discover the project.
