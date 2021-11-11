mod common;

use common::ShortCode;

macro_rules! test_scenario_summary {
    ($in_str:literal, $summary:literal, [$($shortcodes:ident),*]) => {
        let config = config::Config::default_for_test();

        #[allow(unused_mut)]
        let mut tera = tera::Tera::default();

        // Add all shortcodes
        $(
            tera.add_raw_template(
                &format!("shortcodes/{}", $shortcodes.filename()),
                $shortcodes.output
            ).expect("Failed to add raw template");
        )*

        let permalinks = std::collections::HashMap::new();
        let mut context = rendering::RenderContext::new(
            &tera,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );
        let shortcode_def = utils::templates::get_shortcodes(&tera);
        context.set_shortcode_definitions(&shortcode_def);

        let rendered = rendering::render_content($in_str, &context);
        assert!(rendered.is_ok());
        let rendered = rendered.unwrap();

        assert!(rendered.summary_len.is_some());

        let summary_len = rendered.summary_len.unwrap();
        assert_eq!(&rendered.body[..summary_len], $summary);
    }
}

#[test]
fn basic_summary() {
    test_scenario_summary!("Hello World!\n<!-- more -->\nAnd others!", "<p>Hello World!</p>\n", []);
    test_scenario_summary!(
        "Hello World!\n\nWow!\n<!-- more -->\nAnd others!",
        "<p>Hello World!</p>\n<p>Wow!</p>\n",
        []
    );
}

#[test]
fn summary_with_headers() {
    test_scenario_summary!(
        "# Hello World!\n<!-- more -->\nAnd others!",
        "<h1 id=\"hello-world\">Hello World!</h1>\n",
        []
    );
    test_scenario_summary!(
        "# Hello World!\n\nWow!\n<!-- more -->\nAnd others!",
        "<h1 id=\"hello-world\">Hello World!</h1>\n<p>Wow!</p>\n",
        []
    );
}

const MD_SIMPLE: ShortCode =
    ShortCode::new("simple", "A lot of text to insert into the document", true);
const HTML_SIMPLE: ShortCode =
    ShortCode::new("simple", "A lot of text to insert into the document", true);

#[test]
fn summary_with_md_shortcodes() {
    test_scenario_summary!(
        "{{ simple() }}\n<!-- more -->\nAnd others!",
        "<p>A lot of text to insert into the document</p>\n",
        [MD_SIMPLE]
    );
    test_scenario_summary!(
        "{{ simple() }}\n\nWow!\n<!-- more -->\nAnd others!",
        "<p>A lot of text to insert into the document</p>\n<p>Wow!</p>\n",
        [MD_SIMPLE]
    );
}

#[test]
fn summary_with_html_shortcodes() {
    test_scenario_summary!(
        "{{ simple() }}\n<!-- more -->\nAnd others!",
        "<p>A lot of text to insert into the document</p>\n",
        [HTML_SIMPLE]
    );
    test_scenario_summary!(
        "{{ simple() }}\n\nWow!\n<!-- more -->\nAnd others!",
        "<p>A lot of text to insert into the document</p>\n<p>Wow!</p>\n",
        [HTML_SIMPLE]
    );
}

// const INNER: ShortCode = ShortCode::new("inner", "World", false);
//
// const MD_RECURSIVE: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", true);
// const HTML_RECURSIVE: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", false);
//
// #[test]
// fn summary_with_recursive_shortcodes() {
//     test_scenario_summary!(
//         "{{ outer() }}\n<!-- more -->\nAnd others!",
//         "<p>Hello World!</p>\n",
//         [MD_RECURSIVE, INNER]
//     );
//
//     test_scenario_summary!(
//         "{{ outer() }}\n<!-- more -->\nAnd others!",
//         "<p>Hello World!</p>\n",
//         [HTML_RECURSIVE, INNER]
//     );
// }
