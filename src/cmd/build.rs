use std::env;

use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build(config_file: &str) -> Result<()> {
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;
    site.load()?;
    super::notify_site_size(&site);
    super::warn_about_ignored_pages(&site);
    site.build()
}
