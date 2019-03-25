use std::env;
use std::path::PathBuf;

use errors::Result;
use site::Site;

use console;

pub fn build(
    config_file: &str,
    base_path: Option<&str>,
    base_url: Option<&str>,
    output_dir: &str,
) -> Result<()> {
    let bp = base_path.map(PathBuf::from).unwrap_or(env::current_dir().unwrap());
    let mut site = Site::new(bp, config_file)?;
    site.set_output_path(output_dir);
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    site.load()?;
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()
}
