use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;

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
                    handle_comment_token(&pair, &mut comments);
                }
                _ => {
                    // ignore everything else
                }
            }
        }
    }
    comments
}

/// If it's a single-line `# ...`, store exactly that line.
/// If it's a docstring_comment, store each line inside that triple-quoted block.
fn handle_comment_token(pair: &Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col().0; // line number (1-based)
            // The actual text (including `#`)
            let text = span.as_str();
            // remove leading '#'
            let stripped = text.trim_start_matches('#').trim_end().to_string();

            out.push(CommentLine {
                line_number: start_pos,
                text: stripped,
            });
        }
        Rule::docstring_comment => {
            // multi-line docstring
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();

            // remove the surrounding """..."""
            let trimmed = block_text
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"");

            // naive approach: each line is separate
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
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
