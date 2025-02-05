use pest::Parser;
use pest::iterators::Pair;
use crate::aggregator::CommentLine;

#[derive(Parser)]
#[grammar = "languages/ts.pest"]
pub struct TsParser;

pub fn parse_ts_comments(file_content: &str) -> Vec<CommentLine> {
    let parse_result = TsParser::parse(Rule::ts_file, file_content);
    let mut comments = Vec::new();

    if let Ok(pairs) = parse_result {
        for pair in pairs {
            if pair.as_rule() == Rule::comment_ts {
                handle_ts_comment(pair, &mut comments);
            }
        }
    }
    comments
}

fn handle_ts_comment(pair: Pair<Rule>, out: &mut Vec<CommentLine>) {
    match pair.as_rule() {
        Rule::line_comment => {
            let span = pair.as_span();
            let line = span.start_pos().line_col().0;
            let text = span.as_str();
            let stripped = text.trim_start_matches("//").trim();
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
            let trimmed = block_text.trim_start_matches("/*").trim_end_matches("*/");
            let lines = trimmed.split('\n');
            let mut current_line = start_line;
            for l in lines {
                out.push(CommentLine {
                    line_number: current_line,
                    text: l.trim().to_string(),
                });
                current_line += 1;
            }
        }
        _ => {}
    }
}
