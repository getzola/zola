use std::path::Path;

use errors::{Error, Result};
use site::Site;

use crate::messages;
use crate::prompt::ask_bool_timeout;

const BUILD_PROMPT_TIMEOUT_MILLIS: u64 = 10_000;

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
            console::warn(&format!("The directory '{}' already exists. Building to this directory will delete files contained within this directory.", output_dir.display()));

            // Prompt the user to ask whether they want to continue.
            let clear_dir = tokio::runtime::Runtime::new()
                .expect("Tokio runtime failed to instantiate")
                .block_on(ask_bool_timeout(
                    "Are you sure you want to continue?",
                    false,
                    std::time::Duration::from_millis(BUILD_PROMPT_TIMEOUT_MILLIS),
                ))?;

            if !clear_dir {
                return Err(Error::msg(
                    "Cancelled build process because output directory already exists.",
                ));
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
    messages::notify_site_size(&site);
    messages::warn_about_ignored_pages(&site);
    site.build()
}
