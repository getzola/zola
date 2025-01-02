#![allow(dead_code)]

use std::collections::HashMap;

use libs::tera::Tera;

use config::Config;
use errors::Result;
use markdown::{render_content, RenderContext, Rendered};
use templates::ZOLA_TERA;
use utils::types::InsertAnchor;

fn configurable_render(
    content: &str,
    config: Config,
    insert_anchor: InsertAnchor,
) -> Result<Rendered> {
    let mut tera = Tera::default();
    tera.extend(&ZOLA_TERA).unwrap();

    // out_put_id looks like a markdown string
    tera.add_raw_template("shortcodes/out_put_id.html", "{{id}}").unwrap();
    tera.add_raw_template(
        "shortcodes/image.html",
        "<img src='https://placekitten.com/200/300' alt='{{alt}}'></img>",
    )
    .unwrap();
    tera.add_raw_template("shortcodes/split_lines.html", r#"{{ body | split(pat="\n") }}"#)
        .unwrap();
    tera.add_raw_template("shortcodes/ex1.html", "1").unwrap();
    tera.add_raw_template("shortcodes/ex2.html", "2").unwrap();
    tera.add_raw_template("shortcodes/ex3.html", "3").unwrap();
    tera.add_raw_template("shortcodes/with_tabs.html", "<div>\n\tHello World!\n    </div>")
        .unwrap();
    tera.add_raw_template(
        "shortcodes/web_component.html",
        "<bc-authorizer-example><code>{{ body | safe}}</code></bc-authorizer-example>",
    )
    .unwrap();
    tera.add_raw_template("shortcodes/render_md.html", "<div>{{ body | markdown | safe}}</div>")
        .unwrap();
    tera.add_raw_template("shortcodes/a.html", "<p>a: {{ nth }}</p>").unwrap();
    tera.add_raw_template("shortcodes/b.html", "<p>b: {{ nth }}</p>").unwrap();
    tera.add_raw_template("shortcodes/a_md.md", "**a: {{ nth }}**").unwrap();
    tera.add_raw_template("shortcodes/b_md.md", "**b: {{ nth }}**").unwrap();
    tera.add_raw_template("shortcodes/quote.html", "<quote>{{body}}</quote>").unwrap();
    tera.add_raw_template("shortcodes/pre.html", "<pre>{{body}}</pre>").unwrap();
    tera.add_raw_template("shortcodes/four_spaces.html", "    no highlight\n    or there").unwrap();
    tera.add_raw_template("shortcodes/i18n.html", "{{lang}}").unwrap();
    tera.add_raw_template(
        "shortcodes/book.md",
        "![Book cover in {{ lang }}](cover.{{ lang }}.png)",
    )
    .unwrap();
    tera.add_raw_template("shortcodes/md_passthrough.md", "{{body}}").unwrap();
    tera.add_raw_template("shortcodes/nth.html", "{{ nth }}").unwrap();

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
    let shortcode_def = utils::templates::get_shortcodes(&tera);
    context.set_shortcode_definitions(&shortcode_def);
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
