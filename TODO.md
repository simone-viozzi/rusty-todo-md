# TODO
## src/cli.rs
* [src/cli.rs:43](src/cli.rs#L43): add a flag to enable debug logging
* [src/cli.rs:139](src/cli.rs#L139): add tests for this branch

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:211](src/todo_extractor_internal/aggregator.rs#L211): Add new extensions and their corresponding parser calls here: Currently supported extensions: "js", "jsx", "go", "py", "rs". Example for adding a new extension: "ts" | "tsx" => Some(crate::languages::ts::TsParser::parse_comments),

## src/todo_extractor_internal/languages/dockerfile.rs
* [src/todo_extractor_internal/languages/dockerfile.rs:29](src/todo_extractor_internal/languages/dockerfile.rs#L29): now in the tests i need to actually create the file instead of passing a fake path and a content
* [src/todo_extractor_internal/languages/dockerfile.rs:32](src/todo_extractor_internal/languages/dockerfile.rs#L32): and also need to check errors

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection
