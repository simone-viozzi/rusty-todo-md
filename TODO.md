## src/cli.rs
* [src/cli.rs:43](src/cli.rs#L43): add a flag to enable debug logging
* [src/cli.rs:80](src/cli.rs#L80): now it's with _ because we don't use it yet
* [src/cli.rs:133](src/cli.rs#L133): add tests for this branch

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:163](src/todo_extractor_internal/aggregator.rs#L163): Add new extensions and their corresponding parser calls here: "js" => Some(crate::languages::js::JsParser::parse_comments(file_content)), "ts" => Some(crate::languages::ts::TsParser::parse_comments(file_content)),

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection

