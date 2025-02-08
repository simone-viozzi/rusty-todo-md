use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;
use pest_derive::Parser;
use log::{debug, info, error};

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    info!("Starting Rust comment parsing. File length: {}", file_content.len());

    let parse_result = RustParser::parse(Rule::rust_file, file_content);
    let mut comments = Vec::new();

    match parse_result {
        Ok(pairs) => {
            debug!("Parsing successful! Found {} top-level pairs.", pairs.clone().count());
            
            for pair in pairs {
                debug!("Pair: {:?} => '{}'", pair.as_rule(), pair.as_str());

                if pair.as_rule() == Rule::comment_rust {
                    debug!("Extracting Rust comment: '{}'", pair.as_str());
                    handle_rust_comment(pair, &mut comments);
                }
            }
        }
        Err(e) => {
            error!("Rust parsing error: {:?}", e);
        }
    }

    info!("Extracted {} Rust comments", comments.len());
    comments
}


fn handle_rust_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            debug!("Line comment found at line {}: '{}'", line, text);
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
            let block_text = span.as_str();
            debug!("Block comment found starting at line {}: length {}", start_line, block_text.len());
            // remove `/*` and `*/`
            let trimmed = block_text
                .trim_start_matches("/*")
                .trim_end_matches("*/");

            // split by lines
            let lines: Vec<&str> = trimmed.split('\n').collect();
            let mut current_line = start_line;
            for line_text in lines {
                debug!("Block comment line {}: '{}'", current_line, line_text);
                out.push(CommentLine {
                    line_number: current_line,
                    text: line_text.trim_end().to_string(),
                });
                current_line += 1;
            }
        }
        Rule::doc_comment_line => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            debug!("Doc comment line found at line {}: '{}'", line, text);
            // remove leading `///` or `//!`
            let stripped = if text.starts_with("///") {
                &text[3..]
            } else {
                &text[3..] // `//!`
            };
            out.push(CommentLine {
                line_number: line,
                text: stripped.trim_end().to_string(),
            });
        }
        _ => {}
    }
}
