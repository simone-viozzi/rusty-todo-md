use log::debug;
use std::marker::PhantomData;
use std::path::Path;

use crate::languages::{
    common::CommentParser,
    python::PythonParser,
    // go::GoParser,
    // js::JsParser,
    // ts::TsParser,
    rust::RustParser,
};
use log::{error, info};
use pest::Parser;
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
                // Iterate over children of the rust_file or python_file.
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

fn extract_comment_from_pair(
    pair: pest::iterators::Pair<impl pest::RuleType>,
) -> Option<CommentLine> {
    let span = pair.as_span();
    let base_line = span.start_pos().line_col().0; // Get line number
    let text = span.as_str().trim(); // Extract the comment text

    let rule_name = format!("{:?}", pair.as_rule()).to_lowercase();
    // Skip tokens whose rule names contain "non_comment"
    if rule_name.contains("non_comment") {
        return None;
    }
    // Accept tokens if they are a comment or a docstring
    if (rule_name.contains("comment") || rule_name.contains("docstring")) && !text.is_empty() {
        Some(CommentLine {
            line_number: base_line,
            text: text.to_string(),
        })
    } else {
        None
    }
}

/// Given a block of contiguous comment lines, extract the TODO item (if any) by merging
/// the initial TODO line with subsequent indented lines.
fn extract_todo_from_block(block: &[CommentLine]) -> Option<TodoItem> {
    // Regex to capture a TODO line: the marker (non-letters), the whitespace after the marker, then "TODO:" and the text.
    let re = Regex::new(r"^(?P<marker>[^a-zA-Z]*)(?P<ws>\s*)TODO:\s*(?P<text>.*)").unwrap();

    // Find the first line that contains "TODO:".
    let (todo_index, caps) = block
        .iter()
        .enumerate()
        .find_map(|(i, line)| re.captures(&line.text).map(|caps| (i, caps)))?;

    // Extract the marker and initial text.
    let marker = caps.name("marker").map(|m| m.as_str()).unwrap_or("");
    let initial_text = caps.name("text").map(|m| m.as_str()).unwrap_or("");

    // Use the captured whitespace after the marker as the baseline indentation.
    let base_indent = caps.name("ws").map(|m| m.as_str().len()).unwrap_or(0);

    // Start with the normalized initial TODO text.
    let mut message = normalize_text(initial_text, marker);

    // Process subsequent lines in the block.
    for line in block.iter().skip(todo_index + 1) {
        // Compute the raw indentation (number of leading whitespace characters) from the original line.
        let raw_indent = line.text.chars().take_while(|c| c.is_whitespace()).count();

        // We'll decide how to process based on whether the line starts with the same marker.
        if line.text.trim_start().starts_with(marker) {
            // Remove the marker and then measure the indentation of what remains.
            let after_marker = line.text.trim_start_matches(marker);
            let current_indent = after_marker.chars().take_while(|c| c.is_whitespace()).count();
            if current_indent > base_indent {
                let content = after_marker.trim_start();
                if !content.is_empty() {
                    let part = normalize_text(content, marker);
                    if !part.is_empty() {
                        message.push(' ');
                        message.push_str(&part);
                    }
                }
            } else {
                break;
            }
        } else {
            // For lines that don't repeat the marker (e.g. Python docstring continuations),
            // use the raw indentation.
            if raw_indent > base_indent {
                let part = normalize_text(line.text.trim(), marker);
                if !part.is_empty() {
                    message.push(' ');
                    message.push_str(&part);
                }
            } else {
                break;
            }
        }
    }

    Some(TodoItem {
        line_number: block[todo_index].line_number,
        message,
    })
}



/// Normalizes a text fragment by:
/// - Splitting by whitespace and rejoining with a single space,
/// - If the marker indicates a block comment (i.e. contains "/*"),
///   removes a trailing "*/" from the text.
fn normalize_text(text: &str, marker: &str) -> String {
    let mut normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if marker.contains("/*") && normalized.ends_with("*/") {
        normalized = normalized.trim_end_matches("*/").trim().to_string();
    }
    normalized
}

fn split_multiline_comment_line(line: &CommentLine) -> Vec<CommentLine> {
    let mut result = Vec::new();
    // Split the text by newline.
    for (i, part) in line.text.split('\n').enumerate() {
        // Assume that the first part retains the original line number,
        // and subsequent parts increment the line number.
        result.push(CommentLine {
            line_number: line.line_number + i,
            text: part.to_string(),
        });
    }
    result
}

fn flatten_comment_lines(lines: &[CommentLine]) -> Vec<CommentLine> {
    let mut flattened = Vec::new();
    for line in lines {
        if line.text.contains('\n') {
            flattened.extend(split_multiline_comment_line(line));
        } else {
            flattened.push(line.clone());
        }
    }
    flattened
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

    // Call the relevant language parser to extract comment lines.
    let comment_lines = match extension.as_str() {
        "py" => PythonParser::parse_comments(file_content),
        "rs" => RustParser::parse_comments(file_content),
        // "go" => GoParser::parse_comments(file_content),
        // "js" => JsParser::parse_comments(file_content),
        // "ts" => TsParser::parse_comments(file_content),
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
        comment_lines.len(),
        comment_lines
    );

    // Next, find any TODOs among these comment lines.
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
    // Flatten the comments so that multi-line entries become separate lines.
    let flattened_lines = flatten_comment_lines(lines);

    let mut result = Vec::new();
    let mut block: Vec<CommentLine> = Vec::new();

    for line in &flattened_lines {
        if block.is_empty() {
            block.push(line.clone());
        } else {
            // Check if the current line is contiguous with the last one.
            if line.line_number == block.last().unwrap().line_number + 1 {
                block.push(line.clone());
            } else {
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
