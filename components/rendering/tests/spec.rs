mod common;

use common::ShortCode;
use std::path::PathBuf;
use templates::ZOLA_TERA;

macro_rules! test_scenario {
    ($in_str:literal, $out_str:literal, [$($shortcodes:ident),*]) => {
        let config = config::Config::default_for_test();

        #[allow(unused_mut)]
        let mut tera = tera::Tera::default();
        tera.extend(&ZOLA_TERA).unwrap();

        $(
            let ShortCode { name, is_md, output } = $shortcodes;
            tera.add_raw_template(
                &format!("shortcodes/{}.{}", name, if is_md { "md" } else { "html" }),
                &output,
            ).unwrap();
        )*

        let mut permalinks = std::collections::HashMap::new();
        permalinks.insert("".to_string(), "".to_string());

        tera.register_filter(
            "markdown",
            templates::filters::MarkdownFilter::new(
                PathBuf::new(),
                config.clone(),
                permalinks.clone(),
            ).unwrap()
        );

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
        println!("{:?}", rendered);
        assert!(rendered.is_ok());

        let rendered = rendered.unwrap();
        assert_eq!(rendered.body, $out_str.to_string());
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
    test_scenario!("```\nWow Code!\n```", "<pre><code>Wow Code!\n</code></pre>\n", []);

    test_scenario!("    Wow Code!", "<pre><code>Wow Code!</code></pre>\n", []);
}

const MD_SIMPLE: ShortCode = ShortCode::new("simple", "Hello World!", true);
const HTML_SIMPLE: ShortCode = ShortCode::new("simple", "Hello World!", false);

#[test]
fn simple_shortcodes() {
    // Test both MD & HTML plain text shortcodes

    test_scenario!("{{ simple() }}", "<p>Hello World!</p>\n", [MD_SIMPLE]);

    test_scenario!("hey {{ simple() }}", "<p>hey Hello World!</p>\n", [HTML_SIMPLE]);
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
        "<p><div>Hello World!</div> <div>Hello World!</div></p>\n",
        [HTML_DIV]
    );
    test_scenario!(
        "{{ dived() }}\n{{ dived() }}",
        "<p><div>Hello World!</div>\n<div>Hello World!</div></p>\n",
        [HTML_DIV]
    );
    test_scenario!(
        "{{ dived() }}\n\n{{ dived() }}",
        "<div>Hello World!</div><div>Hello World!</div>",
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
        "<p><div>\n\tHello World!\n</div> <div>\n\tHello World!\n</div></p>\n",
        [HTML_TABS_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n{{ multiline() }}",
        "<p><div>\n\tHello World!\n</div>\n<div>\n\tHello World!\n</div></p>\n",
        [HTML_TABS_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n\n{{ multiline() }}",
        "<div>\n\tHello World!\n</div><div>\n\tHello World!\n</div>",
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
        "<p><div>\n    Hello World!\n</div> <div>\n    Hello World!\n</div></p>\n",
        [HTML_SPACES_MULTILINE]
    );

    test_scenario!(
        "{{ multiline() }}\n{{ multiline() }}",
        "<p><div>\n    Hello World!\n</div>\n<div>\n    Hello World!\n</div></p>\n",
        [HTML_SPACES_MULTILINE]
    );

    // a single \n would keep it in the same paragraph as above
    // 2 \n would result in different paragraphs and basically ignoring the 2 \n
    test_scenario!(
        "{{ multiline() }}\n\n{{ multiline() }}",
        "<div>\n    Hello World!\n</div><div>\n    Hello World!\n</div>",
        [HTML_SPACES_MULTILINE]
    );
}

// Related to Issue of recursive shortcodes
// const MD_OUTER: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", true);
// const MD_INNER: ShortCode = ShortCode::new("inner", "World", true);
//
// const HTML_OUTER: ShortCode = ShortCode::new("outer", "Hello {{ inner() }}!", false);
// const HTML_INNER: ShortCode = ShortCode::new("inner", "World", false);
//
// const MD_REUSER: ShortCode = ShortCode::new("reuser", "{{ reuser() }}", true);
// const MD_REFBACK: ShortCode = ShortCode::new("refback", "{{ leveledreuser() }}", true);
// const MD_LEVELED_REUSER: ShortCode = ShortCode::new("leveledreuser", "{{ refback() }}", true);
//
// const HTML_REUSER: ShortCode = ShortCode::new("reuser", "{{ reuser() }}", false);
// const HTML_REFBACK: ShortCode = ShortCode::new("refback", "{{ leveledreuser() }}", false);
// const HTML_LEVELED_REUSER: ShortCode = ShortCode::new("leveledreuser", "{{ refback() }}", false);
//
// #[test]
// fn md_recursive_shortcodes() {
//     // Test recursive shortcodes in a MD context.
//     // This should always work, unless a shortcode is reused
//
//     test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_OUTER, MD_INNER]);
//     test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_INNER, MD_OUTER]);
// }
//
// #[test]
// fn html_recursive_shortcodes() {
//     // Test recursive shortcodes in a HTML context.
//     // One can add HTML shortcodes within html shortcodes, unless a shortcode is reused
//
//     test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_OUTER, HTML_INNER]);
//     test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_INNER, HTML_OUTER]);
// }
//
// #[test]
// fn shortcodes_recursion_stop() {
//     // Test whether recursion stops if a shortcode is reused.
//
//     test_scenario_fail!("{{ reuser() }}", [MD_REUSER]);
//     test_scenario_fail!("{{ leveledreuser() }}", [MD_LEVELED_REUSER, MD_REFBACK]);
//
//     test_scenario_fail!("{{ reuser() }}", [HTML_REUSER]);
//     test_scenario_fail!("{{ leveledreuser() }}", [HTML_LEVELED_REUSER, HTML_REFBACK]);
//
//     test_scenario_fail!("{{ leveledreuser() }}", [HTML_LEVELED_REUSER, MD_REFBACK]);
//     test_scenario_fail!("{{ leveledreuser() }}", [MD_LEVELED_REUSER, HTML_REFBACK]);
// }
//
// #[test]
// fn html_in_md_recursive_shortcodes() {
//     // Test whether we can properly add HTML shortcodes in MD shortcodes
//
//     test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [HTML_INNER, MD_OUTER]);
//     test_scenario!("{{ outer() }}", "<p>Hello World!</p>\n", [MD_OUTER, HTML_INNER]);
// }
//
// #[test]
// fn md_in_html_recursive_shortcodes() {
//     // Test whether we can not add MD shortcodes in HTML shortcodes
//
//     test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [HTML_OUTER, MD_INNER]);
//     test_scenario!("{{ outer() }}", "<p>Hello {{ inner() }}!</p>", [MD_INNER, HTML_OUTER]);
// }

const MD_BODY_SHORTCODE: ShortCode = ShortCode::new("bdy", "*{{ body }}*", true);
const HTML_BODY_SHORTCODE: ShortCode = ShortCode::new("bdy", "<span>{{ body }}</span>", false);

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

    // Should it wrap the shortcode in a `<p>`?
    test_scenario!(
        "abc\n\n{% bdy() %}def{% end %}",
        "<p>abc</p>\n<span>def</span>",
        [HTML_BODY_SHORTCODE]
    );
}

// Related to issue #515
// #[test]
// fn shortcode_in_md_body() {
//     // Test whether we can properly insert a shortcode in a MD shortcode body
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><em>Wow! Hello World!</em></p>\n",
//         [MD_BODY_SHORTCODE, MD_SIMPLE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><em>Wow! Hello World!</em></p>\n",
//         [MD_SIMPLE, MD_BODY_SHORTCODE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><em>Wow! Hello World!</em></p>\n",
//         [MD_BODY_SHORTCODE, HTML_SIMPLE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><em>Wow! Hello World!</em></p>\n",
//         [HTML_SIMPLE, MD_BODY_SHORTCODE]
//     );
// }
//
// #[test]
// fn shortcode_in_html_body() {
//     // Test whether we can properly insert a shortcode in a HTML shortcode body
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><span>Wow! Hello World!</span></p>\n",
//         [HTML_BODY_SHORTCODE, MD_SIMPLE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><span>Wow! Hello World!</span></p>\n",
//         [MD_SIMPLE, HTML_BODY_SHORTCODE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><span>Wow! Hello World!</span></p>\n",
//         [HTML_BODY_SHORTCODE, HTML_SIMPLE]
//     );
//
//     test_scenario!(
//         "{% bdy() %}Wow! {{ simple() }}{% end %}",
//         "<p><span>Wow! Hello World!</span></p>\n",
//         [HTML_SIMPLE, HTML_BODY_SHORTCODE]
//     );
// }

const MD_ARG_SHORTCODE: ShortCode = ShortCode::new(
    "argeater",
    "{{ s }}\n{{ b }}\n{{ f }}\n{{ i }}\n{{ array | join(sep=' // ') | safe }}",
    true,
);
const HTML_ARG_SHORTCODE: ShortCode = ShortCode::new(
    "argeater",
    "{{ s }}\n{{ b }}\n{{ f }}\n{{ i }}\n{{ array | join(sep=' // ') | safe }}",
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
        "Hello World!\ntrue\n3.1415\n42\n1 // 3 // 3 // 7",
        [HTML_ARG_SHORTCODE]
    );
}

// const MD_BDY_OUTER: ShortCode = ShortCode::new("outer", "*{{ body }}*", true);
// const MD_BDY_INNER: ShortCode = ShortCode::new("inner", "**{{ body }}**", true);

// Originally from PR #1475
// #[test]
// fn body_in_body() {
//     test_scenario!(
//         "{% outer() %}\n\tTest text\n\t{% inner() %}\n\t\tHello World!\n\t{% end %}\n{% end %}",
//         "<p><em>Test text <b>Hello World!</b></em></p>\n",
//         [MD_BDY_OUTER, MD_BDY_INNER]
//     );
// }

// https://github.com/getzola/zola/issues/1355
#[test]
fn list_with_shortcode() {
    test_scenario!(
        "* a\n* b\n\t{{ multiline() }}\n*c\n\t{{ multiline() }}\n",
        "<ul>\n<li>a</li>\n<li>b\n<div>\n\tHello World!\n</div>\n*c\n<div>\n\tHello World!\n</div></li>\n</ul>\n",
        [HTML_TABS_MULTILINE]
    );
}

const WEB_COMPONENT_SHORTCODE: ShortCode = ShortCode::new(
    "examplecode",
    "<bc-authorizer-example>
  <code>{{ body | safe}}</code>
</bc-authorizer-example>",
    false,
);
// https://github.com/getzola/zola/issues/1655
#[test]
fn shortcodes_do_not_generate_paragraphs() {
    test_scenario!(
        r#"{% examplecode() %}
some code;
more code;

other code here;
{% end %}"#,
        "<bc-authorizer-example>\n  <code>some code;\nmore code;\n\nother code here;</code>\n</bc-authorizer-example>",
        [WEB_COMPONENT_SHORTCODE]
    );
}

const CODE_BLOCK_SHORTCODE: ShortCode = ShortCode::new(
    "details",
    r#"<details>
<summary>{{summary | markdown(inline=true) | safe}}</summary>
<div class="details-content">
{{ body | markdown | safe}}
</div>
</details>"#,
    false,
);
// https://github.com/getzola/zola/issues/1601
#[test]
fn works_with_code_block_in_shortcode() {
    test_scenario!(
        r#"{% details(summary="hey") %}
```
some code
```
{% end %}"#,
        "<details>\n<summary>hey</summary>\n<div class=\"details-content\">\n<pre><code>some code\n</code></pre>\n\n</div>\n</details>",
        [CODE_BLOCK_SHORTCODE]
    );
}

// https://github.com/getzola/zola/issues/1600
#[test]
fn shortcodes_work_in_quotes() {
    test_scenario!(
        "> test quote\n> {{ vimeo(id=\"124313553\") }}\n> test quote",
        "<blockquote>\n<p>test quote\n<div >\n    <iframe src=\"//player.vimeo.com/video/124313553\" webkitallowfullscreen mozallowfullscreen allowfullscreen></iframe>\n</div>\n\ntest quote</p>\n</blockquote>\n",
        []
    );
}

const GOOGLE_SHORTCODE: ShortCode = ShortCode::new(
    "google",
    r#"<div>
<a href="https://google.com/search?q={{query}}">Google Search</a>
</div>"#,
    false,
);
// https://github.com/getzola/zola/issues/1500
#[test]
fn can_handle_issue_1500() {
    test_scenario!(
        r#"foo {{ google(query="apple") }} bar."#,
        "<p>foo <div>\n<a href=\"https://google.com/search?q=apple\">Google Search</a>\n</div> bar.</p>\n",
        [GOOGLE_SHORTCODE]
    );
}
