use std::path::Path;
use log::debug;

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
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    debug!("extract_todos: extension = '{}'", extension);

    // Call the relevant language parser to extract comment lines
    let comment_lines = match extension.as_str() {
        "py" => parse_python_comments(file_content),
        "rs" => parse_rust_comments(file_content),
        "go" => parse_go_comments(file_content),
        "js" => parse_js_comments(file_content),
        "ts" => parse_ts_comments(file_content),
        _ => {
            debug!("No recognized extension for file {:?}; returning empty list.", path);
            vec![]
        }
    };

    debug!(
        "extract_todos: found {} comment lines from parser",
        comment_lines.len()
    );

    // Next, find any TODOs among these comment lines
    let todos = collect_todos_from_comment_lines(&comment_lines);
    debug!("extract_todos: found {} TODO items total", todos.len());
    todos
}

/// A single comment line with (line_number, entire_comment_text).
#[derive(Debug)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge multi-line TODO lines and produce `TodoItem` for each distinct `TODO:`.
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    let mut result = Vec::new();
    let mut idx = 0;

    while idx < lines.len() {
        let text = &lines[idx].text;
        let line_num = lines[idx].line_number;

        debug!(
            "collect_todos: checking line {} => '{}'",
            line_num, text
        );

        if let Some(pos) = text.find("TODO:") {
            debug!(" -> Found TODO at line {} pos {}", line_num, pos);
            let after_todo = &text[pos + 5..]; // skip "TODO:"
            let mut collected = after_todo.trim_start().to_string();

            idx += 1;
            while idx < lines.len() {
                let cont_text = &lines[idx].text;
                if cont_text.starts_with(' ') || cont_text.starts_with('\t') {
                    debug!(
                        "   continuing multiline TODO at line {} => '{}'",
                        lines[idx].line_number, cont_text
                    );
                    collected.push(' ');
                    collected.push_str(cont_text.trim());
                    idx += 1;
                } else {
                    break;
                }
            }

            let final_msg = collected.trim_end().to_string();
            debug!(
                " -> Final merged TODO from line {} => '{}'",
                line_num, final_msg
            );

            result.push(TodoItem {
                line_number: line_num,
                message: final_msg,
            });
        } else {
            idx += 1;
        }
    }

    result
}
