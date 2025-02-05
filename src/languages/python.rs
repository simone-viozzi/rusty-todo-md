use pest::Parser;
use pest::iterators::{Pairs, Pair};
use once_cell::sync::Lazy;

use crate::aggregator::CommentLine;

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
                    handle_comment_token(&pair, file_content, &mut comments);
                }
                _ => {
                    // ignore everything else
                }
            }
        }
    }
    comments
}

/// Here we figure out which lines in the original `file_content` the comment covers.
/// If it's a single-line `# ...`, we store exactly that line.
/// If it's a docstring_comment, we might store each line inside that triple-quoted block.
fn handle_comment_token(pair: &Pair<Rule>, file_content: &str, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col().0; // line number (1-based)
            // The actual text (excluding the #) if we want. Let's just keep the entire comment for now:
            let text = span.as_str().to_string(); // includes `#`
            // We can remove the leading '#' for clarity:
            let stripped = text.trim_start_matches('#').trim_end().to_string();

            out.push(CommentLine {
                line_number: start_pos,
                text: stripped,
            });
        }
        Rule::docstring_comment => {
            // multi-line docstring
            // We'll iterate line by line in this block.
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let end_line = span.end_pos().line_col().0;

            // The entire triple-quoted block
            let block_text = span.as_str();

            // We remove the surrounding """...""" for clarity
            let trimmed = block_text
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"");

            // Now we split by lines, gather them individually
            // We'll do a naive line matching for line numbers.
            let lines: Vec<&str> = trimmed.split('\n').collect();

            // We'll figure out the offset in the original file. The first line of docstring is start_line,
            // but that line also includes """ so let's keep it simple:
            let total_lines = (end_line - start_line) + 1;
            // We'll match them up
            let mut current_line = start_line;
            // We'll do best effort, each line in lines is presumably one line
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
