use pest::Parser;
use pest::iterators::{Pairs, Pair};
use crate::aggregator::CommentLine;

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = RustParser::parse(Rule::rust_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            match pair.as_rule() {
                Rule::comment_rust => {
                    handle_rust_comment(pair, &mut comments);
                }
                _ => {}
            }
        }
    }
    comments
}

fn handle_rust_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // Remove leading `//`
            let stripped = text.trim_start_matches('/').trim_start_matches('/').trim_start();
            out.push(CommentLine {
                line_number: line,
                text: stripped.to_string(),
            });
        }
        Rule::block_comment => {
            let span = pair.as_span();
            let start_line = span.start_pos().line_col().0;
            let end_line = span.end_pos().line_col().0;
            let block_text = span.as_str();
            // remove `/*` and `*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            // split by lines
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim().to_string(),
                });
                current_line += 1;
            }
        }
        Rule::doc_comment_line => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            // remove leading `///` or `//!`
            let stripped = if text.starts_with("///") {
                &text[3..]
            } else {
                &text[3..] // `//!`
            };
            out.push(CommentLine {
                line_number: line,
                text: stripped.trim().to_string(),
            });
        }
        _ => {}
    }
}
