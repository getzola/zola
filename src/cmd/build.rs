use std::env;

use errors::Result;
use site::Site;

use console;

pub fn build(config_file: &str) -> Result<()> {
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;
    site.load()?;
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()
}
