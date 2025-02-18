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
use crate::languages::common_syntax;


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



/// Given a block of contiguous comment lines, extract the TODO item (if any)
/// by:
/// 1. Removing all lines above the first TODO line.
/// 2. Dedenting the resulting block.
/// 3. Checking if the line after the TODO is indented.
/// 4. Merging the TODO line with all its indented continuation lines.
fn extract_todo_from_block(block: &[CommentLine]) -> Option<TodoItem> {
    // Find the first line that contains the "TODO:" marker.
    let todo_index = block.iter().position(|line| line.text.contains("TODO:"))?;
    
    // Create a candidate block starting from the TODO line to the end.
    let todo_block_lines: Vec<&str> = block[todo_index..]
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    
    // Join the candidate lines into one block (with newline separators).
    let joined_block = todo_block_lines.join("\n");
    
    // Dedent the entire block so that the TODO line becomes column 0.
    let dedented_block = common_syntax::dedent_comment(&joined_block);
    
    // Split the dedented block back into individual lines.
    let dedented_lines: Vec<&str> = dedented_block.lines().collect();
    
    // The first line is the TODO line.
    // For a block TODO, subsequent lines must be indented.
    let mut collected_lines = Vec::new();
    if let Some(first_line) = dedented_lines.first() {
        collected_lines.push(*first_line);
        // Collect all following lines that are indented.
        for line in dedented_lines.iter().skip(1) {
            if line.starts_with(' ') || line.starts_with('\t') {
                collected_lines.push(*line);
            } else {
                break;
            }
        }
    }
    
    // Merge the collected lines into one normalized string.
    let mut merged = common_syntax::merge_comment_lines(&collected_lines);

    if let Some(stripped) = merged.strip_prefix("TODO:") {
        // Optionally trim any leading whitespace after removal.
        merged = stripped.trim_start().to_string();
    }
    
    Some(TodoItem {
        line_number: block[todo_index].line_number,
        message: merged,
    })
}




// TODO: what is this?
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

// TODO: what is this?
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
/// the first comment line.
pub fn collect_todos_from_comment_lines(lines: &[CommentLine]) -> Vec<TodoItem> {
    // Flatten the comments so that multi-line entries become separate lines.
    let flattened_lines = flatten_comment_lines(lines);
    let mut result = Vec::new();
    let mut block: Vec<CommentLine> = Vec::new();

    for line in &flattened_lines {
        if block.is_empty() {
            block.push(line.clone());
        } else {
            // If the current line is contiguous (i.e. the next line number), add it to the block.
            if line.line_number == block.last().unwrap().line_number + 1 {
                block.push(line.clone());
            } else {
                let stripped_block: Vec<CommentLine> = block
                    .iter()
                    .map(|cl| CommentLine {
                        line_number: cl.line_number,
                        text: common_syntax::strip_markers(&cl.text),
                    })
                    .collect();

                // Attempt to extract a TODO from the stripped block.
                if let Some(todo) = extract_todo_from_block(&stripped_block) {
                    result.push(todo);
                }
                block.clear();
                block.push(line.clone());
            }
        }
    }
    if !block.is_empty() {
        let stripped_block: Vec<CommentLine> = block
            .iter()
            .map(|cl| CommentLine {
                line_number: cl.line_number,
                text: common_syntax::strip_markers(&cl.text),
            })
            .collect();
        if let Some(todo) = extract_todo_from_block(&stripped_block) {
            result.push(todo);
        }
    }
    result
}
