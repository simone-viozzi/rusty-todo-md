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
