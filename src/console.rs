use std::time::Instant;

use chrono::Duration;
use term_painter::ToStyle;
use term_painter::Color::*;

use errors::Error;
use site::Site;


pub fn info(message: &str) {
    println!("{}", NotSet.bold().paint(message));
}

pub fn warn(message: &str) {
    println!("{}", Yellow.bold().paint(message));
}

pub fn success(message: &str) {
    println!("{}", Green.bold().paint(message));
}

pub fn error(message: &str) {
    println!("{}", Red.bold().paint(message));
}

/// Display in the console the number of pages/sections in the site
pub fn notify_site_size(site: &Site) {
    println!(
        "-> Creating {} pages ({} orphan), {} sections, and processing {} images",
        site.pages.len(),
        site.get_all_orphan_pages().len(),
        site.sections.len() - 1, // -1 since we do not the index as a section
        site.num_img_ops(),
    );
}

/// Display a warning in the console if there are ignored pages in the site
pub fn warn_about_ignored_pages(site: &Site) {
    let ignored_pages: Vec<_> = site.sections
        .values()
        .flat_map(|s| s.ignored_pages.iter().map(|p| p.file.path.clone()))
        .collect();

    if !ignored_pages.is_empty() {
        warn(&format!(
            "{} page(s) ignored (missing date or order in a sorted section):",
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
    for e in error.iter().skip(1) {
        self::error(&format!("Reason: {}", e));
    }
}
