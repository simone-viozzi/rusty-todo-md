use log::debug;
use std::path::Path;
use std::{marker::PhantomData, path::PathBuf};

use crate::todo_extractor_internal::languages::common::CommentParser;
use crate::todo_extractor_internal::languages::common_syntax;
use log::{error, info};
use pest::Parser;

/// Represents a single found marked item.
#[derive(Debug, PartialEq, Clone)]
pub struct MarkedItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub message: String,
}

/// Configuration for comment markers.
pub struct MarkerConfig {
    pub markers: Vec<String>,
}

impl Default for MarkerConfig {
    fn default() -> Self {
        MarkerConfig {
            markers: vec!["TODO".to_string()],
        }
    }
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

/// Extracts a comment from a given `pest::iterators::Pair`.
///
/// - `pair`: A `pest::iterators::Pair` representing a parsed token.
/// - Returns: An `Option<CommentLine>` containing the extracted comment if successful.
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

/// Given a block of contiguous comment lines (with markers already stripped),
/// extract the marked item (if any) by:
/// 1. Finding the first line containing any marker.
/// 2. Joining that line and any immediately indented following lines.
/// 3. Dedenting and merging them into a single normalized message.
/// 4. Removing the marker prefix and trimming the result.
fn extract_marked_item_from_block(
    block: &[CommentLine],
    config: &MarkerConfig,
    path: &Path,
) -> Option<MarkedItem> {
    debug!("Extracting marked item from block: {:?}", block);

    // Find the index of the first line with any marker.
    let marker_index = block.iter().position(|line| {
        config
            .markers
            .iter()
            .any(|marker| line.text.contains(marker))
    })?;
    debug!("Found marker at index: {}", marker_index);

    // Get the candidate lines from the marker line until the end of the block.
    let candidate_lines: Vec<&str> = block[marker_index..]
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    debug!("Candidate lines: {:?}", candidate_lines);

    // Join the candidate lines into a single string.
    let joined_block = candidate_lines.join("\n");
    debug!("Joined block:\n{}", joined_block);

    // Dedent the block so that the marker line starts at column 0.
    let dedented_block = common_syntax::dedent_comment(&joined_block);
    debug!("Dedented block:\n{}", dedented_block);

    // Split the dedented block into individual lines.
    let dedented_lines: Vec<&str> = dedented_block.lines().collect();
    debug!("Dedented lines: {:?}", dedented_lines);

    // Extract candidate marked lines (first line and any immediately indented following lines).
    let candidate_marked_lines = extract_candidate_marked_lines(&dedented_lines);
    debug!("Candidate marked lines: {:?}", candidate_marked_lines);

    // *** NEW: Check if the first candidate line starts with any marker ***
    if let Some(first_line) = candidate_marked_lines.first() {
        if !config
            .markers
            .iter()
            .any(|marker| first_line.trim_start().starts_with(marker))
        {
            debug!("Candidate line does not start with any marker, skipping block.");
            return None;
        }
    } else {
        return None;
    }

    // Merge the candidate lines into a normalized single-line string.
    let merged = common_syntax::merge_comment_lines(&candidate_marked_lines);
    debug!("Merged marked message: {}", merged);

    // Remove the marker prefix and trim any extra whitespace.
    // Remove the marker prefix and trim any extra whitespace.
    // This version makes an optional colon after the marker removable.
    let final_message = config.markers.iter().fold(merged, |acc, marker| {
        if let Some(s) = acc.strip_prefix(marker) {
            let s = s.trim_start();
            // If a colon is present right after the marker, remove it along with any whitespace.
            let s = if let Some(stripped) = s.strip_prefix(':') { stripped } else { s };
            s.trim().to_string()
        } else {
            acc
        }
    });

    debug!("Final marked message: {}", final_message);

    Some(MarkedItem {
        line_number: block[marker_index].line_number,
        message: final_message,
        file_path: path.to_path_buf(),
    })
}

/// Extracts the candidate marked lines from a list of dedented lines.
/// It always takes the first line (which contains a marker) and then collects
/// all subsequent lines that are indented (starting with a space or tab).
fn extract_candidate_marked_lines<'a>(lines: &'a [&'a str]) -> Vec<&'a str> {
    let mut candidate = Vec::new();
    if let Some(first_line) = lines.first() {
        candidate.push(*first_line);
        for line in lines.iter().skip(1) {
            if line.starts_with(' ') || line.starts_with('\t') {
                candidate.push(*line);
            } else {
                break;
            }
        }
    }
    candidate
}

// Splits a multi-line comment into individual `CommentLine` entries.
//
// - `line`: A `CommentLine` containing multiple lines of text.
// - Returns: A `Vec<CommentLine>` with each line split into a separate entry.
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

// Flattens a list of `CommentLine` entries, splitting any multi-line comments
// into individual `CommentLine` entries.
//
// - `lines`: A slice of `CommentLine` entries.
// - Returns: A `Vec<CommentLine>` with all multi-line comments split into individual entries.
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

/// Returns the comment lines extracted by the appropriate parser based on the file extension.
///
/// - `extension`: The file extension (e.g., "py", "rs").
/// - `file_content`: The source code text.
/// - Returns: An `Option<Vec<CommentLine>>` containing extracted comments if successful.
fn get_parser_comments(extension: &str, file_content: &str) -> Option<Vec<CommentLine>> {
    match extension {
        "py" => Some(
            crate::todo_extractor_internal::languages::python::PythonParser::parse_comments(
                file_content,
            ),
        ),
        "rs" => Some(
            crate::todo_extractor_internal::languages::rust::RustParser::parse_comments(
                file_content,
            ),
        ),
        // Add new extensions and their corresponding parser calls here:
        // "js" => Some(crate::languages::js::JsParser::parse_comments(file_content)),
        // "ts" => Some(crate::languages::ts::TsParser::parse_comments(file_content)),
        _ => None,
    }
}

/// Extracts marked items from the given file content based on its extension.
///
/// - `path`: The path to the file.
/// - `file_content`: The source code text.
/// - `config`: The marker configuration.
/// - Returns: A `Vec<MarkedItem>` containing extracted marked items.
pub fn extract_marked_items(
    path: &Path,
    file_content: &str,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    debug!("extract_marked_items: extension = '{}'", extension);

    // Use the helper function to get the comment lines.
    let comment_lines = match get_parser_comments(extension.as_str(), file_content) {
        Some(lines) => lines,
        None => {
            debug!(
                "No recognized extension for file {:?}; returning empty list.",
                path
            );
            vec![]
        }
    };

    debug!(
        "extract_marked_items: found {} comment lines from parser: {:?}",
        comment_lines.len(),
        comment_lines
    );

    // Continue with the existing logic to collect and merge marked items.
    let marked_items = collect_marked_items_from_comment_lines(&comment_lines, config, path);
    debug!(
        "extract_marked_items: found {} marked items total",
        marked_items.len()
    );
    marked_items
}

/// A single comment line with (line_number, entire_comment_text).
#[derive(Debug, Clone)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge contiguous comment lines into blocks and produce a `MarkedItem` for each block
/// that contains a marker. In a block, the marker’s line number is taken from
/// the first comment line.  
///  
/// **New behavior:** If a marker is encountered in a block that already contains a marker,  
/// the current block is terminated (processed) and a new block is started.
pub fn collect_marked_items_from_comment_lines(
    lines: &[CommentLine],
    config: &MarkerConfig,
    path: &Path,
) -> Vec<MarkedItem> {
    debug!(
        "Starting to collect marked items from comment lines. Total lines: {}",
        lines.len()
    );

    // Flatten multi-line comment entries.
    let flattened_lines = flatten_comment_lines(lines);
    debug!("Flattened lines count: {}", flattened_lines.len());

    let mut marked_items = Vec::new();
    let mut current_block = Vec::new();

    for line in &flattened_lines {
        if current_block.is_empty() {
            debug!("Starting new block with line: {:?}", line);
            current_block.push(line.clone());
        } else if is_contiguous(&current_block, line) {
            // If the current block already contains a marker and the incoming line also contains one,
            // we finalize the current block and start a new one.
            if block_contains_marker(&current_block, &config.markers)
                && config
                    .markers
                    .iter()
                    .any(|marker| line.text.contains(marker))
            {
                debug!(
                    "Found a new marker in a block that already contains one. Splitting block at line: {:?}",
                    line
                );
                process_block(&mut current_block, &mut marked_items, config, path);
                current_block.push(line.clone());
            } else {
                debug!("Adding contiguous line to current block: {:?}", line);
                current_block.push(line.clone());
            }
        } else {
            debug!("Non-contiguous line encountered. Finalizing current block.");
            process_block(&mut current_block, &mut marked_items, config, path);
            debug!("Starting new block with line: {:?}", line);
            current_block.push(line.clone());
        }
    }

    // Process any remaining block.
    if !current_block.is_empty() {
        debug!("Processing final block.");
        process_block(&mut current_block, &mut marked_items, config, path);
    }

    debug!(
        "Finished collecting marked items. Total marked items found: {}",
        marked_items.len()
    );
    marked_items
}

/// Returns true if the current block already contains a line with a marker.
///
/// - `block`: A slice of `CommentLine` entries representing the current block.
/// - `markers`: A slice of marker strings.
/// - Returns: `true` if the block contains a marker, `false` otherwise.
fn block_contains_marker(block: &[CommentLine], markers: &[String]) -> bool {
    let contains = block
        .iter()
        .any(|cl| markers.iter().any(|marker| cl.text.contains(marker)));
    debug!(
        "Checking if current block contains a marker: {} (block: {:?})",
        contains, block
    );
    contains
}

/// Checks whether the given `line` is contiguous (i.e. its line number is exactly one
/// more than the last line in `current_block`).
///
/// - `current_block`: A slice of `CommentLine` entries representing the current block.
/// - `line`: The `CommentLine` to check for contiguity.
/// - Returns: `true` if the line is contiguous, `false` otherwise.
fn is_contiguous(current_block: &[CommentLine], line: &CommentLine) -> bool {
    if let Some(last) = current_block.last() {
        let contiguous = last.line_number + 1 == line.line_number;
        debug!(
            "Checking contiguity: last line {} and current line {} => {}",
            last.line_number, line.line_number, contiguous
        );
        contiguous
    } else {
        false
    }
}

/// Processes a block of contiguous comment lines by stripping markers and
/// extracting a marked item (if present). Clears the block after processing.
///
/// - `block`: A mutable reference to a `Vec<CommentLine>` representing the current block.
/// - `marked_items`: A mutable reference to a `Vec<MarkedItem>` to store extracted marked items.
/// - `config`: The marker configuration.
fn process_block(
    block: &mut Vec<CommentLine>,
    marked_items: &mut Vec<MarkedItem>,
    config: &MarkerConfig,
    path: &Path,
) {
    debug!("Processing block with {} lines: {:?}", block.len(), block);
    let stripped_block = strip_comment_lines(block);
    debug!("Stripped block: {:?}", stripped_block);

    if let Some(marked_item) = extract_marked_item_from_block(&stripped_block, config, path) {
        debug!("Marked item found in block: {:?}", marked_item);
        marked_items.push(marked_item);
    } else {
        debug!("No marked item found in current block.");
    }
    block.clear();
}

/// Strips language-specific markers from each comment line in the block.
///
/// - `block`: A slice of `CommentLine` entries representing the current block.
/// - Returns: A `Vec<CommentLine>` with markers stripped from each line.
fn strip_comment_lines(block: &[CommentLine]) -> Vec<CommentLine> {
    block
        .iter()
        .map(|cl| {
            let stripped_text = common_syntax::strip_markers(&cl.text);
            debug!(
                "Stripping markers from line {}: '{}' -> '{}'",
                cl.line_number, cl.text, stripped_text
            );
            CommentLine {
                line_number: cl.line_number,
                text: stripped_text,
            }
        })
        .collect()
}

#[cfg(test)]
mod aggregator_tests {
    use super::*;
    use crate::logger;
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .format(logger::format_logger)
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });
    }

    #[test]
    fn test_valid_rust_extension() {
        init_logger();
        let src = "// TODO: Implement feature X";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
    }

    #[test]
    fn test_invalid_extension() {
        init_logger();
        let src = "// TODO: This should not be processed";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.unknown"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_merge_multiline_todo() {
        init_logger();
        let src = r#"
// TODO: Fix bug
//     Improve error handling
//     Add logging
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "Fix bug Improve error handling Add logging"
        );
    }

    #[test]
    fn test_stop_merge_on_unindented_line() {
        init_logger();
        let src = r#"
// TODO: Improve API
// Refactor later
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "Improve API"); // Does not merge second line
    }

    #[test]
    fn test_todo_with_line_number() {
        init_logger();
        let src = r#"
// Some comment
// TODO: Implement caching
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].line_number, 3);
        assert_eq!(todos[0].message, "Implement caching");
    }

    #[test]
    fn test_empty_input_no_todos() {
        init_logger();
        let src = "";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_display_todo_output() {
        init_logger();
        let src = "// TODO: Improve logging";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);

        let output = format!("{} - {}", todos[0].line_number, todos[0].message);
        assert_eq!(output, "1 - Improve logging");
    }

    #[test]
    fn test_display_no_todos() {
        init_logger();
        let src = "fn main() {}";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert!(todos.is_empty());
    }

    #[test]
    fn test_basic_framework() {
        init_logger();
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_false_positive_detection() {
        init_logger();
        let src = r#"
let message = "TODO: This should not be detected";
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_multiple_consecutive_todos() {
        init_logger();
        let src = r#"
// TODO: todo1
// TODO: todo2
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);

        assert_eq!(todos.len(), 2);

        // Check their line numbers and messages
        // The first TODO should be on line 2, the second on line 3 (1-based from Pest)
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "todo1");
        assert_eq!(todos[1].line_number, 3);
        assert_eq!(todos[1].message, "todo2");
    }

    #[test]
    fn test_ignore_todo_not_at_beginning() {
        let src = r#"
// This is a comment with a TODO: not at the beginning
fn main() {}
"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(
            todos.len(),
            0,
            "A TODO not at the beginning should not be detected"
        );
    }

    #[test]
    fn test_fixme_with_colon() {
        // Test a comment that uses FIXME with a colon.
        let src = r#"
    // FIXME: Correct the error handling
    "#;
        let config = MarkerConfig {
            markers: vec!["FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].message, "Correct the error handling");
    }

    #[test]
    fn test_fixme_without_colon() {
        // Test a comment that uses FIXME without a colon.
        let src = r#"
    // FIXME Correct the error handling
    "#;
        let config = MarkerConfig {
            markers: vec!["FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].message, "Correct the error handling");
    }

    #[test]
    fn test_mixed_markers() {
        // Test a file that mixes both TODO and FIXME comments,
        // with and without the colon.
        let src = r#"
    // TODO: Implement feature A
    // FIXME: Fix bug in module
    // Some regular comment
    // TODO Implement feature B
    // FIXME Fix another bug
    "#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);

        // We expect four items in order.
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].message, "Implement feature A");
        assert_eq!(items[1].message, "Fix bug in module");
        assert_eq!(items[2].message, "Implement feature B");
        assert_eq!(items[3].message, "Fix another bug");
    }

    #[test]
    fn test_mixed_markers_complex() {
        // This test mixes both TODO and FIXME comments (with and without a colon),
        // includes multiline comment blocks, and interleaves non-comment code.
        let src = r#"
// TODO: Implement feature A

fn some_function() {
    // This is a normal comment
    // FIXME: Fix bug in module
    println!("Hello, world!");
}

/*
   TODO: Implement feature C
       with additional multiline details
*/

/// FIXME Fix critical bug
///   that occurs on edge cases

// TODO Implement feature B

// FIXME Fix another bug
"#;

        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let items = extract_marked_items(Path::new("file.rs"), src, &config);

        // We expect six separate marked items:
        // 1. "Implement feature A"
        // 2. "Fix bug in module"
        // 3. "Implement feature C with additional multiline details"
        // 4. "Fix critical bug that occurs on edge cases"
        // 5. "Implement feature B"
        // 6. "Fix another bug"
        assert_eq!(items.len(), 6);

        assert_eq!(items[0].message, "Implement feature A");
        assert_eq!(items[1].message, "Fix bug in module");
        assert_eq!(
            items[2].message,
            "Implement feature C with additional multiline details"
        );
        assert_eq!(
            items[3].message,
            "Fix critical bug that occurs on edge cases"
        );
        assert_eq!(items[4].message, "Implement feature B");
        assert_eq!(items[5].message, "Fix another bug");
    }
}
