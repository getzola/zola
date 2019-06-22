use std::env;
use std::path::PathBuf;

use errors::Result;
use site::Site;

use console;

pub fn check(config_file: &str, base_path: Option<&str>, base_url: Option<&str>) -> Result<()> {
    let bp = base_path.map(PathBuf::from).unwrap_or(env::current_dir().unwrap());
    let mut site = Site::new(bp, config_file)?;
    // Force the checking of external links
    site.config.check_external_links = true;
    // Disable syntax highlighting since the results won't be used
    // and this operation can be expensive.
    site.config.highlight_code = false;
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    site.load()?;
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    Ok(())
}
