use std::env;
use std::path::PathBuf;

use errors::Result;
use site::Site;

use console;

pub fn check(config_file: &str, base_path: Option<&str>, base_url: Option<&str>) -> Result<()> {
    let bp = base_path.map(PathBuf::from).unwrap_or_else(|| env::current_dir().unwrap());
    let mut site = Site::new(bp, config_file)?;
    // Force the checking of external links
    site.config.enable_check_mode();
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    site.load()?;
    console::notify_site_size_simple(&site);
    console::warn_about_ignored_pages(&site);
    Ok(())
}
