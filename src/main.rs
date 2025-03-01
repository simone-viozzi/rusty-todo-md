use rusty_todo_md::{cli, logger};

fn main() {
    env_logger::Builder::from_default_env()
        .format(logger::format_logger)
        .init();
    cli::run_cli();
}
