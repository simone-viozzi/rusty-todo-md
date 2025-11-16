# TODO
## .github/workflows/release.yml
* [.github/workflows/release.yml:414](.github/workflows/release.yml#L414): This is a smoke test

## src/cli.rs
* [src/cli.rs:65](src/cli.rs#L65): add a flag to enable debug logging
* [src/cli.rs:228](src/cli.rs#L228): simplify this, maybe move to git_utils and maybe do not check if content changed but just try to add it and ignore errors in case it was not modified

## src/todo_extractor_internal/languages/dockerfile.rs
* [src/todo_extractor_internal/languages/dockerfile.rs:32](src/todo_extractor_internal/languages/dockerfile.rs#L32): now in the tests i need to actually create the file instead of passing a fake path and a content
* [src/todo_extractor_internal/languages/dockerfile.rs:35](src/todo_extractor_internal/languages/dockerfile.rs#L35): and also need to check errors

## src/todo_md_internal.rs
* [src/todo_md_internal.rs:6](src/todo_md_internal.rs#L6): generalize in maker collection

## tests/cli_error_tests.rs
* [tests/cli_error_tests.rs:26](tests/cli_error_tests.rs#L26): Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
* [tests/cli_error_tests.rs:55](tests/cli_error_tests.rs#L55): Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
* [tests/cli_error_tests.rs:115](tests/cli_error_tests.rs#L115): Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro

## tests/cli_no_files_tests.rs
* [tests/cli_no_files_tests.rs:24](tests/cli_no_files_tests.rs#L24): Replace Command::cargo_bin() with cargo::cargo_bin_cmd! macro
