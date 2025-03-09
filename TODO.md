## src/cli.rs
* [src/cli.rs:34](src/cli.rs#L34): add a flag to enable debug logging
* [src/cli.rs:35](src/cli.rs#L35): add configuration to specify the Markers to search for

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:163](src/todo_extractor_internal/aggregator.rs#L163): Add new extensions and their corresponding parser calls here: "js" => Some(crate::languages::js::JsParser::parse_comments(file_content)), "ts" => Some(crate::languages::ts::TsParser::parse_comments(file_content)),

## src/todo_md.rs
* [src/todo_md.rs:20](src/todo_md.rs#L20): edit this to return a dict of maked items, where the key is the type of marker (TODO, FIXME, etc) and the value is a list of MarkedItem
* [src/todo_md.rs:27](src/todo_md.rs#L27): what happen here if the file is malformed? is the file is malformed, we should raise an error and rerun with --all-files to regenerate the file from scratch
* [src/todo_md.rs:31](src/todo_md.rs#L31): this will need to be a 3 way scan 1. scan for the markers (# TODO, # FIXME, etc) 2. scan for the file path (## src/main.rs) 3. scan for the marker line (* [src/main.rs:12](src/main.rs#L12): Refactor this function)

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection

