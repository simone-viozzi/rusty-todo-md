use env_logger::fmt::Formatter;
use log::Record;
use std::io::Write;

pub fn format_logger(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    writeln!(
        buf,
        "{}:{} - {}",
        record.file().unwrap_or("unknown"),
        record.line().unwrap_or(0),
        record.args()
    )
}
