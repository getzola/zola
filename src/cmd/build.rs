use std::path::Path;

use errors::{Error, Result};
use site::Site;
use utils::timings;

use crate::messages;

// One parameter per CLI flag; grouping them into a struct would only move the
// same list somewhere else.
#[allow(clippy::too_many_arguments)]
pub fn build(
    root_dir: &Path,
    config_file: &Path,
    base_url: Option<&str>,
    output_dir: Option<&Path>,
    force: bool,
    include_drafts: bool,
    minify: bool,
    show_timings: bool,
) -> Result<()> {
    if show_timings {
        timings::enable();
    }
    let result =
        build_inner(root_dir, config_file, base_url, output_dir, force, include_drafts, minify);
    // Printed even when the build failed: knowing which phase we got to is the
    // point of the flag.
    if let Some(report) = timings::report() {
        println!("{report}");
    }
    result
}

fn build_inner(
    root_dir: &Path,
    config_file: &Path,
    base_url: Option<&str>,
    output_dir: Option<&Path>,
    force: bool,
    include_drafts: bool,
    minify: bool,
) -> Result<()> {
    let mut site = {
        let _span = timings::span("site::new");
        Site::new(root_dir, config_file)?
    };
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
    {
        let _span = timings::span("load");
        site.load()?;
    }
    messages::notify_site_size(&site);
    messages::warn_about_ignored_pages(&site);
    let _span = timings::span("build");
    site.build()
}
