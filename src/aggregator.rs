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

    // ✅ If the comment starts with "TODO:", store it
    let re = Regex::new(r"^\s*[^a-zA-Z]?\s*TODO:").unwrap();
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
        "extract_todos: found {} comment lines from parser",
        comment_lines.len()
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

/// Merge multi-line TODO lines and produce `TodoItem` for each distinct `TODO:`.
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    let mut result = Vec::new();
    let mut idx = 0;

    debug!(
        "Starting to collect TODOs from comment lines. Total lines: {}",
        lines.len()
    );

    while idx < lines.len() {
        let text = &lines[idx].text.trim();
        let line_num = lines[idx].line_number;

        debug!("Processing line {}: '{}'", line_num, text);

        if text.starts_with("TODO:") {
            let mut collected = text[5..].trim().to_string();
            debug!("Found TODO at line {}: '{}'", line_num, collected);

            idx += 1;
            while idx < lines.len() {
                let next_text = &lines[idx].text.trim();
                debug!(
                    "Checking next line {}: '{}'",
                    lines[idx].line_number, next_text
                );

                // Ensure we only merge if the next line is indented
                if next_text.starts_with(" ") || next_text.starts_with("\t") {
                    collected.push(' ');
                    collected.push_str(next_text.trim());
                    debug!(
                        "Merged line {} into TODO: '{}'",
                        lines[idx].line_number, collected
                    );
                    idx += 1;
                } else {
                    debug!(
                        "Line {} is not indented. Stopping merge.",
                        lines[idx].line_number
                    );
                    break;
                }
            }

            debug!(
                "Added TODO item: line {}, message '{}'",
                line_num, collected
            );
            result.push(TodoItem {
                line_number: line_num,
                message: collected,
            });
        } else {
            debug!("Line {} does not start with TODO:, skipping.", line_num);
            idx += 1;
        }
    }

    debug!(
        "Finished collecting TODOs. Total TODO items: {}",
        result.len()
    );
    result
}
