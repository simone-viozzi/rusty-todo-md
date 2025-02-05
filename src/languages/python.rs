use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;
use log::debug;

// Load grammar at compile time
#[derive(Parser)]
#[grammar = "languages/python.pest"]
pub struct PythonParser;

/// Parse the entire Python file content, returning lines of comment text (and line numbers).
pub fn parse_python_comments(file_content: &str) -> Vec<CommentLine> {
    debug!("parse_python_comments: start, input length = {}", file_content.len());

    let parse_result = PythonParser::parse(Rule::python_file, file_content);
    let mut comments = Vec::new();

    match parse_result {
        Ok(pairs) => {
            debug!("parse_python_comments: parse OK, top-level pairs len = {}", pairs.clone().count());
            for pair in pairs {
                debug!(" - pair = {:?}", pair.as_rule());
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
        Err(e) => {
            debug!("parse_python_comments: parse ERR => {}", e);
        }
    }
    debug!(
        "parse_python_comments: returning {} comment lines",
        comments.len()
    );

    comments
}

/// If it's a single-line `# ...`, store exactly that line.
/// If it's a docstring_comment, store each line inside that triple-quoted block.
fn handle_comment_token(pair: &Pair<Rule>, out: &mut Vec<CommentLine>) {
    let rule = pair.as_rule();
    debug!("handle_comment_token: rule = {:?}", rule);

    match rule {
        Rule::line_comment => {
            let span = pair.as_span();
            let start_pos = span.start_pos().line_col().0; // line number (1-based)
            let text = span.as_str();

            debug!(
                " -> line_comment line={} raw='{}'",
                start_pos, text
            );

            // remove leading '#'
            let stripped = text.trim_start_matches('#').trim_end().to_string();

            out.push(CommentLine {
                line_number: start_pos,
                text: stripped,
            });
        }
        Rule::docstring_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let block_text = span.as_str();

            debug!(
                " -> docstring_comment lines start={} raw_len={}",
                start_line,
                block_text.len()
            );

            // remove the surrounding """..."""
            let trimmed = block_text
                .trim_start_matches("\"\"\"")
                .trim_end_matches("\"\"\"");

            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                debug!(
                    "    docstring line={} => '{}'",
                    current_line, line_text
                );
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim_end().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}
