use std::path::Path;

use errors::{Error, Result};
use site::Site;

use crate::console;
use crate::prompt::ask_bool;

pub fn build(
    root_dir: &Path,
    config_file: &Path,
    base_url: Option<&str>,
    output_dir: Option<&Path>,
    include_drafts: bool,
) -> Result<()> {
    let mut site = Site::new(root_dir, config_file)?;
    if let Some(output_dir) = output_dir {
        // Check whether output directory exists or not
        // This way we don't replace already existing files.
        if output_dir.exists() {
            console::warn("The directory you are about build to already exists. Building to this directory will delete files contained within this directory.");

            // Prompt the user to ask whether they want to continue.
            if !ask_bool("Are you you want to continue?", false)? {
                return Err(Error::msg("Cancelled build process because output directory already exists."));
            }
        }

        site.set_output_path(output_dir);
    }
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    if include_drafts {
        site.include_drafts();
    }
    site.load()?;
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()
}
