use std::path::Path;

use tera::Tera;

use crate::Site;
use config::Config;
use errors::{bail, Error, Result};
use templates::{global_fns, ZOLA_TERA};
use utils::templates::rewrite_theme_paths;

pub fn load_tera(path: &Path, config: &Config) -> Result<Tera> {
    let tpl_glob =
        format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "templates/**/*.{*ml,md}");

    // Only parsing as we might be extending templates from themes and that would error
    // as we haven't loaded them yet
    let mut tera =
        Tera::parse(&tpl_glob).map_err(|e| Error::chain("Error parsing templates", e))?;

    if let Some(ref theme) = config.theme {
        // Test that the templates folder exist for that theme
        let theme_path = path.join("themes").join(&theme);
        if !theme_path.join("templates").exists() {
            bail!("Theme `{}` is missing a templates folder", theme);
        }

        let theme_tpl_glob = format!(
            "{}/{}",
            path.to_string_lossy().replace("\\", "/"),
            format!("themes/{}/templates/**/*.{{*ml,md}}", theme)
        );
        let mut tera_theme = Tera::parse(&theme_tpl_glob)
            .map_err(|e| Error::chain("Error parsing templates from themes", e))?;
        rewrite_theme_paths(&mut tera_theme, &theme);

        if theme_path.join("templates").join("robots.txt").exists() {
            tera_theme.add_template_file(theme_path.join("templates").join("robots.txt"), None)?;
        }
        tera.extend(&tera_theme)?;
    }
    tera.extend(&ZOLA_TERA)?;
    tera.build_inheritance_chains()?;

    if path.join("templates").join("robots.txt").exists() {
        tera.add_template_file(path.join("templates").join("robots.txt"), Some("robots.txt"))?;
    }

    Ok(tera)
}

/// Adds global fns that are to be available to shortcodes while rendering markdown
pub fn register_early_global_fns(site: &mut Site) -> Result<()> {
    for (lang, tera) in site.localized_tera.iter_mut() {
        // Split off in the hope that eventually it could be cheaply copied without re-parsing the
        // `.ftl` files, and saving allocation due to the `Arc`s. FIXME
        let loader = global_fns::construct_arc_loader(
            site.config.default_language.clone(),
            site.base_path.clone(),
            site.config.theme.clone(),
        )
        .map_err(|e| Error::from(e.to_string()))?;

        tera.register_function(
            "resize_image",
            global_fns::ResizeImage::new(site.imageproc.clone()),
        );
        tera.register_function(
            "get_image_metadata",
            global_fns::GetImageMeta::new(site.content_path.clone()),
        );
        tera.register_function("load_data", global_fns::LoadData::new(site.base_path.clone()));
        tera.register_function(
            "get_file_hash",
            global_fns::GetFileHash::new(vec![
                site.static_path.clone(),
                site.output_path.clone(),
                site.content_path.clone(),
            ]),
        );
        let cfg = site.config.get_localized(lang).expect("`lang` in config");
        tera.register_function(
            "get_url",
            global_fns::GetUrl::new(
                cfg.clone(),
                site.permalinks.clone(),
                vec![site.static_path.clone(), site.output_path.clone(), site.content_path.clone()],
            ),
        );
        tera.register_function("trans", global_fns::Trans::new(cfg));
        tera.register_function(
            "get_taxonomy_url",
            global_fns::GetTaxonomyUrl::new(
                lang.clone(),
                &site.taxonomies,
                site.config.slugify.taxonomies,
            ),
        );
        if let Some(l) = loader {
            tera.register_function("fluent", global_fns::Fluent::new(l, lang.clone()));
        }
    }

    Ok(())
}

/// Functions filled once we have parsed all the pages/sections only, so not available in shortcodes
pub fn register_tera_global_fns(site: &mut Site) {
    for (lang, tera) in site.localized_tera.iter_mut() {
        tera.register_function(
            "get_page",
            global_fns::GetPage::new(site.base_path.clone(), site.library.clone()),
        );
        tera.register_function(
            "get_section",
            global_fns::GetSection::new(site.base_path.clone(), site.library.clone()),
        );
        tera.register_function(
            "get_taxonomy",
            global_fns::GetTaxonomy::new(
                lang.clone(),
                site.taxonomies.clone(),
                site.library.clone(),
            ),
        );
    }
}
