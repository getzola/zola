use std::env;
use std::io::Write;

use libs::atty;
use libs::once_cell::sync::Lazy;
use libs::termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Termcolor color choice.
/// We do not rely on ColorChoice::Auto behavior
/// as the check is already performed by has_color.
static COLOR_CHOICE: Lazy<ColorChoice> =
    Lazy::new(|| if has_color() { ColorChoice::Always } else { ColorChoice::Never });

pub fn info(message: &str) {
    colorize(message, ColorSpec::new().set_bold(true), StandardStream::stdout(*COLOR_CHOICE));
}

pub fn warn(message: &str) {
    colorize(
        &format!("{}{}", "Warning: ", message),
        ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)),
        StandardStream::stdout(*COLOR_CHOICE),
    );
}

pub fn success(message: &str) {
    colorize(
        message,
        ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)),
        StandardStream::stdout(*COLOR_CHOICE),
    );
}

pub fn error(message: &str) {
    colorize(
        &format!("{}{}", "Error: ", message),
        ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)),
        StandardStream::stderr(*COLOR_CHOICE),
    );
}

/// Print a colorized message to stdout
fn colorize(message: &str, color: &ColorSpec, mut stream: StandardStream) {
    stream.set_color(color).unwrap();
    write!(stream, "{}", message).unwrap();
    stream.set_color(&ColorSpec::new()).unwrap();
    writeln!(stream).unwrap();
}

/// Check whether to output colors
fn has_color() -> bool {
    let use_colors = env::var("CLICOLOR").unwrap_or_else(|_| "1".to_string()) != "0"
        && env::var("NO_COLOR").is_err();
    let force_colors = env::var("CLICOLOR_FORCE").unwrap_or_else(|_| "0".to_string()) != "0";

    force_colors || use_colors && atty::is(atty::Stream::Stdout)
}
