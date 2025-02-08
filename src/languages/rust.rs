use crate::aggregator::{parse_comments, CommentLine};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    parse_comments::<RustParser, Rule>(Rule::rust_file, file_content)
}
