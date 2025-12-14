use std::io::Write;

use anstream::{
    AutoStream,
    stream::{AsLockedWrite, RawStream},
};
use anstyle::{AnsiColor, Color, Style};

pub fn info(message: &str) {
    colorize(message, &Style::new().bold(), AutoStream::auto(std::io::stdout()));
}

pub fn success(message: &str) {
    colorize(
        message,
        &Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green))),
        AutoStream::auto(std::io::stdout()),
    );
}

/// Print a colorized message to stdout
fn colorize<S>(message: &str, color: &Style, mut stream: AutoStream<S>)
where
    S: RawStream + AsLockedWrite,
{
    writeln!(stream, "{color}{}{color:#}", message).unwrap();
}
