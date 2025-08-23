use log::debug;
use std::path::Path;
use std::{marker::PhantomData, path::PathBuf};

use crate::todo_extractor_internal::languages::common::CommentParser;
use crate::todo_extractor_internal::languages::common_syntax;
use log::{error, info};
use pest::Parser;

/// Represents a single found marked item.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct MarkedItem {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub message: String,
    pub marker: String, // NEW: The marker (e.g., TODO, FIXME)
}

/// Configuration for comment markers.
pub struct MarkerConfig {
    pub markers: Vec<String>,
}

impl MarkerConfig {
    /// Normalize all markers: strip trailing colons and whitespace.
    pub fn normalized(markers: Vec<String>) -> Self {
        let markers = markers
            .into_iter()
            .map(|m| m.trim().trim_end_matches(':').trim().to_string())
            .collect();
        MarkerConfig { markers }
    }
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
                    //debug!(
                    //    "Processing child pair: {:?} => '{}'",
                    //    inner_pair.as_rule(),
                    //    inner_pair.as_str().replace('\n', "\\n")
                    //);

                    if let Some(comment) = extract_comment_from_pair(inner_pair) {
                        debug!("Extracted comment: {comment:?}",);
                        comments.push(comment);
                    } else {
                        //debug!("Skipped non-comment pair.");
                    }
                }
            }
        }
        Err(e) => {
            error!("Parsing error: {e:?}");
        }
    }

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

/// Determines the effective extension for a file, handling special cases like Dockerfile.
///
/// - `path`: The file path to analyze.
/// - Returns: The effective extension as a string.
pub fn get_effective_extension(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Handle special filenames like Dockerfile which have no extension
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension.is_empty() && file_name == "dockerfile" {
        "dockerfile".to_string()
    } else {
        extension
    }
}

/// Returns the appropriate parser function for a given file extension.
///
/// - `extension`: The file extension (e.g., "py", "rs").
/// - Returns: An `Option` containing the parser function if supported.
pub fn get_parser_for_extension(
    extension: &str,
    file_path: &Path,
) -> Option<fn(&str) -> Vec<CommentLine>> {
    let result: Option<fn(&str) -> Vec<CommentLine>> = match extension {
        "py" => {
            Some(crate::todo_extractor_internal::languages::python::PythonParser::parse_comments)
        }
        "rs" => Some(crate::todo_extractor_internal::languages::rust::RustParser::parse_comments),
        "js" | "jsx" | "mjs" => {
            Some(crate::todo_extractor_internal::languages::js::JsParser::parse_comments)
        }
        "ts" | "tsx" | "java" | "cpp" | "hpp" | "cc" | "hh" | "cs" | "swift" | "kt" | "kts"
        | "json" => Some(crate::todo_extractor_internal::languages::js::JsParser::parse_comments),
        "go" => Some(crate::todo_extractor_internal::languages::go::GoParser::parse_comments),
        "sh" => Some(crate::todo_extractor_internal::languages::shell::ShellParser::parse_comments),
        "yml" | "yaml" => {
            Some(crate::todo_extractor_internal::languages::yaml::YamlParser::parse_comments)
        }
        "toml" => Some(crate::todo_extractor_internal::languages::toml::TomlParser::parse_comments),
        "dockerfile" => Some(
            crate::todo_extractor_internal::languages::dockerfile::DockerfileParser::parse_comments,
        ),
        "sql" => Some(crate::todo_extractor_internal::languages::sql::SqlParser::parse_comments),
        "md" => Some(
            crate::todo_extractor_internal::languages::markdown::MarkdownParser::parse_comments,
        ),
        // TODO Add new extensions and their corresponding parser calls here:
        //      Currently supported extensions: "js", "jsx", "go", "py", "rs".
        //      Example for adding a new extension:
        //      "ts" | "tsx" => Some(crate::languages::ts::TsParser::parse_comments),
        _ => None,
    };

    // Log the result
    match &result {
        Some(_) => {
            info!("file {:?} have a valid parser", file_path);
        }
        None => {
            debug!(
                "No parser found for extension '{}' in file: {:?}",
                extension, file_path
            );
        }
    }

    result
}

/// Extracts marked items using a provided parser function.
///
/// **RECOMMENDED**: This is the preferred function for extracting marked items as it allows
/// the caller to validate file extensions before reading file content, improving performance
/// for large files with unsupported extensions.
///
/// # Usage
/// ```rust,no_run
/// use std::path::Path;
/// use rusty_todo_md::todo_extractor::get_parser_for_extension;
/// use rusty_todo_md::todo_extractor::get_effective_extension;
/// use rusty_todo_md::todo_extractor::extract_marked_items_with_parser;
///
///
///
/// let path = Path::new("example.rs");
/// if let Some(parser_fn) = get_parser_for_extension(&get_effective_extension(path)) {
///     config = MarkerConfig::default();
///     let content = std::fs::read_to_string(path).unwrap();
///     let items = extract_marked_items_with_parser(path, &content, parser_fn, &config);
/// }
/// ```
///
/// - `path`: The path to the file.
/// - `file_content`: The source code text.
/// - `parser_fn`: The parser function to use for extracting comments (obtained from `get_parser_for_extension`).
/// - `config`: The marker configuration.
/// - Returns: A `Vec<MarkedItem>` containing extracted marked items.
pub fn extract_marked_items_with_parser(
    path: &Path,
    file_content: &str,
    parser_fn: fn(&str) -> Vec<CommentLine>,
    config: &MarkerConfig,
) -> Vec<MarkedItem> {
    debug!("extract_marked_items_with_parser for file {path:?}");

    let comment_lines = parser_fn(file_content);

    debug!(
        "extract_marked_items_with_parser: found {} comment lines from parser: {:?}",
        comment_lines.len(),
        comment_lines
    );

    // Continue with the existing logic to collect and merge marked items.
    let marked_items = collect_marked_items_from_comment_lines(&comment_lines, config, path);
    debug!(
        "extract_marked_items_with_parser: found {} marked items total",
        marked_items.len()
    );
    marked_items
}

pub fn extract_marked_items_from_file(
    file: &Path,
    marker_config: &MarkerConfig,
) -> Result<Vec<MarkedItem>, String> {
    let effective_ext = get_effective_extension(file);
    let parser_fn = match get_parser_for_extension(&effective_ext, file) {
        Some(parser) => parser,
        None => {
            // Skip unsupported file types without reading content
            return Ok(Vec::new());
        }
    };

    match std::fs::read_to_string(file) {
        Ok(content) => {
            let todos = extract_marked_items_with_parser(file, &content, parser_fn, marker_config);
            Ok(todos)
        }
        Err(e) => {
            error!("Warning: Could not read file {file:?}, skipping. Error: {e}");
            Err(format!("Could not read file {:?}: {}", file, e))
        }
    }
}

/// A single comment line with (line_number, entire_comment_text).
#[derive(Debug, Clone)]
pub struct CommentLine {
    pub line_number: usize,
    pub text: String,
}

/// Merge flattened and stripped comment lines into blocks and produce a `MarkedItem` for each block.
/// A block is defined as a group of lines that starts with a marker (e.g. "TODO:" or "FIXME")
/// and includes any immediately indented lines (which are treated as continuations).
pub fn collect_marked_items_from_comment_lines(
    lines: &[CommentLine],
    config: &MarkerConfig,
    path: &Path,
) -> Vec<MarkedItem> {
    // First, flatten multi-line comments and strip language-specific markers.
    let stripped_lines = strip_and_flatten(lines);
    // Group the lines into blocks based on marker lines and their indented continuations.
    let blocks = group_lines_into_blocks_with_marker(stripped_lines, &config.markers);
    // Convert each block into a MarkedItem.
    blocks
        .into_iter()
        .map(|(line_number, marker, block)| MarkedItem {
            file_path: path.to_path_buf(),
            line_number,
            message: process_block_lines(&block, &config.markers),
            marker,
        })
        .collect()
}

/// Utility: Flattens multi-line comment entries and strips language-specific markers from each line.
fn strip_and_flatten(lines: &[CommentLine]) -> Vec<CommentLine> {
    flatten_comment_lines(lines)
        .into_iter()
        .map(|cl| CommentLine {
            line_number: cl.line_number,
            text: common_syntax::strip_markers(&cl.text),
        })
        .collect()
}

/// Utility: Groups stripped comment lines into blocks. Each block is a tuple containing:
/// - The line number where the block starts (i.e. the marker line)
/// - The marker string that matched (always the base marker, no colon)
/// - A vector of strings representing the block’s lines (with markers already stripped)
fn group_lines_into_blocks_with_marker(
    lines: Vec<CommentLine>,
    markers: &[String],
) -> Vec<(usize, String, Vec<String>)> {
    let mut blocks = Vec::new();
    let mut current_block: Option<(usize, String, Vec<String>)> = None;

    for cl in lines {
        let trimmed = cl.text.trim().to_string();
        // Try to match any marker at the start of the line.
        // Accept if the marker is followed by nothing, a space, or a colon.
        // Always store the base marker (no colon) in the result.
        let matched_marker = markers.iter().find_map(|base| {
            if let Some(rest) = trimmed.strip_prefix(base) {
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with(':') {
                    return Some(base.clone());
                }
            }
            None
        });
        if let Some(marker) = matched_marker {
            // If we were already collecting a block, push it before starting a new one.
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }
            // Start a new block with the marker line.
            current_block = Some((cl.line_number, marker, vec![trimmed]));
        } else if let Some((_, _, ref mut block_lines)) = current_block {
            // If the line is indented, treat it as a continuation of the current block.
            if cl.text.starts_with(' ') || cl.text.starts_with('\t') {
                block_lines.push(trimmed);
            } else {
                // If not indented, close the current block.
                blocks.push(current_block.take().unwrap());
            }
        }
        // Lines that are not marker lines and not indented within a block are ignored.
    }

    // Push any remaining block at the end.
    if let Some(block) = current_block {
        blocks.push(block);
    }
    blocks
}

/// Merges the given block lines into a single normalized message and removes the marker prefix.
/// It also removes an optional colon (":") that immediately follows the marker.
/// For example, if the block lines are:
///   ["TODO Implement feature A", "more details"]
/// or
///   ["TODO: Implement feature A", "more details"]
/// the resulting message will be:
///   "Implement feature A more details"
fn process_block_lines(lines: &[String], markers: &[String]) -> String {
    let merged = lines.join(" ");
    markers.iter().fold(merged, |acc, marker| {
        if let Some(stripped) = acc.strip_prefix(marker) {
            // If a colon immediately follows the marker, remove it.
            let stripped = if let Some(rest) = stripped.strip_prefix(":") {
                rest
            } else {
                stripped
            };
            stripped.trim().to_string()
        } else {
            acc
        }
    })
}

#[cfg(test)]
mod aggregator_tests {
    use super::*;
    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_valid_rust_extension() {
        init_logger();
        let src = "// TODO: Implement feature X";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_js_extension() {
        init_logger();
        let src = "// TODO: Implement feature X";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.js"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_jsx_extension() {
        init_logger();
        let src = "// TODO: Add prop validation";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("component.jsx"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_go_extension() {
        init_logger();
        let src = "// TODO: Implement feature X";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("main.go"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_invalid_extension() {
        init_logger();
        let src = "// TODO: This should not be processed";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.unknown"), src, &config);
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "Fix bug Improve error handling Add logging"
        );
        assert_eq!(todos[0].marker, "TODO:");
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(todos.len(), 0);
    }

    #[test]
    fn test_display_todo_output() {
        init_logger();
        let src = "// TODO: Improve logging";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);

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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);

        assert_eq!(todos.len(), 2);

        // Check their line numbers and messages
        // The first TODO should be on line 2, the second on line 3 (1-based from Pest)
        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "todo1");
        assert_eq!(todos[1].line_number, 3);
        assert_eq!(todos[1].message, "todo2");
    }

    #[test]
    fn test_mixed_marker_configurations() {
        // Test a file that mixes TODO and FIXME, with and without colons.
        let src = r#"
// TODO: Implement feature
// FIXME Fix bug
// TODO Add docs
// FIXME: Refactor
"#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let items = test_extract_marked_items(Path::new("file.rs"), src, &config);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].marker, "TODO");
        assert_eq!(items[0].message, "Implement feature");
        assert_eq!(items[1].marker, "FIXME");
        assert_eq!(items[1].message, "Fix bug");
        assert_eq!(items[2].marker, "TODO");
        assert_eq!(items[2].message, "Add docs");
        assert_eq!(items[3].marker, "FIXME");
        assert_eq!(items[3].message, "Refactor");
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
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let items = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let items = test_extract_marked_items(Path::new("file.rs"), src, &config);
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
        let items = test_extract_marked_items(Path::new("file.rs"), src, &config);

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
        let items = test_extract_marked_items(Path::new("file.rs"), src, &config);

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

    #[test]
    fn test_merge_multiline_todo_with_todo_in_str() {
        init_logger();
        let src = r#"
// TODO add a new argument to specify what markers to look for
//      like --markers "TODO, FIXME, HACK"
"#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("file.rs"), src, &config);

        assert_eq!(todos.len(), 1);

        assert_eq!(todos[0].line_number, 2);
        assert_eq!(todos[0].message, "add a new argument to specify what markers to look for like --markers \"TODO, FIXME, HACK\"");
    }

    #[test]
    fn test_valid_sh_extension() {
        init_logger();
        let src = "# TODO: setup\nexit";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("script.sh"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_yaml_extension() {
        init_logger();
        let src = "# TODO: conf\nkey: val";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("config.yaml"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_toml_extension() {
        init_logger();
        let src = "# TODO: fix\nkey=1";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("config.toml"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_sql_extension() {
        init_logger();
        let src = "-- TODO: q\nSELECT 1;";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("query.sql"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_valid_markdown_extension() {
        init_logger();
        let src = "<!-- TODO: doc -->";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("README.md"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }

    #[test]
    fn test_dockerfile_no_extension() {
        init_logger();
        let src = "# TODO: step\nFROM alpine";
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].marker, "TODO:");
    }
}
