use crate::aggregator::{parse_comments, CommentLine};
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "languages/python.pest"]
pub struct PythonParser;

pub fn parse_python_comments(file_content: &str) -> Vec<CommentLine> {
    parse_comments::<PythonParser, Rule>(PhantomData, Rule::python_file, file_content)
}
