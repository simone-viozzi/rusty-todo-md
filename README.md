# **Rusty TODO MD** — A Pre-Commit Hook & CLI for Managing TODOs

[![PyPI - Version](https://img.shields.io/pypi/v/rusty-todo-md.svg)](https://pypi.org/project/rusty-todo-md/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/rusty-todo-md.svg)](https://pypi.org/project/rusty-todo-md/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Rusty TODO MD helps you **find, centralize, and maintain** all your `TODO` comments across your codebase.
It can run as a **[pre-commit](https://pre-commit.com/) hook** or from the **CLI**, automatically extracting TODO-style comments into a structured `TODO.md` file.

Supports a wide range of languages and file types, with **sectioned formatting**, **multi-line support**, and **smart sync**.

---

## 📌 Recommended usage: Pre-commit via shim repo

When `pre-commit` installs a hook from a Git repo, it runs `pip install .` from that repo — which would normally build Rusty TODO MD from source (requiring a Rust toolchain).

The **shim repository** ([`rusty-todo-md-pre-commit`](https://github.com/simone-viozzi/rusty-todo-md-pre-commit)) solves this by depending on the `rusty_todo_md` PyPI package, ensuring **prebuilt wheels** are used.

Add this to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/simone-viozzi/rusty-todo-md-pre-commit
    rev: v1.7.5  # Use the latest upstream tag (shim mirrors upstream)
    hooks:
      - id: rusty-todo-md
```

Then install the hook:

```sh
pre-commit install
```

> ✅ No Rust toolchain is required when using the shim and a supported platform.

---

## ⚙️ CLI installation

You can also install Rusty TODO MD directly for manual CLI use:

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

## 🧩 CLI usage

### Scan staged files
```sh
rusty-todo-md
```

### Scan all tracked files
```sh
rusty-todo-md --all-files
```

### Use multiple markers
```sh
rusty-todo-md --markers TODO FIXME HACK
```

### Custom TODO.md path
```sh
rusty-todo-md --todo-path docs/TODOS.md
```

---

## 📝 Supported languages & extensions

Rusty TODO MD detects comment syntax based on file extension:

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
| **macOS**             | `x86_64` (macOS 13), `aarch64` (macOS 14)  |

---

## 🛠 Troubleshooting

- If no wheel is available for your platform, `pip` will try to build from source — which **requires a Rust toolchain**.
- If you encounter build errors, please:
  1. Check the [latest releases](https://github.com/simone-viozzi/rusty-todo-md/releases) to confirm wheel availability.
  2. Open an [issue](https://github.com/simone-viozzi/rusty-todo-md/issues) with your OS/arch details.

---

## 👩‍💻 Development

If you want to run Rusty TODO MD directly from the main repo via pre-commit (building from source):

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

If you find Rusty TODO MD helpful, please consider giving it a ⭐ on GitHub to help others discover the project.
