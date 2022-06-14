use libs::time::Duration;
use std::convert::TryInto;
use std::time::Instant;

use errors::Error;
use site::Site;

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
        console::warn(&format!("Orphan page found: {}", orphan.path));
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
        console::warn(&format!(
            "{} page(s) ignored (missing date or weight in a sorted section):",
            ignored_pages.len()
        ));
        for path in ignored_pages {
            console::warn(&format!("- {}", path.display()));
        }
    }
}

/// Print the time elapsed rounded to 1 decimal
pub fn report_elapsed_time(instant: Instant) {
    let duration: Duration = instant.elapsed().try_into().unwrap();
    let duration_ms = duration.whole_milliseconds() as f64;

    if duration_ms < 1000.0 {
        console::success(&format!("Done in {}ms.\n", duration_ms));
    } else {
        let duration_sec = duration_ms / 1000.0;
        console::success(&format!("Done in {:.1}s.\n", ((duration_sec * 10.0).round() / 10.0)));
    }
}

/// Display an error message and the actual error(s)
pub fn unravel_errors(message: &str, error: &Error) {
    if !message.is_empty() {
        console::error(message);
    }
    console::error(&error.to_string());
    let mut cause = error.source();
    while let Some(e) = cause {
        console::error(&format!("Reason: {}", e));
        cause = e.source();
    }
}
