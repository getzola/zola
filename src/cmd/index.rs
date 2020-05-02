use std::path::Path;
use std::time::*;

use errors::{Result, Error};
use site::Site;
use crate::console;

pub fn index(
    root_dir: &Path,
    config_file: &str,
    base_url: Option<&str>,
    output_dir: &str,
    include_drafts: bool,
    index_type: &str,
) -> Result<()> {
    let mut site = Site::new(root_dir, config_file)?;
    site.set_output_path(output_dir);

    // TODO: is base_url even necessary for this command?

    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    if include_drafts {
        site.include_drafts();
    }
    site.load()?;
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);

    let do_elastic_index = || -> Result<()> {
        let indexing_start = Instant::now();
        let n_indexed = site.build_search_index()
            .map_err(|e| Error::from(format!("creating elasticlunr index failed: {}", e)))?;
        let indexing_took = Instant::now() - indexing_start;
        console::report_n_pages_indexed(n_indexed, indexing_took);
        Ok(())
    };

    match index_type {
        "elasticlunr" => do_elastic_index()?,
        "tantivy" => {
            let index_dir = Path::new(output_dir).join("tantivy-index");
            if index_dir.exists() {
                std::fs::remove_dir_all(&index_dir)?;
            }
            utils::fs::ensure_directory_exists(&index_dir)?;

            let lang = &site.config.default_language;
            let library = site.library.read().unwrap(); // unwrap originally in Site::build_search_index, just parroting here, no idea if safe

            let indexing_start = Instant::now();
            let n_pages_indexed = search::build_tantivy_index(lang, &library, &index_dir)?;
            let indexing_took = Instant::now() - indexing_start;
            console::report_n_pages_indexed(n_pages_indexed, indexing_took);
        }

        _ => unreachable!()
    }

    Ok(())
}