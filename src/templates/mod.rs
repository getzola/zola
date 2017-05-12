use tera::Tera;

pub mod filters;
pub mod global_fns;

lazy_static! {
    pub static ref GUTENBERG_TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("rss.xml", include_str!("builtins/rss.xml")),
            ("sitemap.xml", include_str!("builtins/sitemap.xml")),
            ("robots.txt", include_str!("builtins/robots.txt")),
            ("anchor-link.html", include_str!("builtins/anchor-link.html")),

            ("shortcodes/youtube.html", include_str!("builtins/shortcodes/youtube.html")),
            ("shortcodes/vimeo.html", include_str!("builtins/shortcodes/vimeo.html")),
            ("shortcodes/gist.html", include_str!("builtins/shortcodes/gist.html")),

            ("internal/alias.html", include_str!("builtins/internal/alias.html")),
        ]).unwrap();
        tera
    };
}
