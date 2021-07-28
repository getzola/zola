mod common;

use common::ShortCode;

macro_rules! test_scenario {
    ($in_str:literal, $out_str:literal, [$($shortcodes:ident),*]) => {
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
        let context = rendering::RenderContext::new(
            &tera,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let rendered = rendering::render_content($in_str, &context);
        assert!(rendered.is_ok());

        let rendered = rendered.unwrap();
        assert_eq!(rendered.body, $out_str.to_string());
    }
}

macro_rules! test_scenario_fail {
    ($in_str:literal, [$($shortcodes:ident),*]) => {
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
        let context = rendering::RenderContext::new(
            &tera,
            &config,
            &config.default_language,
            "",
            &permalinks,
            front_matter::InsertAnchor::None,
        );

        let rendered = rendering::render_content($in_str, &context);
        assert!(rendered.is_err());
    }
}

#[test]
fn plain_text() {
    // Test basic formation of text and paragraphs tags
    // - Plain sentences (Long and broken up)
    // - Multiple paragraphs

    test_scenario!("Hello World!", "<p>Hello World!</p>\n", []);

    test_scenario!("Hello\nWorld!", "<p>Hello\nWorld!</p>\n", []);

    test_scenario!("Hello\n\nWorld!", "<p>Hello</p>\n<p>World!</p>\n", []);

    test_scenario!("Hello\n\nWorld\n\n!", "<p>Hello</p>\n<p>World</p>\n<p>!</p>\n", []);
}

#[test]
fn header() {
    // Test basic header ids
    // - Plain headers
    // - Headers with text

    test_scenario!("# Header1", "<h1 id=\"header1\">Header1</h1>\n", []);

    test_scenario!(
        "## A longer string of text!",
        "<h2 id=\"a-longer-string-of-text\">A longer string of text!</h2>\n",
        []
    );

    test_scenario!(
        "# Header1\nHello World!",
        "<h1 id=\"header1\">Header1</h1>\n<p>Hello World!</p>\n",
        []
    );

    test_scenario!(
        "# Header1\n\nHello World!",
        "<h1 id=\"header1\">Header1</h1>\n<p>Hello World!</p>\n",
        []
    );
}

#[test]
fn code_block() {
    test_scenario!("```\nWow Code!\n```", "<pre><code>Wow Code!</code></pre>\n", []);

    test_scenario!("    Wow Code!", "<pre><code>Wow Code!</code></pre>\n", []);
}

const MD_SIMPLE: ShortCode = ShortCode::new("simple", "Hello World!", true);
const HTML_SIMPLE: ShortCode = ShortCode::new("simple", "Hello World!", false);

#[test]
fn simple_shortcodes() {
    // Test both MD & HTML plain text shortcodes

    test_scenario!("{{ simple() }}", "<p>Hello World!</p>\n", [MD_SIMPLE]);

    test_scenario!("{{ simple() }}", "<p>Hello World!</p>\n", [HTML_SIMPLE]);
}

const MD_LINK: ShortCode = ShortCode::new("link", "[Link...](/)", true);
const HTML_LINK: ShortCode = ShortCode::new("link", "<a href=\"/\">Link...</a>", false);

#[test]
fn md_inline_shortcodes() {
    // Test both MD & HTML inline shortcodes

    test_scenario!(
        "A read more link: {{ link() }}",
        "<p>A read more link: <a href=\"/\">Link...</a></p>\n",
        [MD_LINK]
    );

    test_scenario!(
        "A read more link: {{ link() }}",
        "<p>A read more link: <a href=\"/\">Link...</a></p>\n",
        [HTML_LINK]
    );
}

const HTML_DIV: ShortCode = ShortCode::new("dived", "<div>Hello World!</div>", false);

#[test]
fn html_div_shortcodes() {
    // Test the behaviour of HTML div-ed shortcodes

    test_scenario!("{{ dived() }}", "<div>Hello World!</div>", [HTML_DIV]);

    test_scenario!(
        "{{ dived() }} {{ dived() }}",
        "<p><div>Hello World!</div> <div>Hello World!</div></p>",
        [HTML_DIV]
    );
    test_scenario!(
        "{{ dived() }}\n{{ dived() }}",
        "<p><div>Hello World!</div>\n<div>Hello World!</div></p>",
        [HTML_DIV]
    );
    test_scenario!(
        "{{ dived() }}\n\n{{ dived() }}",
        "<div>Hello World!</div>\n<div>Hello World!</div>",
        [HTML_DIV]
    );
}

const HTML_TABS_MULTILINE: ShortCode =
    ShortCode::new("multiline", "<div>\n\tHello World!\n</div>", false);

const HTML_SPACES_MULTILINE: ShortCode =
    ShortCode::new("multiline", "<div>\n    Hello World!\n</div>", false);

#[test]
fn html_tabs_multiline_shortcodes() {
    // Test the behaviour multiline HTML shortcodes
    // This can cause problems mostly because the 4 spaces sometimes used for tabs also are used
    // to indicate code-blocks

    test_scenario!("{{ multiline() }}", "<div>\n\tHello World!\n</div>", [HTML_TABS_MULTILINE]);

    test_scenario!(
        "{{ multiline() }} {{ multiline() }}",
        "<p><div>\n\tHello World!\n</div> <div>\n\tHello World!\n</div></p>",
        [HTML_TABS_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n{{ multiline() }}",
        "<p><div>\n\tHello World!\n</div> <div>\n\tHello World!\n</div></p>",
        [HTML_TABS_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n\n{{ multiline() }}",
        "<div>\n\tHello World!\n</div>\n\n<div>\n\tHello World!\n</div>",
        [HTML_TABS_MULTILINE]
    );
}

#[test]
fn html_spaces_multiline_shortcodes() {
    // Test the behaviour multiline HTML shortcodes
    // This can cause problems mostly because the 4 spaces sometimes used for tabs also are used
    // to indicate code-blocks

    test_scenario!("{{ multiline() }}", "<div>\n    Hello World!\n</div>", [HTML_SPACES_MULTILINE]);

    test_scenario!(
        "{{ multiline() }} {{ multiline() }}",
        "<p><div>\n    Hello World!\n</div> <div>\n    Hello World!\n</div></p>",
        [HTML_SPACES_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n{{ multiline() }}",
        "<p><div>\n    Hello World!\n</div> <div>\n    Hello World!\n</div></p>",
        [HTML_SPACES_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n\n{{ multiline() }}",
        "<div>\n    Hello World!\n</div>\n\n<div>\n    Hello World!\n</div>",
        [HTML_SPACES_MULTILINE]
    );
}

const MD_OUTER: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", true);
const MD_INNER: ShortCode = ShortCode::new("inner", "World", true);

const HTML_OUTER: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", false);
const HTML_INNER: ShortCode = ShortCode::new("inner", "World", false);

const MD_REUSER: ShortCode = ShortCode::new("reuser", "{{ reuser() }}", true);
const MD_REFBACK: ShortCode = ShortCode::new("refback", "{{ leveledreuser() }}", true);
const MD_LEVELED_REUSER: ShortCode = ShortCode::new("leveledreuser", "{{ refback() }}", true);

const HTML_REUSER: ShortCode = ShortCode::new("reuser", "{{ reuser() }}", false);
const HTML_REFBACK: ShortCode = ShortCode::new("refback", "{{ leveledreuser() }}", false);
const HTML_LEVELED_REUSER: ShortCode = ShortCode::new("leveledreuser", "{{ refback() }}", false);

#[test]
fn md_recursive_shortcodes() {
    // Test recursive shortcodes in a MD context.
    // This should always work, unless a shortcode is reused

    test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_OUTER, MD_INNER]);
    test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_INNER, MD_OUTER]);
}

#[test]
fn html_recursive_shortcodes() {
    // Test recursive shortcodes in a HTML context.
    // One can add HTML shortcodes within html shortcodes, unless a shortcode is reused

    test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_OUTER, HTML_INNER]);
    test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_INNER, HTML_OUTER]);
}

#[test]
fn shortcodes_recursion_stop() {
    // Test whether recursion stops if a shortcode is reused.

    test_scenario_fail!("{{ reuser() }}", [MD_REUSER]);
    test_scenario_fail!("{{ leveledreuser() }}", [MD_LEVELED_REUSER, MD_REFBACK]);

    test_scenario_fail!("{{ reuser() }}", [HTML_REUSER]);
    test_scenario_fail!("{{ leveledreuser() }}", [HTML_LEVELED_REUSER, HTML_REFBACK]);

    test_scenario_fail!("{{ leveledreuser() }}", [HTML_LEVELED_REUSER, MD_REFBACK]);
    test_scenario_fail!("{{ leveledreuser() }}", [MD_LEVELED_REUSER, HTML_REFBACK]);
}

#[test]
fn html_in_md_recursive_shortcodes() {
    // Test whether we can properly add HTML shortcodes in MD shortcodes

    test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [HTML_INNER, MD_OUTER]);
    test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_OUTER, HTML_INNER]);
}

#[test]
fn md_in_html_recursive_shortcodes() {
    // Test whether we can not add MD shortcodes in HTML shortcodes

    test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_OUTER, MD_INNER]);
    test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [MD_INNER, HTML_OUTER]);
}

const MD_BODY_SHORTCODE: ShortCode = ShortCode::new("bdy", "*{{ body }}*", true);
const HTML_BODY_SHORTCODE: ShortCode = ShortCode::new("bdy", "<span>{{ body }}</span", false);

#[test]
fn md_body_shortcodes() {
    // Test basic MD body shortcodes

    test_scenario!("abc {% bdy() %}def{% end %}", "<p>abc <em>def</em></p>\n", [MD_BODY_SHORTCODE]);

    test_scenario!(
        "abc\n\n{% bdy() %}def{% end %}",
        "<p>abc</p>\n<p><em>def</em></p>\n",
        [MD_BODY_SHORTCODE]
    );
}

#[test]
fn html_body_shortcodes() {
    // Test basic HTML body shortcodes

    test_scenario!(
        "abc {% bdy() %}def{% end %}",
        "<p>abc <span>def</span></p>\n",
        [HTML_BODY_SHORTCODE]
    );

    test_scenario!(
        "abc\n\n{% bdy() %}def{% end %}",
        "<p>abc</p>\n<p><span>def</span></p>\n",
        [HTML_BODY_SHORTCODE]
    );
}

#[test]
fn shortcode_in_md_body() {
    // Test whether we can properly insert a shortcode in a MD shortcode body

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><em>Wow! Hello World!</em></p>\n",
        [MD_BODY_SHORTCODE, MD_SIMPLE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><em>Wow! Hello World!</em></p>\n",
        [MD_SIMPLE, MD_BODY_SHORTCODE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><em>Wow! Hello World!</em></p>\n",
        [MD_BODY_SHORTCODE, HTML_SIMPLE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><em>Wow! Hello World!</em></p>\n",
        [HTML_SIMPLE, MD_BODY_SHORTCODE]
    );
}

#[test]
fn shortcode_in_html_body() {
    // Test whether we can properly insert a shortcode in a HTML shortcode body

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><span>Wow! Hello World!</span></p>\n",
        [HTML_BODY_SHORTCODE, MD_SIMPLE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><span>Wow! Hello World!</span></p>\n",
        [MD_SIMPLE, HTML_BODY_SHORTCODE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><span>Wow! Hello World!</span></p>\n",
        [HTML_BODY_SHORTCODE, HTML_SIMPLE]
    );

    test_scenario!(
        "{% bdy() %}Wow! {{ simple() }}{% end %}",
        "<p><span>Wow! Hello World!</span></p>\n",
        [HTML_SIMPLE, HTML_BODY_SHORTCODE]
    );
}

const MD_INV_COUNTER: ShortCode = ShortCode::new("counter", "{{ nth }}", true);
const HTML_INV_COUNTER: ShortCode = ShortCode::new("counter", "{{ nth }}", false);

#[test]
fn invocation_count() {
    // Test whether the invocation counter works properly in both MD & HTML shortcodes
    test_scenario!("{{ counter() }} {{ counter() }}", "<p>1 2</p>\n", [MD_INV_COUNTER]);

    test_scenario!(
        "{{ counter() }}\n\n{{ counter() }\n\n{{ counter() }}}\n\n{{ counter() }}",
        "<p>1 2 3 4</p>\n",
        [MD_INV_COUNTER]
    );

    test_scenario!("{{ counter() }} {{ counter() }}", "<p>1 2</p>\n", [HTML_INV_COUNTER]);

    test_scenario!(
        "{{ counter() }}\n\n{{ counter() }\n\n{{ counter() }}}\n\n{{ counter() }}",
        "<p>1 2 3 4</p>\n",
        [HTML_INV_COUNTER]
    );
}

const MD_COUNTER_ONE: ShortCode = ShortCode::new("counter1", "{{ nth }}", true);
const MD_COUNTER_TWO: ShortCode = ShortCode::new("counter2", "{{ nth }}", true);
const HTML_COUNTER_ONE: ShortCode = ShortCode::new("counter1", "{{ nth }}", false);
const HTML_COUNTER_TWO: ShortCode = ShortCode::new("counter2", "{{ nth }}", false);

#[test]
fn invocation_count_interferance() {
    // Test whether multiple invocation counters don't interfere with each other
    test_scenario!(
        "{{ counter1() }} {{ counter2() }}",
        "<p>1 1</p>\n",
        [MD_COUNTER_ONE, MD_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }} {{ counter1() }} {{ counter1() }} {{ counter2()}} {{ counter2() }}",
        "<p>1 1 2 3 2 3</p>\n",
        [MD_COUNTER_ONE, MD_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }}",
        "<p>1 1</p>\n",
        [HTML_COUNTER_ONE, HTML_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }} {{ counter1() }} {{ counter1() }} {{ counter2()}} {{ counter2() }}",
        "<p>1 1 2 3 2 3</p>\n",
        [HTML_COUNTER_ONE, HTML_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }}",
        "<p>1 1</p>\n",
        [HTML_COUNTER_ONE, MD_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }} {{ counter1() }} {{ counter1() }} {{ counter2()}} {{ counter2() }}",
        "<p>1 1 2 3 2 3</p>\n",
        [HTML_COUNTER_ONE, MD_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }}",
        "<p>1 1</p>\n",
        [MD_COUNTER_ONE, HTML_COUNTER_TWO]
    );

    test_scenario!(
        "{{ counter1() }} {{ counter2() }} {{ counter1() }} {{ counter1() }} {{ counter2()}} {{ counter2() }}",
        "<p>1 1 2 3 2 3</p>\n",
        [MD_COUNTER_ONE, HTML_COUNTER_TWO]
    );
}

const MD_ARG_SHORTCODE: ShortCode = ShortCode::new(
    "argeater",
    "{{ s }}\n{{ b }}\n{{ f }}\n{{ i }}\n{{ a | join(sep=' // ') }}",
    true,
);
const HTML_ARG_SHORTCODE: ShortCode = ShortCode::new(
    "argeater",
    "{{ s }}\n{{ b }}\n{{ f }}\n{{ i }}\n{{ a | join(sep=' // ') }}",
    false,
);

#[test]
fn shortcode_arguments() {
    // Test for properly inserting all shortcodes

    test_scenario!(
        "{{ argeater(s='Hello World!', b=true, f=3.1415, i=42, array=[1, 3, 3, 7]) }}",
        "<p>Hello World!\ntrue\n3.1415\n42\n1 // 3 // 3 // 7</p>\n",
        [MD_ARG_SHORTCODE]
    );

    test_scenario!(
        "{{ argeater(s='Hello World!', b=true, f=3.1415, i=42, array=[1, 3, 3, 7]) }}",
        "<p>Hello World!\ntrue\n3.1415\n42\n1 // 3 // 3 // 7</p>\n",
        [HTML_ARG_SHORTCODE]
    );
}
