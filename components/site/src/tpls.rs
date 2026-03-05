use crate::Site;
use templates::{filters, functions};

/// Adds global fns that are to be available during markdown rendering
pub fn register_early_global_fns(site: &mut Site) {
    site.tera.register_filter(
        "num_format",
        filters::NumFormatFilter::new(&site.config.default_language),
    );

    site.tera.register_function(
        "get_url",
        functions::GetUrl::new(
            site.base_path.clone(),
            site.config.clone(),
            site.permalinks.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "resize_image",
        functions::ResizeImage::new(
            site.base_path.clone(),
            site.imageproc.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "get_image_metadata",
        functions::GetImageMetadata::new(
            site.base_path.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function(
        "load_data",
        functions::LoadData::new(
            site.base_path.clone(),
            site.config.theme.clone(),
            site.output_path.clone(),
        ),
    );
    site.tera.register_function("trans", functions::Trans::new(site.config.clone()));
    site.tera.register_function(
        "get_hash",
        functions::GetHash::new(
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
            site.wikilinks.clone(),
            site.tera.clone(),
        ),
    );
    register_tera_global_fns(site);
}

/// Functions filled once we have parsed all the pages/sections only
pub fn register_tera_global_fns(site: &mut Site) {
    site.tera.register_function(
        "get_page",
        functions::GetPage::new(
            site.base_path.clone(),
            &site.config.default_language,
            site.cache.clone(),
        ),
    );
    site.tera.register_function(
        "get_section",
        functions::GetSection::new(
            site.base_path.clone(),
            &site.config.default_language,
            site.cache.clone(),
        ),
    );
    site.tera.register_function(
        "get_taxonomy",
        functions::GetTaxonomy::new(&site.config.default_language, site.cache.clone()),
    );
    site.tera.register_function(
        "get_taxonomy_url",
        functions::GetTaxonomyUrl::new(
            &site.config.default_language,
            site.cache.clone(),
            site.config.slugify.taxonomies,
        ),
    );
    site.tera.register_function(
        "get_taxonomy_term",
        functions::GetTaxonomyTerm::new(
            &site.config.default_language,
            site.cache.clone(),
            site.config.slugify.taxonomies,
        ),
    );
}
