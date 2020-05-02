use std::path::Path;

use errors::Result;
use site::Site;

//use crate::console;

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

    // TODO: could skipping the theme and/or sass prep end up
    // somehow impacting the search indexing? doesn't seem like
    // it could, but maybe

    match index_type {
        "elasticlunr" => {
            site.build_search_index()?;
        }

        "tantivy" => {
            //if ! Path::new(output_dir).exists() {
            //    std::fs::create_dir_all(output_dir)?;
            //}
            let index_dir = Path::new(output_dir).join("tantivy-index");
            utils::fs::ensure_directory_exists(&index_dir)?;

            let lang = &site.config.default_language;
            let library = site.library.read().unwrap(); // unwrap originally in Site::build_search_index, just parroting here, no idea if safe

            search::build_tantivy_index(lang, &library, output_dir)?;
        }

        _ => unreachable!()
    }

    Ok(())
}