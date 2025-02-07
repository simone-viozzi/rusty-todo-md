#!/usr/bin/env python3
import os
import sys
import click
from pathspec import PathSpec
from pathspec.patterns.gitwildmatch import GitWildMatchPattern


def collect_gitignore_patterns(root_path):
    """
    Recursively walk through the directory tree starting at root_path,
    collect all .gitignore files, and merge their patterns into a single PathSpec.
    """
    all_patterns = [".git/"]
    for dirpath, dirnames, filenames in os.walk(root_path):
        if ".gitignore" in filenames:
            gitignore_path = os.path.join(dirpath, ".gitignore")
            try:
                with open(gitignore_path, "r", encoding="utf-8") as f:
                    patterns = f.read().splitlines()
                # We use GitWildMatchPattern so that .gitignore style patterns are respected
                all_patterns.extend(patterns)
            except Exception as e:
                print(f"Warning: Could not read {gitignore_path} due to error: {e}", file=sys.stderr)

    if not all_patterns:
        return None

    return PathSpec.from_lines(GitWildMatchPattern, all_patterns)


def get_language_for_extension(file_ext):
    """
    Return the code fence language for a given file extension.
    If not recognized, return None (so we know to skip).
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
        ".toml": "toml",
        ".xml": "xml",
        ".kt": "kotlin",
        ".swift": "swift",
        ".tf": "hcl",
        ".lua": "lua",
        ".dockerfile": "dockerfile",
        ".pest": "pest",
        ".csv": "csv",
        ".ini": "ini",
        ".ijs": "jslang",
    }
    # Special-case Dockerfile (if the file name is literally "Dockerfile")
    if file_ext == "" and "Dockerfile" in extension_map:
        return extension_map[".dockerfile"]

    return extension_map.get(file_ext.lower(), None)


def generate_tree(root_path, gitignore_spec):
    """
    Generate a textual tree representation of the folder structure,
    excluding files/dirs matched by the merged .gitignore patterns.
    """
    tree_lines = []

    def _tree(dir_path, prefix=""):
        try:
            entries = sorted(os.listdir(dir_path))
        except OSError as e:
            # If we cannot list the directory, just return
            print(f"Warning: cannot list {dir_path} due to error: {e}", file=sys.stderr)
            return

        for i, entry in enumerate(entries):
            full_path = os.path.join(dir_path, entry)
            rel_path = os.path.relpath(full_path, root_path)

            # If matched by .gitignore, skip
            if gitignore_spec and gitignore_spec.match_file(rel_path):
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


def collect_files_content(root_path, gitignore_spec, output_file):
    """
    Walk the directory structure, respecting gitignore, and collect
    file contents or note them as unrecognized.
    """
    file_sections = []
    unrecognized_files = []

    for dirpath, dirnames, filenames in os.walk(root_path):
        # Filter out directories ignored by .gitignore
        new_dirnames = []
        for d in dirnames:
            rel_path = os.path.relpath(os.path.join(dirpath, d), root_path)
            if not (gitignore_spec and gitignore_spec.match_file(rel_path)):
                new_dirnames.append(d)
        dirnames[:] = new_dirnames

        # Process files
        for filename in filenames:
            full_file_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(full_file_path, root_path)

            # Skip if matched by gitignore
            if gitignore_spec and gitignore_spec.match_file(rel_path):
                continue

            # Skip the output file itself if it appears in the tree
            if filename == os.path.basename(output_file):
                continue

            # Check extension for recognized language
            _, ext = os.path.splitext(filename)
            language = get_language_for_extension(ext)

            if language:
                # Attempt to read the file content
                try:
                    with open(full_file_path, "r", encoding="utf-8", errors="replace") as f:
                        content = f.read()
                except Exception as e:
                    print(f"Skipping file {rel_path} due to read error: {e}", file=sys.stderr)
                    continue

                file_header = f"## File: `{rel_path}`\n*(Relative Path: `{rel_path}`)*"
                fenced_content = f"```{language}\n{content}\n```"
                section = f"{file_header}\n\n{fenced_content}\n\n---\n"
                file_sections.append(section)
            else:
                # Unrecognized extension — we won't read content
                unrecognized_files.append(rel_path)

    return file_sections, unrecognized_files


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

    # Collect all gitignore patterns from the entire repository tree
    gitignore_spec = collect_gitignore_patterns(root_path)

    # Generate a tree representation
    tree_output = generate_tree(root_path, gitignore_spec)

    # Collect file contents and unrecognized files
    file_sections, unrecognized_files = collect_files_content(root_path, gitignore_spec, output_file)

    # Write everything to the output file
    with open(output_file, "w", encoding="utf-8") as out:
        # Print the folder structure
        out.write("# Folder Structure\n\n")
        out.write("```\n")
        out.write(tree_output)
        out.write("\n```\n\n")

        # Print recognized file contents
        for section in file_sections:
            out.write(section)

        # Print unrecognized files
        if unrecognized_files:
            out.write("# Unrecognized Files\n\n")
            out.write(
                "The following files have extensions not recognized by the script, "
                "so their contents were *not* included:\n\n"
            )
            for rel_path in unrecognized_files:
                out.write(f"- `{rel_path}`\n")

    print(f"All files have been merged (where recognized) into {output_file}")
    if unrecognized_files:
        print("Some files were not recognized by extension and were skipped. See 'Unrecognized Files' section.")


if __name__ == "__main__":
    main()
