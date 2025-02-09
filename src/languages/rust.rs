use crate::aggregator::{parse_comments, CommentLine};
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

pub fn parse_rust_comments(file_content: &str) -> Vec<CommentLine> {
    parse_comments::<RustParser, Rule>(PhantomData, Rule::rust_file, file_content)
}
