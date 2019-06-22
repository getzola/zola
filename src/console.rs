use std::env;
use std::error::Error as StdError;
use std::io::Write;
use std::time::Instant;

use atty;
use chrono::Duration;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use errors::Error;
use site::Site;

lazy_static! {
    /// Termcolor color choice.
    /// We do not rely on ColorChoice::Auto behavior
    /// as the check is already performed by has_color.
    static ref COLOR_CHOICE: ColorChoice =
        if has_color() {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        };
}

pub fn info(message: &str) {
    colorize(message, ColorSpec::new().set_bold(true));
}

pub fn warn(message: &str) {
    colorize(message, ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)));
}

pub fn success(message: &str) {
    colorize(message, ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)));
}

pub fn error(message: &str) {
    colorize(message, ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)));
}

/// Print a colorized message to stdout
fn colorize(message: &str, color: &ColorSpec) {
    let mut stdout = StandardStream::stdout(*COLOR_CHOICE);
    stdout.set_color(color).unwrap();
    writeln!(&mut stdout, "{}", message).unwrap();
    stdout.set_color(&ColorSpec::new()).unwrap();
}

/// Display in the console the number of pages/sections in the site
pub fn notify_site_size(site: &Site) {
    let library = site.library.read().unwrap();
    println!(
        "-> Creating {} pages ({} orphan), {} sections, and processing {} images",
        library.pages().len(),
        site.get_number_orphan_pages(),
        library.sections().len() - 1, // -1 since we do not count the index as a section there
        site.num_img_ops(),
    );
}

/// Display a warning in the console if there are ignored pages in the site
pub fn warn_about_ignored_pages(site: &Site) {
    let library = site.library.read().unwrap();
    let ignored_pages: Vec<_> = library
        .sections_values()
        .iter()
        .flat_map(|s| s.ignored_pages.iter().map(|k| library.get_page_by_key(*k).file.path.clone()))
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
    let duration_ms = Duration::from_std(instant.elapsed()).unwrap().num_milliseconds() as f64;

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
