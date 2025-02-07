#!/usr/bin/env python3
import os
import sys
import click
from pathspec import PathSpec
from pathspec.patterns.gitwildmatch import GitWildMatchPattern


def get_gitignore_spec(root_path):
    """
    Parse the .gitignore file and return a PathSpec object for filtering.
    """
    gitignore_path = os.path.join(root_path, ".gitignore")
    if not os.path.exists(gitignore_path):
        return None

    with open(gitignore_path, "r", encoding="utf-8") as f:
        patterns = f.readlines()

    return PathSpec.from_lines(GitWildMatchPattern, patterns)


def get_language_for_extension(file_ext):
    """
    Return the code fence language for a given file extension.
    """
    extension_map = {
        ".py": "python",
        ".cpp": "cpp",
        ".cc": "cpp",
        ".cxx": "cpp",
        ".h": "cpp",
        ".hpp": "cpp",
        ".md": "markdown",
        ".js": "javascript",
        ".ts": "typescript",
        ".json": "json",
        ".yml": "yaml",
        ".yaml": "yaml",
        ".sh": "bash",
        ".bash": "bash",
        ".java": "java",
        ".cs": "csharp",
        ".rb": "ruby",
        ".go": "go",
        ".php": "php",
        ".html": "html",
        ".css": "css",
        ".txt": "plaintext",
        ".rs": "rust",
    }
    return extension_map.get(file_ext.lower(), "plaintext")


def generate_tree(root_path, gitignore_spec):
    """
    Generate a textual tree representation of the folder structure,
    excluding gitignored files.
    """
    tree_lines = []

    def _tree(dir_path, prefix=""):
        entries = sorted(os.listdir(dir_path))
        entries = [e for e in entries if not e.startswith(".") and e != ".git"]

        for i, entry in enumerate(entries):
            full_path = os.path.join(dir_path, entry)

            if gitignore_spec and gitignore_spec.match_file(
                os.path.relpath(full_path, root_path)
            ):
                continue

            connector = "└── " if i == len(entries) - 1 else "├── "
            tree_lines.append(prefix + connector + entry)

            if os.path.isdir(full_path):
                extension = "    " if i == len(entries) - 1 else "│   "
                _tree(full_path, prefix + extension)

    root_basename = os.path.basename(os.path.abspath(root_path)) or root_path
    tree_lines.append(root_basename)
    _tree(root_path)
    return "\n".join(tree_lines)


@click.command()
@click.argument(
    "root_path", type=click.Path(exists=True, file_okay=False, dir_okay=True)
)
@click.option(
    "-o", "--output", "output_file", default="merged_output.md", help="Output file path"
)
def main(root_path, output_file):
    if not os.path.isdir(root_path):
        print(f"Error: {root_path} is not a valid directory.")
        sys.exit(1)

    gitignore_spec = get_gitignore_spec(root_path)
    tree_output = generate_tree(root_path, gitignore_spec)
    file_sections = []

    for dirpath, dirnames, filenames in os.walk(root_path):
        dirnames[:] = [
            d
            for d in dirnames
            if not (
                gitignore_spec
                and gitignore_spec.match_file(
                    os.path.relpath(os.path.join(dirpath, d), root_path)
                )
            )
        ]
        filenames[:] = [
            f
            for f in filenames
            if not (
                gitignore_spec
                and gitignore_spec.match_file(
                    os.path.relpath(os.path.join(dirpath, f), root_path)
                )
            )
        ]

        for filename in filenames:
            full_file_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(full_file_path, root_path)

            if filename == os.path.basename(output_file):
                continue

            _, ext = os.path.splitext(filename)
            language = get_language_for_extension(ext)

            try:
                with open(full_file_path, "r", encoding="utf-8", errors="replace") as f:
                    content = f.read()
            except Exception as e:
                print(f"Skipping file {rel_path} due to read error: {e}")
                continue

            file_header = f"## File: `{rel_path}`\n*(Relative Path: `{rel_path}`)*"
            fenced_content = f"```{language}\n{content}\n```"
            section = f"{file_header}\n\n{fenced_content}\n\n---\n"
            file_sections.append(section)

    with open(output_file, "w", encoding="utf-8") as out:
        out.write("# Folder Structure\n\n")
        out.write("```")
        out.write(tree_output)
        out.write("\n```\n\n")
        for section in file_sections:
            out.write(section)

    print(f"All files have been merged into {output_file}")


if __name__ == "__main__":
    main()
