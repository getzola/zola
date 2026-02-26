#![allow(dead_code)]

use ahash::AHashMap;
use config::Config;
use errors::Result;
use markdown::{MarkdownContext, Rendered, render_content};
use templates::ZOLA_TERA;
use utils::types::InsertAnchor;

fn configurable_render(
    content: &str,
    config: Config,
    insert_anchor: InsertAnchor,
) -> Result<Rendered> {
    let mut tera = ZOLA_TERA.clone();

    let permalinks = AHashMap::from_iter([
        ("pages/about.md".to_owned(), "https://getzola.org/about/".to_owned()),
        ("guides/quickstart.md".to_owned(), "https://getzola.org/guides/quickstart/".to_owned()),
        ("about.md".to_owned(), "https://getzola.org/about/".to_owned()),
    ]);

    let wikilinks = AHashMap::from_iter([
        ("guides/quickstart".to_owned(), "guides/quickstart.md".to_owned()),
        ("quickstart".to_owned(), "guides/quickstart.md".to_owned()),
        ("about".to_owned(), "about.md".to_owned()),
    ]);

    tera.register_filter(
        "markdown",
        templates::filters::MarkdownFilter::new(
            config.clone(),
            permalinks.clone(),
            AHashMap::new(),
            wikilinks.clone(),
            tera.clone(),
        ),
    );
    let colocated_assets = AHashMap::new();
    let context = MarkdownContext {
        tera: &tera,
        config: &config,
        permalinks: &permalinks,
        colocated_assets: &colocated_assets,
        wikilinks: &wikilinks,
        lang: &config.default_language,
        current_permalink: "https://www.getzola.org/test/",
        current_path: "my_page.md",
        insert_anchor,
    };

    render_content(content, &context)
}

pub fn render(content: &str) -> Result<Rendered> {
    configurable_render(content, Config::default_for_test(), InsertAnchor::None)
}

pub fn render_with_config(content: &str, config: Config) -> Result<Rendered> {
    configurable_render(content, config, InsertAnchor::None)
}

pub fn render_with_insert_anchor(content: &str, insert_anchor: InsertAnchor) -> Result<Rendered> {
    configurable_render(content, Config::default_for_test(), insert_anchor)
}
