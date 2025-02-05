#!/usr/bin/env python3
import os
import click


def get_language_for_extension(file_ext):
    """
    Return the code fence language for a given file extension.
    Fall back to 'plaintext' if unknown.
    """
    # Common extensions mapped to GitHub-flavored Markdown languages
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
        # Add more as needed
    }

    return extension_map.get(file_ext.lower(), "plaintext")


def generate_tree(root_path):
    """
    Generate a textual tree representation of the folder structure
    starting at root_path.
    """
    # We want to return a list of lines that make up the tree.
    tree_lines = []

    def _tree(dir_path, prefix=""):
        # Get a sorted list of entries in the current directory
        entries = sorted(os.listdir(dir_path))
        # Remove hidden files/folders and .git directory if needed
        entries = [e for e in entries if not e.startswith('.') and e != '.git']

        # Iterate through each item and recurse if it's a directory
        for i, entry in enumerate(entries):
            full_path = os.path.join(dir_path, entry)
            # Check if this is the last item to adjust the prefix
            connector = "└── " if i == len(entries) - 1 else "├── "

            tree_lines.append(prefix + connector + entry)

            if os.path.isdir(full_path):
                # If it is not the last entry, we continue with "│   ", else "    "
                extension = "    " if i == len(entries) - 1 else "│   "
                _tree(full_path, prefix + extension)

    # Add the top-level directory itself to the tree
    root_basename = os.path.basename(os.path.abspath(root_path))
    if not root_basename:
        # If the root_path ends with a slash or is something like '/', fallback
        root_basename = root_path

    tree_lines.append(root_basename)
    _tree(root_path)
    return "\n".join(tree_lines)


@click.command()
@click.argument('root_path', type=click.Path(exists=True, file_okay=False, dir_okay=True))
@click.option('-o', '--output', 'output_file', default='merged_output.md', help='Output file path')
def main(root_path, output_file):
    if not os.path.isdir(root_path):
        print(f"Error: {root_path} is not a valid directory.")
        sys.exit(1)

    # 1. Generate the folder tree
    tree_output = generate_tree(root_path)

    # 2. Walk through all files in the directory (recursively)
    #    and collect their contents in the desired format.
    file_sections = []
    for dirpath, dirnames, filenames in os.walk(root_path):
        # Sort files and directories to have a consistent order
        dirnames.sort()
        filenames.sort()

        for filename in filenames:
            # Skip .git directory
            if '.git' in dirpath.split(os.sep):
                continue

            # Build relative path
            full_file_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(full_file_path, root_path)

            # Skip the output file itself if it's in the same directory
            if filename == os.path.basename(output_file):
                continue

            # Determine file extension
            _, ext = os.path.splitext(filename)
            language = get_language_for_extension(ext)

            # Read file content
            try:
                with open(full_file_path, "r", encoding="utf-8", errors="replace") as f:
                    content = f.read()
            except Exception as e:
                # If there's an error reading the file, skip or handle as needed
                print(f"Skipping file {rel_path} due to read error: {e}")
                continue

            # Prepare the file section
            file_header = f"## File: `{rel_path}`\n*(Relative Path: `{rel_path}`)*"

            # Wrap content in code fences
            fenced_content = f"```{language}\n{content}\n```"

            section = f"{file_header}\n\n{fenced_content}\n\n---\n"
            file_sections.append(section)

    # 3. Write everything to `merged_output.md`
    with open(output_file, "w", encoding="utf-8") as out:
        # Write the folder structure at the top
        out.write("# Folder Structure\n\n")
        out.write("```\n")
        out.write(tree_output)
        out.write("\n```\n\n")

        # Write each file's content
        for section in file_sections:
            out.write(section)

    print(f"All files have been merged into {output_file}")


if __name__ == "__main__":
    main()
