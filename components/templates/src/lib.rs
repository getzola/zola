use std::path::Path;
use std::sync::LazyLock;

pub mod filters;
pub mod functions;
mod helpers;

use fs_err as fs;
use tera::{Context, Tera};

use crate::functions::{
    GetHash, GetImageMetadata, GetPage, GetSection, GetTaxonomy, GetTaxonomyTerm, GetTaxonomyUrl,
    GetUrl, LoadData, ResizeImage, Trans,
};
use config::Config;
use errors::{Context as ErrorContext, Result, bail};

const REDIRECT_TPL_NAME: &str = "internal/alias.html";

const BUILTIN_TEMPLATES: &[(&str, &str)] = &[
    ("__zola_builtins/404.html", include_str!("builtins/404.html")),
    ("__zola_builtins/atom.xml", include_str!("builtins/atom.xml")),
    ("__zola_builtins/rss.xml", include_str!("builtins/rss.xml")),
    ("__zola_builtins/sitemap.xml", include_str!("builtins/sitemap.xml")),
    ("__zola_builtins/robots.txt", include_str!("builtins/robots.txt")),
    ("__zola_builtins/split_sitemap_index.xml", include_str!("builtins/split_sitemap_index.xml")),
    ("__zola_builtins/anchor-link.html", include_str!("builtins/anchor-link.html")),
    ("__zola_builtins/summary-cutoff.html", include_str!("builtins/summary-cutoff.html")),
    // This is not overridable by the user
    (REDIRECT_TPL_NAME, include_str!("builtins/internal/alias.html")),
];

pub static ZOLA_TERA: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    // tera-contrib stuff
    tera.register_filter("base64_encode", tera_contrib::base64::b64_encode);
    tera.register_filter("base64_decode", tera_contrib::base64::b64_decode);
    tera.register_filter("regex_replace", tera_contrib::regex::RegexReplace::default());
    tera.register_filter("striptags", tera_contrib::regex::striptags);
    tera.register_filter("spaceless", tera_contrib::regex::spaceless);
    tera.register_filter("urlencode", tera_contrib::urlencode::urlencode);
    tera.register_filter("urlencode_strict", tera_contrib::urlencode::urlencode_strict);
    tera.register_filter("json_encode", tera_contrib::json::json_encode);
    tera.register_filter("filesizeformat", tera_contrib::filesize_format::filesize_format);
    tera.register_filter("format", tera_contrib::format::format);
    tera.register_filter("slugify", tera_contrib::slug::slug);
    tera.register_filter("date", tera_contrib::dates::date);

    tera.register_function("get_random", tera_contrib::rand::get_random);

    tera.register_test("matching", tera_contrib::regex::Matching::default());
    tera.register_test("before", tera_contrib::dates::is_before);
    tera.register_test("after", tera_contrib::dates::is_after);

    // zola specific
    tera.register_function("now", functions::Now::new());
    // needed since tera will validate templates we will load that use these functions
    tera.register_function("get_url", GetUrl::default());
    tera.register_function("get_hash", GetHash::default());
    tera.register_function("get_section", GetSection::default());
    tera.register_function("get_page", GetPage::default());
    tera.register_function("get_taxonomy", GetTaxonomy::default());
    tera.register_function("get_taxonomy_term", GetTaxonomyTerm::default());
    tera.register_function("get_taxonomy_url", GetTaxonomyUrl::default());
    tera.register_function("trans", Trans::default());
    tera.register_function("resize_image", ResizeImage::default());
    tera.register_function("get_image_metadata", GetImageMetadata::default());
    tera.register_function("load_data", LoadData::default());

    tera.add_raw_templates(BUILTIN_TEMPLATES.to_vec()).unwrap();

    tera
});

/// Renders the `internal/alias.html` template that will redirect
/// via refresh to the url given
pub fn render_redirect_template(url: &str, tera: &Tera) -> Result<String> {
    let mut context = Context::new();
    context.insert("url", &url);

    tera.render(REDIRECT_TPL_NAME, &context)
        .with_context(|| format!("Failed to render alias for '{}'", url))
}

/// Creates the Tera instance we will use to render things.
///
/// Combines the builtin Zola templates with an optional theme and the user templates.
/// Filters/Functions requiring the site data will be added to it later
pub fn load_tera(path: &Path, config: &Config) -> Result<Tera> {
    let mut tera = Tera::default();
    let mut templates = Vec::new();

    // Set fallback prefixes FIRST (before loading any templates)
    // Order determines priority: theme (1) > builtins (2)
    let mut fallback_prefixes = Vec::new();
    if let Some(ref theme) = config.theme {
        fallback_prefixes.push(format!("{}/templates/", theme));
    }
    fallback_prefixes.push("__zola_builtins/".to_string());
    tera.set_fallback_prefixes(fallback_prefixes);

    // Register filters/tests/functions from ZOLA_TERA
    tera.register_from(&ZOLA_TERA);

    // Add builtin templates
    for (name, content) in BUILTIN_TEMPLATES {
        templates.push((name.to_string(), content.to_string()));
    }

    // Validate theme exists if configured
    let site_tpl_dir = path.join("templates");
    let theme_tpl_dir =
        config.theme.as_ref().map(|t| path.join("themes").join(t).join("templates"));

    if let Some(ref theme_dir) = theme_tpl_dir {
        if !theme_dir.exists() {
            bail!("Theme `{}` is missing a templates folder", config.theme.as_ref().unwrap());
        }
    }

    if !site_tpl_dir.exists() && theme_tpl_dir.is_none() {
        bail!("Either a `templates/` folder or a theme is required");
    }

    // Load theme templates first (lower priority)
    if let Some(ref theme) = config.theme {
        let pattern = format!(
            "{}/themes/{theme}/templates/**/*.{{html,xml,md,txt,json,ics}}",
            path.display()
        );
        for (file_path, name) in tera::load_from_glob(&pattern)? {
            // "page.html" → "sample/templates/page.html"
            let name = format!("{theme}/templates/{name}");
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read '{}'", file_path.display()))?;
            templates.push((name, content));
        }
    }

    // Load site templates (higher priority, will override theme templates)
    if site_tpl_dir.exists() {
        let pattern = format!("{}/templates/**/*.{{html,xml,md,txt,json,ics}}", path.display());
        for (file_path, name) in tera::load_from_glob(&pattern)? {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read '{}'", file_path.display()))?;
            templates.push((name, content));
        }
    }

    tera.add_raw_templates(templates)?;

    Ok(tera)
}
