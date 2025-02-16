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
    let text = span.as_str().trim(); // Extract actual comment text

    debug!(
        "Extracted comment at line {}: '{}'",
        base_line,
        text.replace('\n', "\\n")
    );

    // Updated regex: allow zero or more non-letter characters before "TODO:"
    let re = Regex::new(r"^\s*[^a-zA-Z]*\s*TODO:").unwrap();
    if re.is_match(text) {
        return Some(CommentLine {
            line_number: base_line,
            text: text.to_string(),
        });
    }

    None
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
    // Find the index of the first line in the block that contains "TODO:".
    let mut todo_index = None;
    for (i, line) in block.iter().enumerate() {
        if line.text.contains("TODO:") {
            todo_index = Some(i);
            break;
        }
    }
    if let Some(i) = todo_index {
        // Use a regex to capture the text after "TODO:" on that line.
        let re = Regex::new(r"^\s*[^a-zA-Z]*\s*TODO:\s*(.*)").unwrap();
        let mut message = String::new();
        if let Some(caps) = re.captures(&block[i].text) {
            message = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
        }
        // Append subsequent lines if they are indented.
        for line in block.iter().skip(i + 1) {
            let trimmed = line.text.trim();
            if trimmed.starts_with(" ") || trimmed.starts_with("\t") {
                if !message.is_empty() {
                    message.push(' ');
                }
                message.push_str(trimmed);
            } else {
                break;
            }
        }
        return Some(TodoItem {
            line_number: block[i].line_number,
            message,
        });
    }
    None
}


