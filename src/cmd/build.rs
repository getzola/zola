use std::path::Path;

use errors::{Error, Result};
use site::Site;

use crate::messages;

pub fn build(
    root_dir: &Path,
    config_file: &Path,
    base_url: Option<&str>,
    output_dir: Option<&Path>,
    force: bool,
    include_drafts: bool,
    minify: bool,
) -> Result<()> {
    let mut site = Site::new(root_dir, config_file)?;
    if let Some(output_dir) = output_dir {
        if !force && output_dir.exists() {
            return Err(Error::msg(format!(
                "Directory '{}' already exists. Use --force to overwrite.",
                output_dir.display(),
            )));
        }

        site.set_output_path(output_dir);
    }
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    if include_drafts {
        site.include_drafts();
    }
    if minify {
        site.minify();
    }
    site.load()?;
    messages::notify_site_size(&site);
    messages::warn_about_ignored_pages(&site);
    site.build()
}
