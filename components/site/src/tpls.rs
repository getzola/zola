use crate::Site;
use templates::{filters, global_fns};
use tera::Result as TeraResult;

/// Adds global fns that are to be available to shortcodes while rendering markdown
pub fn register_early_global_fns(site: &mut Site) -> TeraResult<()> {
    site.tera.register_filter(
        "markdown",
        filters::MarkdownFilter::new(
            site.base_path.clone(),
            site.config.clone(),
            site.permalinks.clone(),
        )?,
    );

    site.tera.register_function(
        "get_url",
        global_fns::GetUrl::new(
            site.config.clone(),
            site.permalinks.clone(),
            vec![site.static_path.clone(), site.output_path.clone(), site.content_path.clone()],
        ),
    );
    site.tera
        .register_function("resize_image", global_fns::ResizeImage::new(site.imageproc.clone()));
    site.tera.register_function(
        "get_image_metadata",
        global_fns::GetImageMeta::new(site.content_path.clone()),
    );
    site.tera.register_function("load_data", global_fns::LoadData::new(site.base_path.clone()));
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
        "get_file_hash",
        global_fns::GetFileHash::new(vec![
            site.static_path.clone(),
            site.output_path.clone(),
            site.content_path.clone(),
        ]),
    );

    Ok(())
}

/// Functions filled once we have parsed all the pages/sections only, so not available in shortcodes
pub fn register_tera_global_fns(site: &mut Site) {
    site.tera.register_function(
        "get_page",
        global_fns::GetPage::new(site.base_path.clone(), site.library.clone()),
    );
    site.tera.register_function(
        "get_section",
        global_fns::GetSection::new(site.base_path.clone(), site.library.clone()),
    );
    site.tera.register_function(
        "get_taxonomy",
        global_fns::GetTaxonomy::new(
            &site.config.default_language,
            site.taxonomies.clone(),
            site.library.clone(),
        ),
    );
}
