use std::convert::TryInto;
use std::time::Instant;
use time::Duration;

use errors::Error;
use site::Site;

/// Display in the console the number of pages/sections in the site
pub fn notify_site_size(site: &Site) {
    log::info!(
        "-> Creating {} pages ({} orphan) and {} sections",
        site.library.pages.len(),
        site.library.get_all_orphan_pages().len(),
        site.library.sections.len() - 1, // -1 since we do not count the index as a section there
    );
}

/// Display in the console only the number of pages/sections in the site
pub fn check_site_summary(site: &Site) {
    let orphans = site.library.get_all_orphan_pages();
    log::info!(
        "-> Site content: {} pages ({} orphan), {} sections",
        site.library.pages.len(),
        orphans.len(),
        site.library.sections.len() - 1, // -1 since we do not count the index as a section there
    );

    for orphan in orphans {
        log::warn!("Orphan page found: {}", orphan.path);
    }
}

/// Display a warning in the console if there are ignored pages in the site
pub fn warn_about_ignored_pages(site: &Site) {
    let ignored_pages: Vec<_> = site
        .library
        .sections
        .values()
        .flat_map(|s| s.ignored_pages.iter().map(|k| site.library.pages[k].file.path.clone()))
        .collect();

    if !ignored_pages.is_empty() {
        log::warn!(
            "{} page(s) ignored (missing date or weight in a sorted section):",
            ignored_pages.len()
        );
        for path in ignored_pages {
            log::warn!("- {}", path.display());
        }
    }
}

/// Print the time elapsed rounded to 1 decimal
pub fn report_elapsed_time(instant: Instant) {
    let duration: Duration = instant.elapsed().try_into().unwrap();
    let duration_ms = duration.whole_milliseconds() as f64;

    if duration_ms < 1000.0 {
        log::info!("Done in {duration_ms}ms.\n");
    } else {
        let duration_sec = duration_ms / 1000.0;
        log::info!("Done in {:.1}s.\n", ((duration_sec * 10.0).round() / 10.0));
    }
}

/// Display an error message and the actual error(s)
pub fn unravel_errors(message: &str, error: &Error) {
    if !message.is_empty() {
        log::error!("{message}");
    }
    log::error!("{error}");
    let mut cause = error.source();
    while let Some(e) = cause {
        log::error!("Reason: {e}");
        cause = e.source();
    }
}
