#![allow(dead_code)]

use ahash::AHashMap;
use config::Config;
use errors::Result;
use markdown::{MarkdownContext, Rendered, WikilinkResolver, render_content};
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
        ("guides/index.md".to_owned(), "https://getzola.org/guides/".to_owned()),
        ("about.md".to_owned(), "https://getzola.org/about/".to_owned()),
        ("archive/duplicate.md".to_owned(), "https://getzola.org/archive/duplicate/".to_owned()),
        ("guides/duplicate.md".to_owned(), "https://getzola.org/guides/duplicate/".to_owned()),
    ]);

    let mut wikilinks = WikilinkResolver::default();
    wikilinks.add("guides/quickstart.md", &vec!["/start/".to_string()]);
    wikilinks.add("about.md", &vec![]);
    wikilinks.add("archive/duplicate.md", &vec![]);
    wikilinks.add("guides/duplicate.md", &vec![]);
    wikilinks.add_asset("guides/source.pdf");
    wikilinks.add_asset("archive/manual.pdf");
    wikilinks.add_asset("guides/manual.pdf");

    let colocated_assets = AHashMap::from_iter([(
        "guides/source.pdf".to_owned(),
        ("guides/index.md".to_owned(), "source.pdf".to_owned()),
    )]);

    tera.register_filter(
        "markdown",
        templates::filters::MarkdownFilter::new(
            config.clone(),
            permalinks.clone(),
            colocated_assets.clone(),
            wikilinks.clone(),
            AHashMap::default(),
            tera.clone(),
        ),
    );
    let context = MarkdownContext {
        tera: &tera,
        config: &config,
        permalinks: &permalinks,
        colocated_assets: &colocated_assets,
        wikilinks: &wikilinks,
        taxonomy_permalinks: &AHashMap::default(),
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
