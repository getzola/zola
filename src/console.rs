use std::io::Write;
use std::time::Instant;
use std::{convert::TryInto, env};

use libs::once_cell::sync::Lazy;
use libs::time::Duration;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use errors::Error;
use site::Site;

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
        message,
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
        message,
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

/// Display in the console the number of pages/sections in the site
pub fn notify_site_size(site: &Site) {
    let library = site.library.read().unwrap();
    println!(
        "-> Creating {} pages ({} orphan) and {} sections",
        library.pages.len(),
        library.get_all_orphan_pages().len(),
        library.sections.len() - 1, // -1 since we do not count the index as a section there
    );
}

/// Display in the console only the number of pages/sections in the site
pub fn check_site_summary(site: &Site) {
    let library = site.library.read().unwrap();
    let orphans = library.get_all_orphan_pages();
    println!(
        "-> Site content: {} pages ({} orphan), {} sections",
        library.pages.len(),
        orphans.len(),
        library.sections.len() - 1, // -1 since we do not count the index as a section there
    );

    for orphan in orphans {
        warn(&format!("Orphan page found: {}", orphan.path));
    }
}

/// Display a warning in the console if there are ignored pages in the site
pub fn warn_about_ignored_pages(site: &Site) {
    let library = site.library.read().unwrap();
    let ignored_pages: Vec<_> = library
        .sections
        .values()
        .flat_map(|s| s.ignored_pages.iter().map(|k| library.pages[k].file.path.clone()))
        .collect();

    if !ignored_pages.is_empty() {
        warn(&format!(
            "{} page(s) ignored (missing date or weight in a sorted section):",
            ignored_pages.len()
        ));
        for path in ignored_pages {
            warn(&format!("- {}", path.display()));
        }
    }
}

/// Print the time elapsed rounded to 1 decimal
pub fn report_elapsed_time(instant: Instant) {
    let duration: Duration = instant.elapsed().try_into().unwrap();
    let duration_ms = duration.whole_milliseconds() as f64;

    if duration_ms < 1000.0 {
        success(&format!("Done in {}ms.\n", duration_ms));
    } else {
        let duration_sec = duration_ms / 1000.0;
        success(&format!("Done in {:.1}s.\n", ((duration_sec * 10.0).round() / 10.0)));
    }
}

/// Display an error message and the actual error(s)
pub fn unravel_errors(message: &str, error: &Error) {
    if !message.is_empty() {
        self::error(message);
    }
    self::error(&format!("Error: {}", error));
    let mut cause = error.source();
    while let Some(e) = cause {
        self::error(&format!("Reason: {}", e));
        cause = e.source();
    }
}

/// Check whether to output colors
fn has_color() -> bool {
    let use_colors = env::var("CLICOLOR").unwrap_or_else(|_| "1".to_string()) != "0"
        && env::var("NO_COLOR").is_err();
    let force_colors = env::var("CLICOLOR_FORCE").unwrap_or_else(|_| "0".to_string()) != "0";

    force_colors || use_colors && atty::is(atty::Stream::Stdout)
}
