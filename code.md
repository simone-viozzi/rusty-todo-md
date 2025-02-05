# Folder Structure

```
src
├── aggregator.rs
├── languages
│   ├── go.pest
│   ├── go.rs
│   ├── js.pest
│   ├── js.rs
│   ├── mod.rs
│   ├── python.pest
│   ├── python.rs
│   ├── rust.pest
│   ├── rust.rs
│   ├── ts.pest
│   └── ts.rs
├── lib.rs
└── main.rs
```

## File: `aggregator.rs`
*(Relative Path: `aggregator.rs`)*

```rust
use std::path::Path;
use crate::languages::{
    python::parse_python_comments,
    rust::parse_rust_comments,
    go::parse_go_comments,
    js::parse_js_comments,
    ts::parse_ts_comments,
};

/// Represents a single found TODO item.
#[derive(Debug, PartialEq)]
pub struct TodoItem {
    pub line_number: usize,
    pub message: String,
}

/// Detects file extension and chooses the parser to gather raw comment lines,
/// then extracts multi-line TODOs from those comments.
pub fn extract_todos(path: &Path, file_content: &str) -> Vec<TodoItem> {
    // 1. Identify which language parser to use based on extension
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let comment_lines = match extension.as_str() {
        "py" => parse_python_comments(file_content),
        "rs" => parse_rust_comments(file_content),
        "go" => parse_go_comments(file_content),
        "js" => parse_js_comments(file_content),
        "ts" => parse_ts_comments(file_content),
        _ => {
            // fallback: no recognized extension => no comments
            vec![]
        }
    };

    // 2. Now scan the comment lines for "TODO:" occurrences
    collect_todos_from_comment_lines(&comment_lines)
}

/// A single comment line with (line_number, entire_comment_text).
/// We'll store each line separately even if it's from a block comment, so
/// we can handle multi-line merges (for block comments or consecutive single-line).
#[derive(Debug)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge multi-line TODO lines and produce `TodoItem` for each distinct `TODO:`.
///
/// - If a single comment line contains a `TODO:`, record that line_number
///   and parse everything after `TODO:`. Also see if subsequent lines remain part
///   of the same "comment block" that started with `TODO:` (like a multi-line
///   block comment or consecutive single-line lines that appear to continue).
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    let mut result = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        let text = &lines[idx].text;
        if let Some(pos) = text.find("TODO:") {
            // The line with "TODO:"
            let line_num = lines[idx].line_number;
            // Extract everything *after* "TODO:"
            let after_todo = &text[pos + 5..]; // 5 = len("TODO:")
            // Trim leading spaces
            let mut collected = after_todo.trim_start().to_string();

            // Move to next line(s) if they appear to be "continuations"
            // For single-line comments, we can check if next lines are adjacent or part of same block.
            // We'll do a simple approach: if it's from the same block comment OR consecutive single-line,
            // we keep merging while there's indentation or content.

            idx += 1; // move to next line
            while idx < lines.len() {
                // Heuristic: if the next line is from the *same* block comment (or consecutive single-line),
                // we might want to keep merging. We'll do a simpler approach: if it's the same "group"
                // or it starts with some indentation => keep going. This is language-specific, so adapt as needed.
                let next_text = &lines[idx].text;
                // We'll do a naive approach: if next_text starts with space or is empty, consider it a continuation.
                if next_text.starts_with(' ') || next_text.starts_with('\t') {
                    collected.push(' ');
                    collected.push_str(next_text.trim());
                    idx += 1;
                } else {
                    break;
                }
            }

            // Store result
            result.push(TodoItem {
                line_number: line_num,
                message: collected.trim_end().to_string(),
            });
        } else {
            idx += 1;
        }
    }

    result
}

```

---
## File: `lib.rs`
*(Relative Path: `lib.rs`)*

```rust
pub mod aggregator;
pub mod languages;

pub use aggregator::extract_todos;

```

---
## File: `main.rs`
*(Relative Path: `main.rs`)*

```rust
fn main() {
    println!("Hello, world!");
}

```

---
## File: `languages/go.pest`
*(Relative Path: `languages/go.pest`)*

```plaintext
comment_go = { line_comment | block_comment }

// single-line
line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

// multi-line
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// skip string literals
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

go_file = {
    (comment_go | str_literal | ANY)*
}

```

---
## File: `languages/go.rs`
*(Relative Path: `languages/go.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/go.pest"]
pub struct GoParser;

pub fn parse_go_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = GoParser::parse(Rule::go_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_go {
                handle_go_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_go_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // remove `//`
            let stripped = text.trim_start_matches('/').trim_start_matches('/').trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            // remove `/*`...`*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `languages/js.pest`
*(Relative Path: `languages/js.pest`)*

```plaintext
comment_js = { line_comment | block_comment }

// single-line
line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

// multi-line
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// skip strings
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'" |
    "`" ~ (!"`" ~ ANY)* ~ "`" // backticks in JS
}

js_file = {
    (comment_js | str_literal | ANY)*
}

```

---
## File: `languages/js.rs`
*(Relative Path: `languages/js.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/js.pest"]
pub struct JsParser;

pub fn parse_js_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = JsParser::parse(Rule::js_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_js {
                handle_js_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_js_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            let stripped = text.trim_start_matches("//").trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            let trimmed = block_text.trim_start_matches("/*").trim_end_matches("*/");
            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `languages/mod.rs`
*(Relative Path: `languages/mod.rs`)*

```rust
pub mod ts;
pub mod rust;
pub mod python;
pub mod js;
pub mod go;

pub use ts::parse_ts_comments;
pub use rust::parse_rust_comments;
pub use python::parse_python_comments;
pub use js::parse_js_comments;
pub use go::parse_go_comments;

```

---
## File: `languages/python.pest`
*(Relative Path: `languages/python.pest`)*

```plaintext
// We assume the "grammar-extras" feature is on, so we can define COMMENT or WHITESPACE if we want.
//
// For Python, we'll capture top-level "comment" tokens as well as "docstring_comment".

comment_python = { line_comment | docstring_comment }

// Single-line # comment
line_comment = @{
    "#" ~ (!NEWLINE ~ ANY)*
}

// We'll consider triple quotes as docstring_comment for simplicity.
docstring_comment = @{
    "\"\"\"" ~ (!"\"\"\"" ~ ANY)* ~ "\"\"\""
}

// For normal string literals (to skip them), we define a rough rule to ensure we don't treat them as comments:
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

// The main rule: a Python file can have any mixture of comments, string literals, or code (ANY).
// We only want to capture "comment_python" tokens as recognized comment pairs in the parse tree.
python_file = {
    (comment_python | str_literal | ANY)*
}

```

---
## File: `languages/python.rs`
*(Relative Path: `languages/python.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

// Load grammar at compile time
#[derive(Parser)]
#[grammar = "languages/python.pest"]
pub struct PythonParser;

/// Parse the entire Python file content, returning lines of comment text (and line numbers).
pub fn parse_python_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = PythonParser::parse(Rule::python_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            match pair.as_rule() {
                // line_comment or docstring_comment
                Rule::comment_python => {
                    handle_comment_token(&pair, &mut comments);
                }
                _ => {
                    // ignore everything else
                }
            }
        }
    }
    comments
}

/// If it's a single-line `# ...`, store exactly that line.
/// If it's a docstring_comment, store each line inside that triple-quoted block.
fn handle_comment_token(pair: &Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col().0; // line number (1-based)
            // The actual text (including `#`)
            let text = span.as_str();
            // remove leading '#'
            let stripped = text.trim_start_matches('#').trim_end().to_string();

            out.push(CommentLine {
                line_number: start_pos,
                text: stripped,
            });
        }
        Rule::docstring_comment => {
            // multi-line docstring
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();

            // remove the surrounding """..."""
            let trimmed = block_text
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"");

            // naive approach: each line is separate
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                let cleaned = line_text.trim_end().to_string();
                out.push(CommentLine {
                    line_number: current_line,
                    text: cleaned,
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
## File: `languages/rust.pest`
*(Relative Path: `languages/rust.pest`)*

```plaintext
// Minimal Rust grammar focusing on comment tokens & ignoring string literals.

comment_rust = { line_comment | block_comment | doc_comment_line }

// Single-line: `//` until newline
line_comment = @{ "//" ~ (!NEWLINE ~ ANY)* }

// Block comment: `/* ... */` (this can be multi-line)
block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

// Doc comment lines start with `///` or `//!` (we unify them)
doc_comment_line = @{
    "///" ~ (!NEWLINE ~ ANY)* |
    "//!" ~ (!NEWLINE ~ ANY)*
}

// For ignoring string literals roughly:
str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'"
}

rust_file = {
    (comment_rust | str_literal | ANY)*
}

```

---
## File: `languages/rust.rs`
*(Relative Path: `languages/rust.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = RustParser::parse(Rule::rust_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            match pair.as_rule() {
                Rule::comment_rust => {
                    handle_rust_comment(pair, &mut comments);
                }
                _ => {}
            }
        }
    }
    comments
}

fn handle_rust_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // Remove leading `//`
            let stripped = text.trim_start_matches('/').trim_start_matches('/').trim_start();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            // remove `/*` and `*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            // split by lines
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim().to_string(),
                });
                current_line += 1;
            }
        }
        Rule::doc_comment_line => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // remove leading `///` or `//!`
            let stripped = if text.starts_with("///") {
                &text[3..]
            } else {
                &text[3..] // `//!`
            };
            out.push(CommentLine {
                line_number: line,
                text: stripped.trim().to_string(),
            });
        }
        _ => {}
    }
}

```

---
## File: `languages/ts.pest`
*(Relative Path: `languages/ts.pest`)*

```plaintext
comment_ts = { line_comment | block_comment }

line_comment = @{
    "//" ~ (!NEWLINE ~ ANY)*
}

block_comment = @{
    "/*" ~ (!"*/" ~ ANY)* ~ "*/"
}

str_literal = _{
    "\"" ~ (!"\"" ~ ANY)* ~ "\"" |
    "'" ~ (!"'" ~ ANY)* ~ "'" |
    "`" ~ (!"`" ~ ANY)* ~ "`"
}

ts_file = {
    (comment_ts | str_literal | ANY)*
}

```

---
## File: `languages/ts.rs`
*(Relative Path: `languages/ts.rs`)*

```rust
use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/ts.pest"]
pub struct TsParser;

pub fn parse_ts_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = TsParser::parse(Rule::ts_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_ts {
                handle_ts_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_ts_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            let stripped = text.trim_start_matches("//").trim();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();
            let trimmed = block_text.trim_start_matches("/*").trim_end_matches("*/");
            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}

```

---
