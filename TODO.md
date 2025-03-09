## src/cli.rs
* [src/cli.rs:34](src/cli.rs#L34): add a flag to enable debug logging
* [src/cli.rs:35](src/cli.rs#L35): add configuration to specify the Markers to search for

## src/todo_md.rs
* [src/todo_md.rs:20](src/todo_md.rs#L20): edit this to return a dict of maked items, where the key is the type of marker (TODO, FIXME, etc) and the value is a list of MarkedItem
* [src/todo_md.rs:27](src/todo_md.rs#L27): what happen here if the file is malformed? is the file is malformed, we should raise an error and rerun with --all-files to regenerate the file from scratch

