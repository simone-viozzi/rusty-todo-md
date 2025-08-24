# TODO
## .github/workflows/release.yml
* [.github/workflows/release.yml:403](.github/workflows/release.yml#L403): This is a smoke test

## src/cli.rs
* [src/cli.rs:50](src/cli.rs#L50): add a flag to enable debug logging
* [src/cli.rs:186](src/cli.rs#L186): simplify this, maybe move to git_utils and maybe do not check if content changed but just try to add it and ignore errors in case it was not modified

## src/todo_extractor_internal/aggregator.rs
* [src/todo_extractor_internal/aggregator.rs:208](src/todo_extractor_internal/aggregator.rs#L208): Add new extensions and their corresponding parser calls here: Currently supported extensions: "js", "jsx", "go", "py", "rs". Example for adding a new extension: "ts" | "tsx" => Some(crate::languages::ts::TsParser::parse_comments),

## src/todo_extractor_internal/languages/dockerfile.rs
* [src/todo_extractor_internal/languages/dockerfile.rs:29](src/todo_extractor_internal/languages/dockerfile.rs#L29): now in the tests i need to actually create the file instead of passing a fake path and a content
* [src/todo_extractor_internal/languages/dockerfile.rs:32](src/todo_extractor_internal/languages/dockerfile.rs#L32): and also need to check errors

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection
