pub mod filters;
pub mod global_fns;

use std::path::Path;

use config::Config;
use lazy_static::lazy_static;
use tera::{Context, Tera};

use errors::{bail, Error, Result};
use utils::templates::rewrite_theme_paths;

lazy_static! {
    pub static ref ZOLA_TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("__zola_builtins/404.html", include_str!("builtins/404.html")),
            ("__zola_builtins/atom.xml", include_str!("builtins/atom.xml")),
            ("__zola_builtins/rss.xml", include_str!("builtins/rss.xml")),
            ("__zola_builtins/sitemap.xml", include_str!("builtins/sitemap.xml")),
            ("__zola_builtins/robots.txt", include_str!("builtins/robots.txt")),
            (
                "__zola_builtins/split_sitemap_index.xml",
                include_str!("builtins/split_sitemap_index.xml"),
            ),
            ("__zola_builtins/anchor-link.html", include_str!("builtins/anchor-link.html")),
            (
                "__zola_builtins/shortcodes/youtube.html",
                include_str!("builtins/shortcodes/youtube.html"),
            ),
            (
                "__zola_builtins/shortcodes/vimeo.html",
                include_str!("builtins/shortcodes/vimeo.html"),
            ),
            ("__zola_builtins/shortcodes/gist.html", include_str!("builtins/shortcodes/gist.html")),
            (
                "__zola_builtins/shortcodes/streamable.html",
                include_str!("builtins/shortcodes/streamable.html"),
            ),
            ("internal/alias.html", include_str!("builtins/internal/alias.html")),
        ])
        .unwrap();
        tera.register_filter("base64_encode", filters::base64_encode);
        tera.register_filter("base64_decode", filters::base64_decode);
        tera
    };
}

/// Renders the `internal/alias.html` template that will redirect
/// via refresh to the url given
pub fn render_redirect_template(url: &str, tera: &Tera) -> Result<String> {
    let mut context = Context::new();
    context.insert("url", &url);

    tera.render("internal/alias.html", &context)
        .map_err(|e| Error::chain(format!("Failed to render alias for '{}'", url), e))
}

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
