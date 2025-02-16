// src/languages/rust.rs

use crate::aggregator::{parse_comments, CommentLine};
use crate::languages::common::CommentParser; // Import the trait
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "languages/rust.pest"]
pub struct RustParser;

impl CommentParser for RustParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::rust_file, file_content)
    }
}
