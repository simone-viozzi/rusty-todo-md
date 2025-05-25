## src/cli.rs
* [src/cli.rs:43](src/cli.rs#L43): add a flag to enable debug logging
* [src/cli.rs:139](src/cli.rs#L139): add tests for this branch

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:175](src/todo_extractor_internal/aggregator.rs#L175): Add new extensions and their corresponding parser calls here: "js" => Some(crate::languages::js::JsParser::parse_comments(file_content)), "ts" => Some(crate::languages::ts::TsParser::parse_comments(file_content)),

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection

