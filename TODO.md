* [src//cli.rs:10](src//cli.rs#L10): add a new argument to specify what markers to look for like --markers "TODO, FIXME, HACK"
* [src/todo_extractor_internal//aggregator.rs:19](src/todo_extractor_internal//aggregator.rs#L19): make sure we strip : from markers, so that TODO and TODO: are treated the same
