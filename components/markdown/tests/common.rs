#![allow(dead_code)]

use std::collections::HashMap;

use tera::Tera;

use config::Config;
use errors::Result;
use markdown::{RenderContext, Rendered, render_content};
use templates::ZOLA_TERA;
use utils::types::InsertAnchor;

fn configurable_render(
    content: &str,
    config: Config,
    insert_anchor: InsertAnchor,
) -> Result<Rendered> {
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();

    let mut permalinks = HashMap::new();
    permalinks.insert("pages/about.md".to_owned(), "https://getzola.org/about/".to_owned());

    tera.register_filter(
        "markdown",
        templates::filters::MarkdownFilter::new(config.clone(), permalinks.clone(), tera.clone()),
    );
    let mut context = RenderContext::new(
        &tera,
        &config,
        &config.default_language,
        "https://www.getzola.org/test/",
        &permalinks,
        insert_anchor,
    );
    context.set_current_page_path("my_page.md");

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
