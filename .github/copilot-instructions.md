# Rusty TODO MD - Copilot Instructions

Rusty TODO MD is a multi-language TODO comment extractor written in Rust that automatically scans source code files for TODO/FIXME/HACK comments and maintains them in a centralized TODO.md file. It's designed as a pre-commit hook and supports Python, Rust, JavaScript, Go, and other languages.

**ALWAYS reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Working Effectively

### Bootstrap and Build the Repository
- Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source ~/.cargo/env`
- **Build debug version**: `cargo build` -- takes 2-4 minutes on first run. **NEVER CANCEL.** Set timeout to 300+ seconds.
- **Build release version**: `cargo build --release` -- takes 1-3 minutes. **NEVER CANCEL.** Set timeout to 300+ seconds.

### Testing and Validation
- **Run all tests**: `cargo test` -- takes 2-5 minutes with dependency downloads. **NEVER CANCEL.** Set timeout to 300+ seconds.
- **Generate coverage report**: `cargo tarpaulin --out Lcov --output-dir ./coverage` -- takes 4-6 minutes. **NEVER CANCEL.** Set timeout to 360+ seconds.
- Coverage reports are generated in `./coverage/lcov.info` for VS Code integration

### Code Quality and Linting
- **Format code**: `cargo fmt` -- very fast, ~1 second
- **Check formatting**: `cargo fmt --check` -- very fast, ~1 second
- **Run linter**: `cargo clippy --all-targets --all-features -- -D warnings` -- takes 20-45 seconds
- **Note**: Currently has clippy warnings about format string inlining that need fixing

### Python Package Development
- **Install maturin**: `pip install maturin`
- **Build Python wheel**: `maturin build` -- takes 1-2 minutes. **NEVER CANCEL.** Set timeout to 180+ seconds.
- **Build and install locally**: `maturin develop`
- Generated wheels are in `target/wheels/`

### Pre-commit Hooks
- **Install pre-commit**: `pip install pre-commit`
- **Install hooks**: `pre-commit install`
- **Run all hooks**: `pre-commit run --all-files` -- takes 2-3 minutes. **NEVER CANCEL.** Set timeout to 300+ seconds.
- **Note**: Currently fails due to clippy warnings that need to be fixed

## Validation Scenarios

**ALWAYS test these scenarios after making changes to ensure functionality works correctly:**

### CLI Functionality Test
```bash
# Create test scenario
mkdir -p /tmp/test-scenario && cd /tmp/test-scenario

# Create test files with TODO comments
cat > sample.rs << 'EOF'
// TODO: Implement user authentication
fn main() {
    // FIXME: Handle error cases properly
    println!("Hello, world!");

    /* TODO: Add logging functionality
       for better debugging */
}
EOF

cat > sample.py << 'EOF'
# TODO: Add comprehensive error handling
def main():
    """
    FIXME: This function needs proper documentation
    """
    print("Python sample")

    # HACK: Using hardcoded values for now
    return {"sample": "data"}
EOF

# Initialize git repo
git init && git add . && git config user.email "test@example.com" && git config user.name "Test User" && git commit -m "Initial commit"

# Test CLI functionality
path/to/rusty-todo-md --markers TODO FIXME HACK --todo-path TODO.md sample.rs sample.py

# Verify TODO.md was created with expected sections
cat TODO.md
```

### Expected TODO.md Output Format
The tool should generate a TODO.md file with sections organized by marker type, then by file:
```markdown
# FIXME
## sample.py
* [sample.py:4](sample.py#L4): This function needs proper documentation

## sample.rs
* [sample.rs:5](sample.rs#L5): Handle error cases properly

# HACK
## sample.py
* [sample.py:14](sample.py#L14): Using hardcoded values for now

# TODO
## sample.py
* [sample.py:1](sample.py#L1): Add comprehensive error handling

## sample.rs
* [sample.rs:1](sample.rs#L1): Implement user authentication
* [sample.rs:9](sample.rs#L9): Add logging functionality for better debugging
```

### Pre-commit Integration Test
After changes, always test that the tool works as a pre-commit hook:
```bash
# In a git repository with modified files
pre-commit run rusty-todo-md
```

## Common Tasks

### Repository Structure
```
.
├── .github/workflows/     # CI/CD pipelines (ci.yaml, release.yml)
├── .gitignore
├── .pre-commit-config.yaml  # Pre-commit configuration
├── Cargo.toml             # Rust project configuration
├── LICENSE
├── README.md              # Comprehensive project documentation
├── TODO.md                # Generated TODO file (example)
├── bump-version.sh        # Version management script
├── pyproject.toml         # Python packaging configuration
├── shell.nix              # Nix development environment
├── src/                   # Rust source code
│   ├── main.rs
│   ├── cli.rs
│   ├── git_utils.rs
│   ├── todo_extractor.rs
│   ├── todo_md.rs
│   └── todo_extractor_internal/  # Language-specific parsers
└── tests/                 # Integration and unit tests
```

### Key Source Files
- **`src/main.rs`**: Entry point, sets up logging and calls CLI
- **`src/cli.rs`**: Command-line argument parsing and main workflow
- **`src/todo_extractor.rs`**: Core TODO extraction logic
- **`src/todo_md.rs`**: TODO.md file parsing and generation
- **`src/git_utils.rs`**: Git integration for staged/tracked files
- **`src/todo_extractor_internal/`**: Language-specific comment parsers

### CLI Usage Patterns
```bash
# Process specific files (typically called by pre-commit)
rusty-todo-md file1.rs file2.py

# Use custom markers
rusty-todo-md --markers TODO FIXME HACK file1.rs

# Specify custom TODO file location
rusty-todo-md --todo-path docs/TODOS.md file1.rs

# Debug with logging
RUST_LOG=debug rusty-todo-md file1.rs
```

### Development Environment Setup (Optional - Nix)
```bash
# Enter nix shell for reproducible development environment
nix-shell shell.nix
```

## CI/CD Integration

### GitHub Actions Workflows
- **`ci.yaml`**: Runs on pull requests, tests against stable/beta/nightly Rust
- **`release.yml`**: Complex release workflow that builds cross-platform binaries

### Release Process
The project uses `cargo-release` for automated releases:
```bash
# Trigger release via GitHub Actions workflow_dispatch
# This creates release branch, builds cross-platform wheels, and publishes to PyPI
```

## Troubleshooting

### Common Issues
1. **Git repository errors**: The tool requires a valid git repository with at least one commit
2. **File argument confusion**: CLI expects explicit file arguments, not git staging area discovery
3. **Clippy warnings**: Currently has format string style warnings that need fixing
4. **Permission errors**: On some systems, test files may need proper permissions

### Debug Commands
```bash
# Enable debug logging
export RUST_LOG=debug

# Check git repository status
git status

# Verify tool help
rusty-todo-md --help

# Test with single file
rusty-todo-md --markers TODO sample.rs
```

### Performance Expectations
- **Builds**: 2-4 minutes (first time), 1-2 minutes (subsequent)
- **Tests**: 2-5 minutes (with downloads), ~3 seconds (without)
- **Coverage**: 4-6 minutes (**NEVER CANCEL**)
- **Linting**: 20-45 seconds
- **Python packaging**: 1-2 minutes

**CRITICAL: Always wait for completion of long-running operations. Set appropriate timeouts and never cancel builds, tests, or coverage generation prematurely.**
