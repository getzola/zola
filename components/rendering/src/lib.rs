mod codeblock;
mod context;
mod markdown;
mod shortcode;
mod range_relation;
mod table_of_contents;
mod transform;

use shortcode::{fetch_shortcodes, insert_md_shortcodes};

use errors::Result;

pub use context::RenderContext;
use markdown::markdown_to_html;
pub use table_of_contents::Heading;

pub fn render_content(content: &str, context: &RenderContext) -> Result<markdown::Rendered> {
    // Shortcode render order:
    // 1. MD shortcodes
    // 2. MD -> HTML
    // 3. HTML shortcodes

    // Fetch all the defined shortcodes
    let shortcode_definitions = &context.shortcode_definitions;

    let (content, shortcode_ctxs) = fetch_shortcodes(content, shortcode_definitions);

    // This will render both top-level and embedded MD shortcodes (Step 1).
    let (content, shortcode_ctxs) =
        insert_md_shortcodes(content, shortcode_ctxs, &context.tera_context, &context.tera)
            .map_err(Into::<errors::Error>::into)?;

    // Turn the MD into HTML (Step 2).
    // This will also insert the HTML shortcodes (Step 3).
    let html_context = markdown_to_html(&content, &context, shortcode_ctxs)?;

    // TODO: Here issue #1418 could be implemented
    // if do_warn_about_unprocessed_md {
    //     warn_about_unprocessed_md(unprocessed_md);
    // }

    Ok(html_context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use templates::ZOLA_TERA;

    #[test]
    fn can_unignore_inline_shortcode() {
        let config = config::Config::default_for_test();
        let permalinks = std::collections::HashMap::new();
        let context = RenderContext::new(
            &ZOLA_TERA,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let res = render_content("Hello World {{/* youtube() */}}", &context).unwrap();
        assert_eq!(res.body, "Hello World {{ youtube() }}");
    }

    #[test]
    fn can_unignore_shortcode_with_body() {
        let config = config::Config::default_for_test();
        let permalinks = std::collections::HashMap::new();
        let context = RenderContext::new(
            &ZOLA_TERA,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let res = render_content(
            r#"
Hello World
{%/* youtube() */%}Some body {{ hello() }}{%/* end */%}"#,
            &context,
        ).unwrap().body;
        assert_eq!(res, "\nHello World\n{% youtube() %}Some body {{ hello() }}{% end %}");
    }
    // https://github.com/Keats/gutenberg/issues/383
    #[test]
    fn unignore_shortcode_with_body_does_not_swallow_initial_whitespace() {
        let config = config::Config::default_for_test();
        let permalinks = std::collections::HashMap::new();
        let context = RenderContext::new(
            &ZOLA_TERA,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let res = render_content(
            r#"
Hello World
{%/* youtube() */%}
Some body {{ hello() }}{%/* end */%}"#,
            &context,
        ).unwrap().body;
        assert_eq!(res, "\nHello World\n{% youtube() %}\nSome body {{ hello() }}{% end %}");
    }
}
