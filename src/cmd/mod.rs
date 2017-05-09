mod init;
mod build;
mod serve;

pub use self::init::create_new_project;
pub use self::build::build;
pub use self::serve::serve;

use gutenberg::Site;

use console::warn;

fn notify_site_size(site: &Site) {
    println!("-> Creating {} pages and {} sections", site.pages.len(), site.sections.len());
}

fn warn_about_ignored_pages(site: &Site) {
    let ignored_pages = site.get_ignored_pages();
    if !ignored_pages.is_empty() {
        warn(&format!(
            "{} page(s) ignored (missing date or order in a sorted section):",
            ignored_pages.len()
        ));
        for path in site.get_ignored_pages() {
            warn(&format!("- {}", path.display()));
        }
    }
}
