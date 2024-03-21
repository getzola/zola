use crate::Site;
use libs::tera::Result as TeraResult;
use std::sync::Arc;
use templates::{filters, global_fns};

/// Adds global fns that are to be available to shortcodes while rendering markdown
pub fn register_early_global_fns(site: &mut Site) -> TeraResult<()> {
    site.tera.register_filter(
        "num_format",
        filters::NumFormatFilter::new(&site.config.default_language),
    );

    site.tera.register_function(
        "get_url",
        global_fns::GetUrl::new(
            site.base_path.clone(),
            site.config.clone(),
            site.permalinks.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "resize_image",
        global_fns::ResizeImage::new(
            site.base_path.clone(),
            site.imageproc.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "get_image_metadata",
        global_fns::GetImageMetadata::new(
            site.base_path.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "load_data",
        global_fns::LoadData::new(
            site.base_path.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function("trans", global_fns::Trans::new(site.config.clone()));
    site.tera.register_function(
        "get_taxonomy_url",
        global_fns::GetTaxonomyUrl::new(
            &site.config.default_language,
            &site.taxonomies,
            site.config.slugify.taxonomies,
        ),
    );
    site.tera.register_function(
        "get_hash",
        global_fns::GetHash::new(
            site.base_path.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_filter(
        "markdown",
        filters::MarkdownFilter::new(
            site.config.clone(),
            site.permalinks.clone(),
            site.tera.clone(),
        ),
    );

    Ok(())
}

/// Functions filled once we have parsed all the pages/sections only, so not available in shortcodes
pub fn register_tera_global_fns(site: &mut Site) {
    let language_list: Arc<Vec<String>> =
        Arc::new(site.config.languages.keys().map(|s| s.to_string()).collect());
    site.tera.register_function(
        "get_page",
        global_fns::GetPage::new(
            site.base_path.clone(),
            &site.config.default_language,
            Arc::clone(&language_list),
            site.library.clone(),
        ),
    );
    site.tera.register_function(
        "get_section",
        global_fns::GetSection::new(
            site.base_path.clone(),
            &site.config.default_language,
            Arc::clone(&language_list),
            site.library.clone(),
        ),
    );
    site.tera.register_function(
        "get_taxonomy",
        global_fns::GetTaxonomy::new(
            &site.config.default_language,
            site.taxonomies.clone(),
            site.library.clone(),
        ),
    );
    site.tera.register_function(
        "get_taxonomy_term",
        global_fns::GetTaxonomyTerm::new(
            &site.config.default_language,
            site.taxonomies.clone(),
            site.library.clone(),
        ),
    );
}
