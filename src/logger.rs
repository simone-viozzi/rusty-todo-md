use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

/// Returns the log level as a fixed-width string, optionally wrapped in ANSI color codes.
fn colored_level(level: log::Level, color_enabled: bool) -> String {
    // Use a fixed-width string so that levels align.
    let level_str = match level {
        log::Level::Error => "ERROR",
        log::Level::Warn  => "WARN ",
        log::Level::Info  => "INFO ",
        log::Level::Debug => "DEBUG",
        log::Level::Trace => "TRACE",
    };

    if color_enabled {
        match level {
            log::Level::Error => format!("\x1b[31m{}\x1b[0m", level_str), // red
            log::Level::Warn  => format!("\x1b[33m{}\x1b[0m", level_str), // yellow
            log::Level::Info  => format!("\x1b[32m{}\x1b[0m", level_str), // green
            log::Level::Debug => format!("\x1b[34m{}\x1b[0m", level_str), // blue
            log::Level::Trace => format!("\x1b[35m{}\x1b[0m", level_str), // magenta
        }
    } else {
        level_str.to_string()
    }
}

/// Custom formatter that produces output similar to the default env_logger format,
/// but appends a clickable file:line (plain text) and colors the level.
pub fn format_logger(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    // Determine if color is enabled.
    // Here we check the MY_LOG_STYLE environment variable: if it is "never", disable color.
    // (A more robust solution might also check if stdout is a TTY.)
    let color_enabled = std::env::var("MY_LOG_STYLE")
        .map(|s| s != "never")
        .unwrap_or(true);

    // Get the timestamp (env_logger already formats this).
    let ts = buf.timestamp();

    // Get the colored level as a fixed-width string.
    let level = colored_level(record.level(), color_enabled);

    // Get the target (typically the module path).
    let target = record.target();

    // Build a plain-text file:line string (if available) that VSCode can detect.
    let file_line = match (record.file(), record.line()) {
        (Some(file), Some(line)) => format!("{}:{}", file, line),
        _ => String::new(),
    };

    // Example output:
    // 2025-02-16T17:52:07Z DEBUG [my_crate::module] This is a log message (src/file.rs:42)
    writeln!(
        buf,
        "{} {} [{} - {}] {}",
        ts,
        level,
        target,
        file_line,
        record.args(),
    )
}
