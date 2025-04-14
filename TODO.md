## src/cli.rs
* [src/cli.rs:35](src/cli.rs#L35): add a flag to enable debug logging
* [src/cli.rs:36](src/cli.rs#L36): add configuration to specify the Markers to search for
* [src/cli.rs:120](src/cli.rs#L120): add tests for this branch

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:163](src/todo_extractor_internal/aggregator.rs#L163): Add new extensions and their corresponding parser calls here: "js" => Some(crate::languages::js::JsParser::parse_comments(file_content)), "ts" => Some(crate::languages::ts::TsParser::parse_comments(file_content)),

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection

