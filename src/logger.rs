use env_logger::fmt::style::{AnsiColor, Effects, Style};
use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

use log::Level;

fn colored_level(level: Level, color_enabled: bool) -> String {
    // Use fixed-width strings for alignment.
    let level_str = match level {
        Level::Error => "ERROR",
        Level::Warn => "WARN ",
        Level::Info => "INFO ",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    };

    if color_enabled {
        // Build the style using the re‑exported anstyle types.
        let style: Style = match level {
            Level::Error => AnsiColor::Red.on_default().effects(Effects::BOLD),
            Level::Warn => AnsiColor::Yellow.on_default().effects(Effects::BOLD),
            Level::Info => AnsiColor::Green.on_default(),
            Level::Debug => AnsiColor::Blue.on_default(),
            Level::Trace => AnsiColor::Magenta.on_default(),
        };

        // Format using the style's Display impl:
        // - `{}` outputs the escape code to set the style,
        // - `{:#}` outputs the reset code.
        format!("{style}{level_str}{style:#}")
    } else {
        level_str.to_string()
    }
}

/// Custom formatter that produces output similar to the default env_logger format,
/// but appends a clickable file:line (plain text) and styles the level.
pub fn format_logger(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    // Determine if color is enabled.
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
        (Some(file), Some(line)) => format!("{file}:{line}"),
        _ => String::new(),
    };

    // Example output:
    // 2025-02-16T17:52:07Z DEBUG [my_crate::module - src/file.rs:42] This is a log message
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
