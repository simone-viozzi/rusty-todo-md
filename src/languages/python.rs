// src/languages/python.rs

use crate::aggregator::{parse_comments, CommentLine};
use crate::languages::common::CommentParser; // Import the trait
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "languages/python.pest"]
pub struct PythonParser;

impl CommentParser for PythonParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::python_file, file_content)
    }
}
