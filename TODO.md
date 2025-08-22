# TODO
## src/cli.rs
* [src/cli.rs:43](src/cli.rs#L43): add a flag to enable debug logging
* [src/cli.rs:153](src/cli.rs#L153): add tests for this branch

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:221](src/todo_extractor_internal/aggregator.rs#L221): Add new extensions and their corresponding parser calls here: Currently supported extensions: "js", "jsx", "go", "py", "rs". Example for adding a new extension: "ts" | "tsx" => Some(crate::languages::ts::TsParser::parse_comments),

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection
