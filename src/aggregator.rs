use log::debug;
use std::marker::PhantomData;
use std::path::Path;

use crate::languages::{
    python::parse_python_comments,
    rust::parse_rust_comments,
    //go::parse_go_comments,
    //js::parse_js_comments,
    //ts::parse_ts_comments,
};
use log::{error, info};
use pest::{iterators::Pair, Parser};
use regex::Regex;

/// Represents a single found TODO item.
#[derive(Debug, PartialEq)]
pub struct TodoItem {
    pub line_number: usize,
    pub message: String,
}

/// Generic function to parse comments from source code.
///
/// - `parser`: A `pest::Parser` implementation (e.g., `RustParser`, `PythonParser`).
/// - `rule`: The top-level rule for parsing the file.
/// - `file_content`: The source code text.
/// - Returns: A `Vec<CommentLine>` containing extracted comments.
pub fn parse_comments<P: Parser<R>, R: pest::RuleType>(
    _parser_type: PhantomData<P>,
    rule: R,
    file_content: &str,
) -> Vec<CommentLine> {
    info!(
        "Starting comment parsing. File length: {}",
        file_content.len()
    );

    let parse_result = P::parse(rule, file_content);
    let mut comments = Vec::new();

    match parse_result {
        Ok(pairs) => {
            debug!(
                "Parsing successful! Found {} top-level pairs.",
                pairs.clone().count()
            );

            for pair in pairs {
                // 🔥 NEW: Iterate over children of the rust_file or python_file
                for inner_pair in pair.into_inner() {
                    debug!(
                        "Processing child pair: {:?} => '{}'",
                        inner_pair.as_rule(),
                        inner_pair.as_str().replace('\n', "\\n")
                    );

                    if let Some(comment) = extract_comment_from_pair(inner_pair) {
                        debug!("Extracted comment: {:?}", comment);
                        comments.push(comment);
                    } else {
                        debug!("Skipped non-comment pair.");
                    }
                }
            }
        }
        Err(e) => {
            error!("Parsing error: {:?}", e);
        }
    }

    info!("Extracted {} comments", comments.len());
    comments
}

fn extract_comment_from_pair(pair: Pair<impl pest::RuleType>) -> Option<CommentLine> {
    let span = pair.as_span();
    let base_line = span.start_pos().line_col().0; // Get line number
    let text = span.as_str().trim(); // Extract the comment text

    let rule_name = format!("{:?}", pair.as_rule()).to_lowercase();
    // Skip tokens whose rule names contain "non_comment"
    if rule_name.contains("non_comment") {
        return None;
    }
    if rule_name.contains("comment") && !text.is_empty() {
        Some(CommentLine {
            line_number: base_line,
            text: text.to_string(),
        })
    } else {
        None
    }
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
        //"go" => parse_go_comments(file_content),
        //"js" => parse_js_comments(file_content),
        //"ts" => parse_ts_comments(file_content),
        _ => {
            debug!(
                "No recognized extension for file {:?}; returning empty list.",
                path
            );
            vec![]
        }
    };

    debug!(
        "extract_todos: found {} comment lines from parser: {:?}",
        comment_lines.len(), comment_lines
    );

    // Next, find any TODOs among these comment lines
    let todos = collect_todos_from_comment_lines(&comment_lines);
    debug!("extract_todos: found {} TODO items total", todos.len());
    todos
}

/// A single comment line with (line_number, entire_comment_text).
#[derive(Debug, Clone)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge contiguous comment lines into blocks and produce a `TodoItem` for each block
/// that contains a TODO marker. In a block, the TODO’s line number is taken from
/// the first comment line, and only the TODO text from the first occurrence is used.
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    let mut result = Vec::new();
    let mut block: Vec<CommentLine> = Vec::new();

    for line in lines {
        if block.is_empty() {
            block.push(line.clone());
        } else {
            // Check if the current line is contiguous with the last one (no blank line in between)
            if line.line_number == block.last().unwrap().line_number + 1 {
                block.push(line.clone());
            } else {
                // Process the block
                if let Some(todo) = extract_todo_from_block(&block) {
                    result.push(todo);
                }
                block.clear();
                block.push(line.clone());
            }
        }
    }
    if !block.is_empty() {
        if let Some(todo) = extract_todo_from_block(&block) {
            result.push(todo);
        }
    }
    result
}

fn extract_todo_from_block(block: &[CommentLine]) -> Option<TodoItem> {
    debug!("Starting to extract TODO from block with {} lines", block.len());

    // Use DOTALL mode so that . matches newlines.
    let re = Regex::new(r"(?s)^(?P<marker>[^a-zA-Z]*)(?P<ws>\s*)TODO:\s*(?P<text>.*)").unwrap();

    // Find the first line in the block that contains "TODO:".
    let mut todo_index = None;
    for (i, line) in block.iter().enumerate() {
        if line.text.contains("TODO:") {
            todo_index = Some(i);
            debug!("Found TODO at line {}: '{}'", line.line_number, line.text);
            break;
        }
    }

    if let Some(i) = todo_index {
        let mut message = String::new();
        let mut marker = "";
        let mut base_ws = "";
        if let Some(caps) = re.captures(&block[i].text) {
            marker = caps.name("marker").map(|m| m.as_str()).unwrap_or("");
            base_ws = caps.name("ws").map(|m| m.as_str()).unwrap_or("");
            // Normalize whitespace by splitting and rejoining with a single space.
            message = caps.name("text")
                .map(|m| m.as_str().trim().split_whitespace().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();
            debug!("Captured TODO message: '{}'", message);
            debug!("Captured marker: '{}', base_ws: '{}'", marker, base_ws);
        }

        // Append subsequent lines if they are indented beyond the baseline.
        for line in block.iter().skip(i + 1) {
            let trimmed = line.text.trim_start();
            if marker.len() > 0 && trimmed.starts_with(marker) {
                let after_marker = trimmed.trim_start_matches(marker);
                // Get the leading whitespace of the remainder.
                let continuation_ws: String = after_marker.chars().take_while(|c| c.is_whitespace()).collect();
                if continuation_ws.len() > base_ws.len() {
                    let without_marker = after_marker.trim_start();
                    if !without_marker.is_empty() {
                        if !message.is_empty() {
                            message.push(' ');
                        }
                        message.push_str(without_marker);
                        debug!("Appended line (marker removed): '{}'", without_marker);
                    }
                } else {
                    break;
                }
            } else if trimmed.starts_with(" ") || trimmed.starts_with("\t") {
                // Only merge if the line is indented beyond the baseline.
                // (Here we choose not to merge lines without the comment marker.)
                break;
            } else {
                break;
            }
        }

        debug!(
            "Final TODO item: line_number = {}, message = '{}'",
            block[i].line_number, message
        );

        return Some(TodoItem {
            line_number: block[i].line_number,
            message,
        });
    }

    debug!("No TODO found in the block");
    None
}
